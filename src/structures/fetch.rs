use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Row, StatefulWidget, Table, Widget},
};

use crate::{snapshots::Snapshots, utils::parse_inst};

#[derive(Clone, Debug)]
pub struct Fetch {
    base: String,
    n: usize,
}

impl Fetch {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is fetch
        snapshots.get_var(&format!("{base}.dbg_this_is_fetch"))?;

        let mut n = 0;
        let mut entry_name = format!("{base}.incoming_instrs[{n}]");

        while snapshots.get_scope(&entry_name).is_some() {
            n += 1;
            entry_name = format!("{base}.incoming_instrs[{n}]");
        }
        Some(Self {
            base: base.to_owned(),
            n,
        })
    }

    fn get_pc(&self, snapshots: &Snapshots) -> Line {
        let pc = snapshots
            .get_var(&format!("{}.PC", self.base))
            .unwrap()
            .as_hex();
        let next_pc = snapshots
            .get_var(&format!("{}.PC_n", self.base))
            .unwrap()
            .as_hex();

        let parts = vec![
            "PC: ".blue().bold(),
            pc.magenta(),
            " -> ".into(),
            next_pc.magenta().dim(),
        ];

        Line::from(parts)
    }

    fn get_inst_table(&self, snapshots: &Snapshots) -> Table {
        let header = Row::new(vec!["#", "parsed", "raw"]).bold().on_blue();
        let widths = vec![2, 25, 10];

        let mut rows = Vec::with_capacity(self.n);
        for i in 0..self.n {
            // let valid = snapshots
            //     .get_var(&format!("{}.incoming_instrs_vld[{i}]", self.base))
            //     .unwrap()
            //     .is_high();
            let valid = true; // todo: fix
            let row_base = format!("{}.incoming_instrs[{i}]", self.base);

            let mut cells = Vec::with_capacity(3);

            cells.push(i.to_string());
            cells.push(parse_inst(&row_base, snapshots));
            cells.push(
                snapshots
                    .get_var(&format!("{row_base}.inst"))
                    .unwrap()
                    .as_hex(),
            );

            let mut row = Row::new(cells);
            if !valid {
                row = row.dim();
            }

            rows.push(row);
        }

        let block = Block::bordered().title("incoming_instrs");

        Table::new(rows, widths).header(header).block(block)
    }
}

impl StatefulWidget for Fetch {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let title = Line::from("Fetch").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        let lines = vec![
            self.get_pc(snapshots),
            // self.get_outputs(snapshots),
        ];

        let inst_table = self.get_inst_table(snapshots);

        let areas: [Rect; 2] =
            Layout::vertical([Constraint::Length(lines.len() as u16), Constraint::Fill(1)])
                .areas(inner_area);

        Widget::render(Paragraph::new(lines), areas[0], buf);
        Widget::render(inst_table, areas[1], buf);
    }
}
