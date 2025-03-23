use std::cmp::max;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::{snapshots::Snapshots, trace_dbg};

const HEADERS: [(&str, bool); 2] = [("arch_reg", false), ("phys_reg", false)];

#[derive(Clone)]
pub struct MapTable {
    base: String,
    size: usize,
}

impl MapTable {
    pub fn new(base: &str, snapshots: &Snapshots) -> Self {
        let mut i = 0;
        let mut name = format!("{base}[{i}]");

        while snapshots.get_scope(&name).is_some() {
            i += 1;
            name = format!("{base}[{i}]");
        }

        Self {
            base: base.to_owned(),
            size: i,
        }
    }
}

impl StatefulWidget for MapTable {
    type State = Snapshots;

    fn render(self, area: Rect, buf: &mut Buffer, snapshots: &mut Self::State) {
        let mut widths: Vec<u16> = HEADERS.iter().map(|(x, _)| x.len() as u16).collect();
        let header = Row::new(HEADERS.map(|(x, _)| x)).bold().on_blue();

        let mut rows = Vec::new();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];
            let row_base = format!("{}[{i}]", self.base);

            let mut ready = false;

            for (j, (name, _)) in HEADERS.iter().enumerate() {
                let string: String = match *name {
                    "arch_reg" => i.to_string(),
                    "phys_reg" => {
                        let reg_idx_key = format!("{row_base}.value");
                        let reg_idx = snapshots.get_var(&reg_idx_key).unwrap();
                        let string = reg_idx.as_decimal();

                        let reg_ready_key = format!("{row_base}.ready");
                        let reg_ready = snapshots.get_var(&reg_ready_key).unwrap();
                        ready = reg_ready.is_high();

                        if ready {
                            string + "+"
                        } else {
                            string
                        }
                    }
                    _ => {
                        unreachable!()
                    }
                };

                let width = string.len();
                row_cells.push(Cell::new(string));
                widths[j] = max(widths[j], width as u16);
            }

            let mut row = Row::new(row_cells);
            if ready {
                row = row.on_green();
            }
            rows.push(row)
        }

        let title = Line::from("Map Table").bold().centered();
        let block = Block::bordered().title(title);
        let table = Table::new(rows, widths).header(header).block(block);
        Widget::render(table, area, buf);
    }
}
