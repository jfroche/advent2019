use log::*;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
struct Counter<V>
where
    V: Eq + Hash,
{
    map: HashMap<V, u32>,
}

impl<V> Counter<V>
where
    V: Eq + Hash,
{
    fn new<T: Iterator<Item = V>>(iter: T) -> Counter<V> {
        let mut hm = HashMap::new();
        for v in iter {
            let counter = hm.entry(v).or_insert(0);
            *counter += 1;
        }
        Counter { map: hm }
    }

    fn get(&self, item: V) -> u32 {
        *self.map.get(&item).expect("Cannot find item")
    }
}

impl<T: Hash + Eq> std::ops::Deref for Counter<T> {
    type Target = HashMap<T, u32>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<T: Hash + Eq> std::iter::FromIterator<T> for Counter<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Counter::new(iter.into_iter())
    }
}

#[derive(Debug)]
struct Layer {
    width: usize,
    height: usize,
    code: Vec<u32>,
    count: Counter<u32>,
}

impl Layer {
    fn new(code: &Vec<u32>, width: usize, height: usize) -> Self {
        Self {
            code: code.clone(),
            count: Counter::new(code.iter().map(|c| *c)),
            height,
            width,
        }
    }

    fn count_item(&self, item: u32) -> u32 {
        self.count.get(item)
    }
}

fn create_layers(input: String, width: usize, height: usize) -> Vec<Layer> {
    let layer_size = width * height;
    input
        .chars()
        .collect::<Vec<_>>()
        .chunks(layer_size)
        .map(|l| {
            Layer::new(
                &l.iter()
                    .map(|c| c.to_digit(10).expect("unable to parse int"))
                    .collect(),
                width,
                height,
            )
        })
        .collect::<Vec<Layer>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_instruction() {
        let layer = Layer::new(&vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2], 3, 2);
        assert_eq!(layer.count_item(1), 2);
        assert_eq!(layer.count_item(9), 1);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "day8", about = "Advent of Code - Day 8")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Width of the Image
    #[structopt(short, long)]
    width: usize,

    /// Height of the image
    #[structopt(short, long)]
    height: usize,
}

fn main() {
    advent::init_logging();
    let opt = Opt::from_args();
    let buffer: String = fs::read_to_string(opt.input)
        .expect("Unable to read input file")
        .replace("\n", "");
    let mut layers = create_layers(buffer, opt.width, opt.height);
    layers.sort_by(|a, b| a.count_item(0 as u32).cmp(&b.count_item(0)));
    let selected_layer = layers.first().expect("cannot find layer");
    info!(
        "part one: {:?}",
        selected_layer.count_item(1) * selected_layer.count_item(2)
    );
}
