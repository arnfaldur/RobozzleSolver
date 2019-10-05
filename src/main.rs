use colored::*;
use std::fmt::{Display, Error, Formatter};
use rand::prelude::*;

#[allow(dead_code)]
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
    functions: [usize; 5],
    marks: [bool; 3],
    red: bool,
    green: bool,
    blue: bool,
}

struct State {
    running: bool,
    steps: usize,
    stack: Stack,
    map: Map,
    direction: Direction,
    x: usize,
    y: usize,
}

const RE: Tile = 0b0001;
const GE: Tile = 0b0010;
const BE: Tile = 0b0100;
const RS: Tile = 0b1001;
const GS: Tile = 0b1010;
const BS: Tile = 0b1100;
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
const INS_NOOP: Instruction = 0b00010000;
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
        if *ins != INS_NOOP {
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

fn execute(puzzle: &Puzzle, source: &Source) {
    let mut state: State = State {
        running: true,
        steps: 0,
        map: puzzle.map,
        stack: Stack {
            pointer: 0,
            data: [INS_NOOP; STACK_SIZE],
        },
        direction: puzzle.direction,
        x: puzzle.x,
        y: puzzle.y,
    };
    invoke(&mut state.stack, &source[0]);
    while state.running && state.stack.pointer < STACK_SIZE - 12 && state.steps < MAX_STEPS {
//        print!("{}\n------------------------------------------\n", state);
        state.running = step(&mut state, &source);
    }
    print!("{}", state);
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
    let instruction_set = get_instruction_set(&PUZZLE_656);
    let mut rng = rand::thread_rng();
    for iteration in 0..1024 {
        let mut source: Source = [[INS_NOOP; 10]; 5];
        for i in 0..5 {
            for ins in 0..PUZZLE_656.functions[i] {
                source[i][ins] = instruction_set[rng.gen_range(0,instruction_set.len())];
            }
        }
        show_source(&source);
        execute(&PUZZLE_656, &source);
//        println!("Instruction set:");
//        for instruction in get_instruction_set(&PUZZLE_656) {
//            print!("{}", show_instruction(instruction));
//        }
        println!("");
    }
}

fn score(state: &State) -> f32 {
    let mut result = 0.0;
    for y in 1..13 {
        for x in 1..17 {
            result += is_touched(state.map[y][x]) as i32 as f32 / (12.0 * 16.0);
            result -= has_star(state.map[y][x]) as i32 as f32;
        }
    }
    return result - (state.steps as f32 / MAX_STEPS as f32) / (12.0 * 16.0);// step count only breaks ties
}

fn get_instruction_set(puzzle: &Puzzle) -> Vec<Instruction> {
    let functions = puzzle.functions.iter().fold(0, |count, &val| count + (val > 0) as usize);
    let marks: usize = puzzle.marks.iter().map(|b| *b as usize).sum();
    let colors = 1 + puzzle.red as usize + puzzle.green as usize + puzzle.blue as usize;
    let mut result: Vec<Instruction> = Vec::with_capacity((3 + functions + marks) * colors);
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
    for function in source {
        let mut line = false;
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
        let mut count = 0;
        for i in (1..self.pointer + 1).rev() {
            write!(f, "{}", show_instruction(self.data[i]))?;
            count += 1;
            if count == 100 {
                write!(f, "...");
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
        INS_NOOP,
        INS_MARK_RED,
        INS_MARK_RED | INS_RED_COND,
        INS_MARK_RED | INS_GREEN_COND,
        INS_MARK_RED | INS_BLUE_COND,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
        INS_NOOP,
    ],
];

const PUZZLE_656_SOLUTION: Source = [
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
        INS_TURN_RIGHT | INS_RED_COND,
        INS_TURN_RIGHT | INS_RED_COND,
        INS_F2 | INS_BLUE_COND,
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
        [_N, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 1,
    y: 12,
    functions: [5, 5, 0, 0, 0],
    marks: [false; 3],
    red: true,
    green: false,
    blue: true,
};
