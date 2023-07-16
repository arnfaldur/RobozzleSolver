use std::fs::File;
use std::io::{prelude::*, Write};
use std::ops::{Range, RangeInclusive};
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

use self::errors::SolverError;

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
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async {
        //let mut file = File::create("data/solutions.txt").unwrap();
        //file.write_all(b"Hello, world!").unwrap();

        let mut file = File::options()
            .write(true)
            .create(true)
            .open("data/solutions.txt")
            .expect("unable to open/create solutions file");

        let mut contents = String::new();
        file.read_to_string(&mut contents);
        println!("contents: {}", contents);

        let driver = get_driver().await.unwrap();

        // while let Err(e) = login(&driver).await {
        //     println!("login error: {:?}", e);
        //     sleep(Duration::from_secs(1));
        // }

        solve_puzzle(&driver, puzzle_id).await.unwrap();
        driver.quit().await
    })
    .unwrap();
    return Ok(());
}
pub fn just_fetch_level(puzzle_id: u64) -> Result<Level, errors::SolverError> {
    if let Some(level_json) = get_local_level(puzzle_id) {
        println!("Found cached puzzle {}", puzzle_id);
        Ok(Level::from(level_json))
    } else {
        let mut rt = Runtime::new().unwrap();
        return rt.block_on(async {
            let driver = get_driver().await.unwrap();
            goto_puzzle_url(puzzle_id, &driver).await;
            let level_json = fetch_level_json(&driver, puzzle_id).await.unwrap();
            driver.quit().await.unwrap();
            Ok(Level::from(level_json))
        });
    }
}

pub fn just_fetch_many_levels(
    puzzle_id_range: RangeInclusive<u64>,
) -> Result<Vec<Level>, errors::SolverError> {
    let mut result = Vec::new();
    let mut last_standing = 0;
    for puzzle_id in puzzle_id_range.clone() {
        if let Some(level_json) = get_local_level(puzzle_id) {
            println!("Found cached puzzle {}", puzzle_id);
            result.push(Level::from(level_json));
        } else {
            last_standing = puzzle_id;
            break;
        }
    }
    if last_standing > 0 {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let driver = get_driver().await.unwrap();
            let level_jsons =
                fetch_many_levels_json(&driver, last_standing..=(puzzle_id_range.last().unwrap()))
                    .await
                    .unwrap();
            driver.quit().await.unwrap();
            result.append(&mut level_jsons.into_iter().map(Level::from).collect());
        });
    }
    return Ok(result);
}

async fn get_driver() -> Result<WebDriver, errors::SolverError> {
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();
    let tools = FirefoxTools::new(driver.handle.clone());
    tools
        .install_addon(
            std::fs::canonicalize("data/uBlock0_1.50.0.firefox.signed.xpi")
                .expect("unable to canonicalize addon path")
                .to_str()
                .unwrap(),
            Some(true),
        )
        .await
        .unwrap();
    Ok(driver)
}

