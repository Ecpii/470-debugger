use std::cmp::max;

use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Cell, Paragraph, Row, StatefulWidget, Table, Widget},
};

use crate::{
    snapshots::Snapshots,
    trace_dbg,
    utils::{parse_mem_command, parse_mem_size},
};

// true if we can use the raw name as the key to index
const HEADERS: [(&str, bool); 5] = [
    ("#", false),
    ("dirty", true),
    ("tag", true),
    ("lru", true),
    ("data", false),
];

const MSHR_HEADERS: [(&str, bool); 7] = [
    ("#", false),
    ("mem_tag", true),
    ("bmask", true),
    ("addr", false),
    ("size", true),
    ("is_store", true),
    ("store_data", true),
];

#[derive(Clone, Debug)]
pub struct DCache {
    base: String,
    pub size: usize,
    num_mshrs: usize,
}

impl DCache {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is a dcache
        snapshots.get_var(&format!("{base}.dbg_this_is_dcache"))?;

        let mut size = 0;
        let mut entry_name = format!("{base}.metadata[{size}]");

        while snapshots.get_scope(&entry_name).is_some() {
            size += 1;
            entry_name = format!("{base}.metadata[{size}]");
        }

        let mut num_mshrs = 0;
        let mut entry_name = format!("{base}.waiting_commands[{num_mshrs}]");

        while snapshots.get_scope(&entry_name).is_some() {
            num_mshrs += 1;
            entry_name = format!("{base}.waiting_commands[{num_mshrs}]");
        }

