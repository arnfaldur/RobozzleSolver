use std::fs::File;
use std::io::{prelude::*, ErrorKind, Write};
use std::ops::{Range, RangeInclusive};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::result;
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

// username: Hugsun
// password: 8r4WvSfxHGirMDxH6FBO

pub fn solve_puzzle(puzzle_id: u64, login: bool) -> Result<(), SolverError> {
    let mut rt = Runtime::new()?;
    rt.block_on(async {
        let driver = get_driver().await?;

        if login {
            perform_login(&driver).await?
        }
        let mut url = goto_puzzle_url(puzzle_id, &driver).await;

        // let puzzle =
        //     level_json_to_puzzle(&fetch_level_json(&driver, puzzle_id).await.unwrap()).await;
        let puzzle = get_level(puzzle_id)?.puzzle;
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
        driver.quit().await
    });
    return Ok(());
}
pub fn get_levels(
    puzzle_ids: impl Iterator<Item = u64>,
) -> impl Iterator<Item = Result<Level, SolverError>> {
    //let levels = get_local_levels(puzzle_ids);
    let mut result = Vec::new();
    let mut remaining_puzzle_ids = Vec::new();
    puzzle_ids.for_each(|puzzle_id| match get_local_level(puzzle_id) {
        Ok(level) => result.push(Ok(level)),
        Err(serr) => match serr {
            SolverError::NoPuzzleForId => result.push(Err(serr)),
            SolverError::IOError(ierr) => {
                if ierr.kind() == ErrorKind::NotFound {
                    remaining_puzzle_ids.push(puzzle_id);
                } else {
                    panic!("{:?}, IOError kind: {}", ierr, ierr.kind())
                }
            }
            _ => panic!("{:?}", serr),
        },
    });
    if remaining_puzzle_ids.len() > 0 {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            // let driver = get_driver().await.unwrap();
            result.append(
                &mut get_external_levels(remaining_puzzle_ids.into_iter())
                    .await
                    .collect(),
            );
            // driver.quit().await
        });
    }
    return result.into_iter();
}
pub fn get_level(puzzle_id: u64) -> Result<Level, SolverError> {
    return get_levels(puzzle_id..=puzzle_id).next().unwrap();
}

async fn get_driver() -> Result<WebDriver, SolverError> {
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
    Ok(driver)
}

async fn perform_login(driver: &WebDriver) -> Result<(), SolverError> {
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

async fn goto_puzzle_url(puzzle_id: u64, driver: &WebDriver) -> String {
    let mut url = "http://www.robozzle.com/beta/index.html?puzzle=".to_string();
    url.push_str(puzzle_id.to_string().as_str());
    driver.goto(url.as_str()).await.unwrap();
    url
}

async fn get_external_levels(
    puzzle_ids: impl Iterator<Item = u64>,
) -> impl Iterator<Item = Result<Level, SolverError>> {
    let driver = get_driver().await.unwrap();
    let mut url = "http://www.robozzle.com/beta/index.html".to_string();
    driver.goto(url.as_str()).await.unwrap();
    // Disable top solver fetching
    let json = driver
        .execute("robozzle.topSolvers = () => {};", vec![])
        .await
        .unwrap();
    let mut result = Vec::new();
    for puzzle_id in puzzle_ids {
        println!("Fetching puzzle");
        // switch to different level
        let boi = get_on_page_level(&driver, puzzle_id, true).await;

        result.push(boi.map_err(SolverError::from));
    }
    driver.quit().await.unwrap();
    return result.into_iter();
}

async fn get_on_page_level(
    driver: &WebDriver,
    puzzle_id: u64,
    store_locally: bool,
) -> Result<Level, serde_json::Error> {
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
    if store_locally {
        store_puzzle_locally(&json.to_string(), puzzle_id);
    }
    println!("level: {}", json.to_string());
    serde_json::from_value::<LevelJson>(json.clone()).map(Level::from)
}

fn get_local_level(puzzle_id: u64) -> Result<Level, SolverError> {
    let mut path = PathBuf::from_str("data/puzzles").expect("unable to create puzzle pathbuf");
    path.push(puzzle_id.to_string());
    read_level_from_path(path)
}

fn get_local_levels(
    puzzle_ids: impl Iterator<Item = u64>,
) -> impl Iterator<Item = Result<Level, SolverError>> {
    puzzle_ids.map(get_local_level)
}

fn read_level_from_path(path: PathBuf) -> Result<Level, SolverError> {
    File::options()
        .read(true)
        .open(path)
        .map_err(SolverError::IOError)
        .and_then(|mut file| {
            let mut string = String::new();
            file.read_to_string(&mut string);
            let level_json: Result<Level, _> = serde_json::from_str::<Option<LevelJson>>(&string)
                .map_err(SolverError::Serde)
                .and_then(|opt| opt.ok_or(SolverError::NoPuzzleForId))
                .map(Level::from);
            return level_json;
        })
}

pub fn get_all_local_levels() -> impl Iterator<Item = Level> {
    // let mut result = Vec::new();
    std::fs::read_dir("data/puzzles")
        .expect("unable to read puzzle directory")
        .filter_map(|dir| {
            let path = dir.expect("unable to read dir").path();
            read_level_from_path(path.clone())
                .map_err(|err| {
                    eprintln!(
                        "Read level from path error: {:?}\n reading pathh: {:?}",
                        err, path
                    )
                })
                .ok()
        })
    // for dir in std::fs::read_dir("data/puzzles").expect("unable to read puzzle directory") {
    //     let path = dir.expect("unable to read dir").path();
    //     let level_json = File::options()
    //         .read(true)
    //         .open(path)
    //         .map(|mut file| {
    //             let mut string = String::new();
    //             file.read_to_string(&mut string);
    //             let level_json: LevelJson =
    //                 serde_json::from_str(&string).expect("couldn't deserialize JSON");
    //             return Level::from(level_json);
    //         })
    //         .expect("should be opening an existing file");
    //     result.push(level_json);
    // }
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
