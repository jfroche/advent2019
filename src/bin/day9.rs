use log::*;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone)]
struct Intcode {
    program: Vec<i64>,
    cursor: i64,
    relative_base: i64,
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
    Adjustbase,
}

#[derive(Debug, Clone, PartialEq)]
enum ParameterMode {
    Position,
    Immediate,
    Relative,
}

struct Instruction {
    pub instruction_type: InstructionType,
    pub parameter_mode: Vec<ParameterMode>,
}

impl Instruction {
    fn new(code: i64) -> Self {
        let code_as_str = format!("{}", code);
        let mut code_as_vec: Vec<i64> = code_as_str
            .into_bytes()
            .into_iter()
            .map(|b| b as i64 - 48)
            .collect::<Vec<i64>>();
        debug!("code vec: {:?}", code_as_vec);
        let (mut inst_type, size) = match code_as_vec.pop() {
            Some(1) => (InstructionType::Add, 3),
            Some(2) => (InstructionType::Mul, 3),
            Some(3) => (InstructionType::Input, 1),
            Some(4) => (InstructionType::Output, 1),
            Some(5) => (InstructionType::JumpIfTrue, 2),
            Some(6) => (InstructionType::JumpIfFalse, 2),
            Some(7) => (InstructionType::LessThan, 3),
            Some(8) => (InstructionType::Equals, 3),
            Some(9) => (InstructionType::Adjustbase, 1),
            _ => panic!("Unknown instruction"),
        };
        if let Some(x) = code_as_vec.pop() {
            if x == 9 {
                inst_type = InstructionType::Stop;
            }
        }
        let mut parameter_mode = std::iter::repeat(ParameterMode::Position)
            .take(size)
            .collect::<Vec<_>>();
        code_as_vec.reverse();
        code_as_vec
            .iter()
            .enumerate()
            .map(|(i, v)| {
                debug!("getting: pos {} value {} size {}", i, v, size);
                if *v == 1 {
                    parameter_mode[i] = ParameterMode::Immediate;
                } else if *v == 2 {
                    parameter_mode[i] = ParameterMode::Relative;
                }
                debug! {"done"}
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
            program: code.split(",").map(|s| s.parse::<i64>().unwrap()).collect(),
            cursor: 0,
            relative_base: 0,
        }
    }

    fn get(&mut self, pam: &ParameterMode) -> i64 {
        trace!("getting {:?}", pam);
        debug!("getting: {}", self.cursor);
        let value = self.next().expect("Unable to get value") as usize;
        debug!("got: {}", value);
        match pam {
            ParameterMode::Position => {
                if value > self.program.len() {
                    0
                } else {
                    self.program[value]
                }
            },
            ParameterMode::Immediate => value as i64,
            ParameterMode::Relative => {
                let relative_value = value.wrapping_add(self.relative_base as usize);
                if relative_value > self.program.len() {
                    0
                } else {
                    self.program[relative_value]
                }
            },
        }
    }

    fn size(&self) -> usize {
        self.program.len()
    }

    fn set(&mut self, value: i64, pam: &ParameterMode) -> () {
        let mut pos = self.next().expect("Unable to get position to set") as usize;
        trace!("storing {} in {}", value, pos);
        if *pam == ParameterMode::Relative {
            pos = pos.wrapping_add(self.relative_base as usize);
        }
        if pos >= self.program.len() {
            self.program.resize(pos + 1, 0);
        }
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
                    self.set(result, pm.next().expect("missing operand"));
                    false
                },
                InstructionType::Mul => {
                    let result = self
                        .get(pm.next().expect("Missing operand"))
                        .wrapping_mul(self.get(pm.next().expect("missing operand")));
                    self.set(result, pm.next().expect("missing operand"));
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
                    self.set(result, pm.next().expect("missing operand"));
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
                    if param1 < param2 {
                        self.set(1, pm.next().expect("missing operand"));
                    } else {
                        self.set(0, pm.next().expect("missing operand"));
                    }
                    false
                },
                InstructionType::Equals => {
                    let param1 = self.get(pm.next().expect("Missing operand"));
                    let param2 = self.get(pm.next().expect("Missing operand"));
                    if param1 == param2 {
                        self.set(1, pm.next().expect("missing operand"));
                    } else {
                        self.set(0, pm.next().expect("missing operand"));
                    }
                    false
                },
                InstructionType::Adjustbase => {
                    let value = self.get(pm.next().expect("Missing operand"));
                    info!("Ajust base value: {:?}", value);
                    self.relative_base = self.relative_base + value;
                    false
                },
            };
        }
    }
}

impl Iterator for Intcode {
    type Item = i64;

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
    fn test_instruction_relative() {
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
        let inst2 = Instruction::new(209);
        assert_eq!(inst2.instruction_type, InstructionType::Adjustbase);
        assert_eq!(inst2.parameter_mode, vec![ParameterMode::Relative]);
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
    #[test]
    fn test_day9() {
        let input = b"";
        let output = run_test(
            "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99".to_string(),
            &input[..],
        );
        assert_eq!(
            output,
            "OUTPUT value: 109\nOUTPUT value: 1\nOUTPUT value: 204\nOUTPUT value: -1\nOUTPUT \
             value: 1001\nOUTPUT value: 100\nOUTPUT value: 1\nOUTPUT value: 100\nOUTPUT value: \
             1008\nOUTPUT value: 100\nOUTPUT value: 16\nOUTPUT value: 101\nOUTPUT value: \
             1006\nOUTPUT value: 101\nOUTPUT value: 0\nOUTPUT value: 99\n"
        );
        let input = b"";
        let output = run_test("104,1125899906842624,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 1125899906842624\n");

        let input = b"";
        let output = run_test("1102,34915192,34915192,7,4,7,99,0".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 1219070632396864\n");

        let input = b"";
        let output = run_test("109,-1,4,1,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: -1\n");

        let input = b"";
        let output = run_test("109,-1,104,1,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 1\n");

        let input = b"";
        let output = run_test("109,-1,204,1,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 109\n");

        let input = b"";
        let output = run_test("109,1,9,2,204,-6,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 204\n");

        let input = b"";
        let output = run_test("109,1,109,9,204,-6,99".to_string(), &input[..]);
        assert_eq!(output, "OUTPUT value: 204\n");
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
