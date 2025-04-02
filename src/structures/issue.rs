use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::{snapshots::Snapshots, trace_dbg};

// true if we can use the raw name as the key to index
const HEADERS: [(&str, bool); 7] = [
    ("rs1_val", true),
    ("rs2_val", true),
    ("rd", true),
    ("bmask", true),
    ("rob_num", true),
    ("store_queue_tag", true),
    ("op", false),
];

#[derive(Clone, Debug)]
pub struct Issue {
    base: String,
    num_alus: usize,
    num_mults: usize,
    num_branches: usize,
}

impl Issue {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is a btb
        snapshots.get_var(&format!("{base}.dbg_this_is_issue"))?;

        let mut num_alus = 0;
        let mut num_mults = 0;
        let mut num_branches = 0;

        let mut entry_name = format!("{base}.alu_packets[{num_alus}]");
        while snapshots.get_scope(&entry_name).is_some() {
            num_alus += 1;
            entry_name = format!("{base}.alu_packets[{num_alus}]");
        }

        entry_name = format!("{base}.mult_packets[{num_mults}]");
        while snapshots.get_scope(&entry_name).is_some() {
            num_mults += 1;
            entry_name = format!("{base}.mult_packets[{num_mults}]");
        }

        entry_name = format!("{base}.branch_packets[{num_branches}]");
        while snapshots.get_scope(&entry_name).is_some() {
            num_branches += 1;
            entry_name = format!("{base}.branch_packets[{num_branches}]");
        }

        Some(Self {
            base: base.to_owned(),
            num_alus,
            num_mults,
            num_branches,
        })
    }

    fn parse_fu_input_packet(&self, base: &str, snapshots: &Snapshots) -> Row {
        let mut row_cells: Vec<Cell> = vec![];
        let is_valid = snapshots
            .get_var(&format!("{base}.valid"))
            .unwrap()
            .is_high();

        for (name, is_key) in HEADERS.iter() {
            let full_key = format!("{base}.{name}");
            let string = if *is_key {
                trace_dbg!(&full_key);
                let value = snapshots.get_var(&full_key).unwrap();

                // string that gets displayed in the cell section
                match *name {
                    "pc" | "target_pc" => value.as_hex(),
                    "rs1_val" | "rs2_val" | "rd" | "rob_num" => value.as_decimal(),
                    _ => {
                        format!("{}", value)
                    }
                }
            } else if *name == "op" {
                snapshots.render_opinfo(&full_key)
            } else {
                unreachable!()
            };

            row_cells.push(Cell::new(string));
        }

        let mut row = Row::new(row_cells);

        // formatting, colors
        if !is_valid {
            row = row.dim();
        }

        row.to_owned()
    }

    fn get_alu_table(&self, snapshots: &Snapshots) -> Table {
        let widths: Vec<u16> = vec![15; HEADERS.len()];
        let header = Row::new(HEADERS.map(|(x, _)| x)).bold().on_blue();

        let mut rows = Vec::new();
        for i in 0..self.num_alus {
            let alu_packet_base = format!("{}.alu_packets[{i}]", self.base);
            rows.push(self.parse_fu_input_packet(&alu_packet_base, snapshots));
        }

        let title = Line::from("ALU Packets").bold().centered();
        let block = Block::bordered().title(title);
        Table::new(rows, widths).header(header).block(block)
    }
}

impl StatefulWidget for Issue {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let table = self.get_alu_table(snapshots);

        let title = Line::from("Issue").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        Widget::render(table, inner_area, buf);
    }
}
