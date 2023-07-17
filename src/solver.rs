use crate::game::{Puzzle, Source};

use backtrack::backtrack;

pub mod pruning;

pub mod backtrack;
pub mod carlo;
mod mcts;

pub fn solve(puzzle: Puzzle) -> Vec<(usize, Source)> {
    return backtrack(puzzle, None);
}
