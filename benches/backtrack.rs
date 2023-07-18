#![allow(unused)]

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use solver::solver::backtrack::backtrack;
use solver::web::get_levels;

fn bench_backtrack_easy_puzzles(c: &mut Criterion) {
    let benchmark_group_name = "Easy puzzles 1.0";
    let puzzle_ids = &[123, 27, 46, 45, 220, 24, 52, 264, 147, 222, 68, 202, 138];

    let mut group = c.benchmark_group(benchmark_group_name);
    group.warm_up_time(Duration::from_millis(30));
    group.sample_size(10);
    group.measurement_time(Duration::from_millis(200));

    fun_name(puzzle_ids, group);

    let benchmark_group_name = "Easy puzzles 1.5";
    let puzzle_ids = &[140, 287, 266, 139, 63, 136, 105];

    let mut group = c.benchmark_group(benchmark_group_name);
    group.warm_up_time(Duration::from_millis(30));
    group.sample_size(10);
    group.measurement_time(Duration::from_millis(200));

    fun_name(puzzle_ids, group);
    let benchmark_group_name = "Easy puzzles 2.0";
    let puzzle_ids = &[
        278, 59, 126, 61, 108, 76, 73, 79, 75, 67, 295, 265, 262, 285, 124,
    ];
    let mut group = c.benchmark_group(benchmark_group_name);
    group.warm_up_time(Duration::from_millis(30));
    group.sample_size(10);
    group.measurement_time(Duration::from_millis(200));

    fun_name(puzzle_ids, group);

    let benchmark_group_name = "Easy puzzles 2.5";
    let puzzle_ids = &[
        107, 23, 372, 306, 62, 276, 101, 28, 397, 47, 128, 168, 112, 279, 103, 368, 376, 166, 114,
    ];

    let mut group = c.benchmark_group(benchmark_group_name);
    group.warm_up_time(Duration::from_millis(30));
    group.sample_size(10);
    group.measurement_time(Duration::from_millis(200));

    fun_name(puzzle_ids, group);
    let benchmark_group_name = "Easy puzzles 3.0";
    let puzzle_ids = &[
        307, 109, 398, 420, 301, 49, 330, 289, 58, 69, 70, 74, 224, 118, 363, 389,
    ];
    let mut group = c.benchmark_group(benchmark_group_name);
    group.warm_up_time(Duration::from_millis(30));
    group.sample_size(10);
    group.measurement_time(Duration::from_millis(200));

    fun_name(puzzle_ids, group);
}

fn fun_name(
    puzzle_ids: &[u64],
    mut group: criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) {
    let levels: Vec<_> = get_levels(puzzle_ids.into_iter().map(|e| *e))
        .map(Result::unwrap)
        .collect();
    for (i, level) in levels.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("level", i), &level.puzzle, |b, puzzle| {
            b.iter(|| {
                assert!(backtrack(black_box(*puzzle), None).len() > 0);
            })
        });
    }
    group.finish();
}

fn backtrack_batches(c: &mut Criterion) {
    let benchmark_id = "Easy batch";
    let puzzle_ids = &[123, 27, 46, 45, 220, 24, 52, 264, 147, 222, 68, 202, 138];
    let mut group = c.benchmark_group("Batches");
    group.warm_up_time(Duration::from_millis(500));
    group.sample_size(100);
    group.measurement_time(Duration::from_millis(5000));
    group.noise_threshold(0.02);

    let levels: Vec<_> = get_levels(puzzle_ids.into_iter().map(|e| *e))
        .map(Result::unwrap)
        .collect();
    group.bench_with_input(benchmark_id, &levels, |b, levels| {
        b.iter(|| {
            for (_i, level) in levels.iter().enumerate() {
                assert!(backtrack(black_box(level.puzzle), None).len() > 0);
            }
        })
    });
    let benchmark_id = "Medium batch";
    let puzzle_ids = &[
        140, 287, 266, 139, 63, 136, 105, 278, 59, 126, 61, 108, 76, 73, 79, 75, 67, 295, 265, 262,
        285, 124, 107, 23, 372, 306, 62, 276, 101, 28, 397, 47, 128, 168, 112, 279, 103, 368, 376,
        166,
        114,
        // 140 287 266 139 63 136 105 278 59 126 61 108 76 73 79 75 67 295 265 262 285 124 107 23 372 306 62 276 101 28 397 47 128 168 112 279 103 368 376 166 114
    ];
    group.warm_up_time(Duration::from_millis(1000));
    group.sample_size(10);
    group.measurement_time(Duration::from_millis(10000));
    group.noise_threshold(0.02);

    let levels: Vec<_> = get_levels(puzzle_ids.into_iter().map(|e| *e))
        .map(Result::unwrap)
        .collect();
    group.bench_with_input(benchmark_id, &levels, |b, levels| {
        b.iter(|| {
            for (_i, level) in levels.iter().enumerate() {
                assert!(backtrack(black_box(level.puzzle), None).len() > 0);
            }
        })
    });
    let benchmark_id = "Hard batch";
    let puzzle_ids = &[
        307, 109, 398, 420,
        301, //49, 330, 289, 58, 69, 70, 74, 224, 118, 363, 389,
            // 307 109 398 420 301 49 330 289 58 69 70 74 224 118 363 389
    ];

    group.warm_up_time(Duration::from_millis(1000));
    group.sample_size(10);
    group.measurement_time(Duration::from_millis(10000));
    group.noise_threshold(0.02);

    let levels: Vec<_> = get_levels(puzzle_ids.into_iter().map(|e| *e))
        .map(Result::unwrap)
        .collect();
    group.bench_with_input(benchmark_id, &levels, |b, levels| {
        b.iter(|| {
            for (_i, level) in levels.iter().enumerate() {
                assert!(backtrack(black_box(level.puzzle), None).len() > 0);
            }
        })
    });
    group.finish();
}

criterion_group!(boi, backtrack_batches);
criterion_main!(boi);

// 123 27 46 45 220 24 52 264 147 222 68 202 138
// 140 287 266 139 63 136 105 278 59 126 61 108 76 73 79 75 67 295 265 262 285 124 107 23 372 306 62 276 101 28 397 47 128 168 112 279 103 368 376 166 114
// 307 109 398 420 301 49 330 289 58 69 70 74 224 118 363 389

// 123 27 46 45 220 24 52 264 147 222 68 202 138 140 287 266 139 63 136 105 278 59 126 61 108 76 73 79 75 67 295 265 262 285 124 107 23 372 306 62 276 101 28 397 47 128 168 112 279 103 368 376 166 114 307 109 398 420 301 49 330 289 58 69 70 74 224 118 363 389
