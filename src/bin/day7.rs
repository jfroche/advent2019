use log::*;
use std::fs;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::str;

use futures::executor::block_on;
use futures::future::join_all;
use permutohedron::LexicalPermutation;

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
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
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
            Some(5) => (InstructionType::JumpIfTrue, 2),
            Some(6) => (InstructionType::JumpIfFalse, 2),
            Some(7) => (InstructionType::LessThan, 3),
            Some(8) => (InstructionType::Equals, 3),
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
            program: code.split(",").map(|s| s.parse().unwrap()).collect(),
            cursor: 0,
        }
    }

    fn get(&mut self, pam: &ParameterMode) -> i32 {
        trace!("getting {:?}", pam);
        debug!("getting: {}", self.cursor);
        let value = self.next().expect("Unable to get value") as usize;
        debug!("got: {}", value);
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

    fn set(&mut self, value: i32) -> () {
        let pos = self.next().expect("Unable to get position to set") as usize;
        trace!("storing {} in {}", value, pos);
        self.program[pos] = value
    }

    fn run<R, W>(&mut self, mut reader: R, mut writer: W) -> ()
    where
        R: BufRead,
        W: Write,
    {
        loop {
            let next_instruction = self.next();
            if next_instruction == None {
                break;
            }
            debug!(
                "opcode {:?} - index: {}",
                next_instruction.unwrap() % 1000,
                self.cursor - 1
            );
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
                    debug!("OUTPUT value: {:?}", value);
                    write!(&mut writer, "{}", value).expect("Unable to write");
                    false
                },
                InstructionType::Input => {
                    let mut input = String::new();
                    reader
                        .read_line(&mut input)
                        .expect("Unable to read user input");
                    debug!("input: {}", input);
                    let result = input
                        .trim_end()
                        .parse()
                        .expect("cannot parse integer from input");
                    self.set(result);
                    false
                },
                InstructionType::JumpIfTrue => {
                    let param1 = self.get(pm.next().expect("Missing operand"));
                    if param1 != 0 {
                        let param2 = self.get(pm.next().expect("Missing operand")); // self.next().expect("Unable to get value");
                        debug!("changing cursor to {}", param2);
                        self.cursor = param2;
                    } else {
                        self.next();
                    }
                    false
                },
                InstructionType::JumpIfFalse => {
                    let param1 = self.get(pm.next().expect("Missing operand"));
                    if param1 == 0 {
                        let param2 = self.get(pm.next().expect("Missing operand")); // self.next().expect("Unable to get value");
                        debug!("changing cursor to {}", param2);
                        self.cursor = param2;
                    } else {
                        self.next();
                    }
                    false
                },
                InstructionType::LessThan => {
                    let param1 = self.get(pm.next().expect("Missing operand"));
                    let param2 = self.get(pm.next().expect("Missing operand"));
                    let param3 = self.next().expect("Unable to get value") as usize;
                    debug!("writing to {}", param3);
                    if param1 < param2 {
                        self.program[param3] = 1
                    } else {
                        self.program[param3] = 0
                    }
                    false
                },
                InstructionType::Equals => {
                    let param1 = self.get(pm.next().expect("Missing operand"));
                    let param2 = self.get(pm.next().expect("Missing operand"));
                    let param3 = self.next().expect("Unable to get value") as usize;
                    debug!("writing to {}", param3);
                    if param1 == param2 {
                        self.program[param3] = 1;
                    } else {
                        self.program[param3] = 0;
                    }
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

async fn calculate(code: String, input: Vec<usize>) -> i32 {
    input.iter().fold(0, |last, i| {
        let mut p = Intcode::new(code.clone());
        debug!("running p.run({}, {})", i, last);
        let mut output = Vec::new();
        let str_input = format!("{}\n{}\n", *i, last);
        let buffer = str_input.as_bytes();
        p.run(&buffer[..], &mut output);
        let output: i32 = str::from_utf8(&output)
            .expect("unable to parse output")
            .parse()
            .expect("unable to parse output");
        debug!("output: {:?}", output);
        output
    })
}

async fn run_all(code: String) -> Vec<i32> {
    let mut data = [0, 1, 2, 3, 4];
    let mut permutations = Vec::new();

    loop {
        permutations.push(data.to_vec());
        if !data.next_permutation() {
            break;
        }
    }
    let futures = permutations
        .iter()
        .map(|p| calculate(code.clone(), p.to_vec()));
    debug!("{:?}", futures);
    join_all(futures).await
}

#[derive(Debug, StructOpt)]
#[structopt(name = "day7", about = "Advent of Code - Day 7")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    advent::init_logging();
    let buffer = fs::read_to_string(opt.input)
        .expect("Unable to read input file")
        .replace('\n', "");
    let results = block_on(run_all(buffer));
    info!("part 1: {:?}", results.iter().max());
}
