use crate::game::{Source, Puzzle, State};
use crate::game::instructions::*;
use crate::constants::*;
use crate::carlo::{score_cmp, carlo};

use std::fmt::{Display, Formatter, Error};
use std::collections::{HashSet, HashMap, VecDeque, BinaryHeap};
use once_cell::sync::OnceCell;
use std::borrow::{Borrow, BorrowMut};
use std::cmp::{Ordering, max, min};
use std::thread::{spawn, sleep};
use crossbeam_channel::{unbounded, Receiver, Sender, bounded};
use std::sync::{Barrier, Arc};
use std::time::Duration;

const BACKTRACK_STACK_SIZE: usize = 2200; // 44 *

pub(crate) static REJECTS_2: OnceCell<HashSet<[Ins; 2]>> = OnceCell::new();
pub(crate) static REJECTS_3: OnceCell<HashSet<[Ins; 3]>> = OnceCell::new();
pub(crate) static REJECTS_4: OnceCell<HashSet<[Ins; 4]>> = OnceCell::new();

#[derive(Copy, Clone)]
struct Frame(Source, State);

impl Ord for Frame {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.steps.cmp(&other.1.steps)
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

const THREADS: usize = 24;
const CHANNEL_SIZE: usize = 1 << 12;

pub fn backtrack(puzzle: &'static Puzzle) -> Vec<Source> {
//    let mut tested: HashSet<u64, _> = HashSet::new();
//    for thread in 0..THREADS {
//        thread::spawn(move || {

    let (sender, receiver) = bounded(CHANNEL_SIZE);
    sender.send(Frame(puzzle.empty_source(), puzzle.initial_state(&NOGRAM)));

    let barrier = Arc::new(Barrier::new(THREADS + 1));

    let mut threads = vec![];
    for thread in 0..THREADS {
        let (sclone, rclone, bclone) = (sender.clone(), receiver.clone(), barrier.clone());
        threads.push(spawn(move || {
            return trackback(puzzle, thread, sclone, rclone, &bclone);
        }));
    }
    let mut result = vec![];
    for handle in threads {
        match handle.join() {
            Ok(solutions) => result.extend(solutions),
            Err(error) => println!("Thread joining error: {:?}", error),
        };
    }
    return result;
}

fn trackback(puzzle: &Puzzle, thread_id: usize, sender: Sender<Frame>, receiver: Receiver<Frame>, barrier: &Barrier) -> Vec<Source> {
    let mut result: Vec<Source> = vec![];
    let mut candidates = VecDeque::new();
    let mut considered: u64 = 0;
    let mut executed: u64 = 0;
    let mut rejects: u64 = 0;
    let mut denies: u64 = 0;
    let mut snips: u64 = 0;
    let mut duplicates: u64 = 0;
//    let mut visited: HashMap<_, Source> = HashMap::new();
    while let Ok(outer_frame) = receiver.recv_timeout(Duration::from_millis(1)) {
        candidates.push_back(outer_frame);
        while let Some(Frame(mut candidate, branch_state)) = candidates.pop_back() {
            considered += 1;

            if deny(puzzle, &candidate, false) {
                denies += 1;
                continue;
            }
//        if !tested.insert(candidate.get_hash()) {
//            rejects += 1;
//            continue;
//        }
            executed += 1;
            let mut preferred = [true; 5];
            for i in 1..candidate.0.len() {
                for j in (i + 1)..candidate.0.len() {
                    if candidate.0[i] == candidate.0[j] {
                        preferred[j] = false;
                    }
                }
            }
            let mut state = branch_state;
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
                        let instructions: Vec<Ins> =
                            if nop_branch {
                                puzzle.get_ins_set(state.current_tile().to_condition(), false).iter()
                                    .filter(|&ins| {
                                        !ins.is_function() || preferred[ins.source_index()]
                                    }).chain(
                                    puzzle.get_cond_mask().get_probes(state.current_tile().to_condition()).iter()
                                ).cloned().collect()
                            } else if probe_branch {
                                puzzle.get_ins_set(state.current_tile().to_condition(), false).iter().map(|i| i.as_loosened()).collect()
                            } else if loosening_branch {
                                vec![ins.get_ins().as_loosened()]
                            } else { vec![] };
                        let local = candidate.count_ins() >= 3;
                        for instruction in instructions {
                            let mut temp = candidate.clone();
                            temp[method_index][ins_index] = instruction;
                            if snip_around(puzzle, &temp, ins_pointer) {
                                snips += 1;
                            } else {
                                let branch = Frame(temp.to_owned(), state.clone());
                                if local {
                                    candidates.push_back(branch);
                                } else {
                                    match sender.try_send(branch) {
                                        Ok(o) => {
//                                            println!("sent from thread {}", thread_id);
                                        }
                                        Err(e) => {
                                            candidates.push_front(branch);
                                        }
                                    }
                                }
                            }
                        }
                        branched = true;
                        if nop_branch {
                            // this reduces performance but is necessary to find shorter solutions.
                            for i in ins_index..puzzle.functions[method_index] {
                                candidate[method_index][i] = HALT;
                            }
                            branched = false;
                        } else if loosening_branch {
                            candidate[method_index][ins_index] = candidate[method_index][ins_index].as_loosened();
                            branched = false;
                        } else {
                            executed -= 1;
                        }
                    }
                }
                running = state.step(&candidate, puzzle);
            }
            if considered % 10000 == 0 {
                if receiver.is_empty() {
                    drop(sender);
                    return result;
                } else {
                    if let Some(branch) = candidates.pop_front() {
                        sender.send(branch);
                    }
                }
            }
            if considered % 1000000000 == 0 || state.stars == 0 {
//                if state.stars == 0 { print!("solution: "); }
//                println!("considered: {}, executed: {}, \nrejects: {}, denies: {}, \nsnips: {}, duplicates: {}",
//                         considered, executed, rejects, denies, snips, duplicates);
//                println!("current: {}", candidate);
//            for c in candidates.iter().rev().take(16) {
//                println!("{}", c.0);
//            }
//            print!(" and {}", candidates);
//            if considered > 10000 { return vec![]; }
                if state.stars == 0 {
                    result.push(candidate);
                    while let Ok(dumped) = receiver.recv_timeout(Duration::from_millis(1)) { }
                    drop(sender);
                    return result;

//                println!("candidates length is {}", candidates.len());
//                break;
                }
            }
        }
    }
    return result;
}

