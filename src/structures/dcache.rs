use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, StatefulWidget, Table, Widget},
};

use crate::{
    headers::{DCACHE_META_HEADERS, MSHR_HEADERS},
    snapshots::{Snapshots, VerilogValue},
    utils::{parse_mem_command, parse_mem_size, Columns, TOP_BORDER_SET},
};

#[derive(Clone, Debug)]
pub struct DCache {
    base: String,
    num_ways: usize,
    num_sets: usize,
    num_mshrs: usize,
}

impl DCache {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is a dcache
        snapshots.get_var(&format!("{base}.dbg_this_is_dcache"))?;

        let mut num_ways = 0;
        while snapshots
            .get_scope(&format!("{base}.metadata[0][{num_ways}]"))
            .is_some()
        {
            num_ways += 1;
        }

        let mut num_sets = 0;
        while snapshots
            .get_scope(&format!("{base}.metadata[{num_sets}][0]"))
            .is_some()
        {
            num_sets += 1;
        }

        let mut num_mshrs = 0;
        while snapshots
            .get_scope(&format!("{base}.waiting_commands[{num_mshrs}]"))
            .is_some()
        {
            num_mshrs += 1;
        }

        Some(Self {
            base: base.to_owned(),
            num_ways,
            num_sets,
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

    fn get_mshr_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let columns = Columns::new(MSHR_HEADERS.to_vec());

        let bases = (0..self.num_mshrs)
            .map(|i| format!("{}.waiting_commands[{i}]", self.base))
            .collect();
        let table = columns.create_table(bases, snapshots);

        let title = Line::from("MSHRs").bold().centered();
        let block = Block::new()
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title(title);
        table.block(block)
    }

    fn get_set_table(&self, set_num: usize, snapshots: &Snapshots) -> Table {
        let columns = Columns::new(DCACHE_META_HEADERS.to_vec());
        let widths = columns.get_widths();

        let mut rows = Vec::new();
        for set_index in 0..self.num_ways {
            let mut cells = Vec::<Cell>::new();
            let row_base = format!("{}.metadata[{set_num}][{set_index}]", self.base);
            let index = set_num * self.num_ways + set_index;
            let is_valid = snapshots
                .get_var(&format!("{row_base}.valid"))
                .unwrap()
                .is_high();

            for col in DCACHE_META_HEADERS {
                let string = if let Some(key) = col.key {
                    let full_key = format!("{row_base}.{key}");
                    let value = snapshots.get_var(&full_key).unwrap();

                    value.format(&col.display_type)
                } else {
                    match col.name {
                        "data" => {
                            let key = format!("{}.dcache_mem.memData[{index}]", self.base);
                            let value = snapshots.get_var(&key).unwrap().as_hex();
                            value
                        }
                        "addr" => {
                            let tag = snapshots.get_var(&format!("{row_base}.tag")).unwrap();
                            let set_num = VerilogValue::from_usize(set_num, self.num_sets);
                            let block_offset = VerilogValue::from_usize(0, 3);

                            let addr = tag + &(&set_num + &block_offset);

                            addr.as_hex()
                        }
                        "#" => set_index.to_string(),
                        _ => unreachable!(),
                    }
                };

                cells.push(Cell::new(string))
            }

            let mut row = Row::new(cells);

            if !is_valid {
                row = row.dim();
            }

            rows.push(row);
        }

        Table::new(rows, widths)
    }

    fn render_table(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &Snapshots,
    ) {
        let columns = Columns::new(DCACHE_META_HEADERS.to_vec());
        let header = columns.get_header();
        let widths = columns.get_widths();
        let mut constraints = vec![Constraint::Length(2)];

        let title = Line::from("Metadata").bold().centered();
        let top_block = Block::new()
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title(title);

        let header_table = Table::new([header], widths).block(top_block);
        let mut tables = Vec::with_capacity(self.num_sets);

        for set_num in 0..self.num_sets {
            let title = format!("Set {set_num}");

            let mut block = if set_num != self.num_sets - 1 {
                Block::new().borders(Borders::all().difference(Borders::BOTTOM))
            } else {
                Block::bordered()
            };
            block = block.border_set(TOP_BORDER_SET).title(title);

            tables.push(self.get_set_table(set_num, snapshots).block(block));
            constraints.push(Constraint::Length(self.num_ways as u16 + 1));
        }

        let areas = Layout::vertical(constraints).split(area);

        Widget::render(header_table, areas[0], buf);
        for (table, area) in tables.iter().zip(areas.iter().skip(1)) {
            Widget::render(table, *area, buf);
        }
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

        let mshr_table = self
            .get_mshr_table(snapshots)
            .block(Block::bordered().title(Line::from("MSHRs").bold().centered()));

        let [top, rest] =
            Layout::vertical([Constraint::Length(lines.len() as u16), Constraint::Fill(1)])
                .areas(inner_area);
        Widget::render(Paragraph::new(lines), top, buf);

        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(rest);
        self.render_table(left, buf, snapshots);
        Widget::render(mshr_table, right, buf);
    }
}
