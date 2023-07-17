#![feature(test, core_intrinsics)]
#![allow(dead_code, ellipsis_inclusive_range_patterns)]
#![allow(unused)]
//#![warn(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_must_use)]
#![allow(unreachable_code)]

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::solver::carlo::{score, score_cmp};
use clap::{value_parser, Arg, ArgAction, Command};
use constants::*;
use game::{instructions::*, *};
use solver::backtrack::{self, backtrack};
use solver::{
    pruning::{banned_pair, banned_trio},
    solve,
};
use web::{
    encode_program, get_all_local_levels, get_level, get_levels, puzzle_from_string, solve_puzzle,
};

mod constants;
mod game;
mod solver;
mod web;

#[cfg(test)]
mod tests;

fn cli() -> Command {
    Command::new("solver")
        .bin_name("solver")
        .subcommand_required(true)
        .subcommand(
            Command::new("web")
                .subcommand_required(true)
                .subcommand(
                    Command::new("fetch")
                        .subcommand_negates_reqs(true)
                        .subcommand(
                            Command::new("range").arg(
                                Arg::new("puzzle ID")
                                    .required(true)
                                    .num_args(2)
                                    .value_parser(0..30000),
                            ),
                        )
                        .arg(
                            Arg::new("puzzle ID")
                                .required(true)
                                .num_args(1..=10)
                                .value_parser(0..30000),
                        )
                        .arg(
                            Arg::new("long")
                                .long("long")
                                .short('l')
                                .action(ArgAction::SetTrue),
                        ),
                )
                .subcommand(
                    Command::new("solve")
                        .arg_required_else_help(true)
                        .arg(Arg::new("puzzle ID").value_parser(0..30000)),
                ),
        )
        .subcommand(
            Command::new("puzzles")
                .subcommand_required(true)
                .subcommand(
                    Command::new("list").arg(
                        Arg::new("long")
                            .long("long")
                            .short('l')
                            .action(ArgAction::SetTrue),
                    ),
                ),
        )
        .subcommand(
            Command::new("backtrack")
                .subcommand_negates_reqs(true)
                .arg(
                    Arg::new("timed")
                        .long("timed")
                        .short('t')
                        .global(true)
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .global(true)
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("timeout")
                        .long("timeout")
                        .short('o')
                        .global(true)
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(u128)),
                )
                .subcommand(
                    Command::new("range").arg(
                        Arg::new("puzzle ID")
                            .required(true)
                            .num_args(2)
                            .value_parser(0..30000),
                    ),
                )
                .arg(
                    Arg::new("puzzle ID")
                        .required(true)
                        .num_args(1..=10)
                        .value_parser(0..30000),
                ),
        )
        .subcommand(
            Command::new("misc")
                .subcommand_negates_reqs(true)
                .subcommand(
                    Command::new("range").arg(
                        Arg::new("puzzle ID")
                            .required(true)
                            .num_args(2)
                            .value_parser(0..30000),
                    ),
                )
                .arg(
                    Arg::new("puzzle ID")
                        .required(true)
                        .num_args(1..=10)
                        .value_parser(0..30000),
                ),
        )
}