fn snip_around(puzzle: &Puzzle, temp: &Source, ins_pointer: Ins) -> bool {
    let m = ins_pointer.get_method_index();
    let i = ins_pointer.get_ins_index();
    let mut result = false;
    for j in max(i, 1)..min(i + 1, puzzle.functions[m]) {
        let a = temp[m][j - 1];
        let b = temp[m][j];
        result |= banned_pair(puzzle, a, b, false);
    }
    for j in max(i, 2)..min(i + 2, puzzle.functions[m]) {
        let a = temp[m][j - 2];
        let b = temp[m][j - 1];
        let c = temp[m][j];
        result |= banned_trio(puzzle, a, b, c, false);
    }
    return result;
}

pub(crate) fn deny(puzzle: &Puzzle, program: &Source, show: bool) -> bool {
    let mut denied = false;
    let mut conditioned = [HALT, NOP, NOP, NOP, NOP, ];
    let mut invoked = [1, 0, 0, 0, 0];
    for method in 0..5 {
        for i in 0..puzzle.functions[method] {
            let ins = program.0[method][i];
            if ins.is_function() {
                invoked[ins.source_index()] += 1;
                if conditioned[ins.source_index()] == NOP {
                    conditioned[ins.source_index()] = (ins.get_cond() | ins.with_loosened(true));
                } else if conditioned[ins.source_index()] != (ins.get_cond() | ins.with_loosened(true)) {
                    conditioned[ins.source_index()] = HALT;
                }
            }
        }
    }
    for method in 0..5 {
        for i in 1..puzzle.functions[method] {
            let a = program.0[method][i - 1];
            let b = program.0[method][i];
            if b.is_halt() || b.is_nop() { break; }
//            denied |= banned_pair(puzzle, a, b, show);
//            if show && denied { println!("de1"); }
            if a.is_function() && invoked[a.source_index()] == 1 && conditioned[a.source_index()].is_gray() {
                denied |= banned_pair(puzzle, program.0[a.source_index()][puzzle.functions[a.source_index()] - 1], b, show);
                if show && denied { println!("de3"); }
            }
            if b.is_function() && invoked[b.source_index()] == 1 && conditioned[b.source_index()].is_gray() {
                denied |= banned_pair(puzzle, a, program.0[b.source_index()][0], show);
                if show && denied { println!("de5"); }
            }
        }
        if !conditioned[method].is_nop() && !conditioned[method].is_halt() {
            for i in 0..puzzle.functions[method] {
                denied |= !program.0[method][i].is_cond(conditioned[method].get_cond())
                    && (program.0[method][i].is_loosened() == conditioned[method].is_loosened());
                if !program.0[method][i].is_turn() {
                    break;
                }
            }
            if show && denied { println!("de6"); }
        }
//        for i in 2..puzzle.functions[method] {
//            let a = program.0[method][i - 2];
//            let b = program.0[method][i - 1];
//            let c = program.0[method][i];
//            if c == HALT || c == NOP { break; }
//            denied |= banned_trio(puzzle, a, b, c, show);
//        }
//        if show && denied { println!("de11"); }
    }
    return denied;
}

