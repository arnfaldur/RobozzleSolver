use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::constants::*;
use crate::game::{instructions::*, state::won, Source};
use crate::solver::backtrack::backtrack;
use crate::solver::carlo;
use crate::solver::solutions::read_solution_from_file;
use crate::web::get_local_level;

extern crate test;

#[test]
fn test_order_invariance() {
    for cond in [GRAY_COND, RED_COND, GREEN_COND, BLUE_COND].iter() {
        assert_eq!((FORWARD | *cond).is_order_invariant(), false);
        assert_eq!((LEFT | *cond).is_order_invariant(), true);
        assert_eq!((RIGHT | *cond).is_order_invariant(), true);
        assert_eq!((F1 | *cond).is_order_invariant(), false);
        assert_eq!((F2 | *cond).is_order_invariant(), false);
        assert_eq!((F3 | *cond).is_order_invariant(), false);
        assert_eq!((F4 | *cond).is_order_invariant(), false);
        assert_eq!((F5 | *cond).is_order_invariant(), false);
        assert_eq!((MARK_RED | *cond).is_order_invariant(), true);
        assert_eq!((MARK_GREEN | *cond).is_order_invariant(), true);
        assert_eq!((MARK_BLUE | *cond).is_order_invariant(), true);
        assert_eq!((HALT | *cond).is_order_invariant(), false);
        assert_eq!((NOP | *cond).is_order_invariant(), false);
    }
}

#[test]
fn test_puzzle_42() {
    assert_eq!(true, PUZZLE_42.execute(&PUZZLE_42_SOLUTION, true, won));
}

#[test]
fn test_puzzle_42_instruction_set() {
    assert_eq!(
        vec![FORWARD, LEFT, RIGHT, F1, F2, F3, F4,],
        PUZZLE_42.get_ins_set(GRAY_COND, true)
    );
}

#[test]
fn test_puzzle_536() {
    assert_eq!(true, PUZZLE_536.execute(&PUZZLE_536_SOLUTION, false, won));
}

#[test]
fn test_puzzle_536_instruction_set() {
    assert_eq!(
        vec![
            FORWARD,
            LEFT,
            RIGHT,
            F1,
            F2,
            RED_FORWARD,
            RED_LEFT,
            RED_RIGHT,
            RED_F1,
            RED_F2,
            GREEN_FORWARD,
            GREEN_LEFT,
            GREEN_RIGHT,
            GREEN_F1,
            GREEN_F2,
            BLUE_FORWARD,
            BLUE_LEFT,
            BLUE_RIGHT,
            BLUE_F1,
            BLUE_F2,
        ],
        PUZZLE_536.get_ins_set(INS_COLOR_MASK, true)
    );
}

#[test]
fn test_puzzle_656() {
    assert_eq!(true, PUZZLE_656.execute(&PUZZLE_656_SOLUTION, false, won));
}

#[test]
fn test_puzzle_656_instruction_set() {
    assert_eq!(
        vec![
            FORWARD,
            LEFT,
            RIGHT,
            F1,
            F2,
            RED_FORWARD,
            RED_LEFT,
            RED_RIGHT,
            RED_F1,
            RED_F2,
            BLUE_FORWARD,
            BLUE_LEFT,
            BLUE_RIGHT,
            BLUE_F1,
            BLUE_F2,
        ],
        PUZZLE_656.get_ins_set(INS_COLOR_MASK, true)
    );
}

#[test]
fn test_puzzle_1337() {
    assert_eq!(true, PUZZLE_1337.execute(&PUZZLE_1337_SOLUTION, true, won));
}

#[test]
fn test_puzzle_1337_instruction_set() {
    assert_eq!(
        vec![
            FORWARD,
            LEFT,
            RIGHT,
            F1,
            F2,
            MARK_RED,
            MARK_GREEN,
            MARK_BLUE,
            RED_FORWARD,
            RED_LEFT,
            RED_RIGHT,
            RED_F1,
            RED_F2,
            RED_MARK_GREEN,
            RED_MARK_BLUE,
            GREEN_FORWARD,
            GREEN_LEFT,
            GREEN_RIGHT,
            GREEN_F1,
            GREEN_F2,
            GREEN_MARK_RED,
            GREEN_MARK_BLUE,
            BLUE_FORWARD,
            BLUE_LEFT,
            BLUE_RIGHT,
            BLUE_F1,
            BLUE_F2,
            BLUE_MARK_RED,
            BLUE_MARK_GREEN,
        ],
        PUZZLE_1337.get_ins_set(INS_COLOR_MASK, true)
    );
}

