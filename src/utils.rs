use core::fmt;
use std::{
    fmt::{Display, Formatter},
    fs::{self},
    io::{Error, ErrorKind, Result},
};

use raki::{
    AOpcode, BaseIOpcode, COpcode, InstFormat, Instruction, OpcodeKind, PrivOpcode, ZifenceiOpcode,
};
use ratatui::{
    style::Stylize,
    symbols,
    widgets::{Cell, Row, Table},
};

use crate::{
    snapshots::{Snapshots, VerilogValue},
    trace_dbg,
};

#[derive(Clone, Copy)]
pub enum DisplayType {
    Binary,
    Decimal,
    Hex,
    Custom,
}
impl DisplayType {
    pub fn next(&self) -> Self {
        match self {
            DisplayType::Binary => DisplayType::Decimal,
            DisplayType::Decimal => DisplayType::Hex,
            DisplayType::Hex => DisplayType::Binary,
            DisplayType::Custom => {
                panic!("Shouldn't be able to transition to/from Custom DisplayType")
            }
        }
    }
}
impl Display for DisplayType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DisplayType::Binary => "Binary",
                DisplayType::Decimal => "Decimal",
                DisplayType::Hex => "Hex",
                DisplayType::Custom =>
                    panic!("Shouldn't be able to transition to/from Custom DisplayType"),
            }
        )
    }
}
impl TryFrom<&str> for DisplayType {
    type Error = std::io::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "Binary" => Ok(DisplayType::Binary),
            "Decimal" => Ok(DisplayType::Decimal),
            "Hex" => Ok(DisplayType::Hex),
            _ => Err(Error::new(ErrorKind::InvalidData, "Invalid display type!")),
        }
    }
}

pub fn save_watch_list(watch_list: &[(String, DisplayType)], name: &str) -> Result<()> {
    let mut write_data = String::new();

    for (key, disp_type) in watch_list {
        write_data.push_str(&format!("{key},{disp_type}\n"));
    }
    let filename = format!("debugger_files/{name}.dbg_list");

    fs::write(&filename, write_data)?;

    Ok(())
}

pub fn load_watch_list(name: &str) -> Result<Vec<(String, DisplayType)>> {
    let filename = format!("debugger_files/{name}.dbg_list");
    let data = fs::read_to_string(&filename)?;
    let lines = data.split("\n");

    let mut watch_list = Vec::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(",");

        let Some(key) = parts.next() else {
            return Err(Error::new(ErrorKind::InvalidData, "Empty line in data!"));
        };
        let Some(disp_type_str) = parts.next() else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Display type not found in line!",
            ));
        };
        let disp_type: DisplayType = disp_type_str.try_into()?;

        watch_list.push((key.to_string(), disp_type))
    }

    Ok(watch_list)
}

// new type to allow us to redefine display
#[allow(non_camel_case_types)]
pub struct o3oInst(pub Instruction);