pub fn banned_pair(puzzle: &Puzzle, a: Ins, b: Ins, show: bool) -> bool {
    if b == HALT { return false; }
    let mut banned = false;
    if a.get_cond() == b.get_cond() {
        banned |= a.is_order_invariant() && b.is_order_invariant() && a > b;
        if show && banned {
            println!("conds1 a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.is_turn() && b.is_ins(RIGHT);
        if show && banned {
            println!("conds2 a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.is_mark() && !a.is_gray();
        if show && banned {
            println!("conds3 a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.is_gray() && a.is_turn() && b.is_mark();
        if show && banned {
            println!("conds4 a: {:?} b: {:?}", a, b);
            return true;
        }
    }
    if a.is_turn() && b.is_turn() {
        banned |= a > b; // only let a series of turns have one color order
        if show && banned {
            println!("turns a: {:?} b: {:?}", a, b);
            return true;
        }
    }
    if a.is_mark() && b.is_mark() {
        banned |= a.is_gray() || b.is_gray();
        if show && banned {
            println!("marks1 a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.get_ins() == b.get_ins() && a > b;
        if show && banned {
            println!("marks2 a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.get_mark_as_cond() == b.get_cond() && b.get_mark_as_cond() == a.get_cond();
        banned |= a.get_mark_as_cond() == b.get_mark_as_cond();
        banned |= a.get_mark_as_cond() == b.get_cond() && a.get_cond() != b.get_mark_as_cond() && a.get_cond() != b.get_cond();
        if show && banned {
            println!("marksX a: {:?} b: {:?}", a, b);
            return true;
        }
    }
    banned |= a.is_gray() && a.is_mark() && (!b.is_cond(a.get_mark_as_cond()) || false);
    if show && banned {
        println!("mark then cond a: {:?} b: {:?}", a, b);
        return true;
    }
    if (a.is_turn() && a.is_gray() && b.is_mark()) || (a.is_mark() && b.is_turn() && b.is_gray()) {
        banned |= a > b;
        if show && banned {
            println!("five a: {:?} b: {:?}", a, b);
            return true;
        }
    }
    if !a.is_gray() && !b.is_gray() && a.get_cond() != b.get_cond() {
        banned |= b.is_turn() && a.is_mark() && a.get_mark_as_cond() != b.get_cond();
        if show && banned {
            println!("triple color mark off a: {:?} b: {:?}", a, b);
            return true;
        }
    }
    if (puzzle.red as i32 + puzzle.green as i32 + puzzle.blue as i32) == 3 {
        banned |= a.is_gray() && !b.is_gray() && a.is_turn() && b.is_ins(a.get_ins().other_turn());
        if show && banned {
            println!("negation with all colors a: {:?} b: {:?}", a, b);
            return true;
        }
    } else if puzzle.red as i32 + puzzle.green as i32 + puzzle.blue as i32 == 2 {
        banned |= a.is_gray() && !b.is_gray() && a.is_turn() && b.is_ins(a.get_ins().other_turn());
        if show && banned {
            println!("seven a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.get_ins() == b.get_ins() && !a.is_gray() && !b.is_gray() && a.get_cond() != b.get_cond();
        banned |= (a.is_mark() && a.is_gray()) || (b.is_mark() && b.is_gray());
        banned |= a.is_mark() && !b.has_cond(a.get_mark_as_cond());
        banned |= a.is_order_invariant() && b.is_order_invariant() && a.get_ins() > b.get_ins();
        banned |= a.is_mark() && b.is_mark();
    }
    if show && banned {
        println!("Some other pair issue a: {:?} b: {:?}", a, b);
        return true;
    }
    banned //|| query_rejects_2(&[a, b])
}

pub fn banned_trio(puzzle: &Puzzle, a: Ins, b: Ins, c: Ins, show: bool) -> bool {
    if c.is_debug() { return banned_pair(puzzle, a, b, show); }
    let mut banned = false;
    if a.get_cond() == b.get_cond() && a.get_cond() == c.get_cond() {
        banned |= a.is_turn() && a == b && a == c;
    }
    if a.get_cond() == c.get_cond() {
        banned |= a.is_mark() && b.is_turn();
    }
    if a.is_turn() && a.is_gray() && b.is_mark() && c.is_turn() && c.is_gray() {
        banned |= !a.is_ins(LEFT) || !c.is_ins(LEFT);
    }
    banned |= a.is_mark() && a.is_gray() && b.is_order_invariant() && !c.is_cond(a.get_mark_as_cond());
    if a.is_turn() && b.is_turn() && c.is_turn() {
        banned |= a > b || b > c;
//        banned |= a.get_cond() != b.get_cond() && a.get_cond() != c.get_cond() && b.get_cond() != c.get_cond() && !a.is_gray() && !b.is_gray() && !c.is_gray();
    }
    banned || query_rejects_3(&[a, b, c])
}

fn banned_quartet(puzzle: &Puzzle, a: Ins, b: Ins, c: Ins, d: Ins, show: bool) -> bool {
    if d == HALT { return banned_trio(puzzle, a, b, c, show); }
    query_rejects_4(&[a, b, c, d])
}

pub fn reject(state: &State, puzzle: &Puzzle, program: &Source) -> bool {
    if state.stack.len() > 1 {
        let conditions = state.stack[0].get_cond() == state.stack[1].get_cond();
        let wiggles = (state.stack[0].get_ins() == LEFT && state.stack[1].get_ins() == RIGHT) || (state.stack[0].get_ins() == RIGHT && state.stack[1].get_ins() == LEFT);
        let marks = state.stack[0].is_mark() && state.stack[1].is_mark();
        return conditions && (wiggles || marks);
    } else {
        return false;
    }
}

pub(crate) fn query_rejects_2(query: &[Ins; 2]) -> bool {
    return REJECTS_2.get().unwrap_or_else(|| {
        init_rejects_2();
        return REJECTS_2.get().unwrap();
    }).contains(query);
}

pub(crate) fn query_rejects_3(query: &[Ins; 3]) -> bool {
    let mut q: [Ins; 3] = [HALT; 3];
    for i in 0..query.len() {
        q[i] = if query[i].is_function() { query[i].to_probe() } else { query[i] };
    }
    return REJECTS_3.get().unwrap_or_else(|| {
        init_rejects_3();
        return REJECTS_3.get().unwrap();
    }).contains(&q);
}

pub(crate) fn query_rejects_4(query: &[Ins; 4]) -> bool {
    return REJECTS_4.get().unwrap_or_else(|| {
        init_rejects_4();
        return REJECTS_4.get().unwrap();
    }).contains(query);
}
