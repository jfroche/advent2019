use log::*;
use petgraph::algo::dijkstra;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::*;
use petgraph::Graph;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

// struct OrbitMap {
// orbits: Vec<(String, String)>,
// pub map: DiGraphMap<Node, ()>,
// }
//
// impl OrbitMap {
// fn new(orbits: Vec<(String, String)>) -> Self {
// Self {
// map: DiGraphMap::new(),
// orbits,
// }
// }

// fn build(mut self) {
// let mut orbits: Vec<String> = std::mem::replace(&mut self.orbits, Vec::new());
// for item in orbits {
// let i: Vec<_> = item.split(')').collect();
// self.map.add_edge(i[0], i[1], ());
// }
// }
// fn new(code: String) -> Self {
// let mut g = DiGraphMap::new();
// let c = code.clone();
// let mut omap = OrbitMap { map: g, code: c };
// for item in omap.code.split('\n') {
// let i: Vec<_> = item.split(')').collect();
// omap.map.add_edge(i[0], i[1], ());
// }
// omap
// }
//}

fn run(code: &String) {
    let codes: Vec<(String, String)> = code
        .split('\n')
        .filter(|c| *c != "")
        .map(|c| {
            debug!("c: {:?}", c);
            let o: Vec<_> = c.split(')').collect();
            (o[0].to_string(), o[1].to_string())
        })
        .collect();
    // let mut map = OrbitMap::new(codes);
    let mut g = DiGraphMap::new();
    for ref item in &codes {
        g.add_edge(item.0.as_str(), item.1.as_str(), ());
    }
    let g: DiGraph<&str, ()> = g.into_graph();
    debug!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));
    let node_map = dijkstra(&g, 349.into(), None, |_| 1);
    debug!("{:?}", node_map.values().sum::<i32>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_map() {
        advent::init_logging();
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
        run(&map_string);
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
    run(&buffer);
}
