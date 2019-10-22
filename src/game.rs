use colored::*;
use std::fmt::{Display, Error, Formatter};
use std::ops::{IndexMut, Index};
use std::cmp::{max, min};

use super::constants::*;

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Tile(pub u8);

impl Tile {
    fn touch(&mut self) { self.0 |= TILE_TOUCHED.0; }
    fn clear_star(&mut self) { self.0 &= !TILE_STAR_MASK.0; }
    pub(crate) fn is_touched(&self) -> bool { (self.0 & TILE_TOUCHED.0) > 0 }
    pub(crate) fn has_star(&self) -> bool { (self.0 & TILE_STAR_MASK.0) > 0 }
    fn color(&self) -> Tile { Tile(self.0 & TILE_COLOR_MASK.0) }
    fn is_red(&self) -> bool { self.0 & RE.0 > 0 }
    fn is_green(&self) -> bool { self.0 & GE.0 > 0 }
    fn is_blue(&self) -> bool { self.0 & BE.0 > 0 }
    pub(crate) fn executes(&self, instruction: Instruction) -> bool {
        let color = instruction.color_condition();
        return color == GRAY_COND || color.0 == self.color().0;
    }
    fn mark(&mut self, instruction: Instruction) {
        self.0 = instruction.get_mark_color().0 | TILE_TOUCHED.0
    }
    pub(crate) fn to_condition(&self) -> Instruction { Instruction(self.0 << 5) }
    pub(crate) fn get_probes(&self, colors: Instruction) -> Vec<Instruction> {
        let mask = (colors & Instruction(!self.to_condition().0)).to_probe();
//        let mut result = PROBES.into_vec();
//        result.remove_item(*self.to_condition().to_probe());
        return PROBES.iter().filter(|&ins| (*ins & mask) == *ins).cloned().collect();
    }
}

type Map = [[Tile; 18]; 14];

#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct Instruction(pub u8);

impl Instruction {
    fn color_condition(self) -> Instruction { Instruction(self.0 >> 5) }
    pub(crate) fn source_index(self) -> usize { (self.get_instruction().0 - F1.0) as usize }
    pub(crate) fn get_condition(self) -> Instruction { self & INS_COLOR_MASK }
    pub(crate) fn get_instruction(self) -> Instruction { self & INS_MASK }
    fn get_fun_number(self) -> u8 { self.0 - F1.0 + 1 }
    fn get_mark_color(self) -> Instruction { self & MARK_MASK }
    fn get_marker(self) -> Instruction { self.get_instruction() | NOP }
    fn from_marker(self) -> Instruction { Instruction(self.0 & !NOP.0) }
    fn red(self) -> Instruction { self | RED_COND }
    fn green(self) -> Instruction { self | GREEN_COND }
    fn blue(self) -> Instruction { self | BLUE_COND }
    fn is_red(self) -> bool { (self & RED_COND).0 > 0 }
    fn is_green(self) -> bool { (self & GREEN_COND).0 > 0 }
    fn is_blue(self) -> bool { (self & BLUE_COND).0 > 0 }
    fn to_probe(self) -> Instruction { self.get_condition() | NOP }
    pub(crate) fn is_probe(self) -> bool { self.get_condition().0 > 0 && self.get_instruction() == NOP }
}

impl From<Instruction> for u8 {
    fn from(ins: Instruction) -> Self {
        ins.0
    }
}

impl From<u8> for Instruction {
    fn from(val: u8) -> Self {
        Instruction(val)
    }
}

impl std::ops::BitOr for Instruction {
    type Output = Instruction;
    fn bitor(self, rhs: Self) -> Self::Output {
        Instruction(self.0.bitor(rhs.0))
    }
}

impl std::ops::BitAnd for Instruction {
    type Output = Instruction;
    fn bitand(self, rhs: Self) -> Self::Output {
        Instruction(self.0.bitand(rhs.0))
    }
}

type Method = [Instruction; 10];

#[derive(Eq, PartialEq, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct Source(pub [Method; 5]);

