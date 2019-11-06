use crate::game::{Source, Puzzle};

use backtrack::backtrack;
use mcts::monte_carlo;

pub(crate) mod pruning;

pub(crate) mod carlo;
pub(crate) mod backtrack;
mod mcts;

pub fn solve(puzzle: Puzzle) -> Vec<Source> {
    return monte_carlo(&puzzle);
}