// code copied from raki's impl
// https://docs.rs/raki/latest/src/raki/instruction.rs.html#46
impl Display for o3oInst {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.0.inst_format {
            InstFormat::RFormat | InstFormat::MFormat => {
                write!(
                    f,
                    "{} {}, {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rs1.unwrap()),
                    reg2str(self.0.rs2.unwrap())
                )
            }
            InstFormat::AFormat => match self.0.opc {
                OpcodeKind::A(AOpcode::LR_W | AOpcode::LR_D) => write!(
                    f,
                    "{} {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rs1.unwrap()),
                ),
                _ => write!(
                    f,
                    "{} {}, {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rs1.unwrap()),
                    reg2str(self.0.rs2.unwrap())
                ),
            },
            InstFormat::RShamtFormat => {
                write!(
                    f,
                    "{} {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rs1.unwrap()),
                )
            }
            InstFormat::ClFormat | InstFormat::ALrFormat | InstFormat::IFormat => write!(
                f,
                "{} {}, {}, {}",
                self.0.opc,
                reg2str(self.0.rd.unwrap()),
                reg2str(self.0.rs1.unwrap()),
                self.0.imm.unwrap()
            ),
            InstFormat::CsFormat | InstFormat::SFormat | InstFormat::BFormat => write!(
                f,
                "{} {}, {}({})",
                self.0.opc,
                reg2str(self.0.rs1.unwrap()),
                self.0.imm.unwrap(),
                reg2str(self.0.rs2.unwrap()),
            ),
            InstFormat::CiwFormat => {
                write!(
                    f,
                    "{} {}, sp, {:x}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    self.0.imm.unwrap()
                )
            }
            InstFormat::CssFormat => {
                write!(
                    f,
                    "{} {}, {}(sp)",
                    self.0.opc,
                    reg2str(self.0.rs2.unwrap()),
                    self.0.imm.unwrap()
                )
            }
            InstFormat::UFormat | InstFormat::JFormat => {
                write!(
                    f,
                    "{} {}, {:#x}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    self.0.imm.unwrap()
                )
            }
            InstFormat::CjFormat => {
                write!(f, "{} {}", self.0.opc, self.0.imm.unwrap())
            }
            InstFormat::CiFormat => {
                write!(
                    f,
                    "{} {}, {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rd.unwrap()),
                    self.0.imm.unwrap()
                )
            }
            InstFormat::CrFormat => match self.0.opc {
                OpcodeKind::C(COpcode::JR) => {
                    write!(
                        f,
                        "{} zero, 0({})",
                        self.0.opc,
                        reg2str(self.0.rs1.unwrap()),
                    )
                }
                OpcodeKind::C(COpcode::JALR) => {
                    write!(f, "{} ra, 0({})", self.0.opc, reg2str(self.0.rs1.unwrap()),)
                }
                OpcodeKind::C(COpcode::MV) => write!(
                    f,
                    "{} {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rs2.unwrap())
                ),
                OpcodeKind::C(COpcode::ADD) => write!(
                    f,
                    "{} {}, {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rs2.unwrap())
                ),
                _ => unreachable!(),
            },
            InstFormat::CaFormat => {
                write!(
                    f,
                    "{} {}, {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rd.unwrap()),
                    reg2str(self.0.rs2.unwrap())
                )
            }
            InstFormat::CbFormat => {
                write!(
                    f,
                    "{} {}, {}",
                    self.0.opc,
                    self.0.rs1.unwrap(),
                    self.0.imm.unwrap(),
                )
            }
            InstFormat::CsrFormat => {
                write!(
                    f,
                    "{} {}, {:#x}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    self.0.rs2.unwrap(),
                    reg2str(self.0.rs1.unwrap()),
                )
            }
            InstFormat::CsrUiFormat => {
                write!(
                    f,
                    "{} {}, {}, {}",
                    self.0.opc,
                    reg2str(self.0.rd.unwrap()),
                    self.0.rs2.unwrap(),
                    self.0.imm.unwrap(),
                )
            }
            InstFormat::OnlyRd => {
                write!(f, "{} {}", self.0.opc, reg2str(self.0.rd.unwrap()),)
            }
            InstFormat::OnlyRs1 => {
                write!(f, "{} {}", self.0.opc, reg2str(self.0.rs1.unwrap()),)
            }
            InstFormat::OnlyRs2 => {
                write!(f, "{} {}", self.0.opc, reg2str(self.0.rs2.unwrap()),)
            }
            InstFormat::NoOperand => match self.0.opc {
                OpcodeKind::BaseI(BaseIOpcode::ECALL | BaseIOpcode::EBREAK)
                | OpcodeKind::Zifencei(ZifenceiOpcode::FENCE)
                | OpcodeKind::C(COpcode::NOP | COpcode::EBREAK)
                | OpcodeKind::Priv(
                    PrivOpcode::MRET | PrivOpcode::SRET | PrivOpcode::WFI | PrivOpcode::SFENCE_VMA,
                ) => write!(f, "{}", self.0.opc),
                _ => unreachable!(),
            },
        }
    }
}

/// Convert register number to string.
fn reg2str(rd_value: usize) -> String {
    format!("x{rd_value}")
}

pub fn parse_mem_command(val: &VerilogValue) -> &'static str {
    if val.is_unknown() {
        return "xxxxx";
    }
    match val.as_usize() {
        0b00 => "MEM_NONE",
        0b01 => "MEM_LOAD",
        0b10 => "MEM_STORE",
        _ => "<invalid>",
    }
}