impl Source {
    fn len(&self) -> usize {
        self.0.len()
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

const STACK_SIZE: usize = 1 << 10;
pub(crate) const MAX_STEPS: usize = 1 << 12;

pub struct Stack {
    pointer: usize,
    pub data: [Instruction; STACK_SIZE],
}

impl Index<usize> for Stack {
    type Output = Instruction;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.pointer - index - 1]
    }
}

impl Stack {
    pub(crate) fn push(&mut self, element: Instruction) {
        self.data[self.pointer] = element;
        self.pointer += 1;
    }
    fn pop(&mut self) -> Instruction {
        let result = self.top();
        self.pointer -= 1;
        return result;
    }
    pub(crate) fn top(&self) -> Instruction { self.data[self.pointer - 1] }
    pub(crate) fn len(&self) -> usize { self.pointer }
    fn empty(&self) -> bool { self.pointer == 0 }
    pub(crate) fn clear(&mut self) { self.pointer = 0; }
}

#[derive(Copy, Clone, Hash)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
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

pub struct Puzzle {
    pub(crate) map: Map,
    pub(crate) direction: Direction,
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) stars: usize,
    pub(crate) functions: [usize; 5],
    pub(crate) marks: [bool; 3],
    pub(crate) red: bool,
    pub(crate) green: bool,
    pub(crate) blue: bool,
}

impl Puzzle {
    pub(crate) fn get_instruction_set(&self, colors: Instruction, gray: bool) -> Vec<Instruction> {
        let functions = self.functions.iter().fold(0, |count, &val| count + (val > 0) as usize);
        let marks: usize = self.marks.iter().map(|b| *b as usize).sum();
        let (red, green, blue) = (self.red & colors.is_red(), self.green & colors.is_green(), self.blue & colors.is_blue());
        let colors = gray as usize + red as usize + green as usize + blue as usize;
        let mut result: Vec<Instruction> = Vec::with_capacity((3 + functions + marks) * colors);
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
                if self.functions[i] > 0 {
                    result.push(FUNCTIONS[i] | condition);
                }
            }
            for i in 0..MARKS.len() {
                if self.marks[i] {
                    result.push(MARKS[i] | condition);
                }
            }
        }
        return result;
    }
    pub(crate) fn empty_source(&self) -> Source {
        let mut result = NOGRAM.clone();
        for instructions in 0..self.functions.len() {
            for i in 0..self.functions[instructions] {
                result.0[instructions][i] = NOP;
            }
        }
        return result;
    }
    pub(crate) fn initial_state(&self) -> State {
        State {
            stars: self.stars,
            map: self.map,
            direction: self.direction,
            x: self.x,
            y: self.y,
            ..State::default()
        }
    }
    pub(crate) fn execute<F, R>(&self, source: &Source, scoring: F) -> R where F: Fn(&State, &Puzzle) -> R {
        let mut state = self.initial_state();
        state.stack.push(F1);
        while state.running() {
            state.step(&source, self);
            println!("{}", state);
        }
        return scoring(&state, self);
    }
    pub(crate) fn get_color_mask(&self) -> Instruction {
        Instruction(
            (self.red as u8) * RED_COND.0
                | (self.green as u8) * GREEN_COND.0
                | (self.blue as u8) * BLUE_COND.0)
    }
}

pub struct State {
    pub(crate) steps: usize,
    pub(crate) stars: usize,
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
            stack: Stack {
                pointer: 0,
                data: [HALT; STACK_SIZE],
            },
            map: [[_N; 18]; 14],
            direction: Direction::Up,
            x: 1,
            y: 1,
        }
    }
}

