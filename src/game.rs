use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Index, IndexMut};

use colored::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::constants::*;
use instructions::*;
use std::collections::{HashSet, VecDeque};

pub mod display;
pub mod instructions;

pub type TileType = u32;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct Tile(pub TileType);

impl Tile {
    const MAX_TOUCHES: TileType = u32::MAX >> 4;
    fn touch(&mut self) {
        self.0 += TILE_TOUCHED.0;
    }
    fn untouch(&mut self) {
        self.0 -= TILE_TOUCHED.0;
    }
    pub(crate) fn touches(&self) -> usize {
        (self.0 >> 4) as usize
    }
    fn clear_star(&mut self) {
        self.0 &= !TILE_STAR_MASK.0;
    }
    pub(crate) fn touched(&self) -> usize {
        (self.0 >> 4) as usize
    }
    pub(crate) fn has_star(&self) -> bool {
        (self.0 & TILE_STAR_MASK.0) > 0
    }
    fn color(&self) -> Tile {
        Tile(self.0 & TILE_COLOR_MASK.0)
    }
    fn is_red(&self) -> bool {
        self.0 & RE.0 > 0
    }
    fn is_green(&self) -> bool {
        self.0 & GE.0 > 0
    }
    fn is_blue(&self) -> bool {
        self.0 & BE.0 > 0
    }
    pub(crate) fn executes(&self, instruction: Ins) -> bool {
        let color = instruction.get_cond();
        return color == GRAY_COND || color.has_cond(self.to_condition());
    }
    fn mark(&mut self, instruction: Ins) {
        let color: u8 = instruction.get_mark_color().into();
        self.0 = color as TileType | (self.0 & TILE_TOUCH_MASK.0)
    }
    pub(crate) fn to_condition(&self) -> Ins {
        ((self.0 << 5) & 0b11111111).into()
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct Map(pub [[Tile; 18]; 14]);

pub type Method = [Ins; 10];

#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Serialize, Deserialize)]
pub struct Source(pub [Method; 5]);

impl Source {
    fn len(&self) -> usize {
        self.0.len()
    }
    pub fn has_nop(&self) -> bool {
        let mut result = false;
        for m in self.0.iter() {
            for i in m.iter() {
                result |= i.is_nop();
            }
        }
        return result;
    }
    pub fn count_ins(&self) -> usize {
        let mut result = 0;
        for m in self.0.iter() {
            for i in m.iter() {
                result += !(i.is_nop() || i.is_halt()) as usize;
            }
        }
        return result;
    }
    pub fn count_nop(&self) -> usize {
        let mut result = 0;
        for m in self.0.iter() {
            for i in m.iter() {
                result += !(i.is_nop() || i.is_halt()) as usize;
            }
        }
        return result;
    }
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        return hasher.finish();
    }
    pub fn shade(&mut self, max_ins: usize) {
        let diff = max_ins - self.count_ins();
        for m in 0..5 {
            let mut nops = 0;
            for i in 0..10 {
                nops += self[m][i].is_nop() as usize;
                if nops > diff {
                    self[m][i] = HALT;
                }
            }
        }
    }
    pub fn sanitize(&mut self) {
        for m in 0..5 {
            let mut offset = 0;
            for i in 0..10 {
                if self[m][i].is_nop() || self[m][i].is_probe() {
                    offset += 1;
                }
                self[m][i] = if i + offset < 10 {
                    self[m][i + offset]
                } else {
                    HALT
                }
            } // ↑2↑←↑→5|||,_←↑35←↑→↑→,↑↑→↑4↑→↑←↑,←↑↑→↑←↑↑→↑,↑←↑→↑↑→↑←↑,
        }
    }
}

impl Hash for Source {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for method in self.0.iter() {
            for ins in method {
                ins.hash(state);
            }
        }
    }
}

