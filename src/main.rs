#![feature(test, vec_remove_item, core_intrinsics)]
#![allow(dead_code, ellipsis_inclusive_range_patterns)]

use std::time::Instant;
use constants::*;
use carlo::carlo;
use backtrack::backtrack;
use crate::constants::*;

mod constants;
mod game;
mod tests;

mod carlo;
mod backtrack;

fn main() {
    let now = Instant::now();
    let maybe = backtrack(&PUZZLE_656);
    if let Some(solution) = maybe {
        println!("solved! {}", solution);
    }
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
