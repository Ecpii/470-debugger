use branch_stack::BranchStack;
use branches::Btb;
use complete::Complete;
use crossterm::event::{KeyCode, KeyEvent};
use dcache::DCache;
use facache::FaCache;
use fu::FU;
use issue::Issue;
use memunit::MemUnit;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{StatefulWidget, Tabs, Widget};
use regfile::RegFile;
use rob::ROBTable;
use rs::RSTable;
use store_queue::StoreQueue;
use vcd::ScopeItem;

use crate::snapshots::Snapshots;
use crate::utils::{split_horizontal, split_vertical};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, FromRepr};

mod branch_stack;
mod branches;
mod complete;
mod dcache;
mod facache;
mod fu;
mod issue;
mod map_table;
mod memunit;
mod regfile;
mod rob;
mod rs;
mod store_queue;

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, EnumCountMacro)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "RS/ROB")]
    RsRob,
    #[strum(to_string = "Branch Stack")]
    BStack,
    #[strum(to_string = "Issue/FUs")]
    IssueFUs,
    #[strum(to_string = "Caches")]
    Caches,
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
    is_cpu: bool,
    rs: Option<RSTable>,
    rob: Option<ROBTable>,
    bstack: Option<BranchStack>,
    btb: Option<Btb>,
    issue: Option<Issue>,
    dcache: Option<DCache>,
    facache: Option<FaCache>,
    regfile: Option<RegFile>,
    store_queue: Option<StoreQueue>,
    memunit: Option<MemUnit>,
    fu: Option<FU>,
    complete: Option<Complete>,
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
        let mut issue = None;
        let mut dcache = None;
        let mut fu = None;
        let mut facache = None;
        let mut regfile = None;
        let mut store_queue = None;
        let mut memunit = None;
        let mut complete = None;
        let mut is_cpu = false;

        let base = snapshots.get_base();
        let testbench = snapshots.header.find_scope(&[base.clone()]).unwrap();

        for scope_item in testbench.items.iter() {
            let ScopeItem::Scope(scope) = scope_item else {
                continue;
            };
            let new_base = format!("{base}.{}", scope.identifier);

            let cpu_var = format!("{new_base}.dbg_this_is_cpu");
            is_cpu = snapshots.get_var(&cpu_var).is_some();

            if is_cpu {
                // get all the cpu paths
                rs = RSTable::new(&format!("{new_base}.rs_module"), snapshots);
                rob = ROBTable::new(&format!("{new_base}.rob_module"), snapshots);
                bstack = BranchStack::new(&format!("{new_base}.branch_stack_module"), snapshots);
                btb = Btb::new(&format!("{new_base}.btb"), snapshots);
                issue = Issue::new(&format!("{new_base}.issue_module"), snapshots);
                regfile = RegFile::new(&format!("{new_base}.regfile_module"), snapshots);
                dcache = DCache::new(&format!("{new_base}.dcache_module"), snapshots);
                store_queue = StoreQueue::new(&format!("{new_base}.store_queue_module"), snapshots);
                memunit = MemUnit::new(&format!("{new_base}.memunit_module"), snapshots);
                fu = FU::new(&format!("{new_base}.fu_module"), snapshots);
                complete = Complete::new(&format!("{new_base}.complete_module"), snapshots);

                break;
            } else {
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
                if issue.is_none() {
                    issue = Issue::new(&new_base, snapshots);
                }
                if dcache.is_none() {
                    dcache = DCache::new(&new_base, snapshots);
                }
                if facache.is_none() {
                    facache = FaCache::new(&new_base, snapshots);
                }
                if regfile.is_none() {
                    regfile = RegFile::new(&new_base, snapshots);
                }
                if store_queue.is_none() {
                    store_queue = StoreQueue::new(&new_base, snapshots);
                }
                if memunit.is_none() {
                    memunit = MemUnit::new(&new_base, snapshots);
                }
                if fu.is_none() {
                    fu = FU::new(&new_base, snapshots);
                }
                if complete.is_none() {
                    complete = Complete::new(&new_base, snapshots);
                }
            }
        }

        Self {
            rs,
            rob,
            bstack,
            btb,
            is_cpu,
            issue,
            dcache,
            facache,
            regfile,
            store_queue,
            memunit,
            complete,
            fu,
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
        if self.is_cpu {
            use Constraint::{Length, Min};
            let vertical = Layout::vertical([Length(1), Min(0)]);
            let [header_area, inner_area] = vertical.areas(area);

            let horizontal = Layout::horizontal([Min(0), Length(20)]);
            let [tabs_area, _title_area] = horizontal.areas(header_area);

            self.render_tabs(tabs_area, buf);

            match self.selected_tab {
                SelectedTab::RsRob => {
                    let areas = split_rectangle_horizontal(inner_area);
                    self.rs.unwrap().render(areas[0], buf, state);
                    self.rob.unwrap().render(areas[1], buf, state);
                }
                SelectedTab::BStack => {
                    let areas = split_rectangle_horizontal(inner_area);
                    if let Some(btb) = self.btb {
                        let [top_area, bottom_area] = Layout::vertical([
                            Constraint::Length(btb.size as u16 + 1 + 2),
                            Constraint::Fill(1),
                        ])
                        .areas(inner_area);

                        btb.render(top_area, buf, state);
                        let bottom_areas = split_rectangle_horizontal(bottom_area);
                        self.bstack.unwrap().render(bottom_areas[0], buf, state);
                        self.regfile.unwrap().render(bottom_areas[1], buf, state);
                    } else {
                        self.bstack.unwrap().render(areas[0], buf, state);
                        self.rob.unwrap().render(areas[1], buf, state);
                    }
                }
                SelectedTab::IssueFUs => {
                    let [left_area, right_area] = split_horizontal(inner_area);
                    let [top_left_area, bottom_left_area] = split_vertical(left_area);
                    let [top_right_area, bottom_right_area] = split_vertical(right_area);

                    self.issue.unwrap().render(top_left_area, buf, state);
                    self.fu.unwrap().render(top_right_area, buf, state);
                    self.complete.unwrap().render(bottom_left_area, buf, state);
                    self.memunit.unwrap().render(bottom_right_area, buf, state);
                }
                SelectedTab::Caches => {
                    let areas = split_rectangle_horizontal(inner_area);
                    self.dcache.unwrap().render(areas[0], buf, state);
                    self.store_queue.unwrap().render(areas[1], buf, state);
                }
            }
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
            } else if let Some(issue) = self.issue {
                issue.render(area, buf, state);
            } else if let Some(dcache) = self.dcache {
                dcache.render(area, buf, state);
            } else if let Some(facache) = self.facache {
                facache.render(area, buf, state);
            } else if let Some(regfile) = self.regfile {
                regfile.render(area, buf, state);
            } else if let Some(memunit) = self.memunit {
                memunit.render(area, buf, state);
            } else if let Some(store_queue) = self.store_queue {
                store_queue.render(area, buf, state);
            } else if let Some(complete) = self.complete {
                complete.render(area, buf, state);
            }
        }
    }
}
