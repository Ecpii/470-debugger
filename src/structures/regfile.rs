use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::snapshots::Snapshots;

const HEADERS: [(&str, bool); 2] = [("phys_reg", false), ("data", false)];
const WIDTHS: [u16; 2] = [8, 16];

#[derive(Clone)]
pub struct RegFile {
    base: String,
    size: usize,
}

impl RegFile {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        snapshots.get_var(&format!("{base}.dbg_this_is_regfile"))?;

        let mut size = 0;
        let mut name = format!("{base}.regfile_mem.memData[{size}]");

        while snapshots.get_var(&name).is_some() {
            size += 1;
            name = format!("{base}.regfile_mem.memData[{size}]");
        }

        Some(Self {
            base: base.to_owned(),
            size,
        })
    }
}

impl StatefulWidget for RegFile {
    type State = Snapshots;

    fn render(self, area: Rect, buf: &mut Buffer, snapshots: &mut Self::State) {
        let header = Row::new(HEADERS.map(|(x, _)| x)).bold().on_blue();

        let mut rows = Vec::new();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];

            for (name, is_key) in HEADERS.iter() {
                if *is_key {
                    unreachable!()
                } else {
                    let string = match *name {
                        "phys_reg" => i.to_string(),
                        "data" => snapshots
                            .get_var(&format!("{}.regfile_mem.memData[{i}]", self.base))
                            .unwrap()
                            .as_hex(),
                        _ => unreachable!(),
                    };
                    row_cells.push(Cell::new(string));
                }
            }

            let row = Row::new(row_cells);
            rows.push(row)
        }

        let title = Line::from("Register File").bold().centered();
        let block = Block::bordered().title(title);
        let table = Table::new(rows, WIDTHS).header(header).block(block);
        Widget::render(table, area, buf);
    }
}
