use log::*;
use rust_decimal::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "day10", about = "Advent of Code - Day 10")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Clone)]
struct AsteroidMap {
    asteroids: Vec<Asteroid>,
}

impl AsteroidMap {
    fn new(str_map: String) -> Self {
        Self {
            asteroids: str_map
                .lines()
                .enumerate()
                .flat_map(|(i, l)| {
                    l.chars()
                        .enumerate()
                        .filter(|(_, c)| *c == '#')
                        .map(move |(j, _)| Asteroid {
                            x: j,
                            y: i,
                            ..Default::default()
                        })
                })
                .collect(),
        }
    }

    fn count(&self) -> usize {
        self.asteroids.len()
    }

    fn vaporize(&mut self, asteroid: Asteroid) {
        debug!("vaporize: {:?}", asteroid);
        self.asteroids.retain(|x| *x != asteroid);
    }

    fn best_monitoring_station(&self) -> (usize, &Asteroid) {
        let mut best = &Asteroid {
            x: usize::MAX,
            y: usize::MAX,
        };
        let mut best_score = 0;
        for a in &self.asteroids {
            debug!("{:?}", a);
            let h = a.adjacent_asteroids(self.asteroids.clone());
            if h.len() > best_score {
                best_score = h.len();
                best = a;
            }
        }
        (best_score, best)
    }
}

#[derive(Default, Eq, Debug, Clone, Copy)]
struct Asteroid {
    x: usize,
    y: usize,
}

impl PartialEq for Asteroid {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Asteroid {
    fn distance(&self, o: &Asteroid) -> usize {
        ((self.x as i32 - o.x as i32).abs() + (self.y as i32 - o.y as i32).abs()) as usize
    }

    fn angle(&self, o: &Asteroid) -> Decimal {
        let x1 = (o.x as i32 - self.x as i32) as f64;
        let y1 = (o.y as i32 - self.y as i32) as f64;
        let mut d: f64 = y1.atan2(x1).to_degrees().into();
        if d < 0.0 {
            d = d + 360.0;
        }
        Decimal::from_f64(d).unwrap()
    }

    fn adjacent_asteroids(&self, asteroids: Vec<Asteroid>) -> HashMap<Decimal, Asteroid> {
        let mut m: HashMap<Decimal, Asteroid> = HashMap::new();
        for a in asteroids {
            if a == *self {
                continue;
            }
            let d: Decimal = self.angle(&a);
            match m.get_mut(&d) {
                Some(x) => {
                    if self.distance(&a) < self.distance(&x) {
                        *x = a;
                    }
                },
                None => {
                    m.insert(d, a);
                },
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_map() {
        let a_map = AsteroidMap::new("#..#".to_string());
        assert_eq!(a_map.count(), 2);
        assert_eq!(a_map.asteroids[0].x, 0);
        assert_eq!(a_map.asteroids[0].y, 0);
        assert_eq!(a_map.asteroids[1].x, 3);
        assert_eq!(a_map.asteroids[1].y, 0);
        let b_map = AsteroidMap::new("#..#\n#...".to_string());
        assert_eq!(b_map.count(), 3);
        assert_eq!(b_map.asteroids[2].x, 0);
        assert_eq!(b_map.asteroids[2].y, 1);
    }
    #[test]
    fn test_asteroid_angle() {
        assert_eq!(
            Asteroid { x: 0, y: 0 }.angle(&Asteroid { x: 0, y: 1 }),
            Decimal::from_str("90.0").unwrap()
        );
        assert_eq!(
            Asteroid { x: 0, y: 0 }.angle(&Asteroid { x: 1, y: 1 }),
            Decimal::from_str("45.0").unwrap()
        );
        assert_eq!(
            Asteroid { x: 1, y: 1 }.angle(&Asteroid { x: 0, y: 0 }),
            Decimal::from_str("225.0").unwrap()
        );
        assert_eq!(
            Asteroid { x: 1, y: 1 }.angle(&Asteroid { x: 0, y: 1 }),
            Decimal::from_str("180.0").unwrap()
        );
    }

    #[test]
    fn test_asteroid_distance() {
        assert_eq!(
            Asteroid { x: 0, y: 0 }.distance(&Asteroid { x: 0, y: 1 }),
            1
        );
        assert_eq!(
            Asteroid { x: 0, y: 0 }.distance(&Asteroid { x: 10, y: 1 }),
            11
        )
    }

    #[test]
    fn test_adjacent_asteroids() {
        let m = Asteroid { x: 0, y: 0 }.adjacent_asteroids(vec![
            Asteroid { x: 1, y: 0 },
            Asteroid { x: 2, y: 0 },
            Asteroid { x: 0, y: 0 },
        ]);
        assert_eq!(m.len(), 1);
        // assert_eq!(*m.keys().next().unwrap(), Decimal::from_str("0.0").unwrap());
        assert_eq!(*m.values().next().unwrap(), Asteroid { x: 1, y: 0 });
    }

    #[test]
    fn test_vaporize() {
        advent::init_logging();
        let a_map = AsteroidMap::new(
            ".#....#####...#..
##...##.#####..##
##...#...#.#####.
..#.....#...###..
..#.#.....#....##"
                .to_string(),
        );
        let (_, best) = a_map.best_monitoring_station();
        let nineth = vaporize(a_map.clone(), *best, 9);
        assert_eq!(nineth, Asteroid { x: 15, y: 1 });
        let n2 = vaporize(a_map.clone(), *best, 18);
        assert_eq!(n2, Asteroid { x: 4, y: 4 });
        let last_vaporized = vaporize(a_map.clone(), *best, 200);
        assert_eq!(last_vaporized, Asteroid { x: 14, y: 3 })
    }
}

fn vaporize(mut m: AsteroidMap, a: Asteroid, mut count: usize) -> Asteroid {
    debug!("Vaporize from {:?}", a);
    let h = a.adjacent_asteroids(m.asteroids.clone());
    let mut v: Vec<Decimal> = h.keys().cloned().collect::<Vec<Decimal>>();
    v.sort();
    debug!("angles: {:?}", v);
    let p = v
        .iter()
        .position(|&x| x == Decimal::from_str("270.0").unwrap());
    match p {
        Some(x) => v.rotate_left(x),
        None => (),
    };
    for angle in v {
        debug!("{:?} at {:?} (size: {:?})", count, angle, m.count());
        let a = h[&angle];
        m.vaporize(a);
        count = count - 1;
        if count == 0 || m.count() == 1 {
            return a;
        }
    }
    return vaporize(m, a, count);
}

fn main() {
    advent::init_logging();
    let opt = Opt::from_args();
    let buffer: String = fs::read_to_string(opt.input).expect("Unable to read input file");
    let m = AsteroidMap::new(buffer);
    info!("Asteroid count: {:?}", m.count());
    let (best_score, best) = m.best_monitoring_station();
    info!("Asteroid best: {:?} with {:?}", best, best_score);
    let a = vaporize(m.clone(), *best, 200);
    info!("200th asteroid vaporized: {:?}", a);
    info!("Answer part 2: {:?}", a.x * 100 + a.y)
}
