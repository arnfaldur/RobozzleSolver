#![feature(test, vec_remove_item, core_intrinsics)]
#![allow(dead_code, ellipsis_inclusive_range_patterns)]

use std::time::Instant;
use constants::*;
use carlo::carlo;
use backtrack::backtrack;
use crate::constants::*;
use crate::game::{State, Puzzle, genboi, Ins, Tile, won, Source};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::collections::{HashMap, HashSet};
use crate::backtrack::deny;

mod constants;
mod game;
mod tests;

mod carlo;
mod backtrack;

fn main() {
    let now = Instant::now();
//    denial_test();
    if let Some(solution) = backtrack(&PUZZLE_656) {
        println!("solved! {}", solution);
    } else {
        println!("I couldn't find a solution :(");
    }
//    PUZZLE_1337.execute(&PUZZLE_1337_SOLUTION, won);
//    carlo(&PUZZLE_42, 1 << 14, 1 << 11);
//    PUZZLE_TEST_1.execute(&PUZZLE_TEST_1_SOLUTION, |state, _| state.stars == 0);
    println!("The solver took {} seconds.", now.elapsed().as_secs_f64());
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

fn denial_test() {
    let template_puzzle = genboi(RE, GE, BS);
    let tmp = genboi(RE, GS, BS)
        .get_instruction_set(INS_COLOR_MASK, true);
    let instructions = [HALT].iter().chain(tmp.iter()
        .filter(|&ins| !ins.is_function()));
    let tiles = [RS, GS, BS];
    println!("jsdfk: {}", template_puzzle);
    let mut mipmap = HashMap::new();
    let mut rejects2 = HashSet::new();
    let mut rejects3 = HashSet::new();
    let mut counter = 0;
    for &a in instructions.clone() {
        for &b in instructions.clone() {
            if a == HALT && b != HALT { break; }
            let c = HALT;
            for &c in instructions.clone() {
                if b == HALT && c != HALT { break; }
//                if a.is_mark() || b.is_mark() || c.is_mark() { continue; }
                counter += 1;
                let prog = Source([[a, b, c, HALT, HALT, HALT, HALT, HALT, HALT, HALT], [HALT; 10], [HALT; 10], [HALT; 10], [HALT; 10]]);
                if deny(&template_puzzle, &prog, false) { continue; }
                let mut states = vec![];
                for &ta in tiles.iter() {
                    for &tb in tiles.iter() {
                        for &tc in tiles.iter() {
                            let puz = genboi(ta, tb, tc);
                            puz.execute(&prog, false, |state, puzzle| states.push(state.to_owned()));
                        }
                    }
                }
                if mipmap.contains_key(&states) {
                    let champ: Source = mipmap[&states];
                    let mut champins = champ.0.len() * champ.0[0].len();
                    for i in 0..champ.0.len() {
                        for j in 0..champ.0[i].len() {
                            champins -= (champ.0[i][j] == HALT) as usize;
                        }
                    }
                    let mut progins = prog.0.len() * prog.0[0].len();
                    for i in 0..prog.0.len() {
                        for j in 0..prog.0[i].len() {
                            progins -= (prog.0[i][j] == HALT) as usize;
                        }
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
//                        println!("the program {} \tis already represented by \t{}", prog, champ);
                    }
                } else {
                    mipmap.insert(states, prog);
//                    println!("the program {} is the first of its kind", prog);
                }
//                println!("{},", states);
            }
        }
    }
//    println!("dump: {:?}", rejects3);
    let mut denies = 0;
    let mut nonies = 0;
    for &a in instructions.clone() {
        for &b in instructions.clone() {
            if a == HALT && b != HALT { break; }
            let c = HALT;
            for &c in instructions.clone() {
                if b == HALT && c != HALT { break; }
                let prog = Source([[a, b, c, HALT, HALT, HALT, HALT, HALT, HALT, HALT], [HALT; 10], [HALT; 10], [HALT; 10], [HALT; 10]]);
                let mut states = vec![];
                for &ta in tiles.iter() {
                    for &tb in tiles.iter() {
                        let tc = _N;
                        for &tc in tiles.iter() {
                            let puz = genboi(ta, tb, tc);
                            puz.execute(&prog, false, |state, puzzle| states.push(state.to_owned()));
                        }
                    }
                }
                if mipmap.contains_key(&states) {
//                    println!("the program {} \tis already represented by \t{}", prog, mipmap[&states]);
                } else {
                    print!("the program {} is the first of its kind", prog);
                    if deny(&template_puzzle, &prog, false) {
                        denies += 1;
                        println!(" which was denied");
                        deny(&template_puzzle, &prog, true);
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