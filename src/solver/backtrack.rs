use std::cmp::{max, Ordering};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering as SyncOrdering};
use std::thread::spawn;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use super::pruning::*;
use crate::constants::*;
use crate::game::instructions::*;
use crate::game::{Puzzle, Source, State};
use crate::web::encode_program;

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
    pub max_steps: usize,
    pub max_instructions: usize,
}

impl Frame {
    fn new(puzzle: &Puzzle) -> Frame {
        Frame {
            candidate: puzzle.empty_source(),
            state: puzzle.initial_state(&NOGRAM),
            score: 0,
            max_score: 1,
            max_steps: usize::MAX,
            max_instructions: puzzle.methods.iter().sum(),
        }
    }
}

pub fn backtrack(puzzle: Puzzle, timeout: Option<u128>) -> Vec<(usize, Source)> {
    let mut max_instructions = puzzle.methods.iter().sum();
    let mut max_steps = usize::MAX;
    let mut max_score = 16;
    max_instructions = 9;

    let now = Instant::now();

    let mut result: Vec<(usize, Source)> = vec![];
    'outer: for i in 0..50 {
        let mut outer_frame = Frame::new(&puzzle);
        outer_frame.max_score = 1 << i;
        outer_frame.max_steps = 1 << (5 + i * 2);
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
            coz::progress!("backtrack frame");
            if let Some(timeout) = timeout {
                if now.elapsed().as_millis() > timeout {
                    break 'outer;
                }
            }
            // if considered % (1 << 10) == 0 || solved {
            //     println!(
            //         "score: {}, MAX_INS: {}, MAX_SCORE: {}, steps: {}",
            //         score, max_instructions, max_score, steps
            //     );
            //     println!("candidates: {}, current: {}", candidates.len(), candidate);
            // }
        }
        coz::progress!("outer frame");
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
        max_steps,
        max_instructions,
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
    while running && state.steps < max_steps {
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
                            max_steps,
                            max_instructions,
                        };
                        brancher(branch, nop_branch);
                        coz::progress!("branching");
                    }
                }
                return state.stars == 0;
            }
        }
        coz::progress!("search state step");
        running = state.step(&candidate, puzzle);
    }
    return state.stars == 0;
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