fn main() {
    match cli().get_matches().subcommand() {
        Some(("web", matches)) => match matches.subcommand() {
            Some(("solve", matches)) => {
                let puzzle_id = *matches.get_one::<i64>("puzzle ID").expect("required");
                solve_puzzle(puzzle_id as u64, false).expect("couldn't solve puzzle");
            }
            Some(("fetch", matches)) => {
                let puzzle_ids: Vec<i64> = matches
                    .get_many("puzzle ID")
                    .expect("required")
                    .copied()
                    .collect();
                if puzzle_ids.len() == 1 {
                    let level =
                        get_level(puzzle_ids[0] as u64).expect("unable to fetch puzzle data");
                    print_level(&level, matches.get_flag("long"));
                } else if puzzle_ids.len() > 1 {
                    let level_res: Vec<_> = if matches.subcommand_matches("range").is_some() {
                        get_levels((puzzle_ids[0] as u64)..=(puzzle_ids[1] as u64)).collect()
                    } else {
                        get_levels(puzzle_ids.into_iter().map(|n| n as u64)).collect()
                    };
                    let only_one = level_res.len() == 1;
                    level_res.into_iter().for_each(|level| match level {
                        Ok(level) => print_level(&level, matches.get_flag("long")),
                        Err(err) => {
                            if only_one {
                                eprintln!("level error: {:?}", err)
                            }
                        }
                    });
                } else {
                    panic!(
                        "incorrect arguments puzzle range {:?}, can't be > 10 :(",
                        puzzle_ids
                    );
                }
            }
            _ => todo!(),
        },
        Some(("puzzles", matches)) => match matches.subcommand() {
            Some(("list", matches)) => {
                for level in get_all_local_levels() {
                    print_level(&level, matches.get_flag("long"));
                }
                // face = 5088
                // ternary = 10459
                // knot_3 = 4426
                // scratch = 3558
                // odds_and_evens = 12574
                // playing_with_stacks_4 = 12629
                // center_cut = 12684
                // writers_block = 14874
            }
            _ => todo!(),
        },
        Some(("backtrack", matches)) => {
            let (matches, ranged) = if let Some(("range", matches)) = matches.subcommand() {
                (matches, true)
            } else {
                (matches, false)
            };
            let puzzle_ids: Vec<i64> = matches
                .get_many("puzzle ID")
                .expect("required")
                .copied()
                .collect();
            let timed = matches.get_flag("timed");
            let quiet = matches.get_flag("quiet");
            let timeout = matches.get_one::<u128>("timeout").map(|e| *e);
            if puzzle_ids.len() > 0 {
                let boi: Vec<_> = if ranged {
                    get_levels((puzzle_ids[0] as u64)..=(puzzle_ids[1] as u64)).collect()
                } else {
                    get_levels(puzzle_ids.into_iter().map(|n| n as u64)).collect()
                };
                let mut ids = Vec::new();
                boi.into_iter().for_each(|level| match level {
                    Ok(level) => {
                        let now = Instant::now();
                        print_level(&level, !quiet);
                        let results = backtrack(level.puzzle, timeout);
                        let el = now.elapsed();
                        if !results.is_empty() {
                            ids.push(level.id);
                        }
                        if timed && !results.is_empty() {
                            println!(
                                "Duration: {:02}:{:02}:{:02}.{:03}",
                                el.as_secs() / (3600),
                                (el.as_secs() / (60)) % 60,
                                (el.as_secs()) % 60,
                                el.subsec_millis()
                            );
                        }
                        if !quiet && !results.is_empty() {
                            println!("Solutions:");
                            for result in results {
                                println!(
                                    "steps: {:>2}, solution length: {:>2}, code: {}",
                                    result.0,
                                    result.1.count_ins(),
                                    result.1
                                );
                            }
                        }
                    }
                    Err(err) => eprintln!("level error: {:?}", err),
                });
                //dbg!(ids);
            } else {
                panic!(
                    "incorrect arguments puzzle range {:?}, can't be > 10 :(",
                    puzzle_ids
                );
            }
        }
        _ => {
            println!("no CLI match")
        }
    }
}

fn print_level(level: &web::Level, long_output: bool) {
    println!(
        "Id: {:<5} | Title: {} | About: {}",
        level.id, level.title, level.about
    );
    if long_output {
        println!("{}", level.puzzle);
    }
}

fn old_main() {
    //    println!("puzl: {}", puzzle);
    let puzzles = [
        //PUZZLE_42,
        //PUZZLE_536,
        //PUZZLE_656,
        PUZZLE_1337,
        //puzzle_from_string(scratch),
        // puzzle_from_string(writers_block),
        //        parse_level(),
    ];
    for puzzle in puzzles.iter() {
        let now = Instant::now();
        let solutions = solve(*puzzle);
        if !solutions.is_empty() {
            println!("Solved! The solution(s) are:");
            for solution in solutions {
                println!(
                    "{} steps: {}, code: {}",
                    solution.1,
                    solution.0,
                    encode_program(&solution.1, puzzle)
                );
            }
        } else {
            println!("I couldn't find a solution :(");
        }
        println!("The solver took {} seconds.\n", now.elapsed().as_secs_f64());
    }
    //    println!("prog: {}", encode_program(&PUZZLE_656_SOLUTION, &PUZZLE_656));
}