async fn login(driver: &WebDriver) -> Result<(), errors::SolverError> {
    driver
        .goto("http://www.robozzle.com/beta/index.html")
        .await
        .unwrap();
    driver
        .find(By::Id("menu-signin"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();

    driver
        .find(By::Id("dialog-signin"))
        .await
        .unwrap()
        .wait_until()
        .condition(conditions::element_is_displayed(false))
        .await
        .unwrap();

    let mut signin_form = driver.form(By::Id("dialog-signin")).await.unwrap();
    signin_form.set_by_name("name", "Hugsun").await.unwrap();
    sleep(Duration::from_millis(500));
    signin_form
        .set_by_name("password", "8r4WvSfxHGirMDxH6FBO")
        .await
        .unwrap();
    sleep(Duration::from_millis(500));
    signin_form.submit().await.unwrap();
    sleep(Duration::from_millis(500));

    return Ok(());
}
async fn solve_puzzle(driver: &WebDriver, puzzle_id: u64) -> Result<(), errors::SolverError> {
    let mut url = goto_puzzle_url(puzzle_id, driver).await;

    let puzzle = fetch_puzzle(driver, puzzle_id).await;
    let mut solutions = backtrack(puzzle);
    solutions.sort_unstable_by_key(|sol| sol.0);
    solutions.sort_unstable_by_key(|sol| sol.1.count_ins());
    if let Some(solution) = solutions.pop() {
        println!("Trying solution: {}", solution.1);
        println!("that takes {} steps", solution.0);
        url.push_str("&program=");
        url.push_str(encode_program(&solution.1, &puzzle).as_str());
        driver.goto(url).await.unwrap();

        driver
            .execute("$('#program-speed')[0].value = '10'", vec![])
            .await
            .unwrap();
        driver
            .execute("$('#program-speed').trigger('change')", vec![])
            .await
            .unwrap();
        driver
            .find(By::Id("program-go"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        driver
            .find(By::Id("dialog-solved"))
            .await
            .unwrap()
            .wait_until()
            .condition(conditions::element_is_displayed(true))
            .await
            .unwrap()
    }
    return Ok(());
}
async fn goto_puzzle_url(puzzle_id: u64, driver: &WebDriver) -> String {
    let mut url = "http://www.robozzle.com/beta/index.html?puzzle=".to_string();
    url.push_str(puzzle_id.to_string().as_str());
    driver.goto(url.as_str()).await.unwrap();
    url
}
async fn fetch_puzzle(driver: &WebDriver, puzzle_id: u64) -> Puzzle {
    level_json_to_puzzle(&fetch_level_json(driver, puzzle_id).await.unwrap())
}
async fn fetch_many_levels_json(
    driver: &WebDriver,
    puzzle_id_range: RangeInclusive<u64>,
) -> Result<Vec<LevelJson>, errors::SolverError> {
    let mut result = Vec::new();
    let mut url = "http://www.robozzle.com/beta/index.html".to_string();
    driver.goto(url.as_str()).await.unwrap();
    // Disable top solver fetching
    let json = driver
        .execute("robozzle.topSolvers = () => {};", vec![])
        .await
        .unwrap();
    for puzzle_id in puzzle_id_range {
        if let Some(level_json) = get_local_level(puzzle_id) {
            println!("Found cached puzzle {}", puzzle_id);
            result.push(level_json);
        } else {
            println!("Fetching puzzle");
            // switch to different level
            let json = driver
                .execute_async(
                    &format!(
                        "
                        let done = arguments[0];
                        robozzle.service('GetLevel',
                        {{levelId: {}}},
                        function (result, response) {{
                            done(response.GetLevelResult);
                        }});",
                        puzzle_id
                    ),
                    Vec::new(),
                )
                .await
                .unwrap();
            let json = json.json();
            store_puzzle_locally(&json.to_string(), puzzle_id);
            println!("level: {}", json.to_string());
            serde_json::from_value(json.clone())
                .map_err(|err| {
                    eprintln!(
                        "Error {}: failed to deserialize puzzle {} JSON {}",
                        err,
                        puzzle_id,
                        json.clone()
                    );
                })
                .ok()
                .map(|level_json: LevelJson| {
                    result.push(level_json);
                });
        }
    }
    return Ok(result);
}

async fn fetch_level_json(driver: &WebDriver, puzzle_id: u64) -> Option<LevelJson> {
    if let Some(level_json) = get_local_level(puzzle_id) {
        println!("Found cached puzzle {}", puzzle_id);
        Some(level_json)
    } else {
        println!("Fetching puzzle");
        let json = driver
            .execute("return robozzle.level", vec![])
            .await
            .unwrap();
        let json = json.json();
        store_puzzle_locally(&json.to_string(), puzzle_id);
        println!("level: {}", json.to_string());
        return serde_json::from_value(json.clone())
            .map_err(|err| {
                eprintln!(
                    "Error {}: failed to deserialize puzzle {} JSON {}",
                    err,
                    puzzle_id,
                    json.clone()
                );
            })
            .ok();
    }
}

fn get_local_level(puzzle_id: u64) -> Option<LevelJson> {
    let mut path = PathBuf::from_str("data/puzzles").expect("unable to create puzzle pathbuf");
    path.push(puzzle_id.to_string());
    return File::options()
        .read(true)
        .open(path)
        .ok()
        .and_then(|mut file| {
            let mut string = String::new();
            file.read_to_string(&mut string);
            return serde_json::from_str(&string)
                .map_err(|err| {
                    eprintln!(
                        "Error {}: failed to deserialize puzzle {} JSON {}",
                        err,
                        puzzle_id,
                        string.clone()
                    );
                })
                .ok();
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
                    serde_json::from_str(&string).expect("couldn't deserialize JSON");
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
    File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|err| {
            eprintln!(
                "unable to store puzzle {} locally.\nerror: {}\ndata: {}",
                puzzle_id, err, json
            );
        })
        .map(|mut file| {
            file.write_all(json.as_bytes())
                .expect("unable to write puzzle json to file");
        });
}

pub fn puzzle_from_string(string: &str) -> Puzzle {
    let level_json: LevelJson = serde_json::from_str(string)
        .unwrap_or_else(|err| panic!("couldn't read JSON: {}\n error: {}", string, err));
    return level_json_to_puzzle(&level_json);
}

fn level_json_to_puzzle(level_json: &LevelJson) -> Puzzle {
    let mut map = PUZZLE_NULL.map.clone();
    for y in 0..12 {
        let mut cols = level_json.Colors[y].chars();
        let mut tems = level_json.Items[y].chars();
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
    let direction = match level_json.RobotDir.chars().next().unwrap_or(' ') {
        '0' => Direction::Right,
        '1' => Direction::Down,
        '2' => Direction::Left,
        '3' => Direction::Up,
        _ => Direction::Up,
    };
    let mut methods = [0; 5];
    for m in 0..5 {
        methods[m] = level_json.SubLengths[m].parse().unwrap();
    }
    let mflags: u8 = level_json.AllowedCommands.parse().unwrap();
    return make_puzzle(
        map,
        direction,
        level_json.RobotCol.parse::<usize>().unwrap() + 1,
        level_json.RobotRow.parse::<usize>().unwrap() + 1,
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
    pub comment_count: u64,
    pub difficulty_vote_count: u64,
    pub difficulty_vote_sum: u64,
    pub dislikes: u64,
    pub featured: bool,
    pub id: u64,
    pub likes: u64,
    pub solution_count: u64,
    pub submitted_by: Option<String>,
    pub submitted_date: String,
    pub title: String,
    pub puzzle: Puzzle,
}

impl From<LevelJson> for Level {
    fn from(value: LevelJson) -> Self {
        Level {
            puzzle: level_json_to_puzzle(&value),
            about: value.About,
            comment_count: value
                .CommentCount
                .parse::<u64>()
                .expect("comment count should be u64"),
            difficulty_vote_count: value
                .DifficultyVoteCount
                .parse::<u64>()
                .expect("difficulty vote count should be u64"),
            difficulty_vote_sum: value
                .DifficultyVoteSum
                .parse::<u64>()
                .expect("difficulty vote sum should be u64"),
            dislikes: value
                .Disliked
                .parse::<u64>()
                .expect("dislikes should be u64"),
            featured: value
                .Featured
                .parse::<bool>()
                .expect("featured should be a boolean"),
            id: value.Id.parse::<u64>().expect("puzzle id should be u64"),
            likes: value.Liked.parse::<u64>().expect("likes should be u64"),
            solution_count: value
                .Solutions
                .parse::<u64>()
                .expect("solution count should be u64"),
            submitted_by: value.SubmittedBy,
            submitted_date: value.SubmittedDate,
            title: value.Title,
        }
    }
}

// ---------------------------------------------------------------------------
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct LevelJson {
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
    SubmittedBy: Option<String>,
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
