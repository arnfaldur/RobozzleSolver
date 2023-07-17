use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate solver;

use solver::solver::backtrack::backtrack;
use solver::web::get_levels;

fn bench_backtrack_easy_puzzles(c: &mut Criterion) {
    //let puzzle_ids = [23, 24, 27, 28, 32, 34, 38, 43, 45, 46, 47, 49, 50];
    let puzzle_ids = [23, 24];
    let levels: Vec<_> = get_levels(puzzle_ids.into_iter())
        .map(Result::unwrap)
        .collect();
    for level in levels.iter() {
        c.bench_function("tst", |b| {
            b.iter(|| {
                assert!(backtrack(black_box(level.puzzle), None).len() > 0);
            })
        });
    }
}

criterion_group!(boi, bench_backtrack_easy_puzzles);
criterion_main!(boi);
