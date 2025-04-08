use im::HashMap;
use raki::{Decode, Isa};
use std::fmt::Display;
use std::io::BufReader;
use std::ops;
use std::{fs::File, io};
use vcd::{self, Header, IdCode, Scope, ScopeItem, Value, Vector};

use crate::utils::{o3oInst, DisplayType};
use crate::var_index::VarIndex;

pub enum DifferenceType {
    Addition,
    Clearance,
}

#[derive(Debug, Clone)]
pub enum VerilogValue {
    Scalar(Value),
    Vector(Vector),
}

// todo: honestly just move this struct into a new file
impl VerilogValue {
    pub fn format(&self, display_type: &DisplayType) -> String {
        match display_type {
            DisplayType::Binary => format!("{}", self),
            DisplayType::Decimal => self.as_decimal(),
            DisplayType::Hex => self.as_hex(),
            DisplayType::Custom => panic!("trying to format custom display type"),
        }
    }

    pub fn as_hex(&self) -> String {
        match self {
            VerilogValue::Scalar(value) => {
                format!("{}", value)
            }
            VerilogValue::Vector(vector) => {
                let mut bits = Vec::new();

                for item in vector.iter() {
                    bits.push(match item {
                        Value::V0 => 0,
                        Value::V1 => 1,
                        Value::X => return String::from("X"),
                        Value::Z => return String::from("Z"),
                    })
                }

                let val = bits.iter().fold(0, |res, new| (res << 1) + new);
                format!("{:#x}", val)
            }
        }
    }

    pub fn as_decimal(&self) -> String {
        match self {
            VerilogValue::Scalar(value) => {
                format!("{}", value)
            }
            VerilogValue::Vector(vector) => {
                let mut bits = Vec::new();

                for item in vector.iter() {
                    bits.push(match item {
                        Value::V0 => 0,
                        Value::V1 => 1,
                        Value::X => return String::from("X"),
                        Value::Z => return String::from("Z"),
                    })
                }

                let val = bits.iter().fold(0, |res, new| (res << 1) + new);
                val.to_string()
            }
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            VerilogValue::Scalar(value) => match value {
                Value::V1 => 1,
                _ => 0,
            },
            VerilogValue::Vector(vector) => {
                let mut bits = Vec::new();

                for item in vector.iter() {
                    bits.push(match item {
                        Value::V1 => 1,
                        _ => 0,
                    })
                }

                let val = bits.iter().fold(0, |res, new| (res << 1) + new);
                val
            }
        }
    }

    pub fn is_high(&self) -> bool {
        match self {
            VerilogValue::Scalar(value) => matches!(value, Value::V1),
            VerilogValue::Vector(vector) => vector.iter().all(|x| matches!(x, Value::V1)),
        }
    }

    pub fn is_low(&self) -> bool {
        match self {
            VerilogValue::Scalar(value) => matches!(value, Value::V0),
            VerilogValue::Vector(vector) => vector.iter().all(|x| matches!(x, Value::V0)),
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            VerilogValue::Scalar(value) => matches!(value, Value::X | Value::Z),
            VerilogValue::Vector(vector) => vector.iter().any(|x| matches!(x, Value::X | Value::Z)),
        }
    }

    pub fn from_usize(value: usize, size: usize) -> Self {
        if size == 1 {
            match value & 1 {
                0 => VerilogValue::Scalar(Value::V0),
                1 => VerilogValue::Scalar(Value::V1),
                _ => unreachable!(),
            }
        } else {
            let mut value = value;
            let mut bits = Vec::with_capacity(size);
            for _ in 0..size {
                if value & 1 == 0 {
                    bits.push(Value::V0)
                } else {
                    bits.push(Value::V1)
                }

                value >>= 1;
            }
            bits.reverse();
            VerilogValue::Vector(Vector::from(bits))
        }
    }
}

impl ops::Add<&VerilogValue> for &VerilogValue {
    type Output = VerilogValue;

    fn add(self, rhs: &VerilogValue) -> Self::Output {
        match (self, rhs) {
            (VerilogValue::Scalar(value1), VerilogValue::Scalar(value2)) => {
                VerilogValue::Vector(Vector::from([*value1, *value2]))
            }
            (VerilogValue::Scalar(value), VerilogValue::Vector(vector)) => {
                let mut values = vec![*value];
                values.extend(vector.iter());
                VerilogValue::Vector(Vector::from(values))
            }
            (VerilogValue::Vector(vector), VerilogValue::Scalar(value)) => {
                let mut values: Vec<Value> = vector.iter().collect();
                values.push(*value);
                VerilogValue::Vector(Vector::from(values))
            }
            (VerilogValue::Vector(vector1), VerilogValue::Vector(vector2)) => {
                let mut values: Vec<Value> = vector1.iter().collect();
                values.extend(vector2.iter());
                VerilogValue::Vector(Vector::from(values))
            }
        }
    }
}

