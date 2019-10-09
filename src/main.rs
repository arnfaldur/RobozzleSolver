#![feature(test, vec_remove_item, core_intrinsics)]
#![allow(dead_code, ellipsis_inclusive_range_patterns)]

use colored::*;
use std::fmt::{Display, Error, Formatter};
use rand::{Rng, SeedableRng};
use std::f64::{MIN, MIN_POSITIVE};
use std::cmp::Ordering::Equal;
use std::time::Instant;
use rand::prelude::SliceRandom;
use statrs::distribution::{Normal, InverseCDF, Univariate, Continuous, Beta};
use statrs::prec::F64_PREC;

type Tile = u8;

type Map = [[Tile; 18]; 14];

type Instruction = u8;
type Method = [Instruction; 10];
type Source = [Method; 5];

const STACK_SIZE: usize = 2048;
const MAX_STEPS: usize = 2048;

struct Stack {
    pointer: usize,
    data: [Instruction; STACK_SIZE],
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

struct Puzzle {
    map: Map,
    direction: Direction,
    x: usize,
    y: usize,
    stars: usize,
    functions: [usize; 5],
    marks: [bool; 3],
    red: bool,
    green: bool,
    blue: bool,
}

struct State {
    running: bool,
    steps: usize,
    stars: usize,
    stack: Stack,
    map: Map,
    direction: Direction,
    x: usize,
    y: usize,
}

const RE: Tile = 0b00001;
const GE: Tile = 0b00010;
const BE: Tile = 0b00100;
const RS: Tile = 0b01001;
const GS: Tile = 0b01010;
const BS: Tile = 0b01100;
const _N: Tile = 0b10000;

const TILE_STAR_MASK: Tile = 0b00001000;
const TILE_COLOR_MASK: Tile = 0b00010111;

const TILE_TOUCHED: Tile = 0b00100000;

const INS_FORWARD: Instruction = 0;
const INS_TURN_LEFT: Instruction = 1;
const INS_TURN_RIGHT: Instruction = 2;
const INS_F1: Instruction = 3;
const INS_F2: Instruction = 4;
const INS_F3: Instruction = 5;
const INS_F4: Instruction = 6;
const INS_F5: Instruction = 7;
const INS_MARK_RED: Instruction = 0b00001001;
const INS_MARK_GREEN: Instruction = 0b00001010;
const INS_MARK_BLUE: Instruction = 0b00001100;

const INS_MARK_MASK: Instruction = 0b00000111;
const NOP: Instruction = 0b00010000;
const INS_INS_MASK: Instruction = 0b00011111;

const INS_GRAY_COND: Instruction = 0b00000000;
const INS_RED_COND: Instruction = 0b00100000;
const INS_GREEN_COND: Instruction = 0b01000000;
const INS_BLUE_COND: Instruction = 0b10000000;
const INS_COLOR_MASK: Instruction = 0b11100000;

const INSTRUCTIONS: [Instruction; 11] = [
    INS_FORWARD,
    INS_TURN_LEFT,
    INS_TURN_RIGHT,
    INS_F1,
    INS_F2,
    INS_F3,
    INS_F4,
    INS_F5,
    INS_MARK_RED,
    INS_MARK_GREEN,
    INS_MARK_BLUE,
];
const MOVES: [Instruction; 3] = [
    INS_FORWARD,
    INS_TURN_LEFT,
    INS_TURN_RIGHT,
];
const FUNCTIONS: [Instruction; 5] = [
    INS_F1,
    INS_F2,
    INS_F3,
    INS_F4,
    INS_F5,
];
const MARKS: [Instruction; 3] = [
    INS_MARK_RED,
    INS_MARK_GREEN,
    INS_MARK_BLUE,
];

const NOGRAM: Source = [[NOP; 10]; 5];

fn touch(tile: &mut Tile) {
    *tile |= TILE_TOUCHED;
}

fn clear_star(tile: &mut Tile) {
    *tile &= !TILE_STAR_MASK;
}

fn is_touched(tile: Tile) -> bool {
    return tile & TILE_TOUCHED > 0;
}

fn has_star(tile: Tile) -> bool {
    return tile & TILE_STAR_MASK > 0;
}

fn source_index(ins: Instruction) -> usize {
    return (ins - INS_F1) as usize;
}

fn left(direction: &Direction) -> Direction {
    return match direction {
        Direction::Up => Direction::Left,
        Direction::Left => Direction::Down,
        Direction::Down => Direction::Right,
        Direction::Right => Direction::Up,
    };
}

fn right(direction: &Direction) -> Direction {
    return match direction {
        Direction::Up => Direction::Right,
        Direction::Left => Direction::Up,
        Direction::Down => Direction::Left,
        Direction::Right => Direction::Down,
    };
}

fn invoke(stack: &mut Stack, method: &Method) {
    for ins in method.iter().rev() {
        if *ins != NOP {
            stack.pointer += 1;
            stack.data[stack.pointer] = *ins;
        }
    }
}

fn step(state: &mut State, source: &Source) -> bool {
    let instruction: Instruction = state.stack.data[state.stack.pointer] & INS_INS_MASK;
    let color: u8 = (state.stack.data[state.stack.pointer] & INS_COLOR_MASK) >> 5;
    state.stack.pointer -= 1;
    state.steps += 1;
    let map_color: u8 = state.map[state.y][state.x] & TILE_COLOR_MASK;
    if color != 0 && map_color != 0 && color != map_color {
        return state.stack.pointer > 0;
    }
    match instruction {
        INS_FORWARD => {
            match &state.direction {
                Direction::Up => state.y -= 1,
                Direction::Left => state.x -= 1,
                Direction::Down => state.y += 1,
                Direction::Right => state.x += 1,
            };
            let tile = &mut state.map[state.y][state.x];
            if *tile == _N {
                return false;
            }
            state.stars -= has_star(*tile) as usize;
            clear_star(tile);
            touch(tile);
        }
        INS_TURN_LEFT => state.direction = left(&state.direction),
        INS_TURN_RIGHT => state.direction = right(&state.direction),
        INS_F1...INS_F5 => invoke(&mut state.stack, &source[source_index(instruction)]),
        INS_MARK_RED...INS_MARK_BLUE => state.map[state.y][state.x] = instruction | TILE_TOUCHED,
        _ => (),
    }
    return state.stack.pointer > 0;
}

fn execute(puzzle: &Puzzle, source: &Source) -> f64 {
    let mut state: State = State {
        running: true,
        steps: 0,
        stars: puzzle.stars,
        map: puzzle.map,
        stack: Stack {
            pointer: 0,
            data: [NOP; STACK_SIZE],
        },
        direction: puzzle.direction,
        x: puzzle.x,
        y: puzzle.y,
    };
    invoke(&mut state.stack, &source[0]);
    while state.running && state.stars > 0 && state.stack.pointer < STACK_SIZE - 12 && state.steps < MAX_STEPS {
//        print!("{}\n------------------------------------------\n", state);
        state.running = step(&mut state, &source);
    }
//    print!("{}\nscore: {}", state, score(&state));
    return score(&state, puzzle);
}

#[derive(Clone, PartialEq, Debug)]
struct Leaf {
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
        write!(f, "chance: {}%, mean: {}, dev:{}, samples: {}, divider: {}", self.chance() * 100.0, self.mean(), self.std_dev(), self.samples, self.divider)
    }
}

