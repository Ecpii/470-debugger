use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::snapshots::{Snapshots, VerilogValue};

const HEADERS: [&str; 3] = ["arch_reg", "phys_reg", "data"];
const WIDTHS: [u16; 3] = [8, 8, 10];

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
        let header = Row::new(HEADERS).bold().on_blue();

        let mut rows = Vec::new();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];
            let row_base = format!("{}[{i}]", self.base);

            let mut ready = false;

            for name in HEADERS.iter() {
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
                    "data" => {
                        let reg_idx_key = format!("{row_base}.value");
                        let reg_idx = snapshots.get_var(&reg_idx_key).unwrap().as_usize();

                        let tb_path = snapshots.get_base();
                        // fixme: hardcoded as hell
                        let data_key =
                            format!("{tb_path}.o3o.regfile_module.regfile_mem.memData[{reg_idx}]");

                        snapshots
                            .get_var(&data_key)
                            .map(VerilogValue::as_hex)
                            .unwrap_or(String::from("couldn't get data!"))
                    }
                    _ => {
                        unreachable!()
                    }
                };

                row_cells.push(Cell::new(string));
            }

            let mut row = Row::new(row_cells);
            if ready {
                row = row.on_green();
            }
            rows.push(row)
        }

        let title = Line::from("Map Table").bold().centered();
        let block = Block::bordered().title(title);
        let table = Table::new(rows, WIDTHS).header(header).block(block);
        Widget::render(table, area, buf);
    }
}
