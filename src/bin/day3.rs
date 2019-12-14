//! Solution for Advent of Code 2019 Day 3 (https://adventofcode.com/2019/day/3)
//!
//! We define 3 structs:
//!   - Point
//!   - Coordinates
//!   - Wire

use log::*;
use regex::Regex;
use std::cmp::Ordering;
use std::cmp::PartialOrd;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::BufRead;
use std::iter::FromIterator;
use std::ops::Add;
use std::ops::BitAnd;

/// A point in 2D
#[derive(Clone, Debug, Copy)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    /// Returns a point based on its coordinates
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    /// Returns true if the point is on the origin
    pub fn is_origin(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    pub fn distance(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Point) -> Ordering {
        self.distance().cmp(&other.distance())
    }
}
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl Eq for Point {}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Clone, Debug)]
struct Coordinates(Vec<Point>);

impl Coordinates {
    fn new() -> Self {
        Coordinates(vec![])
    }

    fn last(&self) -> Option<&Point> {
        self.0.last().clone()
    }

    fn iter(&self) -> IterCoordinates {
        IterCoordinates(Box::new(self.0.iter()))
    }

    pub fn push(&mut self, value: Point) {
        self.0.push(value)
    }
}
impl From<Coordinates> for HashSet<Point> {
    fn from(item: Coordinates) -> Self {
        let mut h = HashSet::new();
        for p in item.iter() {
            h.insert(*p);
        }
        h
    }
}

impl From<String> for Coordinates {
    fn from(direction: String) -> Self {
        let re = Regex::new(r"(L|R|U|D)([0-9]+)").unwrap();
        let caps = re.captures(direction.as_str()).unwrap();
        let orientation = caps.get(1).map_or("", |m| m.as_str());
        let distance: i32 = caps
            .get(2)
            .expect("Invalid format")
            .as_str()
            .parse()
            .expect("Invalid format");
        match orientation {
            "L" => {
                debug!("{} {}", orientation, distance);
                (1..distance + 1)
                    .map(|i| Point { x: -i, y: 0 })
                    .collect::<Coordinates>()
                //)
            },
            "R" => {
                debug!("{} {}", orientation, distance);
                (1..distance + 1)
                    .map(|i| Point { x: i, y: 0 })
                    .collect::<Coordinates>()
            },
            "U" => {
                debug!("{} {}", orientation, distance);
                (1..distance + 1)
                    .map(|i| Point { x: 0, y: i })
                    .collect::<Coordinates>()
            },
            "D" => {
                debug!("{} {}", orientation, distance);
                (1..distance + 1)
                    .map(|i| Point { x: 0, y: -i })
                    .collect::<Coordinates>()
            },
            _ => panic!("Invalid orientation"),
        }
    }
}

impl<'a> Extend<&'a Point> for Coordinates {
    fn extend<T: IntoIterator<Item = &'a Point>>(&mut self, iter: T) {
        let default_origin = &Point::new(0, 0);
        let mut new_points = Vec::new();
        let last_coordinates: &Point = self.last().unwrap_or(default_origin);
        for elem in iter {
            if elem.is_origin() {
                continue;
            }
            debug!("{} + {}", last_coordinates, *elem);
            let p = *last_coordinates + *elem;
            debug!("adding {}", p);
            new_points.push(p);
        }
        self.0.extend(new_points);
    }
}

// You can create a new struct which will contain a reference to your set of data.
struct IterCoordinates<'a>(Box<dyn Iterator<Item = &'a Point> + 'a>);

// Now you can just implement the `Iterator` trait on your `IterNewType` struct.
impl<'a> Iterator for IterCoordinates<'a> {
    type Item = &'a Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Coordinates {
    type IntoIter = IterCoordinates<'a>;
    type Item = &'a Point;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl BitAnd for Coordinates {
    type Output = Coordinates;

    fn bitand(self, rhs: Self) -> Self::Output {
        let lhs = HashSet::from(self);
        let rhs = HashSet::from(rhs);
        Coordinates(lhs.intersection(&rhs).cloned().collect())
    }
}

impl FromIterator<Point> for Coordinates {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Point>,
    {
        let mut coordinates = Coordinates::new();
        for v in iter {
            coordinates.push(v.clone());
        }
        coordinates
    }
}

#[derive(Clone, Debug)]
struct Wire {
    path: Vec<String>,
    coordinates: Coordinates,
}

impl Wire {
    fn new(c: Coordinates) -> Self {
        Self {
            path: Vec::new(),
            coordinates: c,
        }
    }

