use im::HashMap;
use std::io::BufReader;
use std::io::ErrorKind::InvalidInput;
use std::{fs::File, io};
use vcd::{self, Header, IdCode, Value, Vector};

#[derive(Debug, Clone)]
pub enum VerilogValue {
    Scalar(Value),
    Vector(Vector),
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub time: u64,
    pub clock_count: usize,
    pub variables: HashMap<IdCode, VerilogValue>,
}

pub struct Snapshots {
    shots: Vec<Snapshot>,
    header: Header,
    // parser: Parser<BufReader<File>>,
    index: usize,
}

impl Snapshots {
    pub fn new() -> io::Result<Self> {
        let file = File::open("sampler.vcd")?;
        let mut parser = vcd::Parser::new(BufReader::new(file));
        let header = parser.parse_header()?;

        let clock = header
            .find_var(&["testbench", "clock"])
            .ok_or_else(|| io::Error::new(InvalidInput, "no wire testbench.clock"))
            .unwrap()
            .code;

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
                    if id_code == clock && matches!(value, Value::V1) {
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
                x => {
                    dbg!(&x);
                }
            }
        }

        Ok(Snapshots {
            shots,
            header,
            index: 0,
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

    pub fn get_var<S>(&self, var_name: &[S]) -> IdCode
    where
        S: std::borrow::Borrow<str>,
    {
        let var_id = self.header.find_var(var_name);
        var_id.unwrap().code
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn scratch() {
//         let file = File::open("sampler.vcd").unwrap();
//         let mut parser = vcd::Parser::new(BufReader::new(file));
//     }
// }
