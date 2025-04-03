use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, Widget},
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

    fn get_rows(&self, snapshots: &Snapshots, range: Range<usize>) -> Vec<Row> {
        let mut rows = Vec::new();

        for i in range {
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

        rows
    }
}

impl StatefulWidget for RegFile {
    type State = Snapshots;

    fn render(self, area: Rect, buf: &mut Buffer, snapshots: &mut Self::State) {
        let header = Row::new(HEADERS.map(|(x, _)| x)).bold().on_blue();

        let first_half_rows = self.get_rows(snapshots, 0..self.size / 2);
        let second_half_rows = self.get_rows(snapshots, self.size / 2..self.size);

        let first_table = Table::new(first_half_rows, WIDTHS)
            .header(header.clone())
            .block(Block::new().borders(Borders::RIGHT));
        let second_table = Table::new(second_half_rows, WIDTHS).header(header);

        let title = Line::from("Register File").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(inner_area);
        Widget::render(first_table, left_area, buf);
        Widget::render(second_table, right_area, buf);
    }
}
