use crate::constants::*;
use crate::game::{instructions::*, won, Source};
use crate::solver::backtrack::{self, backtrack};
use crate::solver::carlo;
use crate::web::{get_level, get_levels, get_local_level};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use test::Bencher;

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
    assert_eq!(true, PUZZLE_42.execute(&PUZZLE_42_SOLUTION, false, won));
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
    assert_eq!(true, PUZZLE_1337.execute(&PUZZLE_1337_SOLUTION, false, won));
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

#[bench]
fn bench_execute_42_times_10(b: &mut Bencher) {
    //        for instruction in PUZZLE_42.get_instruction_set() {
    //            print!("{}", show_instruction(instruction));
    //        }
    //        println!();
    //        for instruction in PUZZLE_536.get_instruction_set() {
    //            print!("{}", show_instruction(instruction));
    //        }
    let instruction_set = PUZZLE_42.get_ins_set(INS_COLOR_MASK, true);
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
    let mut source: Source = Source([[NOP; 10]; 5]);
    for iteration in 0..10 {
        for i in 0..5 {
            for ins in 0..PUZZLE_42.methods[i] {
                source[i][ins] = *instruction_set.choose(&mut rng).unwrap_or(&NOP);
            }
        }
        b.iter(|| PUZZLE_42.execute(&source, false, carlo::score));
    }
}

#[bench]
fn bench_execute_42_solution(b: &mut Bencher) {
    b.iter(|| PUZZLE_42.execute(&PUZZLE_42_SOLUTION, false, carlo::score));
}

//    #[bench]
//    fn bench_42_monte_carlo(b: &mut Bencher) {
//        b.iter(|| carlo(&PUZZLE_42, 1 << 5, 1 << 5));
//    }

#[bench]
fn bench_execute_536_times_10(b: &mut Bencher) {
    let instruction_set = PUZZLE_536.get_ins_set(INS_COLOR_MASK, true);
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(536);
    let mut source: Source = Source([[NOP; 10]; 5]);
    for _iteration in 0..10 {
        for i in 0..5 {
            for ins in 0..PUZZLE_536.methods[i] {
                source[i][ins] = *instruction_set.choose(&mut rng).unwrap_or(&NOP);
            }
        }
        b.iter(|| PUZZLE_536.execute(&source, false, carlo::score));
    }
}

#[bench]
fn bench_execute_536_solution(b: &mut Bencher) {
    b.iter(|| PUZZLE_536.execute(&PUZZLE_536_SOLUTION, false, carlo::score));
}

//    #[bench]
//    fn bench_536_monte_carlo(b: &mut Bencher) {
//        b.iter(|| carlo(&PUZZLE_536, 1 << 5, 1 << 5));
//    }

#[bench]
fn bench_execute_656_times_10(b: &mut Bencher) {
    let instruction_set = PUZZLE_656.get_ins_set(INS_COLOR_MASK, true);
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(656);
    let mut source: Source = Source([[NOP; 10]; 5]);
    for _iteration in 0..10 {
        for i in 0..5 {
            for ins in 0..PUZZLE_656.methods[i] {
                source[i][ins] = *instruction_set.choose(&mut rng).unwrap_or(&NOP);
            }
        }
        b.iter(|| PUZZLE_656.execute(&source, false, carlo::score));
    }
}

#[bench]
fn bench_execute_656_solution(b: &mut Bencher) {
    b.iter(|| PUZZLE_656.execute(&PUZZLE_656_SOLUTION, false, carlo::score));
}

//    #[bench]
//    fn bench_656_monte_carlo(b: &mut Bencher) {
//        b.iter(|| carlo(&PUZZLE_656, 1 << 5, 1 << 5));
//    }

// #[bench]
// fn bench_backtrack_easy_puzzles(b: &mut Bencher) {
//     let puzzle_ids = [23, 24, 27, 28, 32, 34, 38, 43, 45, 46, 47, 49, 50];
//     let puzzle_ids = [23, 24];
//     let levels: Vec<_> = get_levels(puzzle_ids.into_iter())
//         .map(Result::unwrap)
//         .collect();
//     for level in levels.iter() {
//         b.iter(|| {
//             backtrack(level.puzzle, None);
//         });
//     }
// }
// made using:
// for n in [23, 24, 27, 28, 34, 45, 46, 47, 49, 52, 58, 59, 61, 62, 63, 67, 68, 69, 70, 73, 74, 75, 76, 79, 88, 101, 103, 105, 107, 108, 109, 112, 114, 118, 123, 124, 126, 128, 136, 138, 139, 140, 142, 147, 166, 168, 171, 202, 220]:
//     print(f"""#[bench]
//     fn bench_backtrack_{n}(b: &mut Bencher) {{
//         let level = get_local_level({n}).unwrap(
//         b.iter(|| {{
//             backtrack(level.puzzle, None);
//         }});
//     }}
//     """)

#[bench]
fn bench_backtrack_24(b: &mut Bencher) {
    let level = get_local_level(24).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_27(b: &mut Bencher) {
    let level = get_local_level(27).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_45(b: &mut Bencher) {
    let level = get_local_level(45).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_46(b: &mut Bencher) {
    let level = get_local_level(46).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_52(b: &mut Bencher) {
    let level = get_local_level(52).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_59(b: &mut Bencher) {
    let level = get_local_level(59).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_61(b: &mut Bencher) {
    let level = get_local_level(61).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_63(b: &mut Bencher) {
    let level = get_local_level(63).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_67(b: &mut Bencher) {
    let level = get_local_level(67).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_68(b: &mut Bencher) {
    let level = get_local_level(68).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_73(b: &mut Bencher) {
    let level = get_local_level(73).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_75(b: &mut Bencher) {
    let level = get_local_level(75).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_76(b: &mut Bencher) {
    let level = get_local_level(76).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_105(b: &mut Bencher) {
    let level = get_local_level(105).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_108(b: &mut Bencher) {
    let level = get_local_level(108).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_123(b: &mut Bencher) {
    let level = get_local_level(123).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_126(b: &mut Bencher) {
    let level = get_local_level(126).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_136(b: &mut Bencher) {
    let level = get_local_level(136).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_138(b: &mut Bencher) {
    let level = get_local_level(138).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_139(b: &mut Bencher) {
    let level = get_local_level(139).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_140(b: &mut Bencher) {
    let level = get_local_level(140).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_147(b: &mut Bencher) {
    let level = get_local_level(147).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_202(b: &mut Bencher) {
    let level = get_local_level(202).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}

#[bench]
fn bench_backtrack_220(b: &mut Bencher) {
    let level = get_local_level(220).unwrap();
    b.iter(|| {
        assert!(backtrack(level.puzzle, None).len() > 0);
    });
}
