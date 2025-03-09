use std::cmp::max;

use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::{
    snapshots::{Snapshots, VerilogValue},
    trace_dbg,
};

const KEYS: [&str; 6] = ["dest_tag", "rs1_tag", "rs2_tag", "bmask", "fu", "rob_num"];
const FU_TYPES: [&str; 5] = ["NOP", "IALU", "LD", "STR", "MULT"];

#[derive(Clone, Debug)]
pub struct RSTable {
    base: String,
    size: usize,
}

impl RSTable {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that dbg_this_is_rs exists so we know base is an rs
        snapshots.get_var(&format!("{base}.dbg_this_is_rs"))?;

        let mut i = 0;
        let mut entry_name = format!("{base}.entries[{i}]");

        while snapshots.get_scope(&entry_name).is_some() {
            i += 1;
            entry_name = format!("{base}.entries[{i}]");
        }

        Some(Self {
            base: base.to_owned(),
            size: i,
        })
    }

    fn format_fu(&self, value: &VerilogValue) -> String {
        if value.is_unknown() {
            return String::from("xxx");
        }

        let n = value.as_usize();
        String::from(FU_TYPES.get(n).copied().unwrap_or("<invalid>"))
    }
}

impl StatefulWidget for RSTable {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let header = Row::new(KEYS).bold().on_blue();
        let mut widths: Vec<u16> = KEYS.iter().map(|x| x.len() as u16).collect();

        let mut rows = Vec::new();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];
            let mut is_valid = true;
            let row_base = format!("{}.entries[{i}]", self.base);

            for (j, key) in KEYS.iter().enumerate() {
                let full_key = format!("{row_base}.{key}");
                trace_dbg!(&full_key);
                let value = snapshots.get_var(&full_key).unwrap();

                // string that gets displayed in the cell section
                let value_str = match *key {
                    "rs1_tag" => {
                        let plus_key = format!("{row_base}.rs1_ready");
                        trace_dbg!(&plus_key);
                        let plus = snapshots
                            .get_var(&plus_key)
                            .is_some_and(VerilogValue::is_high);

                        value.as_decimal() + if plus { "+" } else { "" }
                    }
                    "rs2_tag" => {
                        let plus_key = format!("{row_base}.rs2_ready");
                        let plus = snapshots
                            .get_var(&plus_key)
                            .is_some_and(VerilogValue::is_high);

                        value.as_decimal() + if plus { "+" } else { "" }
                    }
                    "dest_tag" | "rob_num" => value.as_decimal(),
                    "fu" => self.format_fu(value),
                    _ => {
                        format!("{}", value)
                    }
                };
                let width = value_str.len();

                row_cells.push(Cell::new(value_str));
                widths[j] = max(widths[j], width as u16);

                if *key == "fu" && (value.is_low() || value.is_unknown()) {
                    is_valid = false
                }
            }

            let mut row = Row::new(row_cells);

            // formatting, colors
            if is_valid {
                let rs1_ready = snapshots
                    .get_var(&format!("{row_base}.rs1_ready"))
                    .is_some_and(VerilogValue::is_high);
                let rs2_ready = snapshots
                    .get_var(&format!("{row_base}.rs2_ready"))
                    .is_some_and(VerilogValue::is_high);

                // row = row.not_dim();
                if rs1_ready && rs2_ready {
                    row = row.on_green();
                } else {
                    // row = row.on_light_green()
                }
            } else {
                row = row.dim();
            }

            rows.push(row)
        }

        let title = Line::from("Reservation Station").bold().centered();
        let block = Block::bordered().title(title);
        let table = Table::new(rows, widths).header(header).block(block);
        Widget::render(table, area, buf);
    }
}
