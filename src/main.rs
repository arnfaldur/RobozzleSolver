#![feature(test, vec_remove_item, core_intrinsics)]
#![allow(dead_code, ellipsis_inclusive_range_patterns)]

use std::time::Instant;
use constants::*;
use carlo::carlo;
use backtrack::backtrack;
use crate::constants::*;
use crate::game::{State, Puzzle, genboi, Instruction, Tile, won, Source};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::collections::HashMap;
use crate::backtrack::deny;

mod constants;
mod game;
mod tests;

mod carlo;
mod backtrack;

fn main() {
    let now = Instant::now();
    let template_puzzle = genboi(_N, _N, _N);
    let tmp = genboi(RE, GS, BS)
        .get_instruction_set(INS_COLOR_MASK, true);
    let instructions = [HALT].iter().chain(tmp.iter()
        .filter(|&ins| !ins.is_function()));
    let tiles = [RS, GS, BS];

    let mut mipmap = HashMap::new();
    let mut counter = 0;
    for &a in instructions.clone() {
        for &b in instructions.clone() {
            if a == HALT && b != HALT {
                break;
            }
            for &c in instructions.clone() {
                if b == HALT && c != HALT {
                    break;
                }
                let prog = Source([[a, b, c, HALT, HALT, HALT, HALT, HALT, HALT, HALT], [HALT; 10], [HALT; 10], [HALT; 10], [HALT; 10]]);
                if deny(&template_puzzle, &prog) { break; }
                let mut reslist = vec![];
                for &ta in tiles.iter() {
                    for &tb in tiles.iter() {
                        for &tc in tiles.iter() {
                            let puz = genboi(ta, tb, tc);
                            let mut state = puz.initial_state();
                            counter += 1;
                            puz.execute(&prog, false, |state, puzzle| reslist.push(state.to_owned()));
                        }
                    }
                }
                if mipmap.contains_key(&reslist) {
                    println!("the program {} is already represented by {}", prog, mipmap[&reslist]);
                } else {
                    mipmap.insert(reslist, prog);
                    println!("the program {} is the first of its kind", prog);
                }
//                println!("{},", reslist);
            }
        }
    }
    println!("counter: {}", counter);
//    if let Some(solution) = backtrack(&PUZZLE_42) {
//        println!("solved! {}", solution);
//    } else {
//        println!("I couldn't find a solution :(");
//    }
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
