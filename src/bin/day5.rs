use log::*;
use std::fs;
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone)]
struct Intcode {
    program: Vec<i32>,
    cursor: i32,
}

#[derive(Debug, Clone, PartialEq)]
enum InstructionType {
    Mul,
    Add,
    Input,
    Output,
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
enum ParameterMode {
    Position,
    Immediate,
}

struct Instruction {
    pub instruction_type: InstructionType,
    pub parameter_mode: Vec<ParameterMode>,
}

impl Instruction {
    fn new(code: i32) -> Self {
        let code_as_str = format!("{}", code);
        let mut code_as_vec: Vec<i32> = code_as_str
            .into_bytes()
            .into_iter()
            .map(|b| b as i32 - 48)
            .collect::<Vec<i32>>();
        debug!("code vec: {:?}", code_as_vec);
        let (inst_type, size) = match code_as_vec.pop() {
            Some(1) => (InstructionType::Add, 3),
            Some(2) => (InstructionType::Mul, 3),
            Some(3) => (InstructionType::Input, 1),
            Some(4) => (InstructionType::Output, 1),
            Some(9) => (InstructionType::Stop, 0),
            _ => panic!("Unknown instruction"),
        };
        code_as_vec.pop(); // ignore instruction type prefix
        let mut parameter_mode = std::iter::repeat(ParameterMode::Position)
            .take(size)
            .collect::<Vec<_>>();
        code_as_vec.reverse();
        code_as_vec
            .iter()
            .enumerate()
            .filter(|&(_, v)| v == &1)
            .map(|(i, _)| {
                parameter_mode[i] = ParameterMode::Immediate;
            })
            .for_each(drop);
        Self {
            instruction_type: inst_type,
            parameter_mode,
        }
    }
}

impl Intcode {
    fn new(code: String) -> Self {
        Self {
            program: code.split(",").filter_map(|s| s.parse().ok()).collect(),
            cursor: 0,
        }
    }

    fn get(&mut self, pam: &ParameterMode) -> i32 {
        trace!("getting {:?}", pam);
        let value = self.next().expect("Unable to get value") as usize;
        match pam {
            ParameterMode::Position => self.program[value],
            ParameterMode::Immediate => value as i32,
        }
    }

    #[allow(dead_code)]
    fn get_with_value(&mut self, value: usize, pam: &ParameterMode) -> i32 {
        trace!("getting {:?}", pam);
        match pam {
            ParameterMode::Position => self.program[value],
            ParameterMode::Immediate => value as i32,
        }
    }

    fn size(&self) -> usize {
        self.program.len()
    }

    fn set(&mut self, value: i32) -> () {
        let pos = self.next().expect("Unable to get position to set") as usize;
        trace!("storing {} in {}", value, pos);
        self.program[pos] = value
    }

    fn run(&mut self) -> () {
        loop {
            let next_instruction = self.next();
            if next_instruction == None {
                break;
            }
            let op = Instruction::new(next_instruction.expect("Missing next Instruction"));
            let mut pm = op.parameter_mode.iter();
            match op.instruction_type {
                InstructionType::Stop => break,
                InstructionType::Add => {
                    let result = self.get(pm.next().expect("Missing operand"))
                        + self.get(pm.next().expect("missing operand"));
                    debug!("result for add: {}", result);
                    self.set(result);
                    false
                },
                InstructionType::Mul => {
                    let result = self.get(pm.next().expect("Missing operand"))
                        * self.get(pm.next().expect("missing operand"));
                    self.set(result);
                    false
                },
                InstructionType::Output => {
                    let value = self.get(pm.next().expect("Missing operand"));
                    info!("OUTPUT value: {:?}", value);
                    false
                },
                InstructionType::Input => {
                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("error: unable to read user input");
                    debug!("input: {}", input);
                    let result = input
                        .trim_end()
                        .parse()
                        .expect("cannot parse integer from input");
                    self.set(result);
                    false
                },
            };
        }
    }
}

impl Iterator for Intcode {
    type Item = i32;

    // just return the str reference
    fn next(&mut self) -> Option<Self::Item> {
        debug!("getting next: {} on {:?}", self.cursor, self.program);
        if self.program.len() - 1 > self.cursor as usize {
            let n = self.program[self.cursor as usize];
            self.cursor = self.cursor + 1;
            Some(n)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_instruction() {
        advent::init_logging();
        let inst1 = Instruction::new(1002);
        assert_eq!(inst1.instruction_type, InstructionType::Mul);
        assert_eq!(
            inst1.parameter_mode,
            vec![
                ParameterMode::Position,
                ParameterMode::Immediate,
                ParameterMode::Position
            ]
        );
    }

    #[test]
    fn test_basic_programs() {
        let mut program = Intcode::new("1101,100,-1,4,0".to_string());
        program.run();
        assert_eq!(program.get_with_value(4, &ParameterMode::Position), 99);
    }
}

fn run(code: String) {
    let mut program = Intcode::new(code);
    debug!("size: {:?}", program.size());
    program.run()
}

#[derive(Debug, StructOpt)]
#[structopt(name = "day5", about = "Advent of Code - Day 5")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
    advent::init_logging();
    let buffer = fs::read_to_string(opt.input).expect("Unable to read input file");
    println!("{:?}", buffer);
    run(buffer.clone());
}
