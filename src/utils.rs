use core::fmt;
use std::{
    fmt::{Display, Formatter},
    fs::{self},
    io::{Error, ErrorKind, Result},
};

use raki::{
    AOpcode, BaseIOpcode, COpcode, Decode, InstFormat, Instruction, Isa, OpcodeKind, PrivOpcode,
    ZifenceiOpcode,
};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols,
    widgets::{Cell, Row, Table},
};

use crate::snapshots::{Snapshots, VerilogValue};

#[derive(Clone, Copy)]
pub enum DisplayType {
    Binary,
    Decimal,
    Hex,
    Custom(fn(&VerilogValue) -> String),
}
impl DisplayType {
    pub fn next(&self) -> Self {
        match self {
            DisplayType::Binary => DisplayType::Decimal,
            DisplayType::Decimal => DisplayType::Hex,
            DisplayType::Hex => DisplayType::Binary,
            DisplayType::Custom(_) => {
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
                DisplayType::Custom(_) => panic!("Shouldn't be able to save Custom DisplayType"),
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
            InstFormat::SFormat => write!(
                f,
                "{} {}, {}({})",
                self.0.opc,
                reg2str(self.0.rs2.unwrap()),
                self.0.imm.unwrap(),
                reg2str(self.0.rs1.unwrap()),
            ),
            InstFormat::BFormat => write!(
                f,
                "{} {}, {}, {}",
                self.0.opc,
                reg2str(self.0.rs1.unwrap()),
                reg2str(self.0.rs2.unwrap()),
                self.0.imm.unwrap(),
            ),
            InstFormat::CsFormat => write!(
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

pub fn parse_inst(base: &str, snapshots: &Snapshots) -> String {
    let inst_bits = snapshots
        .get_var(&format!("{base}.inst"))
        .unwrap()
        .as_usize();
    let Ok(inst) = (inst_bits as u32).decode(Isa::Rv32) else {
        return String::from("<invalid>");
    };
    let inst = o3oInst(inst);
    format!("{inst}")
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

pub fn parse_fu_type(val: &VerilogValue) -> String {
    if val.is_unknown() {
        return "xxxxx".to_string();
    }
    match val.as_usize() {
        0b000 => "NOP".to_string(),
        0b001 => "IALU".to_string(),
        0b010 => "LOAD".to_string(),
        0b011 => "STORE".to_string(),
        0b100 => "MULT".to_string(),
        0b101 => "BRANCH".to_string(),
        _ => "<invalid>".to_string(),
    }
}

pub const TOP_BORDER_SET: symbols::border::Set = symbols::border::Set {
    top_left: symbols::line::NORMAL.vertical_right,
    top_right: symbols::line::NORMAL.vertical_left,
    ..symbols::border::PLAIN
};

pub const LEFT_BORDER_SET: symbols::border::Set = symbols::border::Set {
    top_left: symbols::line::NORMAL.horizontal_down,
    bottom_left: symbols::line::NORMAL.horizontal_up,
    ..symbols::border::PLAIN
};

pub fn parse_opinfo(base: &str, snapshots: &Snapshots) -> String {
    let pc = snapshots.get_var(&format!("{base}.PC")).unwrap().as_usize();

    let inst_bits = snapshots
        .get_var(&format!("{base}.inst.inst"))
        .unwrap()
        .as_usize();
    let Ok(inst) = (inst_bits as u32).decode(Isa::Rv32) else {
        return format!("{pc}: <invalid>");
    };
    let inst = o3oInst(inst);

    format!("{pc:x}: {inst}")
}

#[derive(Clone, Copy)]
pub struct Column {
    pub name: &'static str,
    pub key: Option<&'static str>,
    pub width: u16,
    pub display_type: DisplayType,
}

pub struct Columns {
    columns: Vec<Column>,
}

impl Columns {
    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns }
    }

    pub fn get_header(&self) -> Row<'static> {
        Row::new(self.columns.iter().map(|col| col.name))
            .bold()
            .on_blue()
    }

    pub fn get_widths(&self) -> Vec<u16> {
        self.columns.iter().map(|col| col.width).collect()
    }

    pub fn create_row<'a>(
        &self,
        base: &str,
        snapshots: &'a Snapshots,
        // fallback: Option<fn(&str, &Snapshots, &str) -> String>,
    ) -> Row<'a> {
        let mut cells = Vec::<Cell>::new();

        let is_valid = snapshots
            .get_var(&format!("{base}.valid"))
            .unwrap_or(&VerilogValue::Scalar(vcd::Value::V1))
            .is_high();

        for col in self.columns.iter() {
            if let Some(key) = col.key {
                let full_key = format!("{base}.{key}");
                let value = snapshots.get_var(&full_key).unwrap();

                let string = value.format(&col.display_type);
                cells.push(Cell::new(string));
            } else {
                // let string =
                //     fallback.expect("unkeyed column with no fallback!")(base, snapshots, col.name);
                if col.name == "op" {
                    let base = format!("{base}.{}", col.name);
                    cells.push(Cell::new(parse_opinfo(&base, snapshots)));
                } else {
                    panic!("unrecognized unkeyed column {}", col.name)
                }
            }
        }
        let mut row = Row::new(cells);

        // formatting, colors
        if !is_valid {
            row = row.dim();
        }

        row
    }

    pub fn create_table<'a>(&self, bases: Vec<String>, snapshots: &'a Snapshots) -> Table<'a> {
        let mut rows = Vec::with_capacity(bases.len());

        for base in bases {
            rows.push(self.create_row(&base, snapshots));
        }

        Table::new(rows, self.get_widths()).header(self.get_header())
    }

    pub fn create_table_no_header<'a>(
        &self,
        bases: Vec<String>,
        snapshots: &'a Snapshots,
    ) -> Table<'a> {
        let mut rows = Vec::with_capacity(bases.len());

        for base in bases {
            rows.push(self.create_row(&base, snapshots));
        }

        Table::new(rows, self.get_widths())
    }
}

pub fn split_vertical(area: Rect) -> [Rect; 2] {
    Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(area)
}

pub fn split_horizontal(area: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area)
}

pub fn _path_predecessor(path: &str) -> String {
    let parts: Vec<&str> = path.split('.').collect();
    parts[0..parts.len() - 1].join(".")
}
