use std::io::{stdout, Write};

use crate::game::{Puzzle, Source, instructions::*, Direction, make_puzzle, Tile};

use tokio::{prelude::*, runtime::Runtime};
use fantoccini::{Client, Locator};
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::constants::*;

pub fn start_web_solver() {
    fetch_puzzle(14);
}

fn fetch_puzzle(puzzle_id: u64) -> Result<(), fantoccini::error::CmdError> {
    let mut url = "http://www.robozzle.com/beta/index.html?puzzle=".to_string();
    url.push_str(puzzle_id.to_string().as_str());

    let rt = Runtime::new()?;

    rt.block_on(async {
        let mut c = Client::new("http://localhost:4444").await.unwrap_or_else(|hello| panic!("HERE {}", hello));


        c.goto(url.as_ref()).await?;

        let val = c.execute("return robozzle.level", vec![]).await?;
        let level_json: LevelJson = serde_json::from_value(val).unwrap();
        let puzzle = level_to_puzzle(&level_json);
        println!("level: {}", puzzle);

        thread::sleep(Duration::from_secs(10));
        c.close().await
    })
}

pub fn parse_level() -> Puzzle {
    let deseris: LevelJson = serde_json::from_str(LEVEL_JSON).unwrap();
    println!("deresir: {:?}", deseris);
    let puzz = level_to_puzzle(&deseris);
    println!("puzzle: {}", puzz);
    return puzz;
}

fn level_to_puzzle(level: &LevelJson) -> Puzzle {
    let mut map = PUZZLE_NULL.map.clone();
    for y in 0..12 {
        let mut cols = level.Colors[y].chars();
        let mut tems = level.Items[y].chars();
        for x in 0..16 {
            let color = match cols.next().unwrap_or(' ') {
                'R' => RE,
                'G' => GE,
                'B' => BE,
                _ => _N,
            };
            map[y + 1][x + 1] = match tems.next().unwrap_or(' ') {
                '*' => Tile(color.0 | TILE_STAR_MASK.0),
                '.' => color,
                _ => _N,
            }
        }
    }
    let direction = match level.RobotDir.chars().next().unwrap_or(' ') {
        '0' => Direction::Right,
        '1' => Direction::Down,
        '2' => Direction::Left,
        '3' => Direction::Up,
        _ => Direction::Up
    };
    let mut functions = [0; 5];
    for m in 0..5 {
        functions[m] = level.SubLengths[m].parse().unwrap();
    }
    let mflags: u8 = level.AllowedCommands.parse().unwrap();
    return make_puzzle(
        map,
        direction,
        level.RobotCol.parse::<usize>().unwrap() + 1,
        level.RobotRow.parse::<usize>().unwrap() + 1,
        functions,
        [(mflags & 0b1) > 0, (mflags & 0b10) > 0, (mflags & 0b100) > 0],
    );
}

// ---------------------------------------------------------------------------
#[derive(Serialize, Deserialize, Debug)]
struct LevelJson {
    About: String,
    AllowedCommands: String,
    Colors: Vec<String>,
    CommentCount: String,
    DifficultyVoteCount: String,
    DifficultyVoteSum: String,
    Disliked: String,
    Featured: String,
    Id: String,
    Items: Vec<String>,
    Liked: String,
    RobotCol: String,
    RobotDir: String,
    RobotRow: String,
    Solutions: String,
    SubLengths: Vec<String>,
    SubmittedBy: String,
    SubmittedDate: String,
    Title: String,
}

const LEVEL_JSON: &str = "{
\"About\": \"Collect starfruit! (See comments for hints - coming soon)\",
\"AllowedCommands\": \"0\",
\"Colors\": [
\"RRRRRRRRRRRRRRRR\",
\"RRRRRRGGGGRRRRRR\",
\"RRRRRGGGGGGRRRRR\",
\"RRRRRGGGGGGRRRRR\",
\"RRRRRGGRRGGGGGRR\",
\"RRRRRRGRRGGGGGGR\",
\"RRGGGRRRRRGRGGGR\",
\"RGGRRRRRRRRRRGGR\",
\"RGGGGGGRRRGGGGGR\",
\"RGGGGGRRRRRGGGRR\",
\"RRGGGRRRRRRRRRRR\",
\"BBBBBBBRRBBBBBBB\"
],
\"CommentCount\": \"0\",
\"DifficultyVoteCount\": \"0\",
\"DifficultyVoteSum\": \"0\",
\"Disliked\": \"0\",
\"Featured\": \"false\",
\"Id\": \"1874\",
\"Items\": [
\"################\",
\"######****######\",
\"#####*....*#####\",
\"#####*....*#####\",
\"#####*....****##\",
\"######*...*..**#\",
\"##***##..#..*.*#\",
\"#*............*#\",
\"#*....*..#*...*#\",
\"#*...*#..##***##\",
\"##***##..#######\",
\"...****.........\"
],
\"Liked\": \"0\",
\"RobotCol\": \"8\",
\"RobotDir\": \"0\",
\"RobotRow\": \"3\",
\"Solutions\": \"1\",
\"SubLengths\": [
\"10\",
\"6\",
\"4\",
\"2\",
\"0\"
],
\"SubmittedBy\": \"masterluk\",
\"SubmittedDate\": \"2010-04-10T12:56:13.157\",
\"Title\": \"Tree of Balance\"
}";

// ---------------------------------------------------------------------------

struct StateEncoder {
    output: String,
    val: usize,
    bits: usize,
}

