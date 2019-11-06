use std::fmt::{Display, Error, Formatter};
use std::f64::{MIN, MIN_POSITIVE};
use rand::{SeedableRng, Rng};
use statrs::prec::F64_PREC;
use std::cmp::Ordering::Equal;

use crate::game::{*, instructions::*};
use crate::constants::{NOGRAM, _N};

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Leaf {
    source: Source,
    samples: f64,
    accumulator: f64,
    dev: f64,
    divider: f64,
    correction: f64,
}

impl Default for Leaf {
    fn default() -> Leaf {
        Leaf { source: NOGRAM, samples: 1.0, accumulator: MIN_POSITIVE, dev: MIN_POSITIVE, divider: 1.0, correction: 1.0 }
    }
}

impl Display for Leaf {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "chance: {}%, mean: {}, dev:{}, samples: {}, divider: {}, source: {}", self.chance() * 100.0, self.mean(), self.std_dev(), self.samples, self.divider, self.source)
    }
}

impl Leaf {
    fn push(&mut self, newscore: f64) {
        let lastmean = self.mean();
        self.accumulator += newscore;
        self.samples += 1.0;
        self.dev += (newscore - self.mean()) * (newscore - lastmean);

// rolling std dev implementation: https://www.johndcook.com/blog/standard_deviation/
    }
    fn mean(&self) -> f64 {
        self.accumulator / self.samples
    }
    fn divided_mean(&self) -> f64 {
        self.mean() / self.divider
    }
    fn variance(&self) -> f64 {
        if self.samples > 1.5 { self.dev / (self.samples - 1.0) } else { MIN_POSITIVE }
    }
    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
    fn uncorrected_chance(&self) -> f64 {
//        let nml = Normal::new(self.mean(), self.std_dev());
//        return (1.0 - nml.unwrap().cdf(0.5)) / self.divider;
//        let alpha = ((1.0 - self.mean()) / self.variance() - 1.0 / self.mean()) * self.mean().powi(2);
//        let beta = alpha * (1.0 / self.mean() - 1.0);
//        let default = Beta::new(1.0, 4.0).unwrap();
//        let bet = Beta::new(alpha, beta).unwrap_or(Beta::new(1.0, 4.0).unwrap());
//        if bet == default {
//            println!("bad: al: {},\t bet: {},\t m: {},\t v: {},\t s: {}", alpha, beta, self.mean(), self.variance(), self.std_dev());
//        } else {
//            println!("GUD: al: {},\t bet: {},\t m: {},\t v: {},\t s: {}", alpha, beta, self.mean(), self.variance(), self.std_dev());
//        }
//        return (1.0 - bet.cdf(0.999)) / self.divider;// + F64_PREC;
        self.divided_mean()
    }
    fn chance(&self) -> f64 {
        self.uncorrected_chance() / self.correction
    }
    fn children(&self, source: Source, branch_factor: f64) -> Leaf {
        Leaf {
            source,
            accumulator: self.accumulator / branch_factor,
            samples: self.samples / branch_factor,
            divider: self.divider * branch_factor,
            dev: self.dev / branch_factor,
            correction: self.correction,
        }
    }
}

pub fn carlo(puzzle: &Puzzle, max_iters: i32, expansions: i32) -> Option<Source> {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(1337);
    let instruction_set = puzzle.get_ins_set(INS_COLOR_MASK, true);
    let mut stems: Vec<Leaf> = vec![Leaf { accumulator: 1.0, ..Leaf::default() }];
    let mut bestboi = Leaf { accumulator: MIN, ..Leaf::default() };
    let mut bestsource = Leaf::default();
    for _expansion in 0..expansions {
        let leaf_to_branch = stems.first().unwrap().clone();
        branches(&mut stems, puzzle, &instruction_set, &leaf_to_branch);
        let correction: f64 = F64_PREC + stems.iter().map(|l| l.uncorrected_chance()).sum::<f64>();
//        println!("correction: {}", correction);
        if correction == 0.0 {
            panic!("correction = 0!");
        }
//        println!("stems: {}", stems.len());
        let mut _counter = 0;
        let mut newbest = false;
        for stem in &mut stems {
            stem.correction = correction;
            let bonus = 64.0 * ((stem.samples < 64.0) as i64 as f64);
            for _iteration in 0..(bonus + rng.gen_range(-0.5, 0.5) + max_iters as f64 * stem.chance()).round() as usize {
                _counter += 1;
                let fullgram = random_program(puzzle, &stem.source, &instruction_set, &mut rng);
                let newscore = puzzle.execute(&fullgram, false, score);
                if newscore > bestboi.accumulator {
                    bestsource = stem.clone();
                    bestboi = Leaf { source: fullgram, samples: 1.0, accumulator: newscore, ..Leaf::default() };
                    newbest = true;
                    if newscore >= 1.0 {
                        return Some(bestsource.source);
                    }
                }
                stem.push(newscore);
            }
        }
        if newbest {
            branches(&mut stems, puzzle, &instruction_set, &bestsource);
        }
        stems.sort_unstable_by(|a, b| b.source.partial_cmp(&a.source).unwrap_or(Equal));
        stems.dedup_by(|a, b| {
            if a.source == b.source {
                b.accumulator = a.accumulator + b.accumulator;
                b.samples = a.samples + b.samples;
                b.dev = a.dev + b.dev;
                true
            } else { false }
        });
//        println!("counter: {}", counter);
        stems.sort_unstable_by(|a, b| b.chance().partial_cmp(&a.chance()).unwrap_or(Equal));
        let lastboi = stems.first().unwrap();
        println!(" length: {}, lastboi: {}", stems.len(), lastboi);
    }
    for souce in &stems {
        println!(" {}", souce);
    }
    print!("bestboi: ");
    print!(" {}\nbestsource: ", bestboi);
    println!(" {}", bestsource);
    return None;
}

