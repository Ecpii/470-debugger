use std::cmp::max;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::{snapshots::Snapshots, trace_dbg};

const KEYS: [&str; 4] = ["t", "t_old", "bmask", "retire_rdy"];

#[derive(Clone)]
pub struct ROBTable {
    base: String,
    size: usize,
}

fn matches_rob_entry(base: &str, snapshots: &Snapshots) -> bool {
    KEYS.iter()
        .all(|key| snapshots.get_var(&format!("{base}.{key}")).is_some())
}

impl ROBTable {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        let mut i = 0;
        let mut name = format!("{base}.entries[{i}]");

        if !(matches_rob_entry(&name, snapshots)) {
            return None;
        }

        while snapshots.get_scope(&name).is_some() {
            i += 1;
            name = format!("{base}.entries[{i}]");
        }

        Some(Self {
            base: base.to_owned(),
            size: i,
        })
    }
}

impl StatefulWidget for ROBTable {
    type State = Snapshots;

    fn render(self, area: Rect, buf: &mut Buffer, snapshots: &mut Self::State) {
        let header = Row::new(KEYS).bold().on_blue();
        let mut widths: Vec<u16> = KEYS.iter().map(|x| x.len() as u16).collect();

        let mut rows = Vec::new();

        let head_index = snapshots
            .get_var(&format!("{}.head", self.base))
            .unwrap()
            .as_usize();
        let tail_index = snapshots
            .get_var(&format!("{}.tail", self.base))
            .unwrap()
            .as_usize();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];
            let row_base = format!("{}.entries[{i}]", self.base);

            for (j, key) in KEYS.iter().enumerate() {
                let full_key = format!("{row_base}.{key}");
                trace_dbg!(&full_key);
                let value = snapshots.get_var(&full_key).unwrap();

                // string that gets displayed in the cell section
                let value_str = match *key {
                    "t" | "t_old" => value.as_decimal(),
                    _ => {
                        format!("{}", value)
                    }
                };
                let width = value_str.len();

                row_cells.push(Cell::new(value_str));
                widths[j] = max(widths[j], width as u16);
            }

            let mut row = Row::new(row_cells);

            if i == head_index {
                row = row.on_green()
            } else if i == tail_index {
                row = row.on_red()
            } else if (tail_index > head_index && head_index < i && i < tail_index)
                || (tail_index < head_index && !(tail_index < i && i < head_index))
            {
                row = row.on_yellow()
            }

            rows.push(row)
        }

        let title = Line::from("Reorder Buffer").bold().centered();
        let block = Block::bordered().title(title);
        let table = Table::new(rows, widths).header(header).block(block);
        Widget::render(table, area, buf);
    }
}