impl Display for VerilogValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerilogValue::Scalar(value) => {
                write!(f, "{}", value)
            }
            VerilogValue::Vector(vector) => {
                write!(f, "{}", vector)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub time: u64,
    pub clock_count: usize,
    pub variables: HashMap<IdCode, VerilogValue>,
}

pub struct Snapshots {
    shots: Vec<Snapshot>,
    var_index: VarIndex,
    pub header: Header,
    index: usize,
}

pub fn get_header_base(header: &Header) -> String {
    for scope_item in header.items.iter() {
        if let ScopeItem::Scope(s) = scope_item {
            if !s.identifier.starts_with("_") {
                return s.identifier.clone();
            }
        }
    }
    panic!("Couldn't find top level testbench in VCD file!")
}

impl Snapshots {
    pub fn new(filename: &str, start_clock: usize, debugging_length: usize) -> io::Result<Self> {
        let file = File::open(filename)?;
        let mut parser = vcd::Parser::new(BufReader::new(file));
        let header = parser.parse_header()?;

        let base = get_header_base(&header);

        let clock_code;
        if let Some(clock) = header.find_var(&[base.as_str(), "clock"]) {
            clock_code = clock.code;
        } else {
            // give the clock an invalid code if it doesnt exist so always at clock cycle 0
            clock_code = u64::MAX.into();
        };

        let var_index = VarIndex::from_header(&header);
        let mut variables = HashMap::new();

        for var_code in var_index.vars.values() {
            variables.insert(*var_code, VerilogValue::Scalar(Value::X));
        }

        let mut shots = Vec::new();

        let mut snapshot = Snapshot {
            time: 0,
            clock_count: 0,
            variables,
        };

        for command_result in parser {
            let command = command_result.unwrap();
            use vcd::Command::*;

            match command {
                Timestamp(time) => {
                    if time != 0 && snapshot.clock_count >= start_clock {
                        shots.push(snapshot.clone());
                    }
                    snapshot.time = time;
                }
                ChangeScalar(id_code, value) => {
                    if id_code == clock_code && matches!(value, Value::V1) {
                        snapshot.clock_count += 1;
                        if snapshot.clock_count > start_clock + debugging_length {
                            shots.push(snapshot.clone());
                            break;
                        }
                    }
                    snapshot
                        .variables
                        .insert(id_code, VerilogValue::Scalar(value));
                }
                ChangeVector(id_code, value) => {
                    snapshot
                        .variables
                        .insert(id_code, VerilogValue::Vector(value));
                }
                _ => {}
            }
        }
        let index = shots.len() - 1;

        Ok(Snapshots {
            shots,
            var_index,
            header,
            index,
        })
    }

    pub fn get(&self) -> Option<&Snapshot> {
        self.shots.get(self.index)
    }

    pub fn advance(&mut self) -> bool {
        if self.index < self.shots.len() - 1 {
            self.index += 1;
            return true;
        }
        false
    }

    pub fn retreat(&mut self) -> bool {
        if self.index > 0 {
            self.index -= 1;
            return true;
        }
        false
    }

    // note edge behavior favors partial completion
    pub fn advance_n(&mut self, n: usize) -> bool {
        for _ in 0..n {
            let success = self.advance();
            if !success {
                return false;
            }
        }
        true
    }

    pub fn retreat_n(&mut self, n: usize) -> bool {
        for _ in 0..n {
            let success = self.retreat();
            if !success {
                return false;
            }
        }
        true
    }

    pub fn go_to_start(&mut self) {
        self.index = 0;
    }

    pub fn go_to_end(&mut self) {
        self.index = self.shots.len() - 1;
    }

    pub fn get_base(&self) -> String {
        get_header_base(&self.header)
    }

    pub fn get_var(&self, var_name: &str) -> Option<&VerilogValue> {
        let code = self.var_index.get(var_name)?;
        self.shots[self.index].variables.get(&code)
    }

    pub fn get_scope(&self, scope_name: &str) -> Option<&Scope> {
        let name_list: Vec<_> = scope_name.split('.').collect();
        self.header.find_scope(name_list.as_slice())
    }

    pub fn autocomplete_var(&self, var_name: &str) -> Vec<String> {
        self.var_index.engine.search(var_name)
    }

    pub fn render_opinfo(&self, base: &str) -> String {
        let pc = self.get_var(&format!("{base}.PC")).unwrap().as_usize();

        let inst_bits = self
            .get_var(&format!("{base}.inst.inst"))
            .unwrap()
            .as_usize();
        let Ok(inst) = (inst_bits as u32).decode(Isa::Rv32) else {
            return format!("{pc}: <invalid>");
        };
        let inst = o3oInst(inst);

        format!("{pc:x}: {inst}")
    }
}
