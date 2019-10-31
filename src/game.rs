use std::fmt::{Display, Error, Formatter, Debug};
use std::ops::{IndexMut, Index};
use std::cmp::{max, min};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;
use std::mem;

use colored::*;
use serde::{Serialize, Deserialize};

use crate::constants::*;
use instructions::*;

pub(crate) mod instructions;

pub type TileType = u16;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct Tile(pub TileType);

impl Tile {
    fn touch(&mut self) { self.0 += TILE_TOUCHED.0; }
    fn untouch(&mut self) { self.0 -= TILE_TOUCHED.0; }
    fn clear_star(&mut self) { self.0 &= !TILE_STAR_MASK.0; }
    pub(crate) fn touched(&self) -> usize { (self.0 >> 5) as usize}
    pub(crate) fn has_star(&self) -> bool { (self.0 & TILE_STAR_MASK.0) > 0 }
    fn color(&self) -> Tile { Tile(self.0 & TILE_COLOR_MASK.0) }
    fn is_red(&self) -> bool { self.0 & RE.0 > 0 }
    fn is_green(&self) -> bool { self.0 & GE.0 > 0 }
    fn is_blue(&self) -> bool { self.0 & BE.0 > 0 }
    pub(crate) fn executes(&self, instruction: Ins) -> bool {
        let color = instruction.get_cond();
        return color == GRAY_COND || color.has_cond(self.to_condition());
    }
    fn mark(&mut self, instruction: Ins) {
        let color: u8 = instruction.get_mark_color().into();
        self.0 = color as TileType | TILE_TOUCHED.0
    }
    pub(crate) fn to_condition(&self) -> Ins { ((self.0 as u8) << 5).into() }
}

type Map = [[Tile; 18]; 14];

pub type Method = [Ins; 10];

#[derive(Eq, PartialEq, PartialOrd, Copy, Clone, Debug, Serialize, Deserialize)]
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
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        return hasher.finish();
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

    fn index(&self, index: usize) -> &Self::Output { &self.0[index] }
}

impl IndexMut<usize> for Source {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output { &mut self.0[index] }
}

// make the Stack struct exactly 2^10 bytes
const STACK_SIZE: usize = (1 << 10) - mem::size_of::<usize>();
pub(crate) const MAX_STEPS: usize = 1 << 12;
const STACK_MATCH: usize = 1 << 6;

#[derive(Copy, Clone)]
pub struct Stack {
    pointer: usize,
    pub data: [InsPtr; STACK_SIZE],
}

impl PartialEq for Stack {
    fn eq(&self, other: &Self) -> bool {
        if self.pointer <= STACK_MATCH && self.pointer == other.pointer {
            return self.data.get(0..self.pointer) == other.data.get(0..other.pointer);
        } else if self.pointer > STACK_MATCH && other.pointer > STACK_MATCH {
            let start = self.pointer - STACK_MATCH;
            return self.data.get(start..self.pointer) == other.data.get(start..other.pointer);
        } else {
            return false;
        }
    }
}

impl Eq for Stack {}

impl Hash for Stack {
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

impl Index<usize> for Stack {
    type Output = InsPtr;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.pointer - index - 1]
    }
}

impl Stack {
    fn push(&mut self, element: InsPtr) {
        self.data[self.pointer] = element;
        self.pointer += 1;
    }
    fn pop(&mut self) -> InsPtr {
        let &result = self.top();
        self.pointer -= 1;
        return result;
    }
    pub fn top(&self) -> &InsPtr { &self.data[self.pointer - 1] }
    pub(crate) fn len(&self) -> usize { self.pointer }
    fn empty(&self) -> bool { self.pointer == 0 }
    pub(crate) fn clear(&mut self) { self.pointer = 0; }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up = 0b0001,
    Left = 0b0010,
    Down = 0b0100,
    Right = 0b1000,
}

