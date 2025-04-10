use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, StatefulWidget, Table, Widget},
};

use crate::{
    snapshots::Snapshots,
    utils::{Column, Columns, DisplayType, TOP_BORDER_SET},
};

const FU_INPUT_HEADERS: [Column; 8] = [
    Column {
        name: "rd",
        key: Some("rd"),
        width: 3,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "rob_num",
        key: Some("rob_num"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "rs1_val",
        key: Some("rs1_val"),
        width: 10,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "rs2_val",
        key: Some("rs2_val"),
        width: 10,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "bmask",
        key: Some("bmask"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "sq_tag",
        key: Some("store_queue_tag"),
        width: 6,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "mem_blocks",
        key: Some("mem_blocks"),
        width: 10,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "op",
        key: None,
        width: 20,
        display_type: DisplayType::Binary,
    },
];

#[derive(Clone, Debug)]
pub struct Issue {
    base: String,
    num_alus: usize,
    num_mults: usize,
    num_branches: usize,
    num_stores: usize,
}

impl Issue {
    pub fn new(base: &str, snapshots: &Snapshots) -> Option<Self> {
        // check that this is a btb
        snapshots.get_var(&format!("{base}.dbg_this_is_issue"))?;

        let mut num_alus = 0;
        let mut num_mults = 0;
        let mut num_branches = 0;
        let mut num_stores = 0;

        while snapshots
            .get_scope(&format!("{base}.alu_packets[{num_alus}]"))
            .is_some()
        {
            num_alus += 1;
        }

        while snapshots
            .get_scope(&format!("{base}.mult_packets[{num_mults}]"))
            .is_some()
        {
            num_mults += 1;
        }

        while snapshots
            .get_scope(&format!("{base}.branch_packets[{num_branches}]"))
            .is_some()
        {
            num_branches += 1;
        }

        while snapshots
            .get_scope(&format!("{base}.store_packets[{num_stores}]"))
            .is_some()
        {
            num_stores += 1;
        }

        Some(Self {
            base: base.to_owned(),
            num_alus,
            num_mults,
            num_branches,
            num_stores,
        })
    }

    fn get_alu_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let columns = Columns::new(FU_INPUT_HEADERS.to_vec());

        let bases = (0..self.num_alus)
            .map(|i| format!("{}.alu_packets[{i}]", self.base))
            .collect();
        let table = columns.create_table(bases, snapshots);

        let title = Line::from("ALU Packets").bold().centered();
        let block = Block::new()
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title(title);
        table.block(block)
    }

    fn get_mult_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let columns = Columns::new(FU_INPUT_HEADERS.to_vec());

        let bases = (0..self.num_mults)
            .map(|i| format!("{}.mult_packets[{i}]", self.base))
            .collect();
        let table = columns.create_table_no_header(bases, snapshots);

        let title = Line::from("Mult Packets").bold().centered();
        let block = Block::new()
            .border_set(TOP_BORDER_SET)
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title(title);
        table.block(block)
    }

    fn get_store_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let columns = Columns::new(FU_INPUT_HEADERS.to_vec());

        let bases = (0..self.num_stores)
            .map(|i| format!("{}.store_packets[{i}]", self.base))
            .collect();
        let table = columns.create_table_no_header(bases, snapshots);

        let title = Line::from("Store Packets").bold().centered();
        let block = Block::new()
            .border_set(TOP_BORDER_SET)
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title(title);
        table.block(block)
    }

    fn get_load_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let columns = Columns::new(FU_INPUT_HEADERS.to_vec());

        let bases = vec![format!("{}.load_packet", self.base)];
        let table = columns.create_table_no_header(bases, snapshots);

        let title = Line::from("Load Packets").bold().centered();
        let block = Block::new()
            .border_set(TOP_BORDER_SET)
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title(title);
        table.block(block)
    }

    fn get_branch_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let columns = Columns::new(FU_INPUT_HEADERS.to_vec());

        let bases = (0..self.num_branches)
            .map(|i| format!("{}.branch_packets[{i}]", self.base))
            .collect();
        let table = columns.create_table_no_header(bases, snapshots);

        let title = Line::from("Branch Packets").bold().centered();
        let block = Block::new()
            .border_set(TOP_BORDER_SET)
            .borders(Borders::all().difference(Borders::BOTTOM))
            .title(title);
        table.block(block)
    }

    fn get_stalling_branch_table<'a>(&self, snapshots: &'a Snapshots) -> Table<'a> {
        let columns = Columns::new(FU_INPUT_HEADERS.to_vec());

        let bases = (0..self.num_branches)
            .map(|i| format!("{}.stalling_branch_packets[{i}]", self.base))
            .collect();
        let table = columns.create_table_no_header(bases, snapshots);

        let title = Line::from("Stalling Branch Packets").bold().centered();
        let block = Block::bordered().border_set(TOP_BORDER_SET).title(title);
        table.block(block)
    }
}

impl StatefulWidget for Issue {
    type State = Snapshots;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        snapshots: &mut Self::State,
    ) {
        let title = Line::from("Issue").bold().centered();
        let block = Block::bordered().title(title);
        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        let areas: [Rect; 6] = Layout::vertical([
            Constraint::Length((1 + 1 + self.num_alus) as u16),
            Constraint::Length((1 + self.num_mults) as u16),
            Constraint::Length((1 + self.num_stores) as u16),
            Constraint::Length((1 + 1) as u16),
            Constraint::Length((1 + self.num_branches) as u16),
            Constraint::Length((1 + 1 + self.num_branches) as u16),
        ])
        .areas(inner_area);

        Widget::render(self.get_alu_table(snapshots), areas[0], buf);
        Widget::render(self.get_mult_table(snapshots), areas[1], buf);
        Widget::render(self.get_store_table(snapshots), areas[2], buf);
        Widget::render(self.get_load_table(snapshots), areas[3], buf);
        Widget::render(self.get_branch_table(snapshots), areas[4], buf);
        Widget::render(self.get_stalling_branch_table(snapshots), areas[5], buf);
    }
}