        Some(Self {
            base: base.to_owned(),
            size,
            num_mshrs,
        })
    }

    fn get_outputs(&self, snapshots: &Snapshots) -> Line {
        let valid = snapshots.get_var(&format!("{}.valid", self.base)).unwrap();
        let ready = snapshots.get_var(&format!("{}.ready", self.base)).unwrap();
        let data = snapshots
            .get_var(&format!("{}.data", self.base))
            .unwrap()
            .as_hex();

        let mut parts = vec!["Outputs:".blue().bold()];

        if ready.is_low() || ready.is_unknown() {
            parts.push(" ready: ".dim());
            parts.push(format!("{ready}").dim());
        } else {
            parts.push(" ready: ".into());
            parts.push(format!("{ready}").into());
        }

        if valid.is_low() || valid.is_unknown() {
            parts.push(" valid: ".dim());
            parts.push(format!("{valid}").dim());
            parts.push(" data: ".dim());
            parts.push(data.dim());
        } else {
            parts.push(" valid: ".into());
            parts.push(format!("{valid}").into());
            parts.push(" data: ".into());
            parts.push(data.into());
        }

        Line::from(parts)
    }

    fn get_memdp_ports(&self, snapshots: &Snapshots) -> Line {
        let read_enable = snapshots
            .get_var(&format!("{}.read_enable", self.base))
            .unwrap()
            .is_high();
        let read_index = snapshots
            .get_var(&format!("{}.read_index", self.base))
            .unwrap()
            .as_decimal();

        let write_enable = snapshots
            .get_var(&format!("{}.read_enable", self.base))
            .unwrap()
            .is_high();
        let write_index = snapshots
            .get_var(&format!("{}.write_index", self.base))
            .unwrap()
            .as_decimal();
        let write_data = snapshots
            .get_var(&format!("{}.write_data.dbbl_level", self.base))
            .unwrap()
            .as_hex();

        let mut parts = vec!["memDP ports: ".blue().bold()];

        if read_enable {
            parts.push("read at index ".into());
            parts.push(read_index.magenta());
        } else {
            parts.push("read at index ".dim());
            parts.push(read_index.magenta().dim());
        }

        parts.push(" | ".bold());

        if write_enable {
            parts.push("write at index ".into());
            parts.push(write_index.magenta());
            parts.push(" with data ".into());
            parts.push(write_data.magenta());
        } else {
            parts.push("write at index ".dim());
            parts.push(write_index.dim().magenta());
            parts.push(" with data ".dim());
            parts.push(write_data.dim().magenta());
        }

        Line::from(parts)
    }

    fn get_incoming_command(&self, snapshots: &Snapshots) -> Line {
        let command_key = format!("{}.query_command", self.base);
        let command = snapshots.get_var(&command_key).unwrap();

        let command_string = parse_mem_command(command);

        let addr_key = format!("{}.query_addr", self.base);
        let addr = snapshots.get_var(&addr_key).unwrap().as_hex();

        let size_key = format!("{}.query_size", self.base);
        let size = snapshots.get_var(&size_key).unwrap();

        let size_string = parse_mem_size(size);

        let data_key = format!("{}.query_data.dbbl_level", self.base);
        let data = snapshots.get_var(&data_key).unwrap().as_hex();

        let mut parts = vec![
            "Incoming Command: ".blue().bold(),
            command_string.magenta(),
            " of size ".into(),
            size_string.magenta(),
            " at ".into(),
            addr.magenta(),
            " with data ".into(),
            data.magenta(),
        ];

        if command.is_low() || command.is_unknown() {
            #[allow(clippy::needless_range_loop)]
            for i in 1..parts.len() {
                parts[i] = parts[i].clone().dim();
            }
        }
        Line::from(parts)
    }

    fn get_mem_command(&self, snapshots: &Snapshots) -> Line {
        let command_key = format!("{}.mem_command", self.base);
        let command = snapshots.get_var(&command_key).unwrap();

        let command_string = parse_mem_command(command);

        let addr_key = format!("{}.mem_addr", self.base);
        let addr = snapshots.get_var(&addr_key).unwrap().as_hex();

        let data_key = format!("{}.mem_command_data.dbbl_level", self.base);
        let data = snapshots.get_var(&data_key).unwrap().as_hex();

        let mut parts = vec![
            "Memory Command: ".blue().bold(),
            command_string.magenta(),
            " at ".into(),
            addr.magenta(),
            " with data ".into(),
            data.magenta(),
        ];

        if command.is_low() || command.is_unknown() {
            #[allow(clippy::needless_range_loop)]
            for i in 1..parts.len() {
                parts[i] = parts[i].clone().dim();
            }
        }
        Line::from(parts)
    }

    fn get_mshr_table(&self, snapshots: &Snapshots) -> Table {
        let mut widths: Vec<u16> = MSHR_HEADERS.iter().map(|(x, _)| x.len() as u16).collect();
        let header = Row::new(MSHR_HEADERS.map(|(x, _)| x)).bold().on_blue();

        let mut rows = Vec::new();

        for i in 0..self.num_mshrs {
            let mut row_cells: Vec<Cell> = vec![];
            let row_base = format!("{}.waiting_commands[{i}]", self.base);
            let is_valid = snapshots
                .get_var(&format!("{row_base}.valid"))
                .unwrap()
                .is_high();

            for (j, (name, is_key)) in MSHR_HEADERS.iter().enumerate() {
                let string = if *is_key {
                    let full_key = format!("{row_base}.{name}");
                    trace_dbg!(&full_key);
                    let value = snapshots.get_var(&full_key).unwrap();

                    // string that gets displayed in the cell section
                    match *name {
                        "mem_tag" => value.as_decimal(),
                        "store_data" => value.as_hex(),
                        "size" => parse_mem_size(value).to_string(),
                        _ => format!("{}", value),
                    }
                } else if *name == "#" {
                    i.to_string()
                } else if *name == "addr" {
                    let tag = snapshots.get_var(&format!("{row_base}.addr.tag")).unwrap();
                    let block_num = snapshots
                        .get_var(&format!("{row_base}.addr.block_num"))
                        .unwrap();
                    let offset = snapshots
                        .get_var(&format!("{row_base}.addr.block_offset"))
                        .unwrap();

                    let addr = &(tag + block_num) + offset;

                    addr.as_hex()
                } else {
                    unreachable!()
                };

                let width = string.len();
                widths[j] = max(widths[j], width as u16);
                row_cells.push(Cell::new(string));
            }

            let mut row = Row::new(row_cells);

            // formatting, colors
            if !is_valid {
                row = row.dim();
            }

            rows.push(row)
        }

        Table::new(rows, widths).header(header)
    }

    fn get_table(&self, snapshots: &Snapshots) -> Table {
        let mut widths: Vec<u16> = HEADERS.iter().map(|(x, _)| x.len() as u16).collect();
        let header = Row::new(HEADERS.map(|(x, _)| x)).bold().on_blue();

        let mut rows = Vec::new();

        for i in 0..self.size {
            let mut row_cells: Vec<Cell> = vec![];
            let row_base = format!("{}.metadata[{i}]", self.base);
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
                    format!("{}", value)
                } else if *name == "data" {
                    let key = format!("{}.dcache_mem.memData[{i}]", self.base);
                    let value = snapshots.get_var(&key).unwrap().as_hex();
                    value
                } else if *name == "#" {
                    i.to_string()
                } else {
                    unreachable!()
                };

                let width = string.len();
                widths[j] = max(widths[j], width as u16);
                row_cells.push(Cell::new(string));
            }

            let mut row = Row::new(row_cells);

            // formatting, colors
            if !is_valid {
                row = row.dim();
            }

            rows.push(row)
        }

        Table::new(rows, widths).header(header)
    }
}

impl StatefulWidget for DCache {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let title = Line::from("Data Cache").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        let lines = vec![
            self.get_incoming_command(snapshots),
            self.get_mem_command(snapshots),
            self.get_memdp_ports(snapshots),
            self.get_outputs(snapshots),
        ];

        let metadata_table = self
            .get_table(snapshots)
            .block(Block::bordered().title(Line::from("Metadata").bold().centered()));
        let mshr_table = self
            .get_mshr_table(snapshots)
            .block(Block::bordered().title(Line::from("MSHRs").bold().centered()));

        let [top, rest] =
            Layout::vertical([Constraint::Length(lines.len() as u16), Constraint::Fill(1)])
                .areas(inner_area);
        Widget::render(Paragraph::new(lines), top, buf);

        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(rest);
        Widget::render(metadata_table, left, buf);
        Widget::render(mshr_table, right, buf);
    }
}