impl StateEncoder {
    fn encode_bits(&mut self, val: usize, bits: usize) {
        for i in 0..bits {
            self.val |= (if val & (1 << i) > 0 { 1 } else { 0 }) << self.bits;
            self.bits += 1;
            if self.bits == 6 {
                let mut c;
                if self.val < 26 {
                    c = char::from((97 + self.val) as u8);
                } else if self.val < 52 {
                    c = char::from((65 + self.val - 26) as u8);
                } else if self.val < 62 {
                    c = char::from((48 + self.val - 52) as u8);
                } else {
                    c = '-';
                }
                self.output.push(c);
                self.val = 0;
                self.bits = 0;
            }
        }
    }
    fn encode_command(&mut self, cond: char, cmd: char) {
        self.encode_bits(match cond {
            'R' => 1,
            'G' => 2,
            'B' => 3,
            _ => 0,
        }, 2);
        self.encode_bits(match cmd {
            'f' => 1,
            'l' => 2,
            'r' => 3,
            '1' | '2' | '3' | '4' | '5' => 4,
            'R' | 'G' | 'B' => 5,
            _ => 0,
        }, 3);
        let sublen = match cmd {
            '1' | '2' | '3' | '4' | '5' => 3,
            'R' | 'G' | 'B' => 2,
            _ => 0,
        };
        if sublen != 0 {
            self.encode_bits(match cmd {
                '1' => 0,
                '2' | 'R' => 1,
                '3' | 'G' => 2,
                '4' | 'B' => 3,
                '5' => 4,
                _ => 0,
            }, sublen);
        }
    }
}

pub fn encode_program(program: &Source, puzzle: &Puzzle) -> String {
    let mut encode_state = StateEncoder {
        output: "".parse().unwrap(),
        val: 0,
        bits: 0,
    };
// program_length = robozzle.program.length; would be expected to be like this:
// let program_length = puzzle.functions.iter().filter(|&method| *method != 0).count();
    let program_length = 5; // but it seems to be like this always.
    encode_state.encode_bits(0, 3); // Version number = 0
    encode_state.encode_bits(program_length, 3);
    for i in 0..program_length {
        encode_state.encode_bits(puzzle.functions[i], 4);
        for j in 0..puzzle.functions[i] {
            let ins = program[i][j];
            encode_state.encode_command(match ins.get_cond() {
                RED_COND => 'R',
                GREEN_COND => 'G',
                BLUE_COND => 'B',
                _ => ' ',
            }, match ins.get_ins() {
                FORWARD => 'f',
                LEFT => 'l',
                RIGHT => 'r',
                F1 => '1',
                F2 => '2',
                F3 => '3',
                F4 => '4',
                F5 => '5',
                MARK_RED => 'R',
                MARK_GREEN => 'G',
                MARK_BLUE => 'B',
                _ => ' ',
            });
        }
    }
    encode_state.encode_bits(0, 5); // Flush
    return encode_state.output.clone();
}

//robozzle.decodeBits = function (decodeState, bits)
//{
//var val = 0;
//for (var i = 0; i < bits; i + + ) {
//if (decodeState.bits == 0) {
//var c = decodeState.input.charCodeAt(decodeState.index);
//decodeState.index + +;
//if (c > = 97 & & c < 97 + 26) {
//decodeState.val = c - 97;
//} else if (c > = 65 & & c < 65 + 26) {
//decodeState.val = c - 65 + 26;
//} else if (c > = 48 & & c < 48 + 10) {
//decodeState.val = c - 48 + 52;
//} else if (c == 95) {
//decodeState.val = 62;
//} else if (c == 45) {
//decodeState.val = 63;
//} else {
//decodeState.val = 0;
//}
//decodeState.bits = 6;
//}
//if (decodeState.val & (1 < < (6 - decodeState.bits))) {
//val |= (1 < < i);
//}
//decodeState.bits - -;
//}
//return val;
//};
//
//robozzle.decodeCommand = function (decodeState) {
//var cond = robozzle.decodeBits(decodeState, 2);
//switch (cond) {
//case 1: cond = 'R'; break;
//case 2: cond = 'G'; break;
//case 3: cond = 'B'; break;
//default: cond = null; break;
//}
//
//var cmd = robozzle.decodeBits(decodeState, 3);
//switch (cmd) {
//case 1: cmd = 'f'; break;
//case 2: cmd = 'l'; break;
//case 3: cmd = 'r'; break;
//case 4:
//var subcmd = robozzle.decodeBits(decodeState, 3);
//switch (subcmd) {
//case 0: cmd = '1'; break;
//case 1: cmd = '2'; break;
//case 2: cmd = '3'; break;
//case 3: cmd = '4'; break;
//case 4: cmd = '5'; break;
//default: cmd = null; break;
//}
//break;
//case 5:
//var subcmd = robozzle.decodeBits(decodeState, 2);
//switch (subcmd) {
//case 1: cmd = 'R'; break;
//case 2: cmd = 'G'; break;
//case 3: cmd = 'B'; break;
//default: cmd = null; break;
//}
//break;
//default: cmd = null; break;
//}
//
//return [ cond, cmd ];
//};
//
//robozzle.decodeProgram = function (input) {
//if ( ! input) {
//return null;
//}
//
//var decodeState = {
//input: input,
//index: 0,
//val: 0,
//bits: 0
//};
//
//var version = robozzle.decodeBits(decodeState, 3);
//if (version != 0) {
//return null;
//}
//
//var program = [];
//var length = robozzle.decodeBits(decodeState, 3);
//for (var j = 0; j < length; j+ + ) {
//var sub = [];
//var sublen = robozzle.decodeBits(decodeState, 4);
//for (var i = 0; i < sublen; i+ + ) {
//sub.push(robozzle.decodeCommand(decodeState));
//}
//program.push(sub);
//}
//
//return program;
//};