pub fn parse_mem_size(val: &VerilogValue) -> &'static str {
    if val.is_unknown() {
        return "xxxxx";
    }
    match val.as_usize() {
        0b00 => "BYTE",
        0b01 => "HALF",
        0b10 => "WORD",
        0b11 => "DOUBLE",
        _ => "<invalid>",
    }
}

pub fn parse_mem_state(val: &VerilogValue) -> &'static str {
    if val.is_unknown() {
        return "xxxxx";
    }
    match val.as_usize() {
        0b00 => "IDLE",
        0b01 => "DCACHE_PEDNING",
        0b10 => "LOAD_PENDING",
        0b11 => "STORE_PENDING",
        _ => "<invalid>",
    }
}

struct Column {
    name: &'static str,
    key: Option<&'static str>,
    width: u16,
    display_type: DisplayType,
}

/*
 logic valid;
 op_info_t op;
 bmask_t bmask;
 reg_idx_t rd;
 rob_num_t rob_num;
 store_queue_num_t store_queue_tag;
 logic [3:0] mem_blocks;
 DATA alu_result;
 DATA mem_data;
*/
const FU_OUTPUT_HEADERS: [Column; 8] = [
    Column {
        name: "rd",
        key: Some("rd"),
        width: 2,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "bmask",
        key: Some("bmask"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "rob_num",
        key: Some("rob_num"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "sq_tag",
        key: Some("store_queue_tag"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "mem_blocks",
        key: Some("mem_blocks"),
        width: 10,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "alu_result",
        key: Some("alu_result"),
        width: 16,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "mem_data",
        key: Some("mem_data"),
        width: 16,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "op",
        key: None,
        width: 20,
        display_type: DisplayType::Custom,
    },
];

pub fn get_fu_output_header() -> Row<'static> {
    Row::new(FU_OUTPUT_HEADERS.map(|col| col.name))
        .bold()
        .on_blue()
}

pub fn get_fu_col_widths() -> [u16; 8] {
    FU_OUTPUT_HEADERS.map(|col| col.width)
}

pub fn parse_fu_output_packets<'a>(bases: &[&str], snapshots: &'a Snapshots) -> Table<'a> {
    let header = get_fu_output_header();
    let widths = FU_OUTPUT_HEADERS.map(|col| col.width);

    let rows = bases
        .iter()
        .map(|base| parse_fu_output_packet(base, snapshots));

    Table::new(rows, widths).header(header)
}

pub fn parse_fu_output_packet<'a>(base: &str, snapshots: &'a Snapshots) -> Row<'a> {
    let mut cells = Vec::<Cell>::new();

    let is_valid = snapshots
        .get_var(&format!("{base}.valid"))
        .unwrap()
        .is_high();

    for col in FU_OUTPUT_HEADERS.iter() {
        if let Some(key) = col.key {
            let full_key = format!("{base}.{key}");
            let value = snapshots.get_var(&full_key).unwrap();

            let string = match col.display_type {
                DisplayType::Custom => {
                    unreachable!("No keyed columns have custom display type!")
                }
                display_type => value.format(&display_type),
            };

            cells.push(Cell::new(string));
        } else {
            assert!(col.name == "op");

            let string = snapshots.render_opinfo(&format!("{base}.op"));

            cells.push(Cell::new(string));
        }
    }
    let mut row = Row::new(cells);

    // formatting, colors
    if !is_valid {
        row = row.dim();
    }

    row
}

pub const TOP_BORDER_SET: symbols::border::Set = symbols::border::Set {
    top_left: symbols::line::NORMAL.vertical_right,
    top_right: symbols::line::NORMAL.vertical_left,
    ..symbols::border::PLAIN
};

