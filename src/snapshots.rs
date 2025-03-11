use im::HashMap;
use std::fmt::Display;
use std::io::BufReader;
use std::{fs::File, io};
use vcd::{self, Header, IdCode, Scope, ScopeItem, Value, Vector};

use crate::var_index::VarIndex;

pub enum DifferenceType {
    Addition,
    Clearance,
}

type Differences = HashMap<IdCode, DifferenceType>;

#[derive(Debug, Clone)]
pub enum VerilogValue {
    Scalar(Value),
    Vector(Vector),
}

impl VerilogValue {
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
    pub fn new(filename: &str) -> io::Result<Self> {
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

        let mut shots = Vec::new();

        let mut snapshot = Snapshot {
            time: 0,
            clock_count: 0,
            variables: HashMap::new(),
        };

        for command_result in parser {
            let command = command_result.unwrap();
            use vcd::Command::*;
            // dbg!(&command);
            match command {
                Timestamp(time) => {
                    // println!("\nTime: {time}")
                    if time != 0 {
                        shots.push(snapshot.clone());
                        snapshot.time = time;
                    }
                }
                ChangeScalar(id_code, value) => {
                    if id_code == clock_code && matches!(value, Value::V1) {
                        snapshot.clock_count += 1;
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
                _ => {
                    // dbg!(&x);
                }
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
        self.index = self.shots.len()-1;
    }

    pub fn get_base(&self) -> String {
        get_header_base(&self.header)
    }

    pub fn get_var(&self, var_name: &str) -> Option<&VerilogValue> {
        // let name_list: Vec<_> = var_name.split('.').collect();
        // let var = self.header.find_var(&name_list)?;
        // self.shots[self.index].variables.get(&var.code)

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

    pub fn differences(&self) -> Differences {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch() {
        let _ = Snapshots::new("rs.vcd");
    }
}