    fn step_to_intersection(&self, i: Point) -> usize {
        self.coordinates
            .iter()
            .take_while(|p| **p != i)
            .map(|i| {
                debug!("{}", i);
                i
            })
            .count()
            + 1
    }
}

impl fmt::Display for Wire {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?})", self.path)
    }
}

impl FromIterator<Point> for Wire {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Point>,
    {
        let mut coordinates = Coordinates::new();
        for v in iter {
            coordinates.push(v.clone());
        }
        Wire::new(coordinates)
    }
}

impl FromIterator<String> for Wire {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        let mut path = Vec::new();
        let mut path_coordinates = Coordinates::new();
        for v in iter {
            path.push(v.clone());
            path_coordinates.extend(Coordinates::from(v).iter());
        }
        Wire {
            path,
            coordinates: path_coordinates,
        }
    }
}

impl BitAnd for Wire {
    type Output = Wire;

    fn bitand(self, rhs: Self) -> Self::Output {
        Wire::new(self.coordinates.clone() & rhs.coordinates.clone())
    }
}

fn common_points(wires: Vec<Wire>) -> Option<Coordinates> {
    let mut iter = wires.iter();
    match iter
        .next()
        .map(|set| iter.fold((*set).clone(), |acc, set| acc.clone() & (*set).clone()))
    {
        None => None,
        Some(w) => Some(w.coordinates),
    }
}

fn closest_intersection_point(wires: Vec<Wire>) -> Option<Point> {
    let common = common_points(wires);
    info!("common points: {:?}", common);
    match common {
        None => None,
        Some(c) => c.iter().fold(None, |min, x| match min {
            None => Some(*x),
            Some(y) => Some(if x < &y { *x } else { y }),
        }),
    }
}

fn create_wire_from_string(s: String) -> Wire {
    s.split(',')
        .flat_map(|s| s.trim().parse::<String>().ok())
        .collect::<Wire>()
}

fn minimal_steps_to_intersection(wires: Vec<Wire>) -> Option<usize> {
    common_points(wires.clone())
        .expect("no common points")
        .iter()
        .map(|p| {
            wires
                .iter()
                .fold(0, |sum, w| sum + w.step_to_intersection(*p))
        })
        .min()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_examples() {
        let wires: Vec<Wire> = vec![
            create_wire_from_string("R75,D30,R83,U83,L12,D49,R71,U7,L72".to_string()),
            create_wire_from_string("U62,R66,U55,R34,D71,R55,D58,R83".to_string()),
        ];
        assert_eq!(
            closest_intersection_point(wires)
                .expect("Cannot find intersection")
                .distance(),
            159
        );
        let wires: Vec<Wire> = vec![
            create_wire_from_string("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51".to_string()),
            create_wire_from_string("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".to_string()),
        ];
        assert_eq!(
            closest_intersection_point(wires)
                .expect("Cannot find intersection")
                .distance(),
            135
        );
    }

    #[test]
    fn test_step_count() {
        advent::init_logging();
        let w1 = create_wire_from_string("R8,U5,L5,D3".to_string());
        assert_eq!(w1.step_to_intersection(Point::new(3, 0)), 3);
        assert_eq!(w1.step_to_intersection(Point::new(3, 3)), 20);
    }
    #[test]
    fn test_minimal_steps_to_intersection() {
        let wires: Vec<Wire> = vec![
            create_wire_from_string("R8,U5,L5,D3".to_string()),
            create_wire_from_string("U7,R6,D4,L4".to_string()),
        ];
        assert_eq!(minimal_steps_to_intersection(wires), Some(30));

        let wires: Vec<Wire> = vec![
            create_wire_from_string("R75,D30,R83,U83,L12,D49,R71,U7,L72".to_string()),
            create_wire_from_string("U62,R66,U55,R34,D71,R55,D58,R83".to_string()),
        ];
        assert_eq!(minimal_steps_to_intersection(wires), Some(610));

        let wires: Vec<Wire> = vec![
            create_wire_from_string("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51".to_string()),
            create_wire_from_string("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".to_string()),
        ];
        assert_eq!(minimal_steps_to_intersection(wires), Some(410));
    }
}

fn main() {
    advent::init_logging();
    let wires: Vec<Wire> = io::stdin()
        .lock()
        .lines()
        .filter_map(|l| l.ok())
        .map(|line| create_wire_from_string(line))
        .collect();
    match closest_intersection_point(wires.clone()) {
        None => info!("no intersection"),
        Some(c) => {
            info!("shortest intersection: {}", c);
            info!("distance: {}", c.distance());
        },
    }
    // Part two
    info!(
        "minimal steps to intersection: {:?}",
        minimal_steps_to_intersection(wires)
    );
}
