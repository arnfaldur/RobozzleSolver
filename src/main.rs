#![feature(test, core_intrinsics, cfg_target_has_atomic)]
#![allow(dead_code, ellipsis_inclusive_range_patterns)]
#![allow(unused)]
//#![warn(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_must_use)]
#![allow(unreachable_code)]

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::solver::carlo::{score, score_cmp};
use constants::*;
use game::{instructions::*, *};
use solver::{
    pruning::{banned_pair, banned_trio},
    solve,
};
use web::{encode_program, puzzle_from_string};

mod constants;
mod game;
mod tests;

mod solver;

mod web;

fn main() {
    //    start_web_solver(); return;
    //    denial_test();return;
    //    println!("sizes: {}", mem::size_of::<State>());
    //    println!("sizes: {}", mem::size_of::<Frame>());
    //    println!("sizes: {}", mem::size_of::<Source>());
    //    return;
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

const RAND_FUNCS: [Method; 9] = [
    [F1, HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT],
    [
        FORWARD, GREEN_F1, HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT,
    ],
    [LEFT, RED_F1, HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT],
    [RIGHT, F1, HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT],
    [
        FORWARD, FORWARD, FORWARD, RIGHT, FORWARD, MARK_GRAY, FORWARD, MARK_RED, FORWARD, FORWARD,
    ],
    [
        MARK_GREEN, FORWARD, FORWARD, RIGHT, FORWARD, MARK_GRAY, FORWARD, MARK_RED, FORWARD,
        FORWARD,
    ],
    [
        RED_RIGHT,
        GREEN_LEFT,
        FORWARD,
        GREEN_RIGHT,
        GREEN_LEFT,
        FORWARD,
        RED_RIGHT,
        GREEN_LEFT,
        FORWARD,
        HALT,
    ],
    [
        RED_RIGHT,
        GREEN_LEFT,
        FORWARD,
        GREEN_RIGHT,
        GREEN_LEFT,
        FORWARD,
        RED_RIGHT,
        GREEN_LEFT,
        FORWARD,
        MARK_GREEN,
    ],
    [HALT; 10],
];

fn denial_test() {
    let template_puzzle = genboi(RE, GE, BS);
    let tmp = template_puzzle.get_ins_set(INS_COLOR_MASK, true);
    let instructions = [HALT].iter().chain(
        tmp.iter()
            //        .filter(|&ins| !ins.is_function() || ins.get_instruction() == F2));
            .filter(|&ins| !ins.is_function()),
    );
    let tiles = [RS, GS, BS];
    println!("jsdfk: {}", template_puzzle);
    let mut mipmap = HashMap::new();
    let mut rejects2 = HashSet::new();
    let mut rejects3 = HashSet::new();
    let mut counter = 0;
    let mut denies = 0;
    let mut nonies = 0;
    for &a in instructions.clone() {
        for &b in instructions.clone() {
            if a == HALT && b != HALT {
                break;
            }
            for &c in instructions.clone() {
                if b == HALT && c != HALT {
                    break;
                }
                //                if a.is_mark() || b.is_mark() || c.is_mark() { continue; }
                counter += 1;
                let mut states = vec![];
                let function = [a, b, c, HALT, HALT, HALT, HALT, HALT, HALT, HALT];
                if banned_pair(&template_puzzle, a, b, false)
                    || banned_pair(&template_puzzle, b, c, false)
                    || banned_trio(&template_puzzle, a, b, c, false)
                {
                    denies += 1;
                    continue;
                }
                for &routine in RAND_FUNCS.iter() {
                    let prog = Source([function, routine, [HALT; 10], [HALT; 10], [HALT; 10]]);
                    for &ta in tiles.iter() {
                        for &tb in tiles.iter() {
                            for &tc in tiles.iter() {
                                let puz = genboi(ta, tb, tc);
                                puz.execute(&prog, false, |state, _| states.push(state.to_owned()));
                            }
                        }
                    }
                }
                let prog = Source([function, [HALT; 10], [HALT; 10], [HALT; 10], [HALT; 10]]);
                if mipmap.contains_key(&states) {
                    let champ: Source = mipmap[&states];
                    let mut champins = champ.0[0].len();
                    for j in 0..champ.0[0].len() {
                        champins -= (champ.0[0][j] == HALT) as usize;
                    }
                    let mut progins = prog.0[0].len();
                    for j in 0..prog.0[0].len() {
                        progins -= (prog.0[0][j] == HALT) as usize;
                    }

                    if progins < champins {
                        mipmap.insert(states, prog);
                        //                        println!("replacing the champ {} with {}", champ, prog);
                        if champins == 2 {
                            let rej = [champ.0[0][0], champ.0[0][1]];
                            rejects2.insert(rej);
                        } else if champins == 3 {
                            let rej = [champ.0[0][0], champ.0[0][1], champ.0[0][2]];
                            rejects3.insert(rej);
                        } else {
                            panic!("program with {}! instructions!", progins);
                        }
                    } else {
                        if progins == 2 {
                            let rej = [prog.0[0][0], prog.0[0][1]];
                            rejects2.insert(rej);
                        } else if progins == 3 {
                            let rej = [prog.0[0][0], prog.0[0][1], prog.0[0][2]];
                            rejects3.insert(rej);
                        } else {
                            //                            panic!("program with {}! instructions!", progins);
                        }
                        println!(
                            "the program {} \tis already represented by \t{}",
                            prog, champ
                        );
                        let mut thing = State::default();
                        let mut bing = State::default();
                        //                        template_puzzle.execute(&prog, true, |state, _| thing = state.to_owned());
                        //                        println!("finally: {}", thing);
                        //                        template_puzzle.execute(&champ, true, |state, _| bing = state.to_owned());
                        //                        println!("binally: {}", bing);
                        //                        println!("ARE THEY THE SAME? {}", thing == bing);
                    }
                } else {
                    mipmap.insert(states, prog);
                    //                    println!("the program {} is the first of its kind", prog);
                }
                //                println!("{},", states);
            }
        }
    }
    println!(
        "counter: {}, denies: {}, nonies: {}",
        counter, denies, nonies
    );
    //    let mut rejtmp: Vec<[Ins; 2]> = rejects2.iter().cloned().collect();
    //    rejtmp.sort();
    //    for e in rejtmp {
    //        print!("[");
    //        for i in e.iter() {
    //            print!("{:?}, ", i);
    //        }
    //        println!("],");
    //    }
    //    let mut rejtmp: Vec<[Ins; 3]> = rejects3.iter().cloned().collect();
    //    rejtmp.sort();
    //    for e in rejtmp {
    //        print!("[");
    //        for i in e.iter() {
    //            print!("{:?}, ", i);
    //        }
    //        println!("],");
    //    }
    //    println!("rej 2: {:?}", rejects2);
    //    println!("rej 3: {:?}", rejects3);
    denies = 0;
    nonies = 0;
    for &a in instructions.clone() {
        for &b in instructions.clone() {
            if a == HALT && b != HALT {
                break;
            }
            let c = HALT;
            for &c in instructions.clone() {
                if b == HALT && c != HALT {
                    break;
                }
                let mut states = vec![];
                let function = [a, b, c, HALT, HALT, HALT, HALT, HALT, HALT, HALT];
                for &routine in RAND_FUNCS.iter() {
                    let prog = Source([function, routine, [HALT; 10], [HALT; 10], [HALT; 10]]);
                    for &ta in tiles.iter() {
                        for &tb in tiles.iter() {
                            for &tc in tiles.iter() {
                                let puz = genboi(ta, tb, tc);
                                puz.execute(&prog, false, |state, _| states.push(state.to_owned()));
                            }
                        }
                    }
                }
                //                if banned_pair(&template_puzzle, a, b, false)
                //                    || banned_pair(&template_puzzle, b, c, false)
                //                    || banned_trio(&template_puzzle, a, b, c, false) {
                //                    denies += 1;
                //                    continue;
                //                }
                let prog = Source([function, [HALT; 10], [HALT; 10], [HALT; 10], [HALT; 10]]);
                if mipmap.contains_key(&states) {
                    //                    println!("the program {} \tis already represented by \t{}", prog, mipmap[&states]);
                } else {
                    print!("the program {} is the first of its kind", prog);
                    if banned_pair(&template_puzzle, a, b, false)
                        || banned_pair(&template_puzzle, b, c, false)
                        || banned_trio(&template_puzzle, a, b, c, false)
                    {
                        denies += 1;
                        println!(" which was denied");
                        banned_pair(&template_puzzle, a, b, true);
                        banned_pair(&template_puzzle, b, c, true);
                        banned_trio(&template_puzzle, a, b, c, true);
                    } else {
                        nonies += 1;
                        println!(" which wasn't denied");
                    }
                }
                //                println!("{},", states);
            }
        }
    }
    println!(
        "counter: {}, denies: {}, nonies: {}",
        counter, denies, nonies
    );
}
