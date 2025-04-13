use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::{
    headers::{FU_OUTPUT_HEADERS, MEM_INPUT_HEADERS},
    snapshots::Snapshots,
    utils::{parse_mem_command, parse_mem_size, parse_mem_state, Columns, TOP_BORDER_SET},
};

#[derive(Clone, Debug)]
pub struct MemUnit {
    base: String,
}

impl MemUnit {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is a memunit
        snapshots.get_var(&format!("{base}.dbg_this_is_memunit"))?;

        Some(Self {
            base: base.to_owned(),
        })
    }

    fn get_state(&self, snapshots: &Snapshots) -> Line {
        let state = parse_mem_state(
            snapshots
                .get_var(&format!("{}.mem_state", self.base))
                .unwrap(),
        );
        let next_state = parse_mem_state(
            snapshots
                .get_var(&format!("{}.next_mem_state", self.base))
                .unwrap(),
        );

        let parts = vec![
            "State: ".blue().bold(),
            state.magenta(),
            " -> ".into(),
            next_state.magenta().dim(),
        ];

        Line::from(parts)
    }

    fn get_cache_command(&self, snapshots: &Snapshots) -> Line {
        let command_key = format!("{}.cache_query_command", self.base);
        let command = snapshots.get_var(&command_key).unwrap();
        let command_string = parse_mem_command(command);

        let addr_key = format!("{}.cache_query_addr", self.base);
        let addr = snapshots.get_var(&addr_key).unwrap().as_hex();

        let size_key = format!("{}.cache_query_size", self.base);
        let size = snapshots.get_var(&size_key).unwrap();
        let size_string = parse_mem_size(size);

        let data_key = format!("{}.cache_query_data.dbbl_level", self.base);
        let data = snapshots.get_var(&data_key).unwrap().as_hex();

        let mut parts = vec![
            "Cache Command: ".blue().bold(),
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

    fn get_cache_response(&self, snapshots: &Snapshots) -> Line {
        let data = snapshots
            .get_var(&format!("{}.cache_resp_data", self.base))
            .unwrap()
            .as_hex();
        let valid = snapshots
            .get_var(&format!("{}.cache_resp_valid", self.base))
            .unwrap()
            .is_high();
        let ready = snapshots
            .get_var(&format!("{}.cache_query_ready", self.base))
            .unwrap();

        let parts = vec![
            "Cache Response: ".blue().bold(),
            if valid {
                data.magenta()
            } else {
                data.magenta().dim()
            },
            ", ready: ".into(),
            format!("{}", ready).magenta(),
        ];

        Line::from(parts)
    }
}

impl StatefulWidget for MemUnit {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let title = Line::from("Memory Unit").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        let lines = vec![
            self.get_state(snapshots),
            self.get_cache_command(snapshots),
            self.get_cache_response(snapshots),
            // self.get_outputs(snapshots),
        ];

        let columns = Columns::new(MEM_INPUT_HEADERS.to_vec());

        let bases = vec![format!("{}.stored_packet", self.base)];
        let block = Block::new()
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title("stored_packet");
        let stored_packet = columns.create_table(bases, snapshots).block(block);

        let bases = vec![format!("{}.next_stored_packet", self.base)];
        let block = Block::bordered()
            .border_set(TOP_BORDER_SET)
            .title("next_stored_packet");
        let next_stored_packet = columns
            .create_table_no_header(bases, snapshots)
            .block(block);

        let columns = Columns::new(FU_OUTPUT_HEADERS.to_vec());

        let bases = vec![format!("{}.output_packet", self.base)];
        let block = Block::new()
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title("output_packet");
        let output_packet = columns.create_table(bases, snapshots).block(block);

        let bases = vec![format!("{}.next_output_packet", self.base)];
        let block = Block::bordered()
            .border_set(TOP_BORDER_SET)
            .title("next_output_packet");
        let next_output_packet = columns
            .create_table_no_header(bases, snapshots)
            .block(block);

        let areas: [Rect; 6] = Layout::vertical([
            Constraint::Length(lines.len() as u16),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(inner_area);

        Widget::render(Paragraph::new(lines), areas[0], buf);
        Widget::render(stored_packet, areas[1], buf);
        Widget::render(next_stored_packet, areas[2], buf);
        Widget::render(output_packet, areas[4], buf);
        Widget::render(next_output_packet, areas[5], buf);
    }
}
