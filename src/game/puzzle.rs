use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Display, Error, Formatter};

use serde::Deserialize;
use serde::Serialize;

use super::board::Board;
use super::instructions::Ins;
use super::state::State;
use super::Direction;
use super::Map;
use super::Source;
use crate::constants::*;
use crate::game::{instructions::*, Tile};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub board: Board,
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
            board: self.board,
            ..State::default()
        };
        result.initialize(source, self);
        return result;
    }
    /// execute a source for the puzzle, returning a score
    pub fn execute<F, R>(&self, source: &Source, show: bool, mut scoring: F) -> R
    where
        F: FnMut(&State, &Puzzle) -> R,
    {
        coz::begin!("execute");
        let mut state = self.initial_state(source);
        if show {
            println!("{}", state);
            println!("{}", source);
        }
        while state.step(&source, self) {
            // while state.steps(&source, self, 100000, Tile::MAX_TOUCHES as usize) {
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

pub fn make_puzzle(
    Board {
        map,
        direction,
        x,
        y,
    }: Board,
    mut methods: [usize; 5],
    marks: [bool; 3],
) -> Puzzle {
    // must be able to use markable colors
    let [mut red, mut green, mut blue] = marks;
    // test which colors are reachable
    let mut frontier = VecDeque::new();
    frontier.push_front((x, y));
    let mut visited = HashSet::new();
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
        board: Board {
            map: map_out,
            direction,
            x,
            y,
        },
        stars,
        methods,
        actual_methods,
        marks,
        red,
        green,
        blue,
    };
}

pub(crate) fn verify_puzzle(puzzle: &Puzzle) -> bool {
    let (mut red, mut green, mut blue) = (false, false, false);
    for y in 1..13 {
        for x in 1..17 {
            red |= puzzle.board.map.0[y][x].is_red();
            green |= puzzle.board.map.0[y][x].is_green();
            blue |= puzzle.board.map.0[y][x].is_blue();
        }
    }
    let stars: usize = puzzle.board.map.count_stars();
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

impl Display for Puzzle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "stars: {}", self.stars,)?;
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
        if self.red {
            write!(f, "{}", RED_PROBE)?;
        }
        if self.green {
            write!(f, "{}", GREEN_PROBE)?;
        }
        if self.blue {
            write!(f, "{}", BLUE_PROBE)?;
        }
        write!(f, "\nmarks: ")?;
        if self.marks[0] {
            write!(f, "{}", MARK_RED)?;
        }
        if self.marks[1] {
            write!(f, "{}", MARK_GREEN)?;
        }
        if self.marks[2] {
            write!(f, "{}", MARK_BLUE)?;
        }
        writeln!(f, "{}", self.board)?;

        write!(f, "")
    }
}

impl Debug for Puzzle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Puzzle")
            .field("board", &self.board)
            .field("stars", &self.stars)
            .field("methods", &self.methods)
            .field("actual_methods", &self.actual_methods)
            .field("marks", &self.marks)
            .field("red", &self.red)
            .field("green", &self.green)
            .field("blue", &self.blue)
            .finish()
    }
}