impl Leaf {
    fn push(&mut self, newscore: f64) {
//        let lastmean = self.mean();
        self.accumulator += newscore;
        self.samples += 1.0;
//        self.dev += (newscore - self.mean()) * (newscore - lastmean);

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
//        return (1.0 - nml.unwrap().cdf(0.999)) / self.divider;
//        let alpha = ((1.0 - self.mean()) / self.variance() - 1.0 / self.mean()) * self.mean().powi(2);
//        let beta = alpha * (1.0 / self.mean() - 1.0);
//        let bet = Beta::new(alpha, beta).unwrap_or(Beta::new(1.0, 10.0).unwrap());
//        return (1.0 - bet.cdf(0.999)) / self.divider + F64_PREC;
//        self.divided_mean() / self.correction
        return self.divided_mean();
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
            dev: self.dev,
            correction: self.correction,
        }
    }
}

fn carlo(puzzle: &Puzzle, max_iters: i32, expansions: i32) {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(1337);
    let instruction_set = get_instruction_set(puzzle);
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
        let mut counter = 0;
        for stem in &mut stems {
            stem.correction = correction;
            for _iteration in 0..(rng.gen_range(-0.5, 0.5) + max_iters as f64 * stem.chance()).round() as usize {
                counter += 1;
                let fullgram = random_program(puzzle, &stem.source, &instruction_set, &mut rng);
                let newscore = execute(puzzle, &fullgram);
                if newscore > bestboi.accumulator {
                    bestsource = stem.clone();
                    bestboi = Leaf { source: fullgram, samples: 1.0, accumulator: newscore, ..Leaf::default() };
                }
                stem.push(newscore);
            }
        }
        println!("counter: {}", counter);
        stems.sort_unstable_by(|a, b| b.chance().partial_cmp(&a.chance()).unwrap_or(Equal));
        let lastboi = stems.first().unwrap();
        show_source(&lastboi.source);
        println!(" length: {}, lastboi: {}", stems.len(), lastboi);
    }
    for souce in &stems {
        show_source(&souce.source);
        println!(" {}", souce);
    }
    print!("bestboi: ");
    show_source(&bestboi.source);
    print!(" {}\nbestsource: ", bestboi);
    show_source(&bestsource.source);
    println!(" {}", bestsource);
}