/*
 DATA data;
 ADDR addr;
 op_info_t op;
 rob_num_t rob_num;
 bmask_t bmask;
 reg_idx_t rd;
 logic wr;
 logic valid;
 logic [3:0] mem_blocks;
 store_queue_num_t store_queue_tag;
*/
const MEM_INPUT_HEADERS: [Column; 9] = [
    Column {
        name: "rd",
        key: Some("rd"),
        width: 2,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "rob_num",
        key: Some("rob_num"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "wr",
        key: Some("wr"),
        width: 2,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "bmask",
        key: Some("bmask"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "data",
        key: Some("data"),
        width: 16,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "addr",
        key: Some("addr"),
        width: 8,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "sq_tag",
        key: Some("store_queue_tag"),
        width: 7,
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
        display_type: DisplayType::Custom,
    },
];

pub fn parse_mem_input_packets<'a>(bases: &[&str], snapshots: &'a Snapshots) -> Table<'a> {
    let header = Row::new(MEM_INPUT_HEADERS.map(|col| col.name))
        .bold()
        .on_blue();
    let widths = MEM_INPUT_HEADERS.map(|col| col.width);

    let rows = bases
        .iter()
        .map(|base| parse_mem_input_packet(base, snapshots));

    Table::new(rows, widths).header(header)
}

pub fn parse_mem_input_packet<'a>(base: &str, snapshots: &'a Snapshots) -> Row<'a> {
    let mut cells = Vec::<Cell>::new();

    let is_valid = snapshots
        .get_var(&format!("{base}.valid"))
        .unwrap()
        .is_high();

    for col in MEM_INPUT_HEADERS.iter() {
        if let Some(key) = col.key {
            let value = snapshots.get_var(&format!("{base}.{key}")).unwrap();

            let string = match col.display_type {
                DisplayType::Custom => {
                    unreachable!("No keyed columns have custom display type!")
                }
                display_type => value.format(&display_type),
            };

            cells.push(Cell::new(string));
        } else {
            assert!(col.name == "op");

            let string = snapshots.render_opinfo(&format!("{base}.op"));

            cells.push(Cell::new(string));
        }
    }
    let mut row = Row::new(cells);

    // formatting, colors
    if !is_valid {
        row = row.dim();
    }

    row
}

/*
typedef struct packed {
  logic valid;
  logic taken;          // was the branch taken?
  ADDR pc;              // pc of the branch instruction
  ADDR target_pc;       // pc we should go to after this branch instruction
  reg_idx_t rd;
  bmask_t bid;
  rob_num_t rob_num;
  branch_pred_packet branch_packet;
} branch_output_t;
 */

const BRANCH_OUTPUT_HEADERS: [Column; 7] = [
    Column {
        name: "taken",
        key: Some("taken"),
        width: 5,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "pc",
        key: Some("pc"),
        width: 6,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "target_pc",
        key: Some("pc"),
        width: 9,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "rd",
        key: Some("rd"),
        width: 2,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "bid",
        key: Some("bid"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "rob_num",
        key: Some("rob_num"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "pht_index",
        key: Some("branch_packet.pht_index"),
        width: 9,
        display_type: DisplayType::Decimal,
    },
];

pub fn get_branch_output_headers() -> Row<'static> {
    Row::new(BRANCH_OUTPUT_HEADERS.map(|col| col.name))
        .bold()
        .on_blue()
}

pub fn get_branch_output_widths() -> [u16; 7] {
    BRANCH_OUTPUT_HEADERS.map(|col| col.width)
}

pub fn parse_branch_output_packet<'a>(base: &str, snapshots: &'a Snapshots) -> Row<'a> {
    let mut cells = Vec::<Cell>::new();

    let is_valid = snapshots
        .get_var(&format!("{base}.valid"))
        .unwrap()
        .is_high();

    for col in BRANCH_OUTPUT_HEADERS.iter() {
        if let Some(key) = col.key {
            let full_key = format!("{base}.{key}");
            let value = snapshots.get_var(&full_key).unwrap();

            let string = match col.display_type {
                DisplayType::Custom => {
                    unreachable!("No keyed columns have custom display type!")
                }
                display_type => value.format(&display_type),
            };

            cells.push(Cell::new(string));
        } else {
            unreachable!("no unkeyed columns in branch output headers!")
        }
    }
    let mut row = Row::new(cells);

    // formatting, colors
    if !is_valid {
        row = row.dim();
    }

    row
}
