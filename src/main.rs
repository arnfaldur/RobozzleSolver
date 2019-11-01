#![feature(test, vec_remove_item, core_intrinsics, cfg_target_has_atomic)]
#![allow(dead_code, ellipsis_inclusive_range_patterns)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_must_use)]
#![allow(unused)]
#![allow(unreachable_code)]

use std::time::{Instant, Duration};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::mem;

use backtrack::*;
use carlo::carlo;
use web::{encode_program, start_web_solver, puzzle_from_string};

use constants::*;
use game::{*, instructions::*};

mod constants;
mod game;
mod tests;

mod carlo;
mod backtrack;

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

    let puzzle = puzzle_from_string(face);
//    puzzle.execute(&PUZZLE_656_SOLUTION, true, won);
    println!("puzl: {}", puzzle);
    let puzzles = [
//        PUZZLE_42,
//        PUZZLE_536,
//        PUZZLE_656,
//        PUZZLE_1337,
        puzzle,
//        parse_level(),
    ];
    for puzzle in puzzles.iter() {
        let now = Instant::now();
        let solutions = backtrack(*puzzle);
        if !solutions.is_empty() {
            println!("Solved! The solution(s) are:");
            for solution in solutions {
                println!("{} code: {}", solution, encode_program(&solution, puzzle));
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
    [FORWARD, GREEN_F1, HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT],
    [LEFT, RED_F1, HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT],
    [RIGHT, F1, HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT],
    [FORWARD, FORWARD, FORWARD, RIGHT, FORWARD, MARK_GRAY, FORWARD, MARK_RED, FORWARD, FORWARD],
    [MARK_GREEN, FORWARD, FORWARD, RIGHT, FORWARD, MARK_GRAY, FORWARD, MARK_RED, FORWARD, FORWARD],
    [RED_RIGHT, GREEN_LEFT, FORWARD, GREEN_RIGHT, GREEN_LEFT, FORWARD, RED_RIGHT, GREEN_LEFT, FORWARD, HALT],
    [RED_RIGHT, GREEN_LEFT, FORWARD, GREEN_RIGHT, GREEN_LEFT, FORWARD, RED_RIGHT, GREEN_LEFT, FORWARD, MARK_GREEN],
    [HALT; 10],
];

fn denial_test() {
    let template_puzzle = genboi(RE, GE, BS);
    let tmp = template_puzzle
        .get_ins_set(INS_COLOR_MASK, true);
    let instructions = [HALT].iter().chain(tmp.iter()
//        .filter(|&ins| !ins.is_function() || ins.get_instruction() == F2));
        .filter(|&ins| !ins.is_function()));
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
            if a == HALT && b != HALT { break; }
            for &c in instructions.clone() {
                if b == HALT && c != HALT { break; }
//                if a.is_mark() || b.is_mark() || c.is_mark() { continue; }
                counter += 1;
                let mut states = vec![];
                let function = [a, b, c, HALT, HALT, HALT, HALT, HALT, HALT, HALT];
                if banned_pair(&template_puzzle, a, b, false)
                    || banned_pair(&template_puzzle, b, c, false)
                    || banned_trio(&template_puzzle, a, b, c, false) {
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
                        println!("the program {} \tis already represented by \t{}", prog, champ);
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
    println!("counter: {}, denies: {}, nonies: {}", counter, denies, nonies);
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
            if a == HALT && b != HALT { break; }
            let c = HALT;
            for &c in instructions.clone() {
                if b == HALT && c != HALT { break; }
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
                        || banned_trio(&template_puzzle, a, b, c, false) {
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
    println!("counter: {}, denies: {}, nonies: {}", counter, denies, nonies);
}