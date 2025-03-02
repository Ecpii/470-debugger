use std::collections::HashMap;

use indicium::simple::{AutocompleteType, SearchIndex, SearchIndexBuilder, SearchType};
use vcd::{Header, IdCode, Scope, ScopeItem};

use crate::trace_dbg;

pub struct VarIndex {
    pub vars: HashMap<String, IdCode>,
    pub index: SearchIndex<String>,
}

impl VarIndex {
    pub fn from_header(header: Header) -> Self {
        let mut vars = HashMap::new();
        let mut index = SearchIndexBuilder::default()
            .search_type(SearchType::And)
            .autocomplete_type(AutocompleteType::Context)
            .split_pattern(Some(vec!['.']))
            .min_keyword_len(0)
            // .split_pattern(None)
            .exclude_keywords(None)
            .build();

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

        while let Some((prefix, next)) = to_search.pop() {
            for scope_item in next.items {
                match scope_item {
                    ScopeItem::Scope(scope) => {
                        let name = prefix.clone() + "." + &scope.identifier;
                        to_search.push((name, scope.clone()));
                    }
                    ScopeItem::Var(var) => {
                        let name = prefix.clone() + "." + &var.reference;
                        index.insert(&name, &name);
                        trace_dbg!(&name);
                        vars.insert(name, var.code);
                    }
                    _ => {}
                }
            }
        }
        Self { vars, index }
    }

    pub fn get(&self, query: &str) -> Option<IdCode> {
        self.vars.get(query).copied()
    }
}