pub fn branches(tree: &mut Vec<Leaf>, puzzle: &Puzzle, instruction_set: &Vec<Ins>, leaf: &Leaf) {
    tree.remove_item(leaf);
    let mut branch_factor = 0.0;
    for i in 0..puzzle.methods.len() {
        for j in 0..puzzle.methods[i] {
            if leaf.source[i][j] == NOP {
                branch_factor += instruction_set.len() as f64;
                break;
            }
        }
    }
    for i in 0..puzzle.methods.len() {
        for j in 0..puzzle.methods[i] {
            if leaf.source[i][j] == NOP {
                for instruction in instruction_set {
                    let mut temp = leaf.source;
                    temp[i][j] = *instruction;
                    tree.push(leaf.children(temp.to_owned(), branch_factor));
                }
                break;
            }
        }
    }
}

pub fn random_program(puzzle: &Puzzle, base: &Source, instruction_set: &Vec<Ins>, mut _rng: impl Rng) -> Source {
    let  fullgram = *base;
    for i in 0..puzzle.methods.len() {
        for j in 0..puzzle.methods[i] {
//            let mask = (fullgram[i][j] != NOP) as u8;
//            fullgram[i][j].0 = fullgram[i][j].0 * mask + (1 - mask) * u8::from(*instruction_set.choose(&mut rng).unwrap_or(&NOP));
        }
    }
    return fullgram;
}

//pub fn score(state: &State, puzzle: &Puzzle) -> f64 {
//    let mut touched = 0.0;
//    let mut stars = 0.0;
//    for y in 1..13 {
//        for x in 1..17 {
//            touched += state.map[y][x].is_touched() as i32 as f64;
//            stars += state.map[y][x].has_star() as i32 as f64;
//        }
//    }
//    let tiles = 12.0 * 16.0 + 1.0;
//    return ((puzzle.stars as f64 - stars) + (touched / tiles) + ((MAX_STEPS - state.steps) as f64) / MAX_STEPS as f64) / puzzle.stars as f64;
//}

pub fn score_cmp(state: &State, puzzle: &Puzzle) -> usize {
    let mut touched = 0;
    let mut stars = 0;
    let mut tiles = 1;
    for y in 1..13 {
        for x in 1..17 {
            tiles += (state.map[y][x] != _N) as usize;
            touched += state.map[y][x].touched() as usize;
            stars += state.map[y][x].has_star() as usize;
        }
    }
    return (puzzle.stars - stars) * tiles * (MAX_STEPS + 1)
        + touched * (MAX_STEPS + 1)
        + MAX_STEPS - state.steps;
}
pub fn score(state: &State, puzzle: &Puzzle) -> f64 {
    let mut touched = 0;
    let mut stars = 0;
    let mut tiles = 1;
    for y in 1..13 {
        for x in 1..17 {
            tiles += (state.map[y][x] != _N) as usize;
            touched += (state.map[y][x].touched() > 0) as usize;
            stars += state.map[y][x].has_star() as usize;
        }
    }
    return (((puzzle.stars - stars) * tiles * (MAX_STEPS + 1)
        + touched * (MAX_STEPS + 1)
        + MAX_STEPS - state.steps) as f64) / ((puzzle.stars * tiles * (MAX_STEPS + 1)) as f64);
}

//pub fn score(state: &State, puzzle: &Puzzle) -> f64 {
//    return (score_cmp(state, puzzle) as f64) / (puzzle.stars as f64);
//}