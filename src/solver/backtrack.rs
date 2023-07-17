use std::cmp::{max, Ordering};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering as SyncOrdering};
use std::thread::spawn;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::thread_rng;

use super::pruning::*;
use crate::constants::*;
use crate::game::instructions::*;
use crate::game::{Puzzle, Source, State};
use crate::web::encode_program;
use rand::prelude::SliceRandom;
use serde_json::error::Category::Syntax;
use std::f32::MAX;

const BACKTRACK_STACK_SIZE: usize = 2200;
// 44 * 50
static MAX_INS: AtomicUsize = AtomicUsize::new(0);
static MAX_SCORE: AtomicI64 = AtomicI64::new(0);

#[derive(Clone)]
pub struct Frame {
    pub candidate: Source,
    pub state: State,
    pub score: i64,
    pub max_score: i64,
}

impl Frame {
    fn new(puzzle: &Puzzle) -> Frame {
        Frame {
            candidate: puzzle.empty_source(),
            state: puzzle.initial_state(&NOGRAM),
            score: 0,
            max_score: 1,
        }
    }
}

impl Ord for Frame {
    fn cmp(&self, other: &Self) -> Ordering {
        self.state.steps.cmp(&other.state.steps)
    }
}

impl PartialOrd for Frame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Frame {}

const THREADS: usize = 32;

pub fn obacktrack(puzzle: Puzzle) -> Vec<(usize, Source)> {
    //    let mut tested: HashSet<u64, _> = HashSet::new();
    //    for thread in 0..THREADS {
    //        thread::spawn(move || {

    let (sender, receiver) = unbounded();
    //    for i in 2..puzzle.functions.iter().sum() {
    //    sender.send();
    //    }
    MAX_INS.store(puzzle.methods.iter().sum(), SyncOrdering::Relaxed);
    MAX_SCORE.store(16, SyncOrdering::Relaxed);
    MAX_INS.store(9, SyncOrdering::Relaxed);

    //    sender.send(Frame::new(&puzzle));
    let mut threads = vec![];
    for thread in 0..THREADS {
        let (sclone, rclone) = (sender.clone(), receiver.clone());
        threads.push(spawn(move || {
            return obacktrack_thread(&puzzle, thread, sclone, rclone);
        }));
    }
    let mut seeds = VecDeque::new();
    //    for i in 2..=puzzle.methods.iter().sum() {
    //seeds.push_front(Frame::new(&puzzle));
    for i in 0..50 {
        let mut f = Frame::new(&puzzle);
        //f.max_score = fibonacci(i);
        f.max_score = 1 << i;
        seeds.push_front(f);
    }
    //    }
    while let Some(branch) = seeds.pop_back() {
        let solved = osearch(&puzzle, branch, |branch, _| {
            if branch.candidate.count_ins() >= 2 {
                sender.send(branch);
            } else {
                seeds.push_back(branch);
            }
        });
    }
    //    sleep(Duration::from_secs(10));
    //    drop(sender);
    //    drop(receiver);
    let mut result = vec![];
    for handle in threads {
        match handle.join() {
            Ok(solutions) => {
                result.extend(solutions);
            }
            Err(error) => println!("Thread joining error: {:?}", error),
        };
    }
    result.sort();
    result.dedup();
    return result;
}

