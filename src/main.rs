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
use clap::{Arg, Command};
use constants::*;
use game::{instructions::*, *};
use solver::{
    pruning::{banned_pair, banned_trio},
    solve,
};
use web::{
    encode_program, get_all_local_levels, puzzle_from_string, solve_puzzles, start_web_solver,
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
                        .arg_required_else_help(true)
                        .arg(Arg::new("puzzle ID").value_parser(0..20000)),
                )
                .subcommand(
                    Command::new("solve")
                        .arg_required_else_help(true)
                        .arg(Arg::new("puzzle ID").value_parser(0..20000)),
                ),
        )
        .subcommand(
            Command::new("puzzles")
                .subcommand_required(true)
                .subcommand(Command::new("list")),
        )
}

fn main() {
    match cli().get_matches().subcommand() {
        Some(("web", matches)) => match matches.subcommand() {
            Some(("solve", matches)) => {
                let puzzle_id = *matches.get_one::<i64>("puzzle ID").expect("required");
                solve_puzzles(puzzle_id as u64).expect("couldn't solve puzzle");
            }
            Some(("fetch", matches)) => {
                let puzzle_id = *matches.get_one::<i64>("puzzle ID").expect("required");
                todo!();
            }
            _ => todo!(),
        },
        Some(("puzzles", matches)) => match matches.subcommand() {
            Some(("list", matches)) => {
                for level in get_all_local_levels() {
                    println!("Id: {}", level.id);
                    println!("Title: {}", level.title);
                    println!("{}", level.puzzle);
                }
                return;

                let face = "{\"About\":\"face\",\"AllowedCommands\":\"0\",\"Colors\":[\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRBRRRBRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRBR\",\"RRRRRGGGGGGGGRBR\",\"RRRRRRRRRRRRRRBR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\"],\"CommentCount\":\"0\",\"DifficultyVoteCount\":\"10\",\"DifficultyVoteSum\":\"15\",\"Disliked\":\"8\",\"Featured\":\"false\",\"Id\":\"5088\",\"Items\":[\"################\",\"#######....#####\",\"####.....*..*###\",\"####.#*..#*...##\",\"###..########..#\",\"###.####.####..#\",\"##.*##########*#\",\"##.###########.#\",\"##.#.........#.#\",\"#..###########.#\",\"################\",\"################\"],\"Liked\":\"1\",\"RobotCol\":\"1\",\"RobotDir\":\"0\",\"RobotRow\":\"9\",\"Solutions\":\"47\",\"SubLengths\":[\"10\",\"10\",\"10\",\"10\",\"10\"],\"SubmittedBy\":\"oshabott59\",\"SubmittedDate\":\"2014-11-27T11:34:11.98\",\"Title\":\"Ionvoc6\"}";
                let ternary = "{\"About\":\"R=2, G=1, B=0. The most significant bit is the last one. Count green turns.\",\"AllowedCommands\":\"7\",\"Colors\":[\"RRRRRBBBBBBBBBBG\",\"RRRRRGRRRRRRRRRB\",\"BBBBBGBBBBBBBGRB\",\"BRRRRGRRRRRRRBRB\",\"RRGGRBRGGRBBRBRB\",\"BRBBRBRBBRRBRBRB\",\"GRGBBBBBGRRBRGRB\",\"BRRBRBRGBBBGRRRB\",\"BRRBRBRRRRRRRGRB\",\"GBBGRGBBBBBBBBRB\",\"RRRRRRRRRRRRRRRB\",\"BRGBGBBBBBBBBBBG\"],\"CommentCount\":\"0\",\"DifficultyVoteCount\":\"1\",\"DifficultyVoteSum\":\"5\",\"Disliked\":\"0\",\"Featured\":\"false\",\"Id\":\"10459\",\"Items\":[\"#####..........*\",\"#####*#########.\",\".....*.......*#.\",\"*####*#######.#.\",\"*#**#*#**#*.#.#.\",\"*#..#.#..##.#*#.\",\"*#*.....*##.#*#.\",\".##.#.#*...*#*#.\",\".##.#.#######*#.\",\"*..*#*........#.\",\"###############.\",\".****..........*\"],\"Liked\":\"0\",\"RobotCol\":\"0\",\"RobotDir\":\"0\",\"RobotRow\":\"11\",\"Solutions\":\"2\",\"SubLengths\":[\"5\",\"10\",\"10\",\"5\",\"0\"],\"SubmittedBy\":\"scorpio\",\"SubmittedDate\":\"2018-03-27T23:19:44.29\",\"Title\":\"Ternary (4 digits)\"}";
                let knot_3 = "{ \"About\": \"I spun the thread on the colors to mark the mid-points before warping it on left\", \"AllowedCommands\": \"0\", \"Colors\": [ \"RRRRRRRBBBBRBBBB\", \"RRBBBRRBRRRRRRRB\", \"RRBRRBRBRRRRRRRB\", \"RRBRRBRBRRRRRRRB\", \"RRBRRBRGRRRRRRRB\", \"RRGBBRBBBBBGRRRB\", \"RRBRBRGBBRRBRRRB\", \"RRBRBRRBRRRBRRRB\", \"RRBRGBBBBRRBRRRB\", \"RRBRRRRBRRRBRRRB\", \"RRRBBBBGBBBBRRRB\", \"RRRRBBBBBBBBBBBR\" ], \"CommentCount\": \"9\", \"DifficultyVoteCount\": \"2\", \"DifficultyVoteSum\": \"6\", \"Disliked\": \"0\", \"Featured\": \"false\", \"Id\": \"4426\", \"Items\": [ \"#######*********\", \"##****#*#######*\", \"##*##*#*#######*\", \"##*##*#*#######*\", \"##*#**#*#######*\", \"##***#******###*\", \"##*#*#***##*###*\", \"##*#*##*###*###*\", \"##*#******#*###*\", \"##**###*###*###*\", \"###*********###*\", \"####.***********\" ], \"Liked\": \"1\", \"RobotCol\": \"4\", \"RobotDir\": \"0\", \"RobotRow\": \"11\", \"Solutions\": \"6\", \"SubLengths\": [ \"5\", \"5\", \"5\", \"5\", \"5\" ], \"SubmittedBy\": \"shaggy12\", \"SubmittedDate\": \"2013-08-26T12:50:36.717\", \"Title\": \"Knot III\" }";
                let scratch = "{ \"About\": \"\", \"AllowedCommands\": \"0\", \"Colors\": [ \"GBBRBBRRBRBBRBRG\", \"GRRRBRRRBRBBRRBG\", \"GBBRBRRBRBBBRBRG\", \"GRRRBRRRBBRRBBRG\", \"GBRBBRRBRBRBRBRG\", \"GRRRBRRRBRBBRBRG\", \"GBBBRGRRRRRRRRRR\", \"RRRRRGRGRRRRRRRR\", \"RRRGRGRGRRRRRRRR\", \"RRGGGGGGGGRRRRRR\", \"RRRRGRGRGRRRRRRR\", \"RRRRGRRRGRRRRRRR\" ], \"CommentCount\": \"12\", \"DifficultyVoteCount\": \"9\", \"DifficultyVoteSum\": \"27\", \"Disliked\": \"1\", \"Featured\": \"false\", \"Id\": \"3558\", \"Items\": [ \"...............*\", \"*..............*\", \"*..............*\", \"*..............*\", \"*..............*\", \"*..............*\", \"*....*##########\", \"#####*#*########\", \"###*#*#*########\", \"##********######\", \"####*#*#*#######\", \"####*###*#######\" ], \"Liked\": \"10\", \"RobotCol\": \"1\", \"RobotDir\": \"0\", \"RobotRow\": \"0\", \"Solutions\": \"30\", \"SubLengths\": [ \"6\", \"4\", \"3\", \"0\", \"0\" ], \"SubmittedBy\": \"denisb\", \"SubmittedDate\": \"2012-03-29T02:57:04.293\", \"Title\": \"Scratch\" }";
                let odds_and_evens = "{ \"About\": \"Useful for #12522\", \"AllowedCommands\": \"0\", \"Colors\": [ \"BRRRRRRRBBBBBBBG\", \"BRRRRRRRBRRRRRRB\", \"BRRRRRBBGRBBBGRB\", \"BRRRRRBRRRGBRBRB\", \"BRRRBBGRRRRRRBRB\", \"BRRRBRRRRGBBBBRB\", \"BRRRBRRRRBRRRRRB\", \"BRRRBRRRRBRGBBBB\", \"BRRRBRRRRBRBRRRR\", \"BRRRGBBBBBRBRRRR\", \"BRRRRRRRRRRBRRRR\", \"GBBBBBBBBBBBRRRR\" ], \"CommentCount\": \"8\", \"DifficultyVoteCount\": \"7\", \"DifficultyVoteSum\": \"21\", \"Disliked\": \"0\", \"Featured\": \"false\", \"Id\": \"12574\", \"Items\": [ \".#######*.......\", \".#######.######.\", \".#####*..#*...#.\", \".#####.###.*#.#.\", \".###*..######.#.\", \".###.####....*#.\", \".###.####.#####.\", \".###.####.#....*\", \".###.####.#.####\", \".###.....*#.####\", \".##########.####\", \"...........*####\" ], \"Liked\": \"10\", \"RobotCol\": \"0\", \"RobotDir\": \"1\", \"RobotRow\": \"0\", \"Solutions\": \"20\", \"SubLengths\": [ \"5\", \"5\", \"5\", \"0\", \"0\" ], \"SubmittedBy\": \"scorpio\", \"SubmittedDate\": \"2019-10-08T14:48:03.153\", \"Title\": \"Odds and Evens (Lite edition)\" }";
                let playing_with_stacks_4 = "{ \"About\": \"Dr Mazhar's advice\\\" Two stacks;First stack pushs and pops first stack;\\\"\", \"AllowedCommands\": \"0\", \"Colors\": [ \"RRRRRRRRRRRRRRRR\", \"BBBRBBBRBBBRBBBR\", \"BRRRRRBRBRRRRRBR\", \"BRBRBRBRBRBRBRBR\", \"BRBRBRBRBRBRBRBR\", \"BRBRRRRRRRRRBRBR\", \"RRRRBRBRBRBRRRRR\", \"BRBRBRBRBRRRBRBR\", \"BRBRRRBRBRRRBRBR\", \"BRBBRBBRBBRBBRBR\", \"BRRRRRRRRRRRRRBR\", \"BBBBBBBRBBBBBBBR\" ], \"CommentCount\": \"0\", \"DifficultyVoteCount\": \"7\", \"DifficultyVoteSum\": \"16\", \"Disliked\": \"0\", \"Featured\": \"false\", \"Id\": \"12629\", \"Items\": [ \"################\", \".......#.......#\", \".#####.#.#####.#\", \".#...#.#.#...#.#\", \".#.#.#.#.#.#.#.#\", \".#.#.#.#.#.#.#.#\", \".#.#.#.#.#.#.#.#\", \".#.#*#.#.#*#.#.#\", \".#.###.#.###.#.#\", \".#.....#.....#.#\", \".#############.#\", \"...............#\" ], \"Liked\": \"8\", \"RobotCol\": \"0\", \"RobotDir\": \"0\", \"RobotRow\": \"11\", \"Solutions\": \"12\", \"SubLengths\": [ \"8\", \"3\", \"0\", \"0\", \"0\" ], \"SubmittedBy\": \"drmazhar\", \"SubmittedDate\": \"2019-11-15T03:53:45.16\", \"Title\": \"Playing With stacks (Version 4)\" }";
                let center_cut = "{ \"About\": \"Everyone, join the fun! Make a puzzle!\", \"AllowedCommands\": \"0\", \"Colors\": [ \"RRRRRRRRRRRRRRRR\", \"BBRBRRRBRRBBRRRR\", \"BBRBRRRBRRBBRRRR\", \"RRBRRBRRBRRRBBBR\", \"BBRBRRRBRRBBRRRR\", \"BBBBRBRBBRBBBBBR\", \"BBGBBBGBGBBBBBGG\", \"BBRGBBRGRBGBBGRR\", \"BGRBGGRBRGBGGBRR\", \"GBRRBBRRRBRBBRRR\", \"BBRRBBRRRBRBBRRR\", \"BRRRRRRRRRRRRRRR\" ], \"CommentCount\": \"1\", \"DifficultyVoteCount\": \"7\", \"DifficultyVoteSum\": \"20\", \"Disliked\": \"0\", \"Featured\": \"false\", \"Id\": \"12684\", \"Items\": [ \"################\", \"**#*###*##**####\", \"**#*###*##**####\", \"****#*#**#*****#\", \"****#*#**#*****#\", \"***************.\", \"****************\", \"**#***#*#*****##\", \"**#***#*#*****##\", \"**##**###*#**###\", \"**##**###*#**###\", \"*###############\" ], \"Liked\": \"6\", \"RobotCol\": \"15\", \"RobotDir\": \"3\", \"RobotRow\": \"5\", \"Solutions\": \"9\", \"SubLengths\": [ \"6\", \"5\", \"3\", \"0\", \"0\" ], \"SubmittedBy\": \"jnpollack\", \"SubmittedDate\": \"2020-01-17T23:44:48.777\", \"Title\": \"Center Cut\" }";
                let writers_block = "{\"About\":\"\",\"AllowedCommands\":\"0\",\"Colors\":[\"RRRRRRRRRRRRRRRR\",\"RRRRGRBRRRRRRRRR\",\"RRRRBRBRBRGRRRRR\",\"RRBRBRBRBRBRRRRR\",\"BRBRBRBRBRBRBRRR\",\"BBBBBBBBBBBBBBBB\",\"BRBRRRBRBRRRBRBR\",\"BRBRRRBRGRRRBRBR\",\"BRGRRRBRRRRRBRBR\",\"GRRRRRBRRRRRGRBR\",\"RRRRRRBRRRRRRRGR\",\"RRRRRRGRRRRRRRRR\"],\"CommentCount\":\"0\",\"DifficultyVoteCount\":\"6\",\"DifficultyVoteSum\":\"19\",\"Disliked\":\"0\",\"Featured\":\"false\",\"Id\":\"14874\",\"Items\":[\"######*#########\",\"####*#.#*#######\",\"##*#.#.#.#*#####\",\"*#.#.#.#.#.#*###\",\".#.#.#.#.#.#.#*#\",\"...............*\",\".#.###.#.###.#.#\",\".#.###.#*###.#.#\",\".#*###.#####.#.#\",\"*#####.#####*#.#\",\"######.#######*#\",\"######*#########\"],\"Liked\":\"6\",\"RobotCol\":\"0\",\"RobotDir\":\"0\",\"RobotRow\":\"5\",\"Solutions\":\"12\",\"SubLengths\":[\"7\",\"7\",\"0\",\"0\",\"0\"],\"SubmittedBy\":\"axorion\",\"SubmittedDate\":\"2022-04-23T15:26:59.963\",\"Title\":\"Writer’s Block\"}";
                println!("{}", face);
                println!("{}", ternary);
                println!("{}", knot_3);
                println!("{}", scratch);
                println!("{}", odds_and_evens);
                println!("{}", playing_with_stacks_4);
                println!("{}", center_cut);
                println!("{}", writers_block);
            }
            _ => todo!(),
        },
        _ => {
            println!("no CLI match")
        }
    }
}

