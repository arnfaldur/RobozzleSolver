use crate::game::{Puzzle, Source};

use backtrack::backtrack;
use mcts::monte_carlo;

pub(crate) mod pruning;

pub(crate) mod backtrack;
pub(crate) mod carlo;
mod mcts;

pub fn solve(puzzle: Puzzle) -> Vec<(usize, Source)> {
    return backtrack(puzzle);
}