#[test]
fn test_puzzle_test_1() {
    assert_eq!(
        true,
        PUZZLE_TEST_1.execute(&PUZZLE_TEST_1_SOLUTION, false, won)
    );
}

#[test]
fn test_puzzle_test_1_instruction_set() {
    assert_eq!(
        vec![
            FORWARD,
            LEFT,
            RIGHT,
            F1,
            F2,
            F3,
            F4,
            F5,
            MARK_RED,
            MARK_GREEN,
            MARK_BLUE,
            RED_FORWARD,
            RED_LEFT,
            RED_RIGHT,
            RED_F1,
            RED_F2,
            RED_F3,
            RED_F4,
            RED_F5,
            RED_MARK_GREEN,
            RED_MARK_BLUE,
            GREEN_FORWARD,
            GREEN_LEFT,
            GREEN_RIGHT,
            GREEN_F1,
            GREEN_F2,
            GREEN_F3,
            GREEN_F4,
            GREEN_F5,
            GREEN_MARK_RED,
            GREEN_MARK_BLUE,
            BLUE_FORWARD,
            BLUE_LEFT,
            BLUE_RIGHT,
            BLUE_F1,
            BLUE_F2,
            BLUE_F3,
            BLUE_F4,
            BLUE_F5,
            BLUE_MARK_RED,
            BLUE_MARK_GREEN,
        ],
        PUZZLE_TEST_1.get_ins_set(INS_COLOR_MASK, true)
    );
}

#[test]
fn test_debug_printing() {
    let instructions = [
        NOP,
        HALT,
        FORWARD,
        LEFT,
        RIGHT,
        F1,
        F2,
        F3,
        F4,
        F5,
        MARK_RED,
        MARK_GREEN,
        MARK_BLUE,
        RED_FORWARD,
        RED_LEFT,
        RED_RIGHT,
        RED_F1,
        RED_F2,
        RED_F3,
        RED_F4,
        RED_F5,
        RED_MARK_GREEN,
        RED_MARK_BLUE,
        GREEN_FORWARD,
        GREEN_LEFT,
        GREEN_RIGHT,
        GREEN_F1,
        GREEN_F2,
        GREEN_F3,
        GREEN_F4,
        GREEN_F5,
        GREEN_MARK_RED,
        GREEN_MARK_BLUE,
        BLUE_FORWARD,
        BLUE_LEFT,
        BLUE_RIGHT,
        BLUE_F1,
        BLUE_F2,
        BLUE_F3,
        BLUE_F4,
        BLUE_F5,
        BLUE_MARK_RED,
        BLUE_MARK_GREEN,
        RED_PROBE,
        GREEN_PROBE,
        BLUE_PROBE,
    ];
    let instruction_strings = [
        "NOP",
        "HALT",
        "FORWARD",
        "LEFT",
        "RIGHT",
        "F1",
        "F2",
        "F3",
        "F4",
        "F5",
        "MARK_RED",
        "MARK_GREEN",
        "MARK_BLUE",
        "RED_FORWARD",
        "RED_LEFT",
        "RED_RIGHT",
        "RED_F1",
        "RED_F2",
        "RED_F3",
        "RED_F4",
        "RED_F5",
        "RED_MARK_GREEN",
        "RED_MARK_BLUE",
        "GREEN_FORWARD",
        "GREEN_LEFT",
        "GREEN_RIGHT",
        "GREEN_F1",
        "GREEN_F2",
        "GREEN_F3",
        "GREEN_F4",
        "GREEN_F5",
        "GREEN_MARK_RED",
        "GREEN_MARK_BLUE",
        "BLUE_FORWARD",
        "BLUE_LEFT",
        "BLUE_RIGHT",
        "BLUE_F1",
        "BLUE_F2",
        "BLUE_F3",
        "BLUE_F4",
        "BLUE_F5",
        "BLUE_MARK_RED",
        "BLUE_MARK_GREEN",
        "RED_PROBE",
        "GREEN_PROBE",
        "BLUE_PROBE",
    ];
    for i in 0..instructions.len() {
        assert_eq!(format!("{:?}", instructions[i]), instruction_strings[i]);
    }
}

// #[bench]
// fn bench_execute_42_times_10(b: &mut Bencher) {
//     //        for instruction in PUZZLE_42.get_instruction_set() {
//     //            print!("{}", show_instruction(instruction));
//     //        }
//     //        println!();
//     //        for instruction in PUZZLE_536.get_instruction_set() {
//     //            print!("{}", show_instruction(instruction));
//     //        }
//     let instruction_set = PUZZLE_42.get_ins_set(INS_COLOR_MASK, true);
//     let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
//     let mut source: Source = Source([[NOP; 10]; 5]);
//     for iteration in 0..10 {
//         for i in 0..5 {
//             for ins in 0..PUZZLE_42.methods[i] {
//                 source[i][ins] = *instruction_set.choose(&mut rng).unwrap_or(&NOP);
//             }
//         }
//         b.iter(|| PUZZLE_42.execute(&source, false, carlo::score));
//     }
// }