impl State {
    pub(crate) fn current_tile(&mut self) -> &mut Tile { &mut self.map[self.y][self.x] }
    pub(crate) fn running(&self) -> bool {
        !self.stack.empty() && self.stars > 0 && self.stack.len() < STACK_SIZE - 12 && self.steps < MAX_STEPS
    }
    pub(crate) fn step(&mut self, source: &Source, puzzle: &Puzzle) {
        let ins = self.stack.pop();
        self.steps += 1;
        if self.current_tile().executes(ins) {
            match ins.get_instruction() {
                FORWARD => {
                    match &self.direction {
                        Direction::Up => self.y -= 1,
                        Direction::Left => self.x -= 1,
                        Direction::Down => self.y += 1,
                        Direction::Right => self.x += 1,
                    };
                    if *self.current_tile() == _N {
                        self.stack.clear();
                    } else {
                        self.stars -= self.current_tile().has_star() as usize;
                        self.current_tile().clear_star();
                        self.current_tile().touch();
                    }
                }
                LEFT => self.direction = self.direction.left(),
                RIGHT => self.direction = self.direction.right(),
                F1 | F2 | F3 | F4 | F5 => {
                    self.stack.push(ins.get_marker());
                    for i in (0..puzzle.functions[ins.source_index()]).rev() {
                        let ins = source.0[ins.source_index()][i];
                        if ins != HALT {
                            self.stack.push(ins);
                        }
                    }
                }
                MARK_RED | MARK_GREEN | MARK_BLUE => self.current_tile().mark(ins),
                _ => (),
            }
        }
    }
    pub(crate) fn stack_frame(&self) -> Instruction {
        for i in (0..self.stack.pointer).rev() {
            match self.stack.data[i] {
                F1_MARKER | F2_MARKER | F3_MARKER | F4_MARKER | F5_MARKER => return self.stack.data[i].from_marker(),
                _ => (),
            }
        }
        println!("couldn't find a stack frame");
        return F1;
    }
    pub(crate) fn instruction_number(&self) -> usize {
        let mut result = 0;
        for i in (0..self.stack.pointer).rev() {
            match self.stack.data[i] {
                F1_MARKER | F2_MARKER | F3_MARKER | F4_MARKER | F5_MARKER => { return result; }
                _ => result += 1,
            }
        }
        return result;
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
    let (mut red, mut green, mut blue) = (false, false, false);
    for y in 1..13 {
        for x in 1..17 {
            red |= map[y][x].is_red();
            green |= map[y][x].is_green();
            blue |= map[y][x].is_blue();
        }
    }
    let stars: usize = map.iter().map(|row| row.iter().map(|el| el.has_star() as usize).sum::<usize>()).sum();
    return Puzzle { map, direction, x, y, stars, functions, marks, red, green, blue };
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

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let background = match self.get_condition() {
            GRAY_COND => Color::BrightBlack,
            RED_COND => Color::Red,
            GREEN_COND => Color::Green,
            BLUE_COND => Color::Blue,
            _ => Color::Black,
        };
        let ins = self.get_instruction();
        let foreground = match ins {
            MARK_RED => Color::Red,
            MARK_GREEN => Color::Green,
            MARK_BLUE => Color::Blue,
            _ => Color::BrightWhite,
        };
        let string = match ins {
            FORWARD => "↑".to_string(),
            LEFT => "←".to_string(),
            RIGHT => "→".to_string(),
            F1 | F2 | F3 | F4 | F5 => ins.get_fun_number().to_string(),
            MARK_RED | MARK_GREEN | MARK_BLUE => "●".to_string(),
            NOP => match *self {
                RED_PROBE | GREEN_PROBE | BLUE_PROBE => "p".to_string(),
                _ => " ".to_string(),
            }
            _ => " ".to_string(),
        };
        write!(f, "{}", string.color(foreground).on_color(background))
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{")?;
        for i in 0..self.len() {
            let mut separate = false;
            for instruction in self[i].iter() {
                if *instruction == HALT {
                    write!(f, "|")?;
//                    break;
                } else if *instruction == NOP {
                    write!(f, "_")?;
                } else if *instruction != NOP {
                    write!(f, "{}", instruction)?;
                }
                separate = true;
            }
            if separate { write!(f, ",")?; }
        }
        write!(f, "}}")
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Stack: ({}, ", self.pointer)?;
        let mut count = 0;
        for i in (0..self.pointer).rev() {
            write!(f, "{}", self.data[i])?;
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
            "At ({}, {}), stars: {}, Running: {}\n{}\nMap:\n",
            self.x, self.y, self.stars, self.running(), self.stack
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
