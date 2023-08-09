#![allow(unused)]

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use solver::{game::state::won, solver::solutions::read_solution_from_file, web::get_local_level};

fn cached_solutions(c: &mut Criterion) {
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
    let pairs: Vec<_> = puzzle_ids
        .into_iter()
        .map(|puzzle_id| {
            let level = get_local_level(puzzle_id).expect("should have read solved local level");
            let solutions =
                read_solution_from_file(puzzle_id).expect("should have read local puzzle solution");
            (level, solutions)
        })
        .collect();

    // let mut group = c.benchmark_group("Solutions");
    // group.warm_up_time(Duration::from_millis(100));
    // group.sample_size(30);
    // group.measurement_time(Duration::from_millis(200));
    // group.noise_threshold(0.02);

    // for (level, solutions) in pairs {
    //     assert!(!solutions.is_empty());
    //     count += 1;
    //     group.bench_with_input(
    //         BenchmarkId::new("Puzzle Solutions", level.id),
    //         &(level.puzzle, solutions),
    //         |b, (puzzle, solutions)| {
    //             b.iter(|| {
    //                 for solution in solutions {
    //                     assert_eq!(true, puzzle.execute(&solution, false, won));
    //                 }
    //             });
    //         },
    //     );
    // }
    c.bench_with_input(BenchmarkId::new("Solution Set", "x"), &pairs, |b, pairs| {
        b.iter(|| {
            for (level, solutions) in pairs {
                assert!(!solutions.is_empty());
                for solution in solutions {
                    assert_eq!(true, level.puzzle.execute(&solution, false, won));
                }
            }
        });
    });

    // group.finish();
}

criterion_group!(benches, cached_solutions);
criterion_main!(benches);
