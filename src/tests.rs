use crate::constants::*;
use crate::game::{instructions::*, state::won, Source};
use crate::solver::backtrack::backtrack;
use crate::solver::solutions::read_solution_from_file;
use crate::web::get_local_level;

#[test]
fn test_backtracker() {
    let puzzle_ids = [
        23, 24, 27, 28, 45, 46, 47, 49, 52, 58, 59, 61, 62, 63, 67, 68, 69, 70, 73, 74, 75, 76, 79,
        101, 103, 105, 107, 108, 109, 112, 114, 123, 124, 126, 128, 136, 138, 139, 140, 147, 166,
        168, 202, 220, 222, 224, 262, 264, 265, 266, 276, 278, 279, 285, 287, 289, 295, 301, 306,
        307, 330, 363, 368, 372, 376, 397, 398,
    ];
    for puzzle_id in puzzle_ids {
        let level = get_local_level(puzzle_id).expect("should have read solved local level");

        let solutions = backtrack(level.puzzle, None);
        assert!(!solutions.is_empty());
        for (_steps, solution) in solutions {
            assert!(level.puzzle.execute(&solution, false, won));
        }
        assert!(!level.puzzle.execute(&TEST_SOURCE, false, won));
    }
}
