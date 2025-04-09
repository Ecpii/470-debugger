use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, StatefulWidget, Table, Widget},
};

use crate::{
    snapshots::Snapshots,
    utils::{get_fu_col_widths, get_fu_output_header, parse_fu_output_packet, TOP_BORDER_SET},
};

// true if we can use the raw name as the key to index

#[derive(Clone, Debug)]
pub struct FU {
    base: String,
    num_alus: usize,
    num_mults: usize,
    num_branches: usize,
    num_stores: usize,
}

impl FU {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is a btb
        snapshots.get_var(&format!("{base}.dbg_this_is_fu"))?;

        let mut num_alus = 0;
        let mut num_mults = 0;
        let mut num_branches = 0;
        let mut num_stores = 0;

        let mut entry_name = format!("{base}.alu_output_packets[{num_alus}]");
        while snapshots.get_scope(&entry_name).is_some() {
            num_alus += 1;
            entry_name = format!("{base}.alu_output_packets[{num_alus}]");
        }

        entry_name = format!("{base}.mult_output_packets[{num_mults}]");
        while snapshots.get_scope(&entry_name).is_some() {
            num_mults += 1;
            entry_name = format!("{base}.mult_output_packets[{num_mults}]");
        }

        entry_name = format!("{base}.branch_output_packets[{num_branches}]");
        while snapshots.get_scope(&entry_name).is_some() {
            num_branches += 1;
            entry_name = format!("{base}.branch_output_packets[{num_branches}]");
        }

        entry_name = format!("{base}.store_output_packets[{num_stores}]");
        while snapshots.get_scope(&entry_name).is_some() {
            num_stores += 1;
            entry_name = format!("{base}.store_output_packets[{num_stores}]");
        }

        Some(Self {
            base: base.to_owned(),
            num_alus,
            num_mults,
            num_branches,
            num_stores,
        })
    }

    fn get_alu_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let header = get_fu_output_header();

        let mut rows = Vec::new();
        for i in 0..self.num_alus {
            let packet_base = format!("{}.alu_output_packets[{i}]", self.base);
            rows.push(parse_fu_output_packet(&packet_base, snapshots));
        }

        let title = Line::from("ALU Packets").bold().centered();
        let block = Block::new()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title);
        Table::new(rows, get_fu_col_widths())
            .header(header)
            .block(block)
    }

    fn get_mult_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let mut rows = Vec::new();
        for i in 0..self.num_mults {
            let packet_base = format!("{}.mult_output_packets[{i}]", self.base);
            rows.push(parse_fu_output_packet(&packet_base, snapshots));
        }

        let title = Line::from("Mult Packets").bold().centered();
        let block = Block::new()
            .border_set(TOP_BORDER_SET)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title);
        Table::new(rows, get_fu_col_widths()).block(block)
    }

    fn get_branch_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let mut rows = Vec::new();
        for i in 0..self.num_branches {
            let packet_base = format!("{}.branch_output_packets[{i}]", self.base);
            rows.push(parse_fu_output_packet(&packet_base, snapshots));
        }

        let title = Line::from("Branch Packets").bold().centered();
        let block = Block::new()
            .border_set(TOP_BORDER_SET)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title);
        Table::new(rows, get_fu_col_widths()).block(block)
    }

    fn get_store_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let mut rows = Vec::new();
        for i in 0..self.num_stores {
            let packet_base = format!("{}.store_output_packets[{i}]", self.base);
            rows.push(parse_fu_output_packet(&packet_base, snapshots));
        }

        let title = Line::from("Store Packets").bold().centered();
        let block = Block::new()
            .border_set(TOP_BORDER_SET)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title);
        Table::new(rows, get_fu_col_widths()).block(block)
    }

    fn get_load_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let packet_base = format!("{}.load_output_packet", self.base);
        let row = parse_fu_output_packet(&packet_base, snapshots);

        let title = Line::from("Load Packet").bold().centered();
        let block = Block::bordered().border_set(TOP_BORDER_SET).title(title);
        Table::new([row], get_fu_col_widths()).block(block)
    }
}

impl StatefulWidget for FU {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let title = Line::from("Functional Units").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        let areas: [Rect; 4] = Layout::vertical([
            Constraint::Length((1 + 1 + self.num_alus) as u16),
            Constraint::Length((1 + self.num_mults) as u16),
            // Constraint::Length((1 + self.num_branches) as u16),
            Constraint::Length((1 + self.num_stores) as u16),
            Constraint::Length((2 + 1) as u16),
        ])
        .areas(inner_area);

        Widget::render(self.get_alu_table(snapshots), areas[0], buf);
        Widget::render(self.get_mult_table(snapshots), areas[1], buf);
        // Widget::render(self.get_branch_table(snapshots), areas[2], buf);
        Widget::render(self.get_store_table(snapshots), areas[2], buf);
        Widget::render(self.get_load_table(snapshots), areas[3], buf);
    }
}