impl Index<usize> for Source {
    type Output = Method;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Source {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

type Stack = StackArr;

pub(crate) const MAX_STEPS: usize = 1 << 12;
const STACK_MATCH: usize = 1 << 6;

#[derive(Clone, Debug)]
pub struct StackVec(pub SmallVec<[InsPtr; 32]>);

impl Default for StackVec {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl PartialEq for StackVec {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        } else if self.0.len() <= STACK_MATCH {
            return self.0 == other.0;
        } else if self.0.len() > STACK_MATCH {
            return self.0.get((self.0.len() - STACK_MATCH)..self.0.len())
                == other.0.get((self.0.len() - STACK_MATCH)..self.0.len());
        } else {
            return false;
        }
    }
}

impl Eq for StackVec {}

impl Hash for StackVec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0
            .get((self.0.len() - STACK_MATCH).max(0)..self.0.len())
            .hash(state);
    }
}
impl Index<usize> for StackVec {
    type Output = InsPtr;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.0.len() - index - 1]
    }
}
impl StackVec {
    fn push(&mut self, element: InsPtr) {
        self.0.push(element);
    }
    fn pop(&mut self) -> InsPtr {
        self.0.pop().unwrap()
    }
    pub fn last(&self) -> &InsPtr {
        self.0.last().unwrap()
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StackArr {
    pointer: usize,
    pub data: [InsPtr; Self::STACK_SIZE],
}

impl Default for StackArr {
    fn default() -> Self {
        Self {
            pointer: 0,
            data: [INSPTR_NULL; Self::STACK_SIZE],
        }
    }
}

impl PartialEq for StackArr {
    fn eq(&self, other: &Self) -> bool {
        if self.pointer != other.pointer {
            return false;
        } else if self.pointer <= STACK_MATCH {
            return self.data.get(0..self.pointer) == other.data.get(0..other.pointer);
        } else if self.pointer > STACK_MATCH {
            let start = self.pointer - STACK_MATCH;
            return self.data.get(start..self.pointer) == other.data.get(start..other.pointer);
        } else {
            return false;
        }
    }
}

impl Eq for StackArr {}

impl Hash for StackArr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut remaining = STACK_MATCH;
        let mut i = self.pointer;
        while remaining > 0 && i > 0 {
            i -= 1;
            //            if self.data[i] != NOP && self.data[i] != HALT {
            self.data[i].hash(state);
            remaining -= 1;
            //            }
        }
    }
}

impl Index<usize> for StackArr {
    type Output = InsPtr;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.pointer - index - 1]
    }
}