fn obacktrack_thread(
    puzzle: &Puzzle,
    thread_id: usize,
    sender: Sender<Frame>,
    receiver: Receiver<Frame>,
) -> Vec<(usize, Source)> {
    let mut result: Vec<(usize, Source)> = vec![];
    let mut candidates = VecDeque::new();
    let mut considered: u64 = 0;
    while let Ok(outer_frame) = receiver.recv_timeout(Duration::from_millis(100)) {
        if outer_frame.max_score > MAX_SCORE.load(SyncOrdering::Relaxed) {
            continue;
        }
        candidates.push_back(outer_frame);
        while let Some(frame) = candidates.pop_back() {
            let candidate = frame.candidate;
            let score = frame.score;
            let max_score = frame.max_score;
            let steps = frame.state.steps;
            considered += 1;
            if candidate.count_ins() > MAX_INS.load(SyncOrdering::Relaxed) {
                continue;
            }
            let solved = osearch(puzzle, frame, |mut branch, is_nop_branch| {
                // This is executed at the heart of the search function
                if !is_nop_branch
                    || branch.candidate.count_ins() <= MAX_INS.load(SyncOrdering::Relaxed)
                {
                    if branch.score > branch.max_score
                    //&& receiver.len() < (1 << 20)
                    {
                        let new_max =
                            max(branch.max_score * 2, MAX_SCORE.load(SyncOrdering::Relaxed));
                        MAX_SCORE.store(new_max, SyncOrdering::Relaxed);
                    //sender.send(Frame { ..branch });
                    } else if branch.candidate.count_ins() <= MAX_INS.load(SyncOrdering::Relaxed) {
                        // let preshade = candidate.clone();
                        branch.candidate.shade(MAX_INS.load(SyncOrdering::Relaxed));
                        // if (preshade.get_hash() != candidate.get_hash()) {
                        //     println!("before shade: {}", preshade);
                        //     println!(" after shade: {}", candidate);
                        // }
                        //if !deny(&puzzle, &branch.candidate, false) {
                        candidates.push_back(Frame { ..branch });
                        //}
                    }
                }
            });

            if considered % (1 << 14) == 0 {
                if receiver.is_empty() || receiver.len() > 1 << 24 {
                    drop(sender);
                    return result;
                }
            }
            if considered % (1 << 20) == 0 || solved {
                //                if state.stars == 0 { print!("solution: "); }
                //                println!("considered: {}", considered);
                println!(
                    "work queue: {}, score: {}, MAX_INS: {}, MAX_SCORE: {}, steps: {}",
                    sender.len(),
                    score,
                    MAX_INS.load(SyncOrdering::Relaxed),
                    max_score,
                    steps
                );
                println!("candidates: {}, current: {}", candidates.len(), candidate);
                // for c in candidates.iter().rev().take(32) {
                //     println!("{}", c.candidate);
                // }
                //                puzzle.execute(&candidate, true, won);
                //                print!(" and {}", candidates);
                if solved {
                    let mut solution = candidate.clone();
                    solution.sanitize();
                    result.push((steps, solution));
                    MAX_INS.store(solution.count_ins() - 0, SyncOrdering::Relaxed);

                    println!("Solution found! {}", solution);
                    println!("code: {}", encode_program(&solution, puzzle));
                    println!("Candidate was {}", &candidate);
                    //while let Ok(dumped) = receiver.recv_timeout(Duration::from_millis(1)) {}
                    //drop(sender);
                    //return result;
                    //                println!("candidates length is {}", candidates.len());
                    //                break;
                }
            }
        }
    }
    return result;
}

