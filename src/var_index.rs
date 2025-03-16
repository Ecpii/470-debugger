use std::collections::HashMap;

use simsearch::{SearchOptions, SimSearch};
use vcd::{Header, IdCode, Scope, ScopeItem};

use crate::trace_dbg;

pub struct VarIndex {
    pub vars: HashMap<String, IdCode>,
    pub engine: SimSearch<String>,
}

impl VarIndex {
    pub fn from_header(header: &Header) -> Self {
        let mut vars = HashMap::new();

        let mut to_search = Vec::<(String, Scope)>::new();

        for scope_item in header.items.iter() {
            let ScopeItem::Scope(s) = scope_item else {
                continue;
            };
            if s.identifier.starts_with("_vcs") {
                continue;
            }
            to_search.push((s.identifier.clone(), s.clone()))
        }

        let mut engine = SimSearch::new_with(SearchOptions::new().levenshtein(true).threshold(0.3));

        while let Some((prefix, next)) = to_search.pop() {
            for scope_item in next.items {
                match scope_item {
                    ScopeItem::Scope(scope) => {
                        let name = prefix.clone() + "." + &scope.identifier;
                        to_search.push((name, scope.clone()));
                    }
                    ScopeItem::Var(var) => {
                        let name = prefix.clone() + "." + &var.reference;
                        engine.insert(name.clone(), &name);
                        trace_dbg!(&name);
                        vars.insert(name, var.code);
                    }
                    _ => {}
                }
            }
        }
        Self { vars, engine }
    }

    pub fn get(&self, query: &str) -> Option<IdCode> {
        self.vars.get(query).copied()
    }
}
