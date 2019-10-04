use colored::*;
use std::fmt::{Display, Error, Formatter};

type Tile = u8;

type Map = [[Tile; 18]; 14];

type Instruction = u8;
type Method = [Instruction; 10];
type Source = [Method; 5];

const STACK_SIZE: usize = 2048;

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
    functions: [usize; 5],
    marks: [bool; 4],
    red: bool,
    green: bool,
    blue: bool,
}

struct State {
    running: bool,
    stack: Stack,
    map: Map,
    direction: Direction,
    x: usize,
    y: usize,
}

const NE: Tile = 0b0000;
const RE: Tile = 0b0001;
const GE: Tile = 0b0010;
const BE: Tile = 0b0100;
const NS: Tile = 0b1000;
const RS: Tile = 0b1001;
const GS: Tile = 0b1010;
const BS: Tile = 0b1100;
const _N: Tile = 0b10000;
const TILE_STAR_MASK: Tile = 0b00001000;
const TILE_COLOR_MASK: Tile = 0b00010111;

const INS_FORWARD: Instruction = 0;
const INS_TURN_LEFT: Instruction = 1;
const INS_TURN_RIGHT: Instruction = 2;
const INS_F1: Instruction = 3;
const INS_F2: Instruction = 4;
const INS_F3: Instruction = 5;
const INS_F4: Instruction = 6;
const INS_F5: Instruction = 7;
const INS_MARK_GRAY: Instruction = 0b00001000;
const INS_MARK_RED: Instruction = 0b00001001;
const INS_MARK_GREEN: Instruction = 0b00001010;
const INS_MARK_BLUE: Instruction = 0b00001100;

const INS_MARK_MASK: Instruction = 0b00000111;
const INS_NOOP: Instruction = 0b00010000;
const INS_INS_MASK: Instruction = 0b00011111;

const INS_GRAY_COND: Instruction = 0b00000000;
const INS_RED_COND: Instruction = 0b00100000;
const INS_GREEN_COND: Instruction = 0b01000000;
const INS_BLUE_COND: Instruction = 0b10000000;
const INS_COLOR_MASK: Instruction = 0b11100000;

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
        INS_MARK_GREEN | INS_RED_COND,
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
        INS_MARK_GREEN | INS_GREEN_COND,
        INS_MARK_GREEN | INS_BLUE_COND,
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
        INS_MARK_BLUE,
        INS_MARK_BLUE | INS_RED_COND,
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
        INS_MARK_BLUE | INS_GREEN_COND,
        INS_MARK_BLUE | INS_BLUE_COND,
    ],

    [
        INS_NOOP,
        INS_MARK_GRAY,
        INS_MARK_GRAY | INS_RED_COND,
        INS_MARK_GRAY | INS_GREEN_COND,
        INS_MARK_GRAY | INS_BLUE_COND,
        INS_MARK_RED,
        INS_MARK_RED | INS_RED_COND,
        INS_MARK_RED | INS_GREEN_COND,
        INS_MARK_RED | INS_BLUE_COND,
        INS_NOOP,
    ],

];

const PUZZLE_656_SOURCE: Source = [
    [
        INS_TURN_LEFT,
        INS_F2,
        INS_TURN_LEFT,
        INS_FORWARD,
        INS_F1,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
    ],
    [
        INS_FORWARD,
        INS_F2 | INS_BLUE_COND,
        INS_TURN_RIGHT | INS_RED_COND,
        INS_TURN_RIGHT | INS_RED_COND,
        INS_FORWARD,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
    ],
    [INS_NOOP; 10],
    [INS_NOOP; 10],
    [INS_NOOP; 10],
];
const PUZZLE_656_MAP: Map = [
    [
        _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N,
    ],
    [
        _N, RS, _N, _N, _N, RS, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N,
    ],
    [
        _N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, RS, _N, _N, _N, _N,
    ],
    [
        _N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N,
    ],
    [
        _N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, RS, _N, _N,
    ],
    [
        _N, BS, _N, RS, _N, BS, _N, _N, _N, RS, _N, _N, _N, BS, _N, BS, _N, _N,
    ],
    [
        _N, BS, _N, BS, _N, BS, _N, _N, _N, BS, _N, RS, _N, BS, _N, BS, _N, _N,
    ],
    [
        _N, BS, RS, BS, _N, BS, _N, _N, _N, BS, _N, BS, _N, BS, _N, BS, _N, _N,
    ],
    [
        _N, BS, BS, BS, _N, BS, RS, _N, _N, BS, _N, BS, _N, BS, RS, BS, _N, _N,
    ],
    [
        _N, BS, BS, BS, _N, BS, BS, _N, _N, BS, RS, BS, _N, BS, BS, BS, _N, _N,
    ],
    [
        _N, BS, BS, BS, _N, BS, BS, _N, _N, BS, BS, BS, RS, BS, BS, BS, RS, _N,
    ],
    [
        _N, BS, BS, BS, RS, BS, BS, RS, RS, BS, BS, BS, BS, BS, BS, BS, BS, _N,
    ],
    [
        _N, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, _N,
    ],
    [
        _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N,
    ],
];

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
        if *ins == INS_NOOP {
            continue;
        }
        stack.pointer += 1;
        stack.data[stack.pointer] = *ins;
    }
}

