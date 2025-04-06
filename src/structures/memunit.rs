use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::{
    snapshots::Snapshots,
    utils::{
        parse_fu_output_packets, parse_mem_command, parse_mem_input_packets, parse_mem_size,
        parse_mem_state, TOP_BORDER_SET,
    },
};

#[derive(Clone, Debug)]
pub struct MemUnit {
    base: String,
}

impl MemUnit {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is a dcache
        snapshots.get_var(&format!("{base}.dbg_this_is_memunit"))?;

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

        let stored_packet =
            parse_mem_input_packets(&[&format!("{}.stored_packet", self.base)], snapshots).block(
                Block::new()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .title("stored_packet"),
            );
        let next_stored_packet =
            parse_mem_input_packets(&[&format!("{}.next_stored_packet", self.base)], snapshots)
                .block(
                    Block::new()
                        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                        .border_set(TOP_BORDER_SET)
                        .title("next_stored_packet"),
                );
        let output_packet =
            parse_fu_output_packets(&[&format!("{}.output_packet", self.base)], snapshots).block(
                Block::new()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_set(TOP_BORDER_SET)
                    .title("output_packet"),
            );
        let next_output_packet =
            parse_fu_output_packets(&[&format!("{}.next_output_packet", self.base)], snapshots)
                .block(
                    Block::bordered()
                        .border_set(TOP_BORDER_SET)
                        .title("next_output_packet"),
                );

        let areas: [Rect; 5] = Layout::vertical([
            Constraint::Length(lines.len() as u16),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(4),
        ])
        .areas(inner_area);

        Widget::render(Paragraph::new(lines), areas[0], buf);
        Widget::render(stored_packet, areas[1], buf);
        Widget::render(next_stored_packet, areas[2], buf);
        Widget::render(output_packet, areas[3], buf);
        Widget::render(next_output_packet, areas[4], buf);
    }
}
