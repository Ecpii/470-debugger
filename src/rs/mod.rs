use std::cmp::max;

use ratatui::{
    style::Stylize,
    widgets::{Cell, Row, StatefulWidget, Table, Widget},
};
use vcd::Value;

use crate::{
    snapshots::{Snapshots, VerilogValue},
    trace_dbg,
};

const RS_SZ: usize = 8;
const BASE: &str = "res_station_tb.DUT";
const KEYS: [&str; 6] = ["dest_tag", "rs1_tag", "rs2_tag", "bmask", "fu", "rob_num"];

// struct Column {
//   key: String,
//   width: u16,
//   format_fun: impl Fn -> String
// }

pub struct RSTable {}

impl RSTable {
    pub fn new() -> Self {
        Self {}
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
        let header = Row::new(KEYS.iter().map(|&x| x.to_string() + ":"));
        let mut widths: Vec<u16> = KEYS.iter().map(|x| x.len() as u16).collect();

        let mut rows = Vec::new();

        for i in 0..RS_SZ {
            let mut row_cells: Vec<Cell> = vec![];
            let mut is_valid = true;
            let row_base = format!("{BASE}.entries[{i}]");
            for (j, key) in KEYS.iter().enumerate() {
                let full_key = format!("{row_base}.{key}");
                trace_dbg!(&full_key);
                let value = snapshots.get_var(&full_key).unwrap();

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

        let table = Table::new(rows, widths).header(header);
        Widget::render(table, area, buf);
    }
}

impl Default for RSTable {
    fn default() -> Self {
        Self::new()
    }
}