// #[bench]
// fn bench_execute_42_solution(b: &mut Bencher) {
//     b.iter(|| PUZZLE_42.execute(&PUZZLE_42_SOLUTION, false, carlo::score));
// }

// //    #[bench]
// //    fn bench_42_monte_carlo(b: &mut Bencher) {
// //        b.iter(|| carlo(&PUZZLE_42, 1 << 5, 1 << 5));
// //    }

// #[bench]
// fn bench_execute_536_times_10(b: &mut Bencher) {
//     let instruction_set = PUZZLE_536.get_ins_set(INS_COLOR_MASK, true);
//     let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(536);
//     let mut source: Source = Source([[NOP; 10]; 5]);
//     for _iteration in 0..10 {
//         for i in 0..5 {
//             for ins in 0..PUZZLE_536.methods[i] {
//                 source[i][ins] = *instruction_set.choose(&mut rng).unwrap_or(&NOP);
//             }
//         }
//         b.iter(|| PUZZLE_536.execute(&source, false, carlo::score));
//     }
// }

// #[bench]
// fn bench_execute_536_solution(b: &mut Bencher) {
//     b.iter(|| PUZZLE_536.execute(&PUZZLE_536_SOLUTION, false, carlo::score));
// }

// //    #[bench]
// //    fn bench_536_monte_carlo(b: &mut Bencher) {
// //        b.iter(|| carlo(&PUZZLE_536, 1 << 5, 1 << 5));
// //    }

// #[bench]
// fn bench_execute_656_times_10(b: &mut Bencher) {
//     let instruction_set = PUZZLE_656.get_ins_set(INS_COLOR_MASK, true);
//     let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(656);
//     let mut source: Source = Source([[NOP; 10]; 5]);
//     for _iteration in 0..10 {
//         for i in 0..5 {
//             for ins in 0..PUZZLE_656.methods[i] {
//                 source[i][ins] = *instruction_set.choose(&mut rng).unwrap_or(&NOP);
//             }
//         }
//         b.iter(|| PUZZLE_656.execute(&source, false, carlo::score));
//     }
// }

// #[bench]
// fn bench_execute_656_solution(b: &mut Bencher) {
//     b.iter(|| PUZZLE_656.execute(&PUZZLE_656_SOLUTION, false, carlo::score));
// }

//    #[bench]
//    fn bench_656_monte_carlo(b: &mut Bencher) {
//        b.iter(|| carlo(&PUZZLE_656, 1 << 5, 1 << 5));
//    }

#[test]
fn test_cached_solutions() {
    let puzzle_ids = [
        14, 15, 16, 17, 18, 19, 20, 22, 23, 24, 25, 26, 27, 28, 32, 34, 37, 38, 42, 43, 44, 45, 46,
        47, 48, 49, 50, 52, 54, 55, 56, 58, 59, 61, 62, 63, 65, 67, 68, 69, 70, 72, 73, 74, 75, 76,
        79, 80, 81, 82, 83, 86, 87, 88, 92, 94, 96, 101, 103, 105, 107, 108, 109, 112, 113, 114,
        117, 118, 121, 123, 124, 125, 126, 128, 131, 132, 134, 135, 136, 138, 139, 140, 141, 142,
        143, 144, 146, 147, 149, 150, 152, 153, 154, 156, 157, 158, 159, 164, 165, 166, 168, 169,
        171, 173, 174, 175, 177, 178, 181, 183, 184, 188, 201, 202, 204, 205, 206, 207, 208, 214,
        215, 219, 220, 222, 223, 224, 227, 229, 234, 237, 238, 239, 242, 247, 253, 262, 263, 264,
        265, 266, 268, 276, 277, 278, 279, 284, 285, 287, 288, 289, 295, 297, 298,
    ];
    for puzzle_id in puzzle_ids {
        let level = get_local_level(puzzle_id).expect("should have read solved local level");
        let solutions =
            read_solution_from_file(puzzle_id).expect("should have read local puzzle solution");
        for solution in solutions {
            assert!(level.puzzle.execute(&solution, false, won));
        }
        assert!(!level.puzzle.execute(&TEST_SOURCE, false, won));
    }
}
