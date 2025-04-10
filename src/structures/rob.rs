use std::cmp::max;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::{snapshots::Snapshots, utils::parse_opinfo};

// true if we can use the raw name as the key to index
const HEADERS: [(&str, bool); 7] = [
    ("#", false),
    ("h/t", false),
    ("t", true),
    ("t_old", true),
    ("bmask", true),
    ("retire_rdy", true),
    ("op", false),
];

#[derive(Clone)]
pub struct ROBTable {
    base: String,
    size: usize,
}

impl ROBTable {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that dbg_this_is_rob exists so we know base is a rob
        snapshots.get_var(&format!("{base}.dbg_this_is_rob"))?;

        let mut i = 0;
        let mut name = format!("{base}.entries[{i}]");

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
        let mut widths: Vec<u16> = HEADERS.iter().map(|(x, _)| x.len() as u16).collect();
        let header = Row::new(HEADERS.map(|(x, _)| x)).bold().on_blue();

        let mut rows = Vec::new();

        let head_index = snapshots
            .get_var(&format!("{}.head", self.base))
            .unwrap()
            .as_usize();
        let tail_index = snapshots
            .get_var(&format!("{}.tail", self.base))
            .unwrap()
            .as_usize();
        let rob_size = snapshots
            .get_var(&format!("{}.size", self.base))
            .unwrap()
            .as_usize();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];
            let row_base = format!("{}.entries[{i}]", self.base);

            for (j, (name, is_key)) in HEADERS.iter().enumerate() {
                if *name == "h/t" {
                    if i == head_index {
                        if i == tail_index {
                            row_cells.push(Cell::new(Text::from("h|t").centered()).bold())
                        } else {
                            row_cells.push(Cell::new(Text::from(" h ").centered()).bold())
                        }
                    } else if i == tail_index {
                        row_cells.push(Cell::new(Text::from(" t ").centered()).bold())
                    } else {
                        row_cells.push(Cell::new(""))
                    }
                    continue;
                }

                let string = if *is_key {
                    let full_key = format!("{row_base}.{name}");
                    let value = snapshots.get_var(&full_key).unwrap();

                    match *name {
                        "t" | "t_old" => value.as_decimal(),
                        _ => {
                            format!("{}", value)
                        }
                    }
                } else if *name == "#" {
                    i.to_string()
                } else if *name == "op" {
                    let opinfo_base = format!("{row_base}.info");
                    parse_opinfo(&opinfo_base, snapshots)
                } else {
                    unreachable!();
                };

                let width = string.len();
                row_cells.push(Cell::new(string));
                widths[j] = max(widths[j], width as u16);
            }

            let mut row = Row::new(row_cells);

            if i == head_index {
                if i == tail_index {
                    row = row.on_light_magenta()
                } else {
                    row = row.on_green()
                }
            } else if i == tail_index {
                row = row.on_red()
            } else if (tail_index > head_index && head_index < i && i < tail_index)
                || (tail_index < head_index && !(tail_index < i && i < head_index))
                || (head_index == tail_index && rob_size > 0)
            // full case
            {
                row = row.on_yellow()
            } else {
                row = row.dim()
            }

            rows.push(row)
        }

        let title = Line::from("Reorder Buffer").bold().centered();
        let block = Block::bordered().title(title);
        let table = Table::new(rows, widths).header(header).block(block);
        Widget::render(table, area, buf);
    }
}
