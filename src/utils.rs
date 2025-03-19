use std::{
    fs::{self},
    io::Result,
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

// pub enum Opcode {
//     Rv32Load,
//     Rv32Store,
//     Rv32Branch,
//     Rv32LoadFp,
//     Rv32StoreFp,
//     Rv32JalrOp,
//     Rv32Fence,
//     Rv32Amo,
//     Rv32JalOp,
//     Rv32OpImm,
//     Rv32Op,
//     Rv32OpFp,
//     Rv32System,
//     Rv32AuipcOp,
//     Rv32LuiOp,
//     Rv64OpImmW,
//     Rv64OpW,
//     Rv32Custom2,
//     Rv32Custom3,
// }

// impl From<usize> for Opcode {
//     fn from(value: usize) -> Self {
//         match value {
//             0b0000011 => Self::Rv32Load,  //load, self-explanatory
//             0b0100011 => Self::Rv32Store, //store, self-explanatory
//             0b1100011 => Self::Rv32Branch,
//             0b0000111 => Self::Rv32LoadFp,  //floating point load
//             0b0100111 => Self::Rv32StoreFp, //floating point store
//             0b1100111 => RV32_JALR_OP,  //jump and link with return
//             0b0001111 => RV32_FENCE,    //FENCE instruction for enforcing memory consistency
//             0b0101111 => RV32_AMO,      //atomic memory operation
//             0b1101111 => RV32_JAL_OP,   //just jump and link
//             0b0010011 => RV32_OP_IMM,
//             0b0110011 => RV32_OP,
//             0b1010011 => RV32_OP_FP,
//             0b1110011 => RV32_SYSTEM,
//             0b0010111 => RV32_AUIPC_OP,
//             0b0110111 => RV32_LUI_OP,
//             0b0011011 => RV64_OP_IMM_W, //is RV64-specific, used for 32 bit imm operations
//             0b0111011 => RV64_OP_W,     //is RV64-specific, used for 32 bit operations
//             0b1011011 => RV32_CUSTOM_2,
//             0b1111011 => RV32_CUSTOM_3,
//         }
//     }
// }
