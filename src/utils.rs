use core::fmt;
use std::{
    fmt::{Display, Formatter},
    fs::{self},
    io::Result,
};

use raki::{
    AOpcode, BaseIOpcode, COpcode, InstFormat, Instruction, OpcodeKind, PrivOpcode, ZifenceiOpcode,
};

pub fn save_watch_list(watch_list: &[String], name: &str) -> Result<()> {
    let write_data = watch_list.join("\n");
    let filename = format!("debugger_files/{name}.dbg_list");

    fs::write(&filename, write_data)?;

    Ok(())
}

pub fn load_watch_list(name: &str) -> Result<Vec<String>> {
    let filename = format!("debugger_files/{name}.dbg_list");

    let data = fs::read_to_string(&filename)?;

    let watch_list = data.split("\n").map(|x| x.to_owned()).collect();
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
