use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use rob::ROBTable;
use rs::RSTable;
use vcd::{ScopeItem, ScopeType};

use crate::snapshots::Snapshots;

mod rob;
mod rs;

#[derive(Clone)]
pub struct Structures {
    rs: Option<RSTable>,
    rob: Option<ROBTable>,
}

impl Structures {
    pub fn new(snapshots: &Snapshots) -> Self {
        let mut rs = None;
        let mut rob = None;

        let base = snapshots.get_base();
        let testbench = snapshots.header.find_scope(&[base.clone()]).unwrap();

        for scope_item_outer in testbench.items.iter() {
            let ScopeItem::Scope(scope_outer) = scope_item_outer else {
                continue;
            };
            let new_base_outer = format!("{base}.{}", scope_outer.identifier);

            // try to fit each module into rs or rob if they match the shape
            if rs.is_none() {
                rs = RSTable::new(&new_base_outer, snapshots);
            }
            if rob.is_none() {
                rob = ROBTable::new(&new_base_outer, snapshots);
            }

            for scope_item in scope_outer.items.iter() {
                let ScopeItem::Scope(scope) = scope_item else {
                    continue;
                };
                if !matches!(scope.scope_type, ScopeType::Module) {
                    continue;
                }

                let new_base = format!("{base}.{}.{}", scope_outer.identifier, scope.identifier);

                // try to fit each module into rs or rob if they match the shape
                if rs.is_none() {
                    rs = RSTable::new(&new_base, snapshots);
                }
                if rob.is_none() {
                    rob = ROBTable::new(&new_base, snapshots);
                }
            }
        }

        // trace_dbg!(&rs);

        Self { rs, rob }
    }
}

// ai generated(gemini)
fn split_rectangle_horizontal(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area)
        .to_vec()
}

impl StatefulWidget for Structures {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        if let Some(rs) = self.rs {
            if let Some(rob) = self.rob {
                let chunks = split_rectangle_horizontal(area);
                rs.render(chunks[0], buf, state);
                rob.render(chunks[1], buf, state);
            } else {
                rs.render(area, buf, state);
            }
        } else if let Some(rob) = self.rob {
            rob.render(area, buf, state);
        }
    }
}
