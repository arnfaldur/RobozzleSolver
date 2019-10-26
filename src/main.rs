#![feature(test, vec_remove_item, core_intrinsics)]
#![allow(dead_code, ellipsis_inclusive_range_patterns)]

use std::time::Instant;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::collections::{HashMap, HashSet};

use constants::*;
use game::{*, instructions::*};
use web::start_web_client;

use backtrack::*;
use carlo::carlo;

mod constants;
mod game;
mod tests;

mod carlo;
mod backtrack;

mod web;

fn main() {
    for puzzle in [PUZZLE_42, PUZZLE_536, PUZZLE_656].iter() {
        let now = Instant::now();
//    start_web_client();
//    denial_test();
        if let Some(solution) = backtrack(&puzzle) {
            println!("solved! {}", solution);
        } else {
            println!("I couldn't find a solution :(");
        }
        println!("The solver took {} seconds.\n", now.elapsed().as_secs_f64());
    }
//    let mut boiii : Vec<[Ins;2]> = get_rejects_2().iter().cloned().collect();
//    boiii.sort();
//    for e in boiii {
//        for i in e.iter() {
//            print!("{}",i);
//        }
//        println!();
//    }
//    PUZZLE_1337.execute(&PUZZLE_1337_SOLUTION, won);
//    carlo(&PUZZLE_42, 1 << 14, 1 << 11);
//    PUZZLE_TEST_1.execute(&PUZZLE_TEST_1_SOLUTION, |state, _| state.stars == 0);
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
// I = 9  -> MarkRed
// I = 10 -> MarkGreen
// I = 11 -> MarkBlue

// S = star
// S = 0 -> No Star
// S = 1 -> Star
const RAND_FUNCS: [Method; 5] = [
    [FORWARD, FORWARD, FORWARD, RIGHT, FORWARD, MARK_GRAY, FORWARD, MARK_RED, FORWARD, FORWARD],
    [MARK_BLUE, FORWARD, FORWARD, RIGHT, FORWARD, MARK_GRAY, FORWARD, MARK_RED, FORWARD, FORWARD],
    [RED_RIGHT, BLUE_LEFT, FORWARD, GREEN_RIGHT, BLUE_LEFT, FORWARD, RED_RIGHT, GREEN_LEFT, FORWARD, HALT],
    [RED_RIGHT, BLUE_LEFT, FORWARD, GREEN_RIGHT, BLUE_LEFT, FORWARD, RED_RIGHT, GREEN_LEFT, FORWARD, MARK_GREEN],
    [HALT; 10],
];

fn denial_test() {
    let template_puzzle = genboi(RE, GE, BS);
    let tmp = genboi(RE, GS, BS)
        .get_instruction_set(INS_COLOR_MASK, true);
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
            let c = HALT;
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
                            panic!("program with {}! instructions!", progins);
                        }
                        println!("the program {} \tis already represented by \t{}", prog, champ);
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
    let mut rejtmp: Vec<[Ins; 2]> = rejects2.iter().cloned().collect();
    rejtmp.sort();
    for e in rejtmp {
        print!("[");
        for i in e.iter() {
            print!("{:?}, ", i);
        }
        println!("],");
    }
    let mut rejtmp: Vec<[Ins; 3]> = rejects3.iter().cloned().collect();
    rejtmp.sort();
    for e in rejtmp {
        print!("[");
        for i in e.iter() {
            print!("{:?}, ", i);
        }
        println!("],");
    }
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
                if banned_pair(&template_puzzle, a, b, true)
                    || banned_pair(&template_puzzle, b, c, true)
                    || banned_trio(&template_puzzle, a, b, c, false) {
                    denies += 1;
                    continue;
                }
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