use std::fs::File;
use std::io::{prelude::*, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thirtyfour::extensions::addons::firefox::FirefoxTools;
use thirtyfour::extensions::query::conditions;
use thirtyfour::fantoccini::error::CmdError;
use thirtyfour::prelude::*;
use tokio::runtime::Runtime;
use webdriver::common::LocatorStrategy::CSSSelector;

use crate::constants::*;
use crate::game::{instructions::*, make_puzzle, Direction, Puzzle, Source, Tile};
use crate::solver::backtrack::backtrack;
use tokio::io::AsyncRead;

mod errors;
#[cfg(test)]
mod tests;

struct Storage {
    solutions: Vec<Solution>,
}

enum Solution {
    Invalid,
    Unsolved(Puzzle),
    Solved(Puzzle, Source),
}

pub fn start_web_solver() {
    // let mut gecko = Command::new("/usr/bin/geckodriver")
    //     .stdin(Stdio::piped())
    //     .stdout(Stdio::piped())
    //     .stderr(Stdio::null()) // silence
    //     .spawn()
    //     .expect("unable to start geckodriver");

    // ctrlc::set_handler(move || {
    //     gecko.wait().expect("command wasn't running");
    // });

    solve_puzzles(81);

    // gecko.kill();
}

// username: Hugsun
// password: 8r4WvSfxHGirMDxH6FBO

pub fn solve_puzzles(puzzle_id: u64) -> Result<(), errors::SolverError> {
    let mut rt = Runtime::new()?;
    rt.block_on(async {
        //let mut file = File::create("data/solutions.txt")?;
        //file.write_all(b"Hello, world!")?;

        let mut file = File::options()
            .write(true)
            .create(true)
            .open("data/solutions.txt")
            .expect("unable to open/create solutions file");

        let mut contents = String::new();
        file.read_to_string(&mut contents);
        println!("contents: {}", contents);

        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;
        let tools = FirefoxTools::new(driver.handle.clone());
        tools
            .install_addon(
                std::fs::canonicalize("data/uBlock0_1.50.0.firefox.signed.xpi")
                    .expect("unable to canonicalize addon path")
                    .to_str()
                    .unwrap(),
                Some(true),
            )
            .await?;

        // while let Err(e) = login(&driver).await {
        //     println!("login error: {:?}", e);
        //     sleep(Duration::from_secs(1));
        // }

        solve_puzzle(&driver, puzzle_id).await?;
        driver.quit().await
    })?;
    return Ok(());
}

async fn login(driver: &WebDriver) -> Result<(), errors::SolverError> {
    driver
        .goto("http://www.robozzle.com/beta/index.html")
        .await?;
    driver.find(By::Id("menu-signin")).await?.click().await?;

    driver
        .find(By::Id("dialog-signin"))
        .await?
        .wait_until()
        .condition(conditions::element_is_displayed(false))
        .await?;

    let mut signin_form = driver.form(By::Id("dialog-signin")).await?;
    signin_form.set_by_name("name", "Hugsun").await?;
    sleep(Duration::from_millis(500));
    signin_form
        .set_by_name("password", "8r4WvSfxHGirMDxH6FBO")
        .await?;
    sleep(Duration::from_millis(500));
    signin_form.submit().await?;
    sleep(Duration::from_millis(500));

    return Ok(());
}
async fn solve_puzzle(driver: &WebDriver, puzzle_id: u64) -> Result<(), errors::SolverError> {
    let mut url = "http://www.robozzle.com/beta/index.html?puzzle=".to_string();
    url.push_str(puzzle_id.to_string().as_str());
    driver.goto(url.as_str()).await?;

    let puzzle = fetch_puzzle(driver, puzzle_id).await?;
    let mut solutions = backtrack(puzzle);
    solutions.sort_unstable_by_key(|sol| sol.0);
    solutions.sort_unstable_by_key(|sol| sol.1.count_ins());
    if let Some(solution) = solutions.pop() {
        println!("Trying solution: {}", solution.1);
        println!("that takes {} steps", solution.0);
        url.push_str("&program=");
        url.push_str(encode_program(&solution.1, &puzzle).as_str());
        driver.goto(url).await?;

        driver
            .execute("$('#program-speed')[0].value = '10'", vec![])
            .await?;
        driver
            .execute("$('#program-speed').trigger('change')", vec![])
            .await?;
        driver.find(By::Id("program-go")).await?.click().await?;
        driver
            .find(By::Id("dialog-solved"))
            .await?
            .wait_until()
            .condition(conditions::element_is_displayed(true))
            .await?
    }
    return Ok(());
}
async fn fetch_puzzle(driver: &WebDriver, puzzle_id: u64) -> Result<Puzzle, errors::SolverError> {
    if let Some(level) = get_local_level(puzzle_id) {
        let puzzle = level_to_puzzle(&level);
        println!("Found cached puzzle");
        println!("puzzle: {}", puzzle);
        Ok(puzzle)
    } else {
        println!("Fetching puzzle");
        let json = driver.execute("return robozzle.level", vec![]).await?;
        let json = json.json();
        if let Value::Null = json {
            return Err(errors::SolverError::Error(
                "Puzzle doesn't exist".to_string(),
            ));
        }
        store_puzzle_locally(&json.to_string(), puzzle_id);
        println!("level: {}", json.to_string());
        let level_json: LevelJson =
            serde_json::from_value(json.clone()).expect("couldn't read JSON");
        let puzzle = level_to_puzzle(&level_json);
        println!("puzzle: {}", puzzle);
        Ok(puzzle)
    }
}

fn get_local_level(puzzle_id: u64) -> Option<LevelJson> {
    let mut path = PathBuf::from_str("data/puzzles").expect("unable to create puzzle pathbuf");
    path.push(puzzle_id.to_string());
    return File::options().read(true).open(path).ok().map(|mut file| {
        let mut string = String::new();
        file.read_to_string(&mut string);
        println!("level: {}", string);
        let level_json: LevelJson = serde_json::from_str(&string).expect("couldn't read JSON");
        return level_json;
    });
}

pub fn get_all_local_levels() -> Vec<Level> {
    let mut result = Vec::new();
    for dir in std::fs::read_dir("data/puzzles").expect("unable to read puzzle directory") {
        let path = dir.expect("unable to read dir").path();
        let level_json = File::options()
            .read(true)
            .open(path)
            .map(|mut file| {
                let mut string = String::new();
                file.read_to_string(&mut string);
                let level_json: LevelJson =
                    serde_json::from_str(&string).expect("couldn't read JSON");
                return Level::from(level_json);
            })
            .expect("should be opening an existing file");
        result.push(level_json);
    }
    result.sort_by_key(|lvl| lvl.id.clone());
    return result;
}

fn store_puzzle_locally(json: &str, puzzle_id: u64) {
    let mut path = PathBuf::from_str("data/puzzles").expect("unable to create puzzle pathbuf");
    path.push(puzzle_id.to_string());
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .expect("unable to open puzzle file");
    file.write_all(json.as_bytes())
        .expect("unable to write puzzle json to file");
}

pub fn puzzle_from_string(string: &str) -> Puzzle {
    let level_json: LevelJson = serde_json::from_str(string)
        .unwrap_or_else(|err| panic!("couldn't read JSON: {}\n error: {}", string, err));
    return level_to_puzzle(&level_json);
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
        _ => Direction::Up,
    };
    let mut methods = [0; 5];
    for m in 0..5 {
        methods[m] = level.SubLengths[m].parse().unwrap();
    }
    let mflags: u8 = level.AllowedCommands.parse().unwrap();
    return make_puzzle(
        map,
        direction,
        level.RobotCol.parse::<usize>().unwrap() + 1,
        level.RobotRow.parse::<usize>().unwrap() + 1,
        methods,
        [
            (mflags & 0b1) > 0,
            (mflags & 0b10) > 0,
            (mflags & 0b100) > 0,
        ],
    );
}

pub struct Level {
    pub about: Value,
    pub comment_count: String,
    pub difficulty_vote_count: String,
    pub difficulty_vote_sum: String,
    pub disliked: String,
    pub featured: String,
    pub id: u64,
    pub liked: String,
    pub solutions: String,
    pub submitted_by: String,
    pub submitted_date: String,
    pub title: String,
    pub puzzle: Puzzle,
}

impl From<LevelJson> for Level {
    fn from(value: LevelJson) -> Self {
        Level {
            puzzle: level_to_puzzle(&value),
            about: value.About,
            comment_count: value.CommentCount,
            difficulty_vote_count: value.DifficultyVoteCount,
            difficulty_vote_sum: value.DifficultyVoteSum,
            disliked: value.Disliked,
            featured: value.Featured,
            id: value.Id.parse::<u64>().expect("puzzle id should be u64"),
            liked: value.Liked,
            solutions: value.Solutions,
            submitted_by: value.SubmittedBy,
            submitted_date: value.SubmittedDate,
            title: value.Title,
        }
    }
}

// ---------------------------------------------------------------------------
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct LevelJson {
    About: Value,
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
        self.encode_bits(
            match cond {
                'R' => 1,
                'G' => 2,
                'B' => 3,
                _ => 0,
            },
            2,
        );
        self.encode_bits(
            match cmd {
                'f' => 1,
                'l' => 2,
                'r' => 3,
                '1' | '2' | '3' | '4' | '5' => 4,
                'R' | 'G' | 'B' => 5,
                _ => 0,
            },
            3,
        );
        let sublen = match cmd {
            '1' | '2' | '3' | '4' | '5' => 3,
            'R' | 'G' | 'B' => 2,
            _ => 0,
        };
        if sublen != 0 {
            self.encode_bits(
                match cmd {
                    '1' => 0,
                    '2' | 'R' => 1,
                    '3' | 'G' => 2,
                    '4' | 'B' => 3,
                    '5' => 4,
                    _ => 0,
                },
                sublen,
            );
        }
    }
}

fn actualize_solution(program: &Source, puzzle: &Puzzle) -> Source {
    let mut result = *program;
    if puzzle.methods != puzzle.actual_methods {
        let (mut mapping, mut invmap) = ([5; 5], [5; 5]);
        let mut marked = [false; 5];
        for i in 0..5 {
            for j in 0..5 {
                if puzzle.actual_methods[i] == puzzle.methods[j] && !marked[j] {
                    mapping[i] = j;
                    marked[j] = true;
                    break;
                }
            }
            invmap[mapping[i]] = i;
        }
        for m in 0..5 {
            result[m] = program[mapping[m]];
            for i in 0..10 {
                if result[m][i].is_function() {
                    result[m][i] = result[m][i].get_cond()
                        | Ins::fun_from_index(invmap[result[m][i].source_index()]);
                }
            }
        }
    }
    return result;
}

pub fn encode_program(program: &Source, puzzle: &Puzzle) -> String {
    let solution = actualize_solution(program, puzzle);
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
        encode_state.encode_bits(puzzle.methods[i], 4);
        for j in 0..puzzle.methods[i] {
            let ins = solution[i][j];
            encode_state.encode_command(
                match ins.get_cond() {
                    RED_COND => 'R',
                    GREEN_COND => 'G',
                    BLUE_COND => 'B',
                    _ => ' ',
                },
                match ins.get_ins() {
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
                },
            );
        }
    }
    encode_state.encode_bits(0, 5); // Flush
    return encode_state.output.clone();
}