fn step(state: &mut State, source: &Source) {
    let instruction: Instruction = state.stack.data[state.stack.pointer] & INS_INS_MASK;
    let color: u8 = (state.stack.data[state.stack.pointer] & INS_COLOR_MASK) >> 5;
    state.stack.pointer -= 1;
    let map_color: u8 = state.map[state.y][state.x] & TILE_COLOR_MASK;
    if color != 0 && map_color != 0 && color != map_color {
        return;
    }
    match instruction {
        INS_FORWARD => {
            match &state.direction {
                Direction::Up => state.y -= 1,
                Direction::Left => state.x -= 1,
                Direction::Down => state.y += 1,
                Direction::Right => state.x += 1,
            };
            state.map[state.y][state.x] &= TILE_COLOR_MASK;
            state.running = state.map[state.y][state.x] != _N;
        }
        INS_TURN_LEFT => state.direction = left(&state.direction),
        INS_TURN_RIGHT => state.direction = right(&state.direction),
        INS_F1...INS_F5 => invoke(&mut state.stack, &source[usize::from(instruction - INS_F1)]),
        INS_MARK_GRAY...INS_MARK_BLUE => state.map[state.y][state.x] = instruction & INS_MARK_MASK,
        _ => return,
    }
}

fn execute(puzzle: &Puzzle, source: &Source) {
    let mut state: State = State {
        running: true,
        map: puzzle.map,
        stack: Stack {
            pointer: 0,
            data: [INS_NOOP; STACK_SIZE],
        },
        direction: puzzle.direction,
        x: puzzle.x,
        y: puzzle.y,
    };
    invoke(&mut state.stack, &PUZZLE_656_SOURCE[0]);
    while state.running && state.stack.pointer > 0 && state.stack.pointer < STACK_SIZE - 12 {
//        print!("{}\n------------------------------------------\n", state);
        step(&mut state, &PUZZLE_656_SOURCE);
    }
    print!("{}", state);
}

fn make_puzzle(
    map: Map,
    direction: Direction,
    x: usize,
    y: usize,
    functions: [usize; 5],
    marks: [bool; 4],
) -> Puzzle {
    let monocolor = !marks.iter().any(|i| *i)
        && [RE, GE, BE]
        .iter()
        .map(|col| {
            map.iter()
                .all(|row| row.iter().all(|tile| ((tile & TILE_COLOR_MASK) & col) > 0))
        })
        .any(|i| i);
    let red = !monocolor
        && map
        .iter()
        .any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & RE) > 0));
    let green = !monocolor
        && map
        .iter()
        .any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & GE) > 0));
    let blue = !monocolor
        && map
        .iter()
        .any(|row| row.iter().any(|tile| ((tile & TILE_COLOR_MASK) & BE) > 0));
    return Puzzle {
        map,
        direction,
        x,
        y,
        functions,
        marks,
        red,
        green,
        blue,
    };
}

fn main() {
    let puzzle = make_puzzle(
        PUZZLE_656_MAP,
        Direction::Right,
        1,
        12,
        [5, 5, 0, 0, 0],
        [false, false, false, false],
    );
    show_source(&TEST_SOURCE);
    execute(&puzzle, &PUZZLE_656_SOURCE);
}

fn get_all_instructions(puzzle: &Puzzle) -> Vec<Instruction> {
    let functions = puzzle
        .functions
        .iter()
        .fold(0, |count, &val| count + if val > 0 { 1 } else { 0 });
    let marks: usize = puzzle.marks.iter().map(|b| if *b { 1 } else { 0 }).sum();
    let colors = 1
        + if puzzle.red { 1 } else { 0 }
        + if puzzle.green { 1 } else { 0 }
        + if puzzle.blue { 1 } else { 0 };
    let mut result = Vec::with_capacity((3 + functions + marks) * colors);

    return result;
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
        INS_MARK_GRAY => Color::BrightBlack,
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
        INS_MARK_GRAY...INS_MARK_BLUE => "●".to_string(),
        _ => " ".to_string(),
    };
    return string.color(foreground).on_color(background);
}

fn show_source(source: &Source) {
    let mut line = false;
    for function in source {
        for instruction in function {
            if *instruction != INS_NOOP {
                line = true;
                print!("{}", show_instruction(*instruction));
            }
        }
        if line {
            print!("\n")
        }
    }
}


impl Display for Stack {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Stack: ({}, ", self.pointer)?;
        for i in (1..self.pointer + 1).rev() {
            write!(f, "{}", show_instruction(self.data[i]))?;
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
                    NE => Color::BrightBlack,
                    RE => Color::Red,
                    GE => Color::Green,
                    BE => Color::Blue,
                    _ => Color::Black,
                };
                let foreground = if tile & TILE_STAR_MASK == 0 {
                    if self.y == y && self.x == x {
                        Color::BrightWhite
                    } else {
                        background
                    }
                } else {
                    Color::Yellow
                };
                write!(f, "{}", string.color(foreground).on_color(background))?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
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
