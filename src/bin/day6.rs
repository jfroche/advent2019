use log::*;
use petgraph::algo::dijkstra;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::*;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

struct OrbitMap<'a> {
    pub map: Graph<&'a str, ()>,
}

impl<'a> OrbitMap<'a> {
    fn new(g: DiGraphMap<&'a str, ()>) -> Self {
        Self {
            map: g.into_graph(),
        }
    }

    fn distance(&self, origin: &str, destination: Option<&str>) -> usize {
        let origin_node = self
            .map
            .node_indices()
            .find(|i| self.map[*i] == origin)
            .unwrap();

        let destination_node = match destination {
            Some(d) => self.map.node_indices().find(|i| self.map[*i] == d),
            None => None,
        };
        debug!(
            "Trying to find path between {:?} and {:?}",
            origin_node, destination_node
        );
        let values = dijkstra(&self.map, origin_node, destination_node, |_| 1);
        debug!("{:?}", values.clone());
        if destination == None {
            let values = values.values();
            values.sum::<usize>()
        } else {
            values[&destination_node.expect("unable to find destination")]
        }
    }

    #[allow(dead_code)]
    fn draw(&self) -> Dot<&Graph<&str, ()>> {
        Dot::with_config(&self.map, &[Config::EdgeNoLabel])
    }
}

fn distance(code: &String, origin: &str, destination: Option<&str>) -> usize {
    let codes: Vec<(String, String)> = code
        .split('\n')
        .filter(|c| *c != "")
        .map(|c| {
            let o: Vec<_> = c.split(')').collect();
            (o[0].to_string(), o[1].to_string())
        })
        .collect();
    let mut g = DiGraphMap::new();
    for ref item in &codes {
        g.add_edge(item.0.as_str(), item.1.as_str(), ());
        g.add_edge(item.1.as_str(), item.0.as_str(), ());
    }
    OrbitMap::new(g).distance(origin, destination)
}

fn part_one(code: &String) -> usize {
    distance(code, "COM", None)
}

fn part_two(code: &String) -> usize {
    distance(code, "SAN", Some("YOU")) - 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_map() {
        let map_string = "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L"
        .to_string();
        assert_eq!(distance(&map_string, "COM", None), 42);
        assert_eq!(distance(&map_string, "B", Some("E")), 3);
    }

    #[test]
    fn test_shortest_path() {
        let map_string = "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
K)YOU
I)SAN"
            .to_string();
        assert_eq!(distance(&map_string, "YOU", Some("SAN")), 6);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "day6", about = "Advent of Code - Day 6")]
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
    let buffer = fs::read_to_string(opt.input).expect("Unable to read input file");
    debug!("{:?}", buffer);
    let sum = part_one(&buffer);
    info!("part one: {}", sum);
    let sum = part_two(&buffer);
    info!("part two: {}", sum);
}