impl StackArr {
    // make the StackArr struct exactly 2^10 bytes
    const STACK_SIZE: usize = (1 << 9) - mem::size_of::<usize>();
    fn push(&mut self, element: InsPtr) {
        self.data[self.pointer] = element;
        self.pointer += 1;
    }
    fn pop(&mut self) -> InsPtr {
        let &result = self.last();
        self.pointer -= 1;
        return result;
    }
    pub fn last(&self) -> &InsPtr {
        &self.data[self.pointer - 1]
    }
    pub(crate) fn len(&self) -> usize {
        self.pointer
    }
    fn is_empty(&self) -> bool {
        self.pointer == 0
    }
    pub(crate) fn clear(&mut self) {
        self.pointer = 0;
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up = 0b00,
    Left = 0b01,
    Down = 0b10,
    Right = 0b11,
}

//#[derive()]
impl Direction {
    fn left(&self) -> Direction {
        return unsafe { std::mem::transmute((*self as u8 + 1) & 0b11) };
        // match self {
        //     Direction::Up => Direction::Left,
        //     Direction::Left => Direction::Down,
        //     Direction::Down => Direction::Right,
        //     Direction::Right => Direction::Up,
        // }
    }
    fn right(&self) -> Direction {
        return unsafe { std::mem::transmute((*self as u8 + 3) & 0b11) };
        // match self {
        //     Direction::Up => Direction::Right,
        //     Direction::Left => Direction::Up,
        //     Direction::Down => Direction::Left,
        //     Direction::Right => Direction::Down,
        // }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub map: Map,
    pub direction: Direction,
    pub x: usize,
    pub y: usize,
    pub stars: usize,
    pub methods: [usize; 5],
    pub actual_methods: [usize; 5],
    pub marks: [bool; 3],
    pub red: bool,
    pub green: bool,
    pub blue: bool,
}

impl Puzzle {
    pub(crate) fn get_ins_set(&self, colors: Ins, gray: bool) -> Vec<Ins> {
        coz::begin!("get instruction set");
        let functions = self
            .methods
            .iter()
            .fold(0, |count, &val| count + (val > 0) as usize);
        let marks: usize = self.marks.iter().map(|b| *b as usize).sum();
        let (red, green, blue) = (
            self.red && colors.has_cond(RED_COND),
            self.green && colors.has_cond(GREEN_COND),
            self.blue && colors.has_cond(BLUE_COND),
        );
        let colors = gray as usize + red as usize + green as usize + blue as usize;
        let mut result: Vec<Ins> = Vec::with_capacity((3 + functions + marks) * colors);
        let mut conditionals = if gray { vec![GRAY_COND] } else { vec![] };
        for (present, ins) in [(red, RED_COND), (green, GREEN_COND), (blue, BLUE_COND)].iter() {
            if *present {
                conditionals.push(*ins);
            }
        }
        for condition in conditionals {
            for ins in &MOVES {
                result.push(*ins | condition);
            }
            for i in 0..FUNCTIONS.len() {
                if self.methods[i] > 0 {
                    result.push(FUNCTIONS[i] | condition);
                }
            }
            for i in 0..MARKS.len() {
                if self.marks[i] && MARKS[i].get_mark_color() != condition.condition_to_color() {
                    result.push(MARKS[i] | condition);
                }
            }
        }
        coz::end!("get instruction set");
        return result;
    }
    pub(crate) fn empty_source(&self) -> Source {
        let mut result = NOGRAM.clone();
        for instructions in 0..self.methods.len() {
            for i in 0..self.methods[instructions] {
                result.0[instructions][i] = NOP;
            }
        }
        return result;
    }
    pub(crate) fn initial_state(&self, source: &Source) -> State {
        let mut result = State {
            stars: self.stars,
            map: self.map,
            direction: self.direction,
            x: self.x,
            y: self.y,
            ..State::default()
        };
        result.initialize(source, self);
        return result;
    }
    /// execute a source for the puzzle, returning a score
    pub(crate) fn execute<F, R>(&self, source: &Source, show: bool, mut scoring: F) -> R
    where
        F: FnMut(&State, &Puzzle) -> R,
    {
        coz::begin!("execute");
        let mut state = self.initial_state(source);
        if show {
            println!("{}", state);
        }
        while state.step(&source, self) {
            if show {
                println!("{}", state);
            }
        }
        if show {
            println!("{}", state);
        }
        let result = scoring(&state, self);
        coz::end!("execute");
        return result;
    }
    pub(crate) fn get_cond_mask(&self) -> Ins {
        if (self.red as u8) + (self.green as u8) + (self.blue as u8) > 1 {
            with_conds(self.red, self.green, self.blue)
        } else {
            GRAY_COND
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct State {
    pub(crate) steps: usize, // number of instructions executed
    pub(crate) stars: usize, // number of stars remaining
    pub stack: Stack,
    pub(crate) map: Map,
    direction: Direction,
    x: usize,
    y: usize,
}

impl Default for State {
    fn default() -> Self {
        State {
            steps: 0,
            stars: 1,
            stack: Default::default(),
            map: Map([[_N; 18]; 14]),
            direction: Direction::Up,
            x: 1,
            y: 1,
        }
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stars.hash(state);
        self.stack.hash(state);
        self.map.hash(state);
        self.direction.hash(state);
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl State {
    pub fn initialize(&mut self, source: &Source, puzzle: &Puzzle) {
        self.invoke(source, puzzle, F1.source_index());
    }
    fn clear_star(&mut self) {
        self.map.0[self.y][self.x].clear_star();
    }
    fn touch(&mut self) {
        self.map.0[self.y][self.x].touch();
    }
    fn untouch(&mut self) {
        self.map.0[self.y][self.x].untouch();
    }
    fn touches(&self) -> usize {
        self.map.0[self.y][self.x].touches()
    }
    fn mark(&mut self, ins: Ins) {
        self.map.0[self.y][self.x].mark(ins)
    }
    pub fn ins_pointer(&self) -> &InsPtr {
        let ins = self.stack.last();
        //        let ins = source[ins.get_method_index()][ins.get_ins_index()];
        return ins;
    }
    pub fn current_ins(&self, source: &Source) -> Ins {
        let ins = self.ins_pointer();
        let ins = source[ins.get_method_index()][ins.get_ins_index()];
        return ins;
    }
    pub(crate) fn current_tile(&self) -> &Tile {
        &self.map.0[self.y][self.x]
    }
    fn running(&self) -> bool {
        !self.stack.is_empty()
            && self.stars > 0
            && self.map.0[self.y][self.x].touches() < Tile::MAX_TOUCHES as usize
            && self.stack.len() < StackArr::STACK_SIZE - 10
            && self.steps < MAX_STEPS
    }
    pub(crate) fn step(&mut self, source: &Source, puzzle: &Puzzle) -> bool {
        coz::begin!("step");
        let ins = self.current_ins(source).as_vanilla();
        self.stack.pop();
        self.steps += 1;
        if self.current_tile().executes(ins) {
            match ins.get_ins() {
                FORWARD => {
                    self.y = (self.y as i32 + [-1, 0, 1, 0][self.direction as usize]) as usize;
                    self.x = (self.x as i32 + [0, -1, 0, 1][self.direction as usize]) as usize;
                    if *self.current_tile() == _N {
                        coz::end!("step");
                        return false;
                    } else {
                        self.stars -= self.current_tile().has_star() as usize;
                        self.clear_star();
                    }
                }
                LEFT => self.direction = self.direction.left(),
                RIGHT => self.direction = self.direction.right(),
                F1 | F2 | F3 | F4 | F5 => {
                    self.invoke(source, puzzle, ins.source_index());
                    //                    self.max_stack = max(self.max_stack, self.stack.pointer);
                }
                MARK_GRAY | MARK_RED | MARK_GREEN | MARK_BLUE => self.mark(ins),
                HALT => {
                    coz::end!("step");
                    return self.running();
                }
                _ => (),
            }
        }
        self.touch();
        coz::end!("step");
        return self.running();
    }
    fn intersection_cost(&self, intersections: i64) -> i64 {
        return if intersections == 1 {
            -(1 << 2)
        } else {
            -(1 << 2) + intersections * intersections
        };
    }
    fn invoke(&mut self, source: &Source, puzzle: &Puzzle, method: usize) {
        for i in (0..puzzle.methods[method]).rev() {
            let ins = source.0[method][i];
            self.stack.push(InsPtr::new(method, i));
        }
    }
    pub(crate) fn get_hash(&self) -> u64 {
        let mut state = DefaultHasher::new();
        self.hash(&mut state);
        return state.finish();
    }
}

pub fn won(state: &State, _: &Puzzle) -> bool {
    return state.stars == 0;
}

pub fn genboi(ta: Tile, tb: Tile, tc: Tile) -> Puzzle {
    return make_puzzle(
        Map([
            [
                _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, ta, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
            ],
            [
                _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N,
            ],
        ]),
        Direction::Right,
        5,
        6,
        [3, 10, 0, 0, 0],
        [true, true, true],
    );
}

pub fn make_puzzle(
    map: Map,
    direction: Direction,
    x: usize,
    y: usize,
    mut methods: [usize; 5],
    marks: [bool; 3],
) -> Puzzle {
    // must be able to use markable colors
    let [mut red, mut green, mut blue] = marks;
    // test which colors are reachable
    let mut frontier = VecDeque::new();
    frontier.push_front((x, y));
    let mut visited = HashSet::new();
    visited.insert((x, y));
    while let Some((x, y)) = frontier.pop_back() {
        for (dx, dy) in &[(1, 0), (0, 1), (-1, 0), (0, -1)] {
            let (nx, ny) = ((x as isize + dx) as usize, (y as isize + dy) as usize);
            if map.0[ny][nx] != _N && !visited.contains(&(nx, ny)) {
                visited.insert((nx, ny));
                frontier.push_front((nx, ny));
                red |= map.0[ny][nx].is_red();
                green |= map.0[ny][nx].is_green();
                blue |= map.0[ny][nx].is_blue();
            }
        }
    }
    // check which colors are on the map
    //for y in 1..13 {
    //    for x in 1..17 {
    //        red |= map[y][x].is_red();
    //        green |= map[y][x].is_green();
    //        blue |= map[y][x].is_blue();
    //    }
    //}
    let actual_methods = methods;
    methods[1..5].sort_unstable_by(|a, b| b.cmp(a));
    let mut map_out = map.clone();
    map_out.0[y][x].clear_star();
    map_out.0[y][x].touch();
    let stars: usize = map_out
        .0
        .iter()
        .map(|row| row.iter().map(|el| el.has_star() as usize).sum::<usize>())
        .sum();
    return Puzzle {
        map: map_out,
        direction,
        x,
        y,
        stars,
        methods,
        actual_methods,
        marks,
        red,
        green,
        blue,
    };
}

fn verify_puzzle(puzzle: &Puzzle) -> bool {
    let (mut red, mut green, mut blue) = (false, false, false);
    for y in 1..13 {
        for x in 1..17 {
            red |= puzzle.map.0[y][x].is_red();
            green |= puzzle.map.0[y][x].is_green();
            blue |= puzzle.map.0[y][x].is_blue();
        }
    }
    let stars: usize = puzzle
        .map
        .0
        .iter()
        .map(|row| row.iter().map(|el| el.has_star() as usize).sum::<usize>())
        .sum();
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