fn osearch<F>(
    puzzle: &Puzzle,
    Frame {
        mut candidate,
        mut state,
        score,
        max_score,
    }: Frame,
    mut brancher: F,
) -> bool
where
    F: FnMut(Frame, bool),
{
    //    candidate.shade(MAX_INS.load(SyncOrdering::Relaxed));
    //    if deny(puzzle, &candidate, false) { return false; }
    let mut preferred = [true; 5];
    for i in 1..candidate.0.len() {
        for j in (i + 1)..candidate.0.len() {
            if candidate.0[i] == candidate.0[j] {
                preferred[j] = false;
            }
        }
    }
    let mut branched = false;
    let mut running = true;
    while running {
        if !branched {
            let ins_pointer = state.ins_pointer();
            let ins = state.current_ins(&candidate);
            let method_index = ins_pointer.get_method_index();
            let ins_index = ins_pointer.get_ins_index();
            let nop_branch = ins.is_nop();
            let probe_branch = ins.is_probe() && state.current_tile().clone().executes(ins);
            let loosening_branch = !ins.is_debug()
                && !ins.is_loosened()
                && !state.current_tile().to_condition().is_cond(ins.get_cond());
            if nop_branch || probe_branch || loosening_branch {
                // instructions for branches of current program
                let mut instructions: Vec<Ins> = if nop_branch {
                    // Noop (unallocated) instruction hit, branches are all puzzle-legal
                    // commands of the color of the current tile and a probe instruction.
                    [
                    // HALT // including halt is questionable
                    ]
                    .iter()
                    .chain(
                        puzzle
                            .get_ins_set(state.current_tile().to_condition(), false)
                            .iter()
                            .filter(|&ins| !ins.is_function() || preferred[ins.source_index()]),
                    )
                    .chain(
                        puzzle
                            .get_cond_mask()
                            .get_probes(state.current_tile().to_condition())
                            .iter(),
                    )
                    .cloned()
                    .collect()
                } else if probe_branch {
                    // probe instruction hit(a nop with a color),
                    // add a branch for each command of the current tile.
                    puzzle
                        .get_ins_set(state.current_tile().to_condition(), false)
                        .iter()
                        .map(|i| i.as_loosened())
                        .chain(
                            if candidate[method_index][ins_index]
                                .remove_cond(state.current_tile().to_condition())
                                .is_probe()
                            {
                                vec![candidate[method_index][ins_index]
                                    .remove_cond(state.current_tile().to_condition())]
                            } else {
                                vec![]
                            },
                        )
                        .collect()
                } else if loosening_branch {
                    // try to make current instruction gray.
                    vec![ins.as_loosened(), ins.get_ins().as_loosened()]
                } else {
                    vec![]
                };
                //let mut rng = thread_rng();
                //instructions.shuffle(&mut rng);
                branched = true;
                if nop_branch {
                    //                    if candidate.count_ins() > MAX_INS {
                    //                        return state.stars == 0;
                    //                    }
                } else if probe_branch {
                    //                    candidate[method_index][ins_index].remove_cond(state.current_tile().to_condition());
                    //                    if !candidate[method_index][ins_index].is_gray() {
                    //                        branched = false;
                    //                    }
                }
                for &instruction in instructions.iter().rev() {
                    let mut temp = candidate.clone();
                    temp[method_index][ins_index] = instruction;
                    if !snip_around(puzzle, &temp, *ins_pointer, false)
                        && !deny(puzzle, &candidate, false)
                    {
                        let branch = Frame {
                            candidate: temp.to_owned(),
                            state: state.clone(),
                            score: state.map.0.iter().fold(0, |acc, &x| {
                                acc + x.iter().fold(0, |ac, &y| -> i64 {
                                    ac + {
                                        let ts: i64 =
                                            std::convert::TryInto::try_into(y.touches()).unwrap();
                                        ts * ts - 2 * ts
                                    } as i64
                                })
                            }) + state.steps as i64
                                + 4 * (state.stars as i64 - puzzle.stars as i64),
                            max_score,
                        };
                        brancher(branch, nop_branch);
                    }
                }
                //                return state.stars == 0;
                if nop_branch {
                    // this reduces performance but is necessary to find shorter solutions.
                    //                    let left = MAX_INS
                    //                    for i in ins_index..puzzle.methods[method_index] {
                    //                        candidate[method_index][i] = HALT;
                    //                    }
                    //                    branched = false;
                } else if loosening_branch {
                    //                    candidate[method_index][ins_index] = candidate[method_index][ins_index].as_loosened();
                    //                    branched = false;
                }
                return state.stars == 0;
            }
        }
        running = state.step(&candidate, puzzle);
    }
    return state.stars == 0;
}

pub fn backtrack(puzzle: Puzzle) -> Vec<(usize, Source)> {
    let mut max_instructions = puzzle.methods.iter().sum();
    let mut max_steps = usize::MAX;
    let mut max_score = 16;
    max_instructions = 9;

    let now = Instant::now();

    let mut result: Vec<(usize, Source)> = vec![];
    'outer: for i in 0..50 {
        let mut outer_frame = Frame::new(&puzzle);
        outer_frame.max_score = 1 << i;
        let puzzle = &puzzle;
        let mut candidates = VecDeque::new();
        let mut considered: u64 = 0;
        if result.len() == 0 {
            candidates.push_back(outer_frame);
        }
        while let Some(frame) = candidates.pop_back() {
            let candidate = frame.candidate;
            let score = frame.score;
            let steps = frame.state.steps;
            considered += 1;
            if candidate.count_ins() > max_instructions {
                continue;
            }
            let solved = search(puzzle, frame, |mut branch, is_nop_branch| {
                // This is executed at the heart of the search function
                if !is_nop_branch || branch.candidate.count_ins() <= max_instructions {
                    if branch.score > branch.max_score {
                        max_score = max(branch.max_score * 2, max_score);
                    } else if branch.candidate.count_ins() <= max_instructions {
                        branch.candidate.shade(max_instructions);
                        candidates.push_back(Frame { ..branch });
                    }
                }
            });

            if solved {
                let mut solution = candidate.clone();
                solution.sanitize();
                result.push((steps, solution));
                max_instructions = solution.count_ins();
            }
            if now.elapsed().as_nanos() > 5_000_000 {
                break 'outer;
            }
            // if considered % (1 << 20) == 0 || solved {
            //     println!(
            //         "score: {}, MAX_INS: {}, MAX_SCORE: {}, steps: {}",
            //         score, max_instructions, max_score, steps
            //     );
            //     if solved {
            //         let mut solution = candidate.clone();
            //         solution.sanitize();
            //         result.push((steps, solution));
            //         max_instructions = solution.count_ins();

            //         println!("Solution found! {}", solution);
            //         println!("code: {}", encode_program(&solution, puzzle));
            //     } else {
            //         println!("candidates: {}, current: {}", candidates.len(), candidate);
            //     }
            // }
        }
    }
    result.sort();
    result.dedup();
    return result;
}