fn main() {
    let now = Instant::now();
    carlo(&PUZZLE_656, 1 << 16, 1 << 8);
//    show_source(&TEST_SOURCE);
    println!("The solver took {} seconds.", now.elapsed().as_secs_f64());
}

fn branches(tree: &mut Vec<Leaf>, puzzle: &Puzzle, instruction_set: &Vec<Instruction>, leaf: &Leaf) {
    tree.remove_item(leaf);
    let mut branch_factor = 0.0;
    for i in 0..puzzle.functions.len() {
        for j in 0..puzzle.functions[i] {
            if leaf.source[i][j] == NOP {
                branch_factor += instruction_set.len() as f64;
                break;
            }
        }
    }
    for i in 0..puzzle.functions.len() {
        for j in 0..puzzle.functions[i] {
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

fn random_program(puzzle: &Puzzle, base: &Source, instruction_set: &Vec<Instruction>, mut rng: impl Rng) -> Source {
    let mut fullgram = *base;
    for i in 0..puzzle.functions.len() {
        for j in 0..puzzle.functions[i] {
            let mask = (fullgram[i][j] != NOP) as Instruction;
            fullgram[i][j] = fullgram[i][j] * mask + (1 - mask) * *instruction_set.choose(&mut rng).unwrap_or(&NOP);
        }
    }
    return fullgram;
}

fn score(state: &State, puzzle: &Puzzle) -> f64 {
    let mut touched = 0.0;
    let mut stars = 0.0;
    for y in 1..13 {
        for x in 1..17 {
            touched += is_touched(state.map[y][x]) as i32 as f64;
            stars += has_star(state.map[y][x]) as i32 as f64;
        }
    }
    let tiles = 12.0 * 16.0 + 1.0;
    return ((puzzle.stars as f64 - stars) + (touched / tiles) + ((MAX_STEPS - state.steps) as f64) / MAX_STEPS as f64) / puzzle.stars as f64;
}

fn get_instruction_set(puzzle: &Puzzle) -> Vec<Instruction> {
    let functions = puzzle.functions.iter().fold(0, |count, &val| count + (val > 0) as usize);
    let marks: usize = puzzle.marks.iter().map(|b| *b as usize).sum();
    let colors = 1 + puzzle.red as usize + puzzle.green as usize + puzzle.blue as usize;
    let mut result: Vec<Instruction> = Vec::with_capacity((3 + functions + marks) * colors);
    result.push(NOP);
    let mut conditionals = vec![INS_GRAY_COND];
    for (present, ins) in [(puzzle.red, INS_RED_COND), (puzzle.green, INS_GREEN_COND), (puzzle.blue, INS_BLUE_COND)].iter() {
        if *present {
            conditionals.push(*ins);
        }
    }
    for condition in conditionals {
        for ins in MOVES.iter() {
            result.push(ins | condition);
        }
        for i in 0..FUNCTIONS.len() {
            if puzzle.functions[i] > 0 {
                result.push(FUNCTIONS[i] | condition);
            }
        }
        for i in 0..MARKS.len() {
            if puzzle.marks[i] {
                result.push(MARKS[i] | condition);
            }
        }
    }
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    extern crate test;

    #[test]
    fn test_puzzle_42() {
        assert_eq!(0.11967216, execute(&PUZZLE_42, &PUZZLE_42_SOLUTION));
    }

    #[test]
    fn test_puzzle_42_instruction_set() {
        assert_eq!(vec![
            INS_FORWARD,
            INS_TURN_LEFT,
            INS_TURN_RIGHT,
            INS_F1,
            INS_F2,
            INS_F3,
            INS_F4,
        ], get_instruction_set(&PUZZLE_42));
    }

    #[test]
    fn test_puzzle_536() {
        assert_eq!(0.48914117, execute(&PUZZLE_536, &PUZZLE_536_SOLUTION));
    }

    #[test]
    fn test_puzzle_536_instruction_set() {
        assert_eq!(vec![
            INS_FORWARD,
            INS_TURN_LEFT,
            INS_TURN_RIGHT,
            INS_F1,
            INS_F2,
            INS_FORWARD | INS_RED_COND,
            INS_TURN_LEFT | INS_RED_COND,
            INS_TURN_RIGHT | INS_RED_COND,
            INS_F1 | INS_RED_COND,
            INS_F2 | INS_RED_COND,
            INS_FORWARD | INS_GREEN_COND,
            INS_TURN_LEFT | INS_GREEN_COND,
            INS_TURN_RIGHT | INS_GREEN_COND,
            INS_F1 | INS_GREEN_COND,
            INS_F2 | INS_GREEN_COND,
            INS_FORWARD | INS_BLUE_COND,
            INS_TURN_LEFT | INS_BLUE_COND,
            INS_TURN_RIGHT | INS_BLUE_COND,
            INS_F1 | INS_BLUE_COND,
            INS_F2 | INS_BLUE_COND,
        ], get_instruction_set(&PUZZLE_536));
    }

    #[test]
    fn test_puzzle_656() {
        assert_eq!(0.5143868, execute(&PUZZLE_656, &PUZZLE_656_SOLUTION));
    }

    #[test]
    fn test_puzzle_656_instruction_set() {
        assert_eq!(vec![
            INS_FORWARD,
            INS_TURN_LEFT,
            INS_TURN_RIGHT,
            INS_F1,
            INS_F2,
            INS_FORWARD | INS_RED_COND,
            INS_TURN_LEFT | INS_RED_COND,
            INS_TURN_RIGHT | INS_RED_COND,
            INS_F1 | INS_RED_COND,
            INS_F2 | INS_RED_COND,
            INS_FORWARD | INS_BLUE_COND,
            INS_TURN_LEFT | INS_BLUE_COND,
            INS_TURN_RIGHT | INS_BLUE_COND,
            INS_F1 | INS_BLUE_COND,
            INS_F2 | INS_BLUE_COND,
        ], get_instruction_set(&PUZZLE_656));
    }

    #[bench]
    fn bench_execute_42_times_10(b: &mut Bencher) {
        for instruction in get_instruction_set(&PUZZLE_42) {
            print!("{}", show_instruction(instruction));
        }
        println!();
        for instruction in get_instruction_set(&PUZZLE_536) {
            print!("{}", show_instruction(instruction));
        }
        let instruction_set = get_instruction_set(&PUZZLE_42);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
        let mut source: Source = [[NOP; 10]; 5];
        for iteration in 0..10 {
            for i in 0..5 {
                for ins in 0..PUZZLE_42.functions[i] {
                    source[i][ins] = *instruction_set.choose(rng).unwrap_or(&NOP);
                }
            }
            b.iter(|| execute(&PUZZLE_42, &source));
        }
    }

    #[bench]
    fn bench_execute_42_solution(b: &mut Bencher) {
        b.iter(|| execute(&PUZZLE_42, &PUZZLE_42_SOLUTION));
    }

    #[bench]
    fn bench_42_monte_carlo(b: &mut Bencher) {
        b.iter(|| carlo(&PUZZLE_42, 1 << 5, 1 << 5));
    }

    #[bench]
    fn bench_execute_536_times_10(b: &mut Bencher) {
        let instruction_set = get_instruction_set(&PUZZLE_536);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(536);
        let mut source: Source = [[NOP; 10]; 5];
        for _iteration in 0..10 {
            for i in 0..5 {
                for ins in 0..PUZZLE_536.functions[i] {
                    source[i][ins] = *instruction_set.choose(rng).unwrap_or(&NOP);
                }
            }
            b.iter(|| execute(&PUZZLE_536, &source));
        }
    }

    #[bench]
    fn bench_execute_536_solution(b: &mut Bencher) {
        b.iter(|| execute(&PUZZLE_536, &PUZZLE_536_SOLUTION));
    }

    #[bench]
    fn bench_536_monte_carlo(b: &mut Bencher) {
        b.iter(|| carlo(&PUZZLE_536, 1 << 5, 1 << 5));
    }

    #[bench]
    fn bench_execute_656_times_10(b: &mut Bencher) {
        let instruction_set = get_instruction_set(&PUZZLE_656);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(656);
        let mut source: Source = [[NOP; 10]; 5];
        for _iteration in 0..10 {
            for i in 0..5 {
                for ins in 0..PUZZLE_656.functions[i] {
                    source[i][ins] = *instruction_set.choose(rng).unwrap_or(&NOP);
                }
            }
            b.iter(|| execute(&PUZZLE_656, &source));
        }
    }

    #[bench]
    fn bench_execute_656_solution(b: &mut Bencher) {
        b.iter(|| execute(&PUZZLE_656, &PUZZLE_656_SOLUTION));
    }

    #[bench]
    fn bench_656_monte_carlo(b: &mut Bencher) {
        b.iter(|| carlo(&PUZZLE_656, 1 << 5, 1 << 5));
    }
}

fn show_instruction(instruction: Instruction) -> colored::ColoredString {
    let background = match instruction & INS_COLOR_MASK {
        INS_GRAY_COND => Color::BrightBlack,
        INS_RED_COND => Color::Red,
        INS_GREEN_COND => Color::Green,
        INS_BLUE_COND => Color::Blue,
        _ => Color::Black,
    };
    let foreground = match instruction & !INS_COLOR_MASK {
        INS_FORWARD...INS_F5 => Color::BrightWhite,
        INS_MARK_RED => Color::Red,
        INS_MARK_GREEN => Color::Green,
        INS_MARK_BLUE => Color::Blue,
        _ => Color::BrightWhite,
    };
    let string = match instruction & !INS_COLOR_MASK {
        INS_FORWARD => "↑".to_string(),
        INS_TURN_LEFT => "←".to_string(),
        INS_TURN_RIGHT => "→".to_string(),
        fun @ INS_F1...INS_F5 => (fun - INS_F1 + 1).to_string(),
        INS_MARK_RED...INS_MARK_BLUE => "●".to_string(),
        _ => " ".to_string(),
    };
    return string.color(foreground).on_color(background);
}

fn show_source(source: &Source) {
    print!("{{");
    for i in 0..source.len() {
        let mut to_print = vec![format!("F{}:", i + 1).normal()];
        for instruction in source[i].iter() {
            if *instruction != NOP {
                to_print.push(show_instruction(*instruction));
            }
        }
        if to_print.len() > 1 {
            for piece in to_print {
                print!("{}", piece);
            }
            print!(",");
        }
    }
    print!("}}");
}


impl Display for Stack {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Stack: ({}, ", self.pointer)?;
        let mut count = 0;
        for i in (1..self.pointer + 1).rev() {
            write!(f, "{}", show_instruction(self.data[i]))?;
            count += 1;
            if count == 100 {
                write!(f, "...")?;
                break;
            }
        }
        write!(f, ")")
        //        write!(f, "{}]", self.data[self.data.len() - 1])
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "At ({}, {}), Running: {}\n{}\nMap:\n",
            self.x, self.y, self.running, self.stack
        )?;
        for y in 1..13 {
            for x in 1..17 {
                let tile = self.map[y][x];
                let string = if self.y == y && self.x == x {
                    match self.direction {
                        Direction::Up => "↑",
                        Direction::Left => "←",
                        Direction::Down => "↓",
                        Direction::Right => "→",
                    }
                } else {
                    "★"
                };
                let background = match tile & TILE_COLOR_MASK {
                    RE => Color::Red,
                    GE => Color::Green,
                    BE => Color::Blue,
                    _ => Color::Black,
                };
                let foreground = if self.y == y && self.x == x {
                    Color::BrightWhite
                } else if has_star(tile) {
                    Color::Yellow
                } else {
                    background
                };
                write!(f, "{}", string.color(foreground).on_color(background))?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}

fn make_puzzle(
    map: Map,
    direction: Direction,
    x: usize,
    y: usize,
    functions: [usize; 5],
    marks: [bool; 3],
) -> Puzzle {
//    let monocolor = !marks.iter().any(|i| *i) &&
//        [RE, GE, BE].iter().map(|col| { map.iter().all(|row| row.iter().all(|tile| ((tile & TILE_COLOR_MASK) & col) > 0)) }).any(|i| i);
//    let red = !monocolor && map.iter().any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & RE) > 0));
//    let green = !monocolor && map.iter().any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & GE) > 0));
//    let blue = !monocolor && map.iter().any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & BE) > 0));
    let (mut red, mut green, mut blue) = (false, false, false);
    for y in 1..13 {
        for x in 1..17 {
            red |= (map[y][x] & RE) > 0;
            green |= (map[y][x] & GE) > 0;
            blue |= (map[y][x] & BE) > 0;
        }
    }
//    let monocolor = [red, green, blue].iter().fold(0, |acc, b| acc + (*b as usize)) == 1;
    let stars: usize = map.iter().map(|row| row.iter().map(|el| has_star(*el) as usize).sum::<usize>()).sum();
    return Puzzle {
        map,
        direction,
        x,
        y,
        stars,
        functions,
        marks,
        red,
        green,
        blue,
    };
}

fn verify_puzzle(puzzle: &Puzzle) -> bool {
//    let monocolor = !marks.iter().any(|i| *i) &&
//        [RE, GE, BE].iter().map(|col| { map.iter().all(|row| row.iter().all(|tile| ((tile & TILE_COLOR_MASK) & col) > 0)) }).any(|i| i);
//    let red = !monocolor && map.iter().any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & RE) > 0));
//    let green = !monocolor && map.iter().any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & GE) > 0));
//    let blue = !monocolor && map.iter().any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & BE) > 0));
    let (mut red, mut green, mut blue) = (false, false, false);
    for y in 1..13 {
        for x in 1..17 {
            red |= (puzzle.map[y][x] & RE) > 0;
            green |= (puzzle.map[y][x] & GE) > 0;
            blue |= (puzzle.map[y][x] & BE) > 0;
        }
    }
    let monocolor = [red, green, blue].iter().fold(0, |acc, b| acc + (*b as usize)) == 1;
    if monocolor {
        red = false;
        green = false;
        blue = false;
    }
    let stars: usize = puzzle.map.iter().map(|row| row.iter().map(|el| has_star(*el) as usize).sum::<usize>()).sum();
    if red != puzzle.red {
        panic!("red bad! {} {}", red, puzzle.red);
    }
    if green != puzzle.green {
        panic!("green bad! {} {}", green, puzzle.green);
    }
    if blue != puzzle.blue {
        panic!("blue bad! {} {}", blue, puzzle.blue);
    }
    if stars != puzzle.stars {
        panic!("stars bad! {} {}", stars, puzzle.stars);
    }
    return true;
}
// Instructions
// 0b 00 00 00 00
//    CC C_ II II

// Tiles
// 0b 00 00 00 00
//    __ __ SC CC

// C = color
// C = 0 -> Gray
// C = 1 -> Red
// C = 2 -> Green
// C = 3 -> Blue

// I = instruction
// I = 0  -> Forward
// I = 1  -> TurnLeft
// I = 2  -> TurnRight
// I = 3  -> F1
// I = 4  -> F2
// I = 5  -> F3
// I = 6  -> F4
// I = 7  -> F5
// I = 8  -> MarkGray
// I = 9  -> MarkRed
// I = 10 -> MarkGreen
// I = 11 -> MarkBlue

// S = star
// S = 0 -> No Star
// S = 1 -> Star


const TEST_SOURCE: Source = [
    [
        INS_FORWARD,
        INS_FORWARD | INS_RED_COND,
        INS_FORWARD | INS_GREEN_COND,
        INS_FORWARD | INS_BLUE_COND,
        INS_TURN_LEFT,
        INS_TURN_LEFT | INS_RED_COND,
        INS_TURN_LEFT | INS_GREEN_COND,
        INS_TURN_LEFT | INS_BLUE_COND,
        INS_MARK_GREEN,
        INS_MARK_BLUE,
    ],
    [
        INS_TURN_RIGHT,
        INS_TURN_RIGHT | INS_RED_COND,
        INS_TURN_RIGHT | INS_GREEN_COND,
        INS_TURN_RIGHT | INS_BLUE_COND,
        INS_F1,
        INS_F1 | INS_RED_COND,
        INS_F1 | INS_GREEN_COND,
        INS_F1 | INS_BLUE_COND,
        INS_MARK_GREEN | INS_RED_COND,
        INS_MARK_BLUE | INS_RED_COND,
    ],
    [
        INS_F2,
        INS_F2 | INS_RED_COND,
        INS_F2 | INS_GREEN_COND,
        INS_F2 | INS_BLUE_COND,
        INS_F3,
        INS_F3 | INS_RED_COND,
        INS_F3 | INS_GREEN_COND,
        INS_F3 | INS_BLUE_COND,
        INS_MARK_GREEN | INS_GREEN_COND,
        INS_MARK_BLUE | INS_GREEN_COND,
    ],
    [
        INS_F4,
        INS_F4 | INS_RED_COND,
        INS_F4 | INS_GREEN_COND,
        INS_F4 | INS_BLUE_COND,
        INS_F5,
        INS_F5 | INS_RED_COND,
        INS_F5 | INS_GREEN_COND,
        INS_F5 | INS_BLUE_COND,
        INS_MARK_GREEN | INS_BLUE_COND,
        INS_MARK_BLUE | INS_BLUE_COND,
    ],
    [
        NOP,
        INS_MARK_RED,
        INS_MARK_RED | INS_RED_COND,
        INS_MARK_RED | INS_GREEN_COND,
        INS_MARK_RED | INS_BLUE_COND,
        NOP, NOP, NOP, NOP, NOP, ],
];

//const PUZZLE_NULL: Puzzle = Puzzle {
//    map: [
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//    ],
//    direction: Direction::Down,
//    x: 1,
//    y: 1,
//    stars: 1,
//    functions: [0, 0, 0, 0, 0],
//    marks: [false; 3],
//    red: false,
//    green: false,
//    blue: false,
//};
const PUZZLE_42: Puzzle = Puzzle {
    map: [
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BS, BS, BS, BS, BS, BS, BS, BS, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BE, BS, BS, BS, BS, BS, BS, BS, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 5,
    y: 9,
    stars: 23,
    functions: [5, 2, 2, 2, 0],
    marks: [false; 3],
    red: false,
    green: false,
    blue: false,
};
const PUZZLE_42_SOLUTION: Source = [
    [
        INS_F2,
        INS_TURN_LEFT,
        INS_F3,
        INS_TURN_LEFT,
        INS_F1,
        NOP, NOP, NOP, NOP, NOP, ],
    [
        INS_F3,
        INS_F3,
        NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, ],
    [
        INS_F4,
        INS_F4,
        NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, ],
    [
        INS_FORWARD,
        INS_FORWARD,
        NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, ],
    [NOP; 10],
];
const PUZZLE_536: Puzzle = Puzzle {
    map: [
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, BE, BE, BE, BE, BE, BE, BE, GE, BE, BE, BE, BE, BE, BE, BE, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, BE, _N, _N, ],
        [_N, BE, BE, BE, BE, BE, BE, RE, BE, BE, BE, BE, BE, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, BE, BE, BE, BE, GE, BE, BE, BE, BE, _N, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, BE, _N, _N, _N, _N, _N, _N, _N, RE, _N, GE, _N, RE, _N, _N, ],
        [_N, RE, _N, GE, _N, BS, BE, BE, BE, BE, BE, BE, _N, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, BE, _N, _N, _N, _N, _N, _N, _N, _N, _N, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, BE, BE, BE, BE, BE, RE, BE, BE, BE, BE, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, BE, _N, _N, ],
        [_N, BE, BE, BE, BE, BE, BE, BE, GE, BE, BE, BE, BE, BE, BE, BE, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 1,
    y: 1,
    stars: 1,
    functions: [3, 3, 0, 0, 0],
    marks: [false; 3],
    red: true,
    green: true,
    blue: true,
};
const PUZZLE_536_SOLUTION: Source = [
    [
        INS_F2,
        INS_TURN_RIGHT,
        INS_F1,
        NOP, NOP, NOP, NOP, NOP, NOP, NOP, ],
    [
        INS_FORWARD,
        INS_F2 | INS_BLUE_COND,
        INS_FORWARD,
        NOP, NOP, NOP, NOP, NOP, NOP, NOP, ],
    [NOP; 10],
    [NOP; 10],
    [NOP; 10],
];
const PUZZLE_656: Puzzle = Puzzle {
    map: [
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, RS, _N, _N, _N, RS, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, RS, _N, _N, _N, _N, ],
        [_N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N, ],
        [_N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, RS, _N, _N, ],
        [_N, BS, _N, RS, _N, BS, _N, _N, _N, RS, _N, _N, _N, BS, _N, BS, _N, _N, ],
        [_N, BS, _N, BS, _N, BS, _N, _N, _N, BS, _N, RS, _N, BS, _N, BS, _N, _N, ],
        [_N, BS, RS, BS, _N, BS, _N, _N, _N, BS, _N, BS, _N, BS, _N, BS, _N, _N, ],
        [_N, BS, BS, BS, _N, BS, RS, _N, _N, BS, _N, BS, _N, BS, RS, BS, _N, _N, ],
        [_N, BS, BS, BS, _N, BS, BS, _N, _N, BS, RS, BS, _N, BS, BS, BS, _N, _N, ],
        [_N, BS, BS, BS, _N, BS, BS, _N, _N, BS, BS, BS, RS, BS, BS, BS, RS, _N, ],
        [_N, BS, BS, BS, RS, BS, BS, RS, RS, BS, BS, BS, BS, BS, BS, BS, BS, _N, ],
        [_N, BE, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 1,
    y: 12,
    stars: 98,
    functions: [5, 5, 0, 0, 0],
    marks: [false; 3],
    red: true,
    green: false,
    blue: true,
};
const PUZZLE_656_SOLUTION: Source = [
    [
        INS_TURN_LEFT,
        INS_F2,
        INS_TURN_LEFT,
        INS_FORWARD,
        INS_F1,
        NOP, NOP, NOP, NOP, NOP, ],
    [
        INS_FORWARD,
        INS_TURN_RIGHT | INS_RED_COND,
        INS_TURN_RIGHT | INS_RED_COND,
        INS_F2 | INS_BLUE_COND,
        INS_FORWARD,
        NOP, NOP, NOP, NOP, NOP, ],
    [NOP; 10],
    [NOP; 10],
    [NOP; 10],
];