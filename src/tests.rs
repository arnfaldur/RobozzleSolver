#[cfg(test)]
mod tests {
    use test::Bencher;
    use crate::game::{Ins, Source};
    use crate::constants::*;
    use crate::carlo;
    use crate::backtrack;
    use rand::SeedableRng;
    use rand::seq::SliceRandom;

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
        assert_eq!(true, backtrack::accept(&PUZZLE_42, &PUZZLE_42_SOLUTION));
    }

    #[test]
    fn test_puzzle_42_instruction_set() {
        assert_eq!(vec![
            FORWARD,
            LEFT,
            RIGHT,
            F1,
            F2,
            F3,
            F4,
        ], PUZZLE_42.get_instruction_set(GRAY_COND, true));
    }

    #[test]
    fn test_puzzle_536() {
        assert_eq!(true, backtrack::accept(&PUZZLE_536, &PUZZLE_536_SOLUTION));
    }

    #[test]
    fn test_puzzle_536_instruction_set() {
        assert_eq!(vec![
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
        ], PUZZLE_536.get_instruction_set(INS_COLOR_MASK, true));
    }

    #[test]
    fn test_puzzle_656() {
        assert_eq!(true, backtrack::accept(&PUZZLE_656, &PUZZLE_656_SOLUTION));
    }

    #[test]
    fn test_puzzle_656_instruction_set() {
        assert_eq!(vec![
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
        ], PUZZLE_656.get_instruction_set(INS_COLOR_MASK, true));
    }

    #[test]
    fn test_puzzle_1337() {
        assert_eq!(true, backtrack::accept(&PUZZLE_1337, &PUZZLE_1337_SOLUTION));
    }

    #[test]
    fn test_puzzle_1337_instruction_set() {
        assert_eq!(vec![
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
        ], PUZZLE_1337.get_instruction_set(INS_COLOR_MASK, true));
    }

    #[test]
    fn test_puzzle_test_1() {
        assert_eq!(true, backtrack::accept(&PUZZLE_TEST_1, &PUZZLE_TEST_1_SOLUTION));
    }

    #[test]
    fn test_puzzle_test_1_instruction_set() {
        assert_eq!(vec![
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
        ], PUZZLE_TEST_1.get_instruction_set(INS_COLOR_MASK, true));
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
        let instruction_set = PUZZLE_42.get_instruction_set(INS_COLOR_MASK, true);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
        let mut source: Source = Source([[NOP; 10]; 5]);
        for iteration in 0..10 {
            for i in 0..5 {
                for ins in 0..PUZZLE_42.functions[i] {
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
        let instruction_set = PUZZLE_536.get_instruction_set(INS_COLOR_MASK, true);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(536);
        let mut source: Source = Source([[NOP; 10]; 5]);
        for _iteration in 0..10 {
            for i in 0..5 {
                for ins in 0..PUZZLE_536.functions[i] {
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
        let instruction_set = PUZZLE_656.get_instruction_set(INS_COLOR_MASK, true);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(656);
        let mut source: Source = Source([[NOP; 10]; 5]);
        for _iteration in 0..10 {
            for i in 0..5 {
                for ins in 0..PUZZLE_656.functions[i] {
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
}
