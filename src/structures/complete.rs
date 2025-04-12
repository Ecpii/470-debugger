use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, StatefulWidget, Table, Widget},
};

use crate::utils::{Columns, DisplayType, LEFT_BORDER_SET};
use crate::{snapshots::Snapshots, utils::Column};

#[derive(Clone)]
pub struct Complete {
    base: String,
    n: usize,
}

impl Complete {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        snapshots.get_var(&format!("{base}.dbg_this_is_complete"))?;

        let mut n = 0;

        while snapshots.get_scope(&format!("{base}.cdb[{n}]")).is_some() {
            n += 1;
        }

        Some(Self {
            base: base.to_owned(),
            n,
        })
    }

    fn get_cdb_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let cdb_headers = vec![
            Column {
                name: "rd",
                key: Some("dest_reg_idx"),
                width: 3,
                display_type: DisplayType::Decimal,
            },
            Column {
                name: "bmask",
                key: Some("bmask"),
                width: 7,
                display_type: DisplayType::Binary,
            },
        ];
        let cdb_columns = Columns::new(cdb_headers);

        let bases = (0..self.n)
            .map(|i| format!("{}.cdb[{i}]", self.base))
            .collect();

        cdb_columns.create_table(bases, snapshots)
    }

    fn get_cdb_etb_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let cdb_etb_headers = vec![
            Column {
                name: "rd",
                key: Some("dest_reg_idx"),
                width: 3,
                display_type: DisplayType::Decimal,
            },
            Column {
                name: "bmask",
                key: Some("bmask"),
                width: 7,
                display_type: DisplayType::Binary,
            },
            Column {
                name: "value",
                key: Some("value"),
                width: 10,
                display_type: DisplayType::Hex,
            },
        ];
        let cdb_etb_columns = Columns::new(cdb_etb_headers);

        let bases = (0..self.n)
            .map(|i| format!("{}.cdb_etb[{i}]", self.base))
            .collect();

        cdb_etb_columns.create_table(bases, snapshots)
    }
}

impl StatefulWidget for Complete {
    type State = Snapshots;

    fn render(self, area: Rect, buf: &mut Buffer, snapshots: &mut Self::State) {
        let title = Line::from("Complete").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);

        Widget::render(block, area, buf);

        let title = Line::from("CDB").bold().centered();
        let block = Block::new()
            .borders(Borders::all().difference(Borders::RIGHT))
            .title(title);
        let cdb_table = self.get_cdb_table(snapshots).block(block);

        let title = Line::from("CDB ETB").bold().centered();
        let block = Block::bordered().border_set(LEFT_BORDER_SET).title(title);
        let cdb_etb_table = self.get_cdb_etb_table(snapshots).block(block);

        let [top_area, _rest] = Layout::vertical([
            Constraint::Length((2 + self.n + 1).try_into().unwrap()),
            Constraint::Min(0),
        ])
        .areas(inner_area);

        let [top_left_area, top_right_area] =
            Layout::horizontal([Constraint::Length(12), Constraint::Min(0)]).areas(top_area);

        Widget::render(cdb_table, top_left_area, buf);
        Widget::render(cdb_etb_table, top_right_area, buf);
    }
}
