use std::collections::{VecDeque};
use std::cmp::{Ordering};
use std::thread::{spawn};
use std::time::Duration;
use std::sync::{atomic::{AtomicUsize, Ordering as SyncOrdering}};

use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::thread_rng;

use crate::game::{Source, Puzzle, State};
use crate::game::instructions::*;
use crate::constants::*;
use super::pruning::*;
use crate::web::encode_program;

const BACKTRACK_STACK_SIZE: usize = 2200;
// 44 * 50
static MAX_INS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct Frame { candidate: Source, state: State, inters: usize }

impl Frame {
    fn new(puzzle: &Puzzle) -> Frame {
        Frame {
            candidate: puzzle.empty_source(),
            state: puzzle.initial_state(&NOGRAM),
            inters: 2,
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

const THREADS: usize = 10;

pub fn backtrack(puzzle: Puzzle) -> Vec<Source> {
//    let mut tested: HashSet<u64, _> = HashSet::new();
//    for thread in 0..THREADS {
//        thread::spawn(move || {

    let (sender, receiver) = unbounded();
//    for i in 2..puzzle.functions.iter().sum() {
//    sender.send();
//    }
    MAX_INS.store(puzzle.methods.iter().sum(), SyncOrdering::Relaxed);
//    MAX_INS.store(19, SyncOrdering::Relaxed);

//    sender.send(Frame::new(&puzzle));
    let mut threads = vec![];
    for thread in 0..THREADS {
        let (sclone, rclone) = (sender.clone(), receiver.clone());
        threads.push(spawn(move || {
            return backtrack_thread(&puzzle, thread, sclone, rclone);
        }));
    }
    let mut seeds = VecDeque::new();
//    for i in 2..=puzzle.methods.iter().sum() {
    seeds.push_front(Frame::new(&puzzle));
//    }
    while let Some(branch) = seeds.pop_back() {
        let solved = search(
            &puzzle, branch,
            |branch, _| if branch.candidate.count_ins() >= 2 {
                sender.send(branch);
            } else {
                seeds.push_back(branch);
            },
        );
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
    return result;
}

fn backtrack_thread(puzzle: &Puzzle, thread_id: usize,
                    sender: Sender<Frame>, receiver: Receiver<Frame>) -> Vec<Source> {
    let mut result: Vec<Source> = vec![];
    let mut candidates = VecDeque::new();
    let mut considered: u64 = 0;
//    let mut visited: HashMap<_, Source> = HashMap::new();
    while let Ok(outer_frame) = receiver.recv_timeout(Duration::from_millis(100)) {
        candidates.push_back(outer_frame);
        while let Some(frame) = candidates.pop_back() {
            let candidate = frame.candidate;
            let inters = frame.inters;
            let steps = frame.state.steps;
            considered += 1;
            if candidate.count_ins() > MAX_INS.load(SyncOrdering::Relaxed) { continue; }
            let solved = search(
                puzzle,
                frame,
                |mut branch, is_nop| {
                    if !is_nop || branch.candidate.count_ins() <= MAX_INS.load(SyncOrdering::Relaxed) {
                        if (branch.state.inters >= branch.inters) && (receiver.len() < (1 << 20)) {
                            sender.send(Frame { inters: branch.inters + 1, ..branch });
                        } else {
                            branch.candidate.shade(MAX_INS.load(SyncOrdering::Relaxed));
                            candidates.push_back(Frame {
                                inters: branch.inters +
                                    if branch.state.inters >= branch.inters { 1 } else { 0 },
                                ..branch
                            });
                        }
                    }
                },
            );

            if considered % (1 << 14) == 0 {
                if receiver.is_empty() || receiver.len() > 1 << 24 {
                    drop(sender);
                    return result;
                }
            }
            if considered % (1 << 22) == 0 || solved {
//                if state.stars == 0 { print!("solution: "); }
//                println!("considered: {}", considered);
                println!("work queue: {}, inters: {}, MAX_INS: {}, steps: {}",
                         sender.len(), inters, MAX_INS.load(SyncOrdering::Relaxed), steps);
                println!("candidates: {}, current: {}", candidates.len(), candidate);
                for c in candidates.iter().rev().take(32) {
                    println!("{}", c.candidate);
                }
//                puzzle.execute(&candidate, true, won);
//                print!(" and {}", candidates);
                if solved {
                    result.push(candidate);
                    MAX_INS.store(candidate.count_ins() - 1, SyncOrdering::Relaxed);
                    println!("Solution found! {} {}", candidate, encode_program(&candidate, puzzle));
//                    while let Ok(dumped) = receiver.recv_timeout(Duration::from_millis(1)) {}
//                    drop(sender);
//                    return result;
//                println!("candidates length is {}", candidates.len());
//                break;
                }
            }
        }
    }
    return result;
}

fn search<F>(puzzle: &Puzzle, Frame { mut candidate, mut state, inters }: Frame,
             mut brancher: F) -> bool where F: FnMut(Frame, bool) {
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
            let loosening_branch = !ins.is_debug() && !ins.is_loosened()
                && !state.current_tile().to_condition().is_cond(ins.get_cond());
            if nop_branch || probe_branch || loosening_branch {
                let mut instructions: Vec<Ins> =
                    if nop_branch {
                        [HALT].iter().chain(
                            puzzle.get_ins_set(state.current_tile().to_condition(), false).iter()
                                .filter(|&ins| {
                                    !ins.is_function() || preferred[ins.source_index()]
                                })).chain(
                            puzzle.get_cond_mask().get_probes(state.current_tile()
                                .to_condition()).iter()
                        ).cloned().collect()
                    } else if probe_branch {
                        puzzle.get_ins_set(state.current_tile().to_condition(), false)
                            .iter().map(|i| i.as_loosened()).chain((
                            if candidate[method_index][ins_index]
                                .remove_cond(state.current_tile().to_condition()).is_probe() {
                                vec![candidate[method_index][ins_index]
                                    .remove_cond(state.current_tile().to_condition())]
                            } else { vec![] })
                        ).collect()
                    } else if loosening_branch {
                        vec![ins.as_loosened(), ins.get_ins().as_loosened()]
                    } else { vec![] };
//                let mut rng = thread_rng();
//                instructions.shuffle(&mut rng);
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
                        && !deny(puzzle, &candidate, false) {
                        let branch = Frame { candidate: temp.to_owned(), state: state.clone(), inters };
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
