use std::io;

#[derive(Debug, Clone)]
struct Intcode {
    program: Vec<i32>,
    cursor: i32,
}

impl Intcode {
    fn new(code: String) -> Self {
        Self {
            program: code.split(",").filter_map(|s| s.parse().ok()).collect(),
            cursor: 0,
        }
    }

    fn restore_state(&mut self) -> () {
        self.program[1] = 12;
        self.program[2] = 2;
    }

    fn get(&self, pos: usize) -> i32 {
        println!("getting {}", pos);
        self.program[pos]
    }

    fn set(&mut self, pos: usize, value: i32) -> () {
        println!("storing {} in {}", value, pos);
        self.program[pos] = value
    }

    fn run(&mut self) -> i32 {
        loop {
            let op = self.next();
            if op == Some(99) || op == None {
                break;
            }
            let mut value: usize = self.next().expect("Unable to fetch left value") as usize;
            let left = self.get(value);
            value = self.next().expect("Unable to fetch right value") as usize;
            let right = self.get(value);
            let result;
            let _ = match op {
                Some(1) => {
                    println!("{} + {}", left, right);
                    result = left + right;
                },
                Some(2) => {
                    println!("{} * {}", left, right);
                    result = left * right;
                },
                Some(99) => break,
                _ => break,
            };
            let position = self.next().expect("Unable to fetch position to store");
            self.set(position as usize, result)
        }
        return self.program[0];
    }
}

impl Iterator for Intcode {
    type Item = i32;

    // just return the str reference
    fn next(&mut self) -> Option<Self::Item> {
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
    fn test_basic_programs() {
        let mut program = Intcode::new("1,2,3,4,99".to_string());
        assert_eq!(program.run(), 1);
        program = Intcode::new("1,0,0,0,99".to_string());
        assert_eq!(program.run(), 2);
        program = Intcode::new("1,1,1,4,99,5,6,0,99".to_string());
        assert_eq!(program.run(), 30);
    }
}

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read input !");
    let mut program = Intcode::new(buffer);
    program.restore_state();
    let result = program.run();
    println!("Step 1 first item: {}", result);
}
