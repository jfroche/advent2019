use std::io;
use std::io::BufRead;
use std::num::ParseIntError;
use std::str::FromStr;

struct Module {
    pub mass: i32,
}

impl Module {
    pub fn required_fuel(&self) -> i32 {
        ((self.mass as f32) / 3.0).floor() as i32 - 2
    }
}

pub fn recursive_required_fuel(mass: i32) -> i32 {
    let fuel = Module { mass }.required_fuel();
    return match fuel {
        m if m > 0 => fuel + recursive_required_fuel(fuel),
        _ => return 0,
    };
}

impl FromStr for Module {
    type Err = ParseIntError;

    fn from_str(_src: &str) -> Result<Self, Self::Err> {
        Ok(Module {
            mass: FromStr::from_str(_src).unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_fuel() -> Result<(), String> {
        assert_eq!(Module { mass: 12 }.required_fuel(), 2);
        assert_eq!(Module { mass: 14 }.required_fuel(), 2);
        assert_eq!(Module { mass: 1969 }.required_fuel(), 654);
        assert_eq!(Module { mass: 100756 }.required_fuel(), 33583);
        Ok(())
    }

    #[test]
    fn test_recursive_required_fuel() -> Result<(), String> {
        assert_eq!(recursive_required_fuel(14), 2);
        assert_eq!(recursive_required_fuel(1969), 966);
        assert_eq!(recursive_required_fuel(100756), 50346);
        Ok(())
    }
}

fn main() {
    let modules: Vec<Module> = io::stdin()
        .lock()
        .lines()
        .filter_map(|l| l.ok())
        .filter_map(|l| l.parse().ok())
        .collect();
    let sum: i32 = modules.iter().map(|m| m.required_fuel()).sum();
    println!("Step 1 sum: {}", sum);
    let sum: i32 = modules
        .iter()
        .map(|m| recursive_required_fuel(m.mass))
        .sum();
    println!("Step 2 sum: {}", sum);
}
