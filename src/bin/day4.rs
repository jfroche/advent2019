use itertools::Itertools;
use log::*;
use std::io;

fn min_max(range: String) -> (i32, i32) {
    let mut m: Vec<_> = range.trim().split('-').collect();
    let max: i32 = m
        .pop()
        .expect("Unable to get min")
        .to_string()
        .parse::<i32>()
        .unwrap();
    let min: i32 = m
        .pop()
        .expect("Unable to get max")
        .to_string()
        .parse()
        .unwrap();
    (min, max)
}

fn has_adjacent(i: &Vec<char>) -> bool {
    let mut v2: Vec<_> = i.clone();
    v2.dedup();
    i != &v2
}

fn has_adjacent_ingroup(i: &Vec<char>) -> bool {
    let res: Vec<usize> = i
        .clone()
        .into_iter()
        .group_by(|elt| *elt)
        .into_iter()
        .map(|(_, r)| r.collect::<Vec<_>>().len())
        .filter(|c| c == &2)
        .collect();
    debug!("{:?}", res);
    res.len() != 0
}

fn never_decrease(i: &Vec<char>) -> bool {
    let mut v2: Vec<_> = i.clone();
    v2.dedup_by(|a, b| b > a);
    i == &v2
}

fn int_to_char(i: i32) -> Vec<char> {
    i.to_string().chars().collect()
}

fn filter_passwords(min: i32, max: i32) -> Vec<Vec<char>> {
    (min..max)
        .map(|i| int_to_char(i))
        .filter(|x| never_decrease(x))
        .filter(|x| has_adjacent(x))
        .collect()
}

fn filter_passwords_p2(min: i32, max: i32) -> Vec<Vec<char>> {
    (min..max)
        .map(|i| int_to_char(i))
        .filter(|x| never_decrease(x))
        .filter(|x| has_adjacent_ingroup(x))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacent() {
        assert!(has_adjacent(&int_to_char(11)));
        assert!(has_adjacent(&int_to_char(12)) == false);
        assert!(has_adjacent(&int_to_char(123345)));
    }

    #[test]
    fn test_never_decrease() {
        assert!(never_decrease(&int_to_char(11)));
        assert!(never_decrease(&int_to_char(21)) == false);
        assert!(never_decrease(&int_to_char(123345)));
        assert!(never_decrease(&int_to_char(111111)));
        assert!(never_decrease(&int_to_char(123325)) == false);
    }

    #[test]
    fn test_filter_password() {
        advent::init_logging();
        let pwd = filter_passwords(111111, 111112);
        info!("{:?}", pwd);
        assert!(pwd == vec![int_to_char(111111)]);
    }

    #[test]
    fn test_has_adjacent_ingroup() {
        assert!(has_adjacent_ingroup(&int_to_char(112233)));
        assert!(has_adjacent_ingroup(&int_to_char(123444)) == false);
        assert!(has_adjacent_ingroup(&int_to_char(111122)));
    }
}

fn main() {
    advent::init_logging();
    let mut range = String::new();
    io::stdin()
        .read_line(&mut range)
        .expect("Failed to read input !");
    let m = min_max(range);
    debug!("min: {} max: {}", m.0, m.1);
    let passwords = filter_passwords(m.0, m.1);
    info!("Passwords: {}", passwords.len());
    // Part 2
    let passwords = filter_passwords_p2(m.0, m.1);
    info!("Part 2 passwords: {}", passwords.len());
}
