use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use crate::{
    snapshots::{Snapshots, VerilogValue},
    utils::{parse_fu_type, parse_opinfo},
};

// true if we can use the raw name as the key to index
const HEADERS: [(&str, bool); 10] = [
    ("#", false),
    ("dest_tag", true),
    ("rs1_tag", true),
    ("rs2_tag", true),
    ("bmask", true),
    ("fu", true),
    ("rob_num", true),
    ("sq_tag", false),
    ("mem_blocks", true),
    ("op", false),
];
const WIDTHS: [u16; 10] = [2, 8, 7, 7, 7, 6, 7, 6, 10, 20];

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
}

impl StatefulWidget for RSTable {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let header = Row::new(HEADERS.map(|(x, _)| x)).bold().on_blue();

        let mut rows = Vec::new();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];
            let mut is_valid = true;
            let row_base = format!("{}.entries[{i}]", self.base);

            for (name, is_key) in HEADERS.iter() {
                let string = if *is_key {
                    let full_key = format!("{row_base}.{name}");
                    let value = snapshots.get_var(&full_key).unwrap();

                    if *name == "fu" && (value.is_low() || value.is_unknown()) {
                        is_valid = false
                    }

                    // string that gets displayed in the cell section
                    match *name {
                        "rs1_tag" => {
                            let plus_key = format!("{row_base}.rs1_ready");
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
                        "fu" => parse_fu_type(value).to_owned(),
                        _ => {
                            format!("{}", value)
                        }
                    }
                } else if *name == "#" {
                    i.to_string()
                } else if *name == "op" {
                    let opinfo_base = format!("{row_base}.op");
                    parse_opinfo(&opinfo_base, snapshots)
                } else if *name == "sq_tag" {
                    let full_key = format!("{row_base}.store_queue_tag");
                    let value = snapshots.get_var(&full_key).unwrap();
                    value.as_decimal()
                } else {
                    unreachable!()
                };

                row_cells.push(Cell::new(string));
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
        let table = Table::new(rows, WIDTHS).header(header).block(block);
        Widget::render(table, area, buf);
    }
}
