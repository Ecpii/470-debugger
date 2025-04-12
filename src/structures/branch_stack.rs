use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::snapshots::Snapshots;

use super::map_table::MapTable;

#[derive(Clone)]
pub struct BranchStack {
    base: String,
    map_table: MapTable,
}

impl BranchStack {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        snapshots.get_var(&format!("{base}.dbg_this_is_bstack"))?;

        Some(Self {
            base: base.to_owned(),
            map_table: MapTable::new(&format!("{base}.active_checkpoint.map_table"), snapshots),
        })
    }

    fn get_masks(&self, snapshots: &Snapshots) -> Line {
        let bmask_clear = snapshots
            .get_var(&format!("{}.bmask_clear", self.base))
            .unwrap()
            .as_binary();

        let bmask_squash = snapshots
            .get_var(&format!("{}.bmask_squash", self.base))
            .unwrap()
            .as_binary();

        Line::from(vec![
            "bmask_clear: ".into(),
            bmask_clear.magenta(),
            " bmask_squash: ".into(),
            bmask_squash.magenta(),
        ])
        .centered()
    }
}

impl StatefulWidget for BranchStack {
    type State = Snapshots;

    fn render(self, area: Rect, buf: &mut Buffer, snapshots: &mut Self::State) {
        let title = Line::from("Branch Stack").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);

        let masks = self.get_masks(snapshots);

        let [top_area, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner_area);

        Widget::render(block, area, buf);
        Widget::render(masks, top_area, buf);
        StatefulWidget::render(self.map_table, rest, buf, snapshots);
    }
}