fn search<F>(
    puzzle: &Puzzle,
    Frame {
        mut candidate,
        mut state,
        score,
        max_score,
    }: Frame,
    mut brancher: F,
) -> bool
where
    F: FnMut(Frame, bool),
{
    let mut preferred = [true; 5];
    for i in 1..candidate.0.len() {
        for j in (i + 1)..candidate.0.len() {
            if candidate.0[i] == candidate.0[j] {
                preferred[j] = false;
            }
        }
    }
    let mut branched = false;
    let mut running = true;
    while running {
        if !branched {
            let ins_pointer = state.ins_pointer();
            let ins = state.current_ins(&candidate);
            let method_index = ins_pointer.get_method_index();
            let ins_index = ins_pointer.get_ins_index();
            let nop_branch = ins.is_nop();
            let probe_branch = ins.is_probe() && state.current_tile().clone().executes(ins);
            let loosening_branch = !ins.is_debug()
                && !ins.is_loosened()
                && !state.current_tile().to_condition().is_cond(ins.get_cond());
            if nop_branch || probe_branch || loosening_branch {
                // instructions for branches of current program
                let mut instructions: Vec<Ins> = if nop_branch {
                    // Noop (unallocated) instruction hit, branches are all puzzle-legal
                    // commands of the color of the current tile and a probe instruction.
                    [
                    // HALT // including halt is questionable
                    ]
                    .iter()
                    .chain(
                        puzzle
                            .get_ins_set(state.current_tile().to_condition(), false)
                            .iter()
                            .filter(|&ins| !ins.is_function() || preferred[ins.source_index()]),
                    )
                    .chain(
                        puzzle
                            .get_cond_mask()
                            .get_probes(state.current_tile().to_condition())
                            .iter(),
                    )
                    .cloned()
                    .collect()
                } else if probe_branch {
                    // probe instruction hit(a nop with a color),
                    // add a branch for each command of the current tile.
                    puzzle
                        .get_ins_set(state.current_tile().to_condition(), false)
                        .iter()
                        .map(|i| i.as_loosened())
                        .chain(
                            if candidate[method_index][ins_index]
                                .remove_cond(state.current_tile().to_condition())
                                .is_probe()
                            {
                                vec![candidate[method_index][ins_index]
                                    .remove_cond(state.current_tile().to_condition())]
                            } else {
                                vec![]
                            },
                        )
                        .collect()
                } else if loosening_branch {
                    // try to make current instruction gray.
                    vec![ins.as_loosened(), ins.get_ins().as_loosened()]
                } else {
                    vec![]
                };
                branched = true;
                for &instruction in instructions.iter().rev() {
                    let mut temp = candidate.clone();
                    temp[method_index][ins_index] = instruction;
                    if !snip_around(puzzle, &temp, *ins_pointer, false)
                        && !deny(puzzle, &candidate, false)
                    {
                        let branch = Frame {
                            candidate: temp.to_owned(),
                            state: state.clone(),
                            score: state.map.0.iter().fold(0, |acc, &x| {
                                acc + x.iter().fold(0, |ac, &y| -> i64 {
                                    ac + {
                                        let ts: i64 =
                                            std::convert::TryInto::try_into(y.touches()).unwrap();
                                        ts * ts - 2 * ts
                                    } as i64
                                })
                            }) + state.steps as i64
                                + 4 * (state.stars as i64 - puzzle.stars as i64),
                            max_score,
                        };
                        brancher(branch, nop_branch);
                    }
                }
                return state.stars == 0;
            }
        }
        running = state.step(&candidate, puzzle);
    }
    return state.stars == 0;
}