fn old_main() {
    //    println!("sizes: {}", mem::size_of::<State>());
    //    println!("sizes: {}", mem::size_of::<Frame>());
    //    println!("sizes: {}", mem::size_of::<Source>());
    let face = "{\"About\":\"face\",\"AllowedCommands\":\"0\",\"Colors\":[\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRBRRRBRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRBR\",\"RRRRRGGGGGGGGRBR\",\"RRRRRRRRRRRRRRBR\",\"RRRRRRRRRRRRRRRR\",\"RRRRRRRRRRRRRRRR\"],\"CommentCount\":\"0\",\"DifficultyVoteCount\":\"10\",\"DifficultyVoteSum\":\"15\",\"Disliked\":\"8\",\"Featured\":\"false\",\"Id\":\"5088\",\"Items\":[\"################\",\"#######....#####\",\"####.....*..*###\",\"####.#*..#*...##\",\"###..########..#\",\"###.####.####..#\",\"##.*##########*#\",\"##.###########.#\",\"##.#.........#.#\",\"#..###########.#\",\"################\",\"################\"],\"Liked\":\"1\",\"RobotCol\":\"1\",\"RobotDir\":\"0\",\"RobotRow\":\"9\",\"Solutions\":\"47\",\"SubLengths\":[\"10\",\"10\",\"10\",\"10\",\"10\"],\"SubmittedBy\":\"oshabott59\",\"SubmittedDate\":\"2014-11-27T11:34:11.98\",\"Title\":\"Ionvoc6\"}";
    let ternary = "{\"About\":\"R=2, G=1, B=0. The most significant bit is the last one. Count green turns.\",\"AllowedCommands\":\"7\",\"Colors\":[\"RRRRRBBBBBBBBBBG\",\"RRRRRGRRRRRRRRRB\",\"BBBBBGBBBBBBBGRB\",\"BRRRRGRRRRRRRBRB\",\"RRGGRBRGGRBBRBRB\",\"BRBBRBRBBRRBRBRB\",\"GRGBBBBBGRRBRGRB\",\"BRRBRBRGBBBGRRRB\",\"BRRBRBRRRRRRRGRB\",\"GBBGRGBBBBBBBBRB\",\"RRRRRRRRRRRRRRRB\",\"BRGBGBBBBBBBBBBG\"],\"CommentCount\":\"0\",\"DifficultyVoteCount\":\"1\",\"DifficultyVoteSum\":\"5\",\"Disliked\":\"0\",\"Featured\":\"false\",\"Id\":\"10459\",\"Items\":[\"#####..........*\",\"#####*#########.\",\".....*.......*#.\",\"*####*#######.#.\",\"*#**#*#**#*.#.#.\",\"*#..#.#..##.#*#.\",\"*#*.....*##.#*#.\",\".##.#.#*...*#*#.\",\".##.#.#######*#.\",\"*..*#*........#.\",\"###############.\",\".****..........*\"],\"Liked\":\"0\",\"RobotCol\":\"0\",\"RobotDir\":\"0\",\"RobotRow\":\"11\",\"Solutions\":\"2\",\"SubLengths\":[\"5\",\"10\",\"10\",\"5\",\"0\"],\"SubmittedBy\":\"scorpio\",\"SubmittedDate\":\"2018-03-27T23:19:44.29\",\"Title\":\"Ternary (4 digits)\"}";
    let knot_3 = "{ \"About\": \"I spun the thread on the colors to mark the mid-points before warping it on left\", \"AllowedCommands\": \"0\", \"Colors\": [ \"RRRRRRRBBBBRBBBB\", \"RRBBBRRBRRRRRRRB\", \"RRBRRBRBRRRRRRRB\", \"RRBRRBRBRRRRRRRB\", \"RRBRRBRGRRRRRRRB\", \"RRGBBRBBBBBGRRRB\", \"RRBRBRGBBRRBRRRB\", \"RRBRBRRBRRRBRRRB\", \"RRBRGBBBBRRBRRRB\", \"RRBRRRRBRRRBRRRB\", \"RRRBBBBGBBBBRRRB\", \"RRRRBBBBBBBBBBBR\" ], \"CommentCount\": \"9\", \"DifficultyVoteCount\": \"2\", \"DifficultyVoteSum\": \"6\", \"Disliked\": \"0\", \"Featured\": \"false\", \"Id\": \"4426\", \"Items\": [ \"#######*********\", \"##****#*#######*\", \"##*##*#*#######*\", \"##*##*#*#######*\", \"##*#**#*#######*\", \"##***#******###*\", \"##*#*#***##*###*\", \"##*#*##*###*###*\", \"##*#******#*###*\", \"##**###*###*###*\", \"###*********###*\", \"####.***********\" ], \"Liked\": \"1\", \"RobotCol\": \"4\", \"RobotDir\": \"0\", \"RobotRow\": \"11\", \"Solutions\": \"6\", \"SubLengths\": [ \"5\", \"5\", \"5\", \"5\", \"5\" ], \"SubmittedBy\": \"shaggy12\", \"SubmittedDate\": \"2013-08-26T12:50:36.717\", \"Title\": \"Knot III\" }";
    let scratch = "{ \"About\": \"\", \"AllowedCommands\": \"0\", \"Colors\": [ \"GBBRBBRRBRBBRBRG\", \"GRRRBRRRBRBBRRBG\", \"GBBRBRRBRBBBRBRG\", \"GRRRBRRRBBRRBBRG\", \"GBRBBRRBRBRBRBRG\", \"GRRRBRRRBRBBRBRG\", \"GBBBRGRRRRRRRRRR\", \"RRRRRGRGRRRRRRRR\", \"RRRGRGRGRRRRRRRR\", \"RRGGGGGGGGRRRRRR\", \"RRRRGRGRGRRRRRRR\", \"RRRRGRRRGRRRRRRR\" ], \"CommentCount\": \"12\", \"DifficultyVoteCount\": \"9\", \"DifficultyVoteSum\": \"27\", \"Disliked\": \"1\", \"Featured\": \"false\", \"Id\": \"3558\", \"Items\": [ \"...............*\", \"*..............*\", \"*..............*\", \"*..............*\", \"*..............*\", \"*..............*\", \"*....*##########\", \"#####*#*########\", \"###*#*#*########\", \"##********######\", \"####*#*#*#######\", \"####*###*#######\" ], \"Liked\": \"10\", \"RobotCol\": \"1\", \"RobotDir\": \"0\", \"RobotRow\": \"0\", \"Solutions\": \"30\", \"SubLengths\": [ \"6\", \"4\", \"3\", \"0\", \"0\" ], \"SubmittedBy\": \"denisb\", \"SubmittedDate\": \"2012-03-29T02:57:04.293\", \"Title\": \"Scratch\" }";
    let odds_and_evens = "{ \"About\": \"Useful for #12522\", \"AllowedCommands\": \"0\", \"Colors\": [ \"BRRRRRRRBBBBBBBG\", \"BRRRRRRRBRRRRRRB\", \"BRRRRRBBGRBBBGRB\", \"BRRRRRBRRRGBRBRB\", \"BRRRBBGRRRRRRBRB\", \"BRRRBRRRRGBBBBRB\", \"BRRRBRRRRBRRRRRB\", \"BRRRBRRRRBRGBBBB\", \"BRRRBRRRRBRBRRRR\", \"BRRRGBBBBBRBRRRR\", \"BRRRRRRRRRRBRRRR\", \"GBBBBBBBBBBBRRRR\" ], \"CommentCount\": \"8\", \"DifficultyVoteCount\": \"7\", \"DifficultyVoteSum\": \"21\", \"Disliked\": \"0\", \"Featured\": \"false\", \"Id\": \"12574\", \"Items\": [ \".#######*.......\", \".#######.######.\", \".#####*..#*...#.\", \".#####.###.*#.#.\", \".###*..######.#.\", \".###.####....*#.\", \".###.####.#####.\", \".###.####.#....*\", \".###.####.#.####\", \".###.....*#.####\", \".##########.####\", \"...........*####\" ], \"Liked\": \"10\", \"RobotCol\": \"0\", \"RobotDir\": \"1\", \"RobotRow\": \"0\", \"Solutions\": \"20\", \"SubLengths\": [ \"5\", \"5\", \"5\", \"0\", \"0\" ], \"SubmittedBy\": \"scorpio\", \"SubmittedDate\": \"2019-10-08T14:48:03.153\", \"Title\": \"Odds and Evens (Lite edition)\" }";
    let playing_with_stacks_4 = "{ \"About\": \"Dr Mazhar's advice\\\" Two stacks;First stack pushs and pops first stack;\\\"\", \"AllowedCommands\": \"0\", \"Colors\": [ \"RRRRRRRRRRRRRRRR\", \"BBBRBBBRBBBRBBBR\", \"BRRRRRBRBRRRRRBR\", \"BRBRBRBRBRBRBRBR\", \"BRBRBRBRBRBRBRBR\", \"BRBRRRRRRRRRBRBR\", \"RRRRBRBRBRBRRRRR\", \"BRBRBRBRBRRRBRBR\", \"BRBRRRBRBRRRBRBR\", \"BRBBRBBRBBRBBRBR\", \"BRRRRRRRRRRRRRBR\", \"BBBBBBBRBBBBBBBR\" ], \"CommentCount\": \"0\", \"DifficultyVoteCount\": \"7\", \"DifficultyVoteSum\": \"16\", \"Disliked\": \"0\", \"Featured\": \"false\", \"Id\": \"12629\", \"Items\": [ \"################\", \".......#.......#\", \".#####.#.#####.#\", \".#...#.#.#...#.#\", \".#.#.#.#.#.#.#.#\", \".#.#.#.#.#.#.#.#\", \".#.#.#.#.#.#.#.#\", \".#.#*#.#.#*#.#.#\", \".#.###.#.###.#.#\", \".#.....#.....#.#\", \".#############.#\", \"...............#\" ], \"Liked\": \"8\", \"RobotCol\": \"0\", \"RobotDir\": \"0\", \"RobotRow\": \"11\", \"Solutions\": \"12\", \"SubLengths\": [ \"8\", \"3\", \"0\", \"0\", \"0\" ], \"SubmittedBy\": \"drmazhar\", \"SubmittedDate\": \"2019-11-15T03:53:45.16\", \"Title\": \"Playing With stacks (Version 4)\" }";
    let center_cut = "{ \"About\": \"Everyone, join the fun! Make a puzzle!\", \"AllowedCommands\": \"0\", \"Colors\": [ \"RRRRRRRRRRRRRRRR\", \"BBRBRRRBRRBBRRRR\", \"BBRBRRRBRRBBRRRR\", \"RRBRRBRRBRRRBBBR\", \"BBRBRRRBRRBBRRRR\", \"BBBBRBRBBRBBBBBR\", \"BBGBBBGBGBBBBBGG\", \"BBRGBBRGRBGBBGRR\", \"BGRBGGRBRGBGGBRR\", \"GBRRBBRRRBRBBRRR\", \"BBRRBBRRRBRBBRRR\", \"BRRRRRRRRRRRRRRR\" ], \"CommentCount\": \"1\", \"DifficultyVoteCount\": \"7\", \"DifficultyVoteSum\": \"20\", \"Disliked\": \"0\", \"Featured\": \"false\", \"Id\": \"12684\", \"Items\": [ \"################\", \"**#*###*##**####\", \"**#*###*##**####\", \"****#*#**#*****#\", \"****#*#**#*****#\", \"***************.\", \"****************\", \"**#***#*#*****##\", \"**#***#*#*****##\", \"**##**###*#**###\", \"**##**###*#**###\", \"*###############\" ], \"Liked\": \"6\", \"RobotCol\": \"15\", \"RobotDir\": \"3\", \"RobotRow\": \"5\", \"Solutions\": \"9\", \"SubLengths\": [ \"6\", \"5\", \"3\", \"0\", \"0\" ], \"SubmittedBy\": \"jnpollack\", \"SubmittedDate\": \"2020-01-17T23:44:48.777\", \"Title\": \"Center Cut\" }";
    let writers_block = "{\"About\":\"\",\"AllowedCommands\":\"0\",\"Colors\":[\"RRRRRRRRRRRRRRRR\",\"RRRRGRBRRRRRRRRR\",\"RRRRBRBRBRGRRRRR\",\"RRBRBRBRBRBRRRRR\",\"BRBRBRBRBRBRBRRR\",\"BBBBBBBBBBBBBBBB\",\"BRBRRRBRBRRRBRBR\",\"BRBRRRBRGRRRBRBR\",\"BRGRRRBRRRRRBRBR\",\"GRRRRRBRRRRRGRBR\",\"RRRRRRBRRRRRRRGR\",\"RRRRRRGRRRRRRRRR\"],\"CommentCount\":\"0\",\"DifficultyVoteCount\":\"6\",\"DifficultyVoteSum\":\"19\",\"Disliked\":\"0\",\"Featured\":\"false\",\"Id\":\"14874\",\"Items\":[\"######*#########\",\"####*#.#*#######\",\"##*#.#.#.#*#####\",\"*#.#.#.#.#.#*###\",\".#.#.#.#.#.#.#*#\",\"...............*\",\".#.###.#.###.#.#\",\".#.###.#*###.#.#\",\".#*###.#####.#.#\",\"*#####.#####*#.#\",\"######.#######*#\",\"######*#########\"],\"Liked\":\"6\",\"RobotCol\":\"0\",\"RobotDir\":\"0\",\"RobotRow\":\"5\",\"Solutions\":\"12\",\"SubLengths\":[\"7\",\"7\",\"0\",\"0\",\"0\"],\"SubmittedBy\":\"axorion\",\"SubmittedDate\":\"2022-04-23T15:26:59.963\",\"Title\":\"Writer’s Block\"}";

    //    println!("puzl: {}", puzzle);
    let puzzles = [
        //PUZZLE_42,
        //PUZZLE_536,
        //PUZZLE_656,
        //PUZZLE_1337,
        //puzzle_from_string(scratch),
        puzzle_from_string(writers_block),
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