impl Direction {
    fn left(&self) -> Direction {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }
    fn right(&self) -> Direction {
        match self {
            Direction::Up => Direction::Right,
            Direction::Left => Direction::Up,
            Direction::Down => Direction::Left,
            Direction::Right => Direction::Down,
        }
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
        let functions = self.methods.iter().fold(0, |count, &val| count + (val > 0) as usize);
        let marks: usize = self.marks.iter().map(|b| *b as usize).sum();
        let (red, green, blue) = (self.red && colors.has_cond(RED_COND),
                                  self.green && colors.has_cond(GREEN_COND),
                                  self.blue && colors.has_cond(BLUE_COND));
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
    pub(crate) fn execute<F, R>(&self, source: &Source, show: bool, mut scoring: F) -> R where F: FnMut(&State, &Puzzle) -> R {
        let mut state = self.initial_state(source);
        if show { println!("{}", state); }
        while state.step(&source, self) {
            if show { println!("{}", state); }
        }
        return scoring(&state, self);
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
    pub(crate) steps: usize,
    pub(crate) stars: usize,
    pub stack: Vec<InsPtr>,
    pub(crate) map: Map,
    direction: Direction,
    x: usize,
    y: usize,
    inters: usize,
}

impl Default for State {
    fn default() -> Self {
        State {
            steps: 0,
            stars: 1,
            stack: vec![],
            map: [[_N; 18]; 14],
            direction: Direction::Up,
            x: 1,
            y: 1,
            inters: 0,
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
    fn clear_star(&mut self) { self.map[self.y][self.x].clear_star(); }
    fn touch(&mut self) { self.map[self.y][self.x].touch(); }
    fn untouch(&mut self) { self.map[self.y][self.x].untouch(); }
    fn mark(&mut self, ins: Ins) { self.map[self.y][self.x].mark(ins) }
    pub fn ins_pointer(&self) -> &InsPtr {
        let ins = self.stack.last().unwrap_or(&INSPTR_NULL);
//        let ins = source[ins.get_method_index()][ins.get_ins_index()];
        return ins;
    }
    pub fn current_ins(&self, source: &Source) -> Ins {
        let ins = self.ins_pointer();
        let ins = source[ins.get_method_index()][ins.get_ins_index()];
        return ins;
    }
    pub(crate) fn current_tile(&self) -> &Tile { &self.map[self.y][self.x] }
    fn running(&self) -> bool {
        !self.stack.is_empty() && self.stars > 0 && self.stack.len() < STACK_SIZE - 12 && self.steps < MAX_STEPS
    }
    pub(crate) fn step(&mut self, source: &Source, puzzle: &Puzzle) -> bool {
        let ins = self.current_ins(source).as_vanilla();
        self.stack.pop();
        self.steps += 1;
        if self.current_tile().executes(ins) {
            match ins.get_ins() {
                FORWARD => {
                    match &self.direction {
                        Direction::Up => self.y -= 1,
                        Direction::Left => self.x -= 1,
                        Direction::Down => self.y += 1,
                        Direction::Right => self.x += 1,
                    };
                    if *self.current_tile() == _N {
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
                HALT => return self.running(),
                _ => (),
            }
        }
        self.touch();
        self.inters = max(self.inters, self.current_tile().touched());
        return self.running();
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
        [
            [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, ta, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N, ],
            [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        ],
        Direction::Right,
        5, 6, [3, 10, 0, 0, 0, ], [true, true, true, ],
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
    let [mut red, mut green, mut blue] = marks;
    for y in 1..13 {
        for x in 1..17 {
            red |= map[y][x].is_red();
            green |= map[y][x].is_green();
            blue |= map[y][x].is_blue();
        }
    }
    let actual_methods = methods;
    methods[1..5].sort_unstable_by(|a, b| b.cmp(a));
    let mut map_out = map.clone();
    map_out[y][x].clear_star();
    map_out[y][x].touch();
    let stars: usize = map_out.iter().map(|row| row.iter()
        .map(|el| el.has_star() as usize).sum::<usize>()).sum();
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
            red |= puzzle.map[y][x].is_red();
            green |= puzzle.map[y][x].is_green();
            blue |= puzzle.map[y][x].is_blue();
        }
    }
    let stars: usize = puzzle.map.iter().map(|row| row.iter().map(|el| el.has_star() as usize).sum::<usize>()).sum();
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

impl Display for Source {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{")?;
        for i in 0..self.len() {
            let mut separate = false;
            for &instruction in self[i].iter() {
                if instruction == HALT {
                    write!(f, "|")?;
//                    break;
                } else if instruction == NOP {
                    write!(f, "_")?;
                } else if instruction != NOP {
                    write!(f, "{}", instruction)?;
                }
                separate = true;
            }
            if separate { write!(f, ",")?; }
        }
        write!(f, "}}")
    }
}

//impl Display for Stack {
//    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
//        write!(f, "Stack: ({}, ", self.pointer)?;
//        let mut count = 0;
//        for i in (0..self.pointer).rev() {
//            write!(f, "{}", self.data[i])?;
//            count += 1;
//            if count == 100 {
//                write!(f, "...")?;
//                break;
//            }
//        }
//        write!(f, ")")
//        //        write!(f, "{}]", self.data[self.data.len() - 1])
//    }
//}
//
//impl Debug for Stack {
//    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
//        write!(f, "Stack: (P:{}, ", self.pointer)?;
//        let mut count = 0;
//        for i in (0..self.pointer).rev() {
//            write!(f, "{:?}, ", self.data[i])?;
//            count += 1;
//            if count == 100 {
//                write!(f, "...")?;
//                break;
//            }
//        }
//        write!(f, ")")
//        //        write!(f, "{}]", self.data[self.data.len() - 1])
//    }
//}

impl Display for Puzzle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "stars: {}", self.stars, )?;
        write!(f, "{{")?;
        for &me in self.methods.iter() {
            if me > 0 {
                for i in 0..me {
                    write!(f, "_")?;
                }
                write!(f, ",")?;
            }
        }
        write!(f, "}}\nconditions: ")?;
        if self.red { write!(f, "{}", RED_PROBE)?; }
        if self.green { write!(f, "{}", GREEN_PROBE)?; }
        if self.blue { write!(f, "{}", BLUE_PROBE)?; }
        write!(f, "\nmarks: ")?;
        if self.marks[0] { write!(f, "{}", MARK_RED)?; }
        if self.marks[1] { write!(f, "{}", MARK_GREEN)?; }
        if self.marks[2] { write!(f, "{}", MARK_BLUE)?; }
        writeln!(f, "\nmap:")?;
        let (mut miny, mut minx, mut maxy, mut maxx) = (14, 18, 0, 0);
        for y in 1..13 {
            for x in 1..17 {
                if self.map[y][x] != _N {
                    miny = min(miny, y);
                    minx = min(minx, x);
                    maxy = max(maxy, y + 1);
                    maxx = max(maxx, x + 1);
                }
            }
        }
        for y in miny..maxy {
            for x in minx..maxx {
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
                let background = match tile.color() {
                    RE => Color::Red,
                    GE => Color::Green,
                    BE => Color::Blue,
                    _ => Color::Black,
                };
                let foreground = if self.y == y && self.x == x {
                    Color::BrightWhite
                } else if tile.has_star() {
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

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "At ({}, {}), stars: {}, Running: {}\nMap:\n",
            self.x, self.y, self.stars, self.running()
        )?;
        let (mut miny, mut minx, mut maxy, mut maxx) = (14, 18, 0, 0);
        for y in 1..13 {
            for x in 1..17 {
                if self.map[y][x] != _N {
                    miny = min(miny, y);
                    minx = min(minx, x);
                    maxy = max(maxy, y + 1);
                    maxx = max(maxx, x + 1);
                }
            }
        }
        for y in miny..maxy {
            for x in minx..maxx {
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
                let background = match tile.color() {
                    RE => Color::Red,
                    GE => Color::Green,
                    BE => Color::Blue,
                    _ => Color::Black,
                };
                let foreground = if self.y == y && self.x == x {
                    Color::BrightWhite
                } else if tile.has_star() {
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
