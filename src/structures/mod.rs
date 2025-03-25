use branch_stack::BranchStack;
use branches::Btb;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{StatefulWidget, Tabs, Widget};
use rob::ROBTable;
use rs::RSTable;
use vcd::{ScopeItem, ScopeType};

use crate::snapshots::Snapshots;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, FromRepr};

mod branch_stack;
mod branches;
mod map_table;
mod rob;
mod rs;

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, EnumCountMacro)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "RS/ROB")]
    RsRob,
    #[strum(to_string = "Branch Stack")]
    BStack,
}

impl SelectedTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = if current_index == 0 {
            SelectedTab::COUNT - 1
        } else {
            current_index - 1
        };
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = if current_index == SelectedTab::COUNT - 1 {
            0
        } else {
            current_index + 1
        };
        Self::from_repr(next_index).unwrap_or(self)
    }
}

#[derive(Clone)]
pub struct Structures {
    rs: Option<RSTable>,
    rob: Option<ROBTable>,
    bstack: Option<BranchStack>,
    btb: Option<Btb>,
    selected_tab: SelectedTab,
}

impl Structures {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = SelectedTab::iter().map(|x| x.to_string());
        let highlight_style = Style::new().bg(Color::Blue);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }

    pub fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::BackTab) => self.selected_tab = self.selected_tab.previous(),
            (_, KeyCode::Tab) => self.selected_tab = self.selected_tab.next(),
            _ => {}
        }
    }

    pub fn new(snapshots: &Snapshots) -> Self {
        let mut rs = None;
        let mut rob = None;
        let mut bstack = None;
        let mut btb = None;

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
            if bstack.is_none() {
                bstack = BranchStack::new(&new_base_outer, snapshots);
            }
            if btb.is_none() {
                btb = Btb::new(&new_base_outer, snapshots);
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
                if bstack.is_none() {
                    bstack = BranchStack::new(&new_base, snapshots);
                }
                if btb.is_none() {
                    btb = Btb::new(&new_base, snapshots);
                }
            }
        }

        Self {
            rs,
            rob,
            bstack,
            btb,
            selected_tab: SelectedTab::default(),
        }
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
        // dirty assumption that if we have both rs and rob, it must be a cpu test
        let is_cpu = self.rs.is_some() && self.rob.is_some();

        if is_cpu {
            use Constraint::{Length, Min};
            let vertical = Layout::vertical([Length(1), Min(0)]);
            let [header_area, inner_area] = vertical.areas(area);

            let horizontal = Layout::horizontal([Min(0), Length(20)]);
            let [tabs_area, _title_area] = horizontal.areas(header_area);

            // render_title(title_area, buf);
            self.render_tabs(tabs_area, buf);

            match self.selected_tab {
                SelectedTab::RsRob => {
                    let areas = split_rectangle_horizontal(inner_area);
                    self.rs.unwrap().render(areas[0], buf, state);
                    self.rob.unwrap().render(areas[1], buf, state);
                }
                SelectedTab::BStack => {
                    if let Some(btb) = self.btb {
                        let [top_area, bottom_area] = Layout::vertical([
                            Constraint::Length(btb.size as u16 + 1 + 2),
                            Constraint::Fill(1),
                        ])
                        .areas(inner_area);

                        btb.render(top_area, buf, state);
                        self.bstack.unwrap().render(bottom_area, buf, state);
                    } else {
                        self.bstack.unwrap().render(inner_area, buf, state);
                    }
                }
            }

            // render_footer(footer_area, buf);
        } else {
            // assumption: just a single module test (though this could change in the future)
            if let Some(rs) = self.rs {
                rs.render(area, buf, state);
            } else if let Some(rob) = self.rob {
                rob.render(area, buf, state);
            } else if let Some(bstack) = self.bstack {
                bstack.render(area, buf, state);
            } else if let Some(btb) = self.btb {
                btb.render(area, buf, state);
            }
        }
    }
}
