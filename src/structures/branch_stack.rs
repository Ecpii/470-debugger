use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::snapshots::Snapshots;

use super::map_table::MapTable;

#[derive(Clone)]
pub struct BranchStack {
    _base: String,
    map_table: MapTable,
}

impl BranchStack {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        snapshots.get_var(&format!("{base}.dbg_this_is_bstack"))?;

        Some(Self {
            _base: base.to_owned(),
            map_table: MapTable::new(
                &format!("{base}.active_checkpoint.map_table"),
                snapshots,
            ),
        })
    }
}

impl StatefulWidget for BranchStack {
    type State = Snapshots;

    fn render(self, area: Rect, buf: &mut Buffer, snapshots: &mut Self::State) {
        let title = Line::from("Branch Stack").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);

        Widget::render(block, area, buf);
        StatefulWidget::render(self.map_table, inner_area, buf, snapshots);
    }
}
