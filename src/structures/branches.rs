use std::cmp::max;

use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::{snapshots::Snapshots, trace_dbg};

// true if we can use the raw name as the key to index
const HEADERS: [(&str, bool); 2] = [("pc", true), ("target_pc", true)];

#[derive(Clone, Debug)]
pub struct Btb {
    base: String,
    pub size: usize,
}

impl Btb {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is a btb
        snapshots.get_var(&format!("{base}.dbg_this_is_btb"))?;

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
}

impl StatefulWidget for Btb {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let mut widths: Vec<u16> = HEADERS.iter().map(|(x, _)| x.len() as u16).collect();
        let header = Row::new(HEADERS.map(|(x, _)| x)).bold().on_blue();

        let mut rows = Vec::new();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];
            let row_base = format!("{}.entries[{i}]", self.base);
            let is_valid = snapshots
                .get_var(&format!("{row_base}.valid"))
                .unwrap()
                .is_high();

            for (j, (name, is_key)) in HEADERS.iter().enumerate() {
                let string = if *is_key {
                    let full_key = format!("{row_base}.{name}");
                    trace_dbg!(&full_key);
                    let value = snapshots.get_var(&full_key).unwrap();

                    // string that gets displayed in the cell section
                    match *name {
                        "pc" | "target_pc" => value.as_hex(),
                        _ => {
                            format!("{}", value)
                        }
                    }
                } else {
                    // if we add stuff to the btb that isn't an exact key
                    unreachable!()
                };

                let width = string.len();
                row_cells.push(Cell::new(string));
                widths[j] = max(widths[j], width as u16);
            }

            let mut row = Row::new(row_cells);

            // formatting, colors
            if !is_valid {
                row = row.dim();
            }

            rows.push(row)
        }

        let title = Line::from("Branch Target Buffer").bold().centered();
        let block = Block::bordered().title(title);
        let table = Table::new(rows, widths).header(header).block(block);
        Widget::render(table, area, buf);
    }
}
