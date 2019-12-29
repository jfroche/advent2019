use log::*;
use std::fs;
use std::io::{self, BufRead, Write};
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

    fn size(&self) -> usize {
        self.program.len()
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
                    info!("OUTPUT value: {:?}", value);
                    write!(&mut writer, "OUTPUT value: {}\n", value).expect("Unable to write");
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_instruction() {
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
        let input = b"2";
        let mut output = Vec::new();
        program.run(&input[..], &mut output);
        assert_eq!(program.get_with_value(4, &ParameterMode::Position), 99);
    }

    fn run_test<R>(program: String, input: R) -> String
    where
        R: BufRead,
    {
        let mut program = Intcode::new(program);
        let mut output = Vec::new();
        program.run(input, &mut output);
        String::from_utf8(output).expect("Not UTF-8")
    }

    #[test]
    fn test_equals() {
        let input = b"2";
        let output = run_test("3,9,8,9,10,9,4,9,99,-1,8".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 0\n");

        let input = b"8";
        let output = run_test("3,9,8,9,10,9,4,9,99,-1,8".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 1\n");
    }

    #[test]
    fn test_equals_immediate() {
        let input = b"2";
        let output = run_test("3,3,1108,-1,8,3,4,3,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 0\n");

        let input = b"8";
        let output = run_test("3,3,1108,-1,8,3,4,3,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 1\n");
    }

    #[test]
    fn test_lessthan() {
        let input = b"2";
        let output = run_test("3,9,7,9,10,9,4,9,99,-1,8".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 1\n");

        let input = b"9";
        let output = run_test("3,9,7,9,10,9,4,9,99,-1,8".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 0\n");
    }
    #[test]
    fn test_lessthan_immediate() {
        let input = b"2";
        let output = run_test("3,3,1107,-1,8,3,4,3,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 1\n");

        let input = b"9";
        let output = run_test("3,3,1107,-1,8,3,4,3,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 0\n");
    }
    #[test]
    fn test_jump() {
        let input = b"2";
        let output = run_test(
            "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9".to_string(),
            &input[..],
        );
        assert_eq!(output, "OUTPUT value: 1\n");

        let input = b"0";
        let output = run_test(
            "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9".to_string(),
            &input[..],
        );
        assert_eq!(output, "OUTPUT value: 0\n");
    }
    #[test]
    fn test_jump_immediate() {
        let input = b"2";
        let output = run_test(
            "3,3,1105,-1,9,1101,0,0,12,4,12,99,1".to_string(),
            &input[..],
        );
        assert_eq!(output, "OUTPUT value: 1\n");

        let input = b"0";
        let output = run_test(
            "3,3,1105,-1,9,1101,0,0,12,4,12,99,1".to_string(),
            &input[..],
        );
        assert_eq!(output, "OUTPUT value: 0\n");
    }

    #[test]
    fn test_larger_example() {
        let input = b"2";
        let output = run_test(
            "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,\
             4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99"
                .to_string(),
            &input[..],
        );
        assert_eq!(output, "OUTPUT value: 999\n");

        let input = b"8";
        let output = run_test(
            "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,\
             4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99"
                .to_string(),
            &input[..],
        );
        assert_eq!(output, "OUTPUT value: 1000\n");

        let input = b"9";
        let output = run_test(
            "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,\
             4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99"
                .to_string(),
            &input[..],
        );
        assert_eq!(output, "OUTPUT value: 1001\n");
    }
    #[test]
    fn test_others() {
        let input = b"";
        let output = run_test("101,-1,7,7,4,7,1105,11,0,99".to_string(), &input[..]);
        assert_eq!(
            output,
            "OUTPUT value: 10\nOUTPUT value: 9\nOUTPUT value: 8\nOUTPUT value: 7\nOUTPUT value: \
             6\nOUTPUT value: 5\nOUTPUT value: 4\nOUTPUT value: 3\nOUTPUT value: 2\nOUTPUT value: \
             1\nOUTPUT value: 0\n"
        );
    }
}

fn run(code: String) {
    let mut program = Intcode::new(code);
    debug!("size: {:?}", program.size());
    let stdio = io::stdin();
    let input = stdio.lock();
    let output = io::stdout();
    program.run(input, output)
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
    advent::init_logging();
    let buffer = fs::read_to_string(opt.input)
        .expect("Unable to read input file")
        .replace('\n', "");
    run(buffer.clone());
}
