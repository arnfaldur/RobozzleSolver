use std::fmt::{Display, Formatter, Error};
use std::collections::{HashSet, HashMap, VecDeque, BinaryHeap};
use std::borrow::{Borrow, BorrowMut};
use std::cmp::{Ordering, max, min};
use std::thread::{spawn, sleep};
use std::time::Duration;
use std::sync::{Barrier, Arc, atomic::{AtomicUsize, Ordering as SyncOrdering}};

use once_cell::sync::OnceCell;
use crossbeam_channel::{unbounded, Receiver, Sender, bounded};

use crate::game::{Source, Puzzle, State, TileType, won};
use crate::game::instructions::*;
use crate::constants::*;
use crate::carlo::{score_cmp, carlo};
use crate::web::encode_program;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::mem;

const BACKTRACK_STACK_SIZE: usize = 2200;
// 44 * 50
static MAX_INS: AtomicUsize = AtomicUsize::new(0);

pub(crate) static REJECTS_2: OnceCell<HashSet<[Ins; 2]>> = OnceCell::new();
pub(crate) static REJECTS_3: OnceCell<HashSet<[Ins; 3]>> = OnceCell::new();
pub(crate) static REJECTS_4: OnceCell<HashSet<[Ins; 4]>> = OnceCell::new();

#[derive(Clone)]
pub struct Frame { candidate: Source, inters: usize }

impl Frame {
    fn new(puzzle: &Puzzle) -> Frame {
        Frame {
            candidate: puzzle.empty_source(),
            inters: 2,
        }
    }
}

const THREADS: usize = 1;

pub fn backtrack(puzzle: Puzzle) -> Vec<Source> {
//    let mut tested: HashSet<u64, _> = HashSet::new();
//    for thread in 0..THREADS {
//        thread::spawn(move || {

    let (sender, receiver) = unbounded();
//    for i in 2..puzzle.functions.iter().sum() {
//    sender.send();
//    }
//    MAX_INS.store(puzzle.methods.iter().sum(), SyncOrdering::Relaxed);
    MAX_INS.store(25, SyncOrdering::Relaxed);

//    sender.send(Frame::new(&puzzle));
    let mut threads = vec![];
    for thread in 0..THREADS {
        let (sclone, rclone) = (sender.clone(), receiver.clone());
        threads.push(spawn(move || {
            return backtrack_thread(&puzzle, thread, sclone, rclone);
        }));
    }
    let mut seeds = VecDeque::new();
//    for i in 2..=10 {
    let i = 10;
        let mut fra = Frame::new(&puzzle);
        fra.inters = fibonacci(i + 30);
        seeds.push_front(fra);
//    }
    while let Some(branch) = seeds.pop_back() {
        let solved = search(
            &puzzle, branch,
            |branch, _| if branch.candidate.count_ins() >= 3 {
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
            considered += 1;
            if candidate.count_ins() > MAX_INS.load(SyncOrdering::Relaxed) { continue; }
            let solved = search(
                puzzle,
                frame,
                |mut branch, is_nop| {
                    if !is_nop || branch.candidate.count_ins() <= MAX_INS.load(SyncOrdering::Relaxed) {
                        branch.candidate.shade(MAX_INS.load(SyncOrdering::Relaxed));
                        candidates.push_back(Frame {
                            ..branch
                        });
                    }
                },
            );
            if considered % (1 << 16) == 0 {
                if receiver.is_empty() || receiver.len() > 1 << 24 {
                    drop(sender);
                    return result;
                }
            }
            if considered % (1 << 20) == 0 || solved {
//                println!("considered: {}", considered);
                println!("work queue: {}, inters: {}, MAX_INS: {}",
                         sender.len(), inters, MAX_INS.load(SyncOrdering::Relaxed));
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

fn search<F>(puzzle: &Puzzle, Frame { mut candidate, inters }: Frame,
             mut brancher: F) -> bool where F: FnMut(Frame, bool) {
//    candidate.shade(MAX_INS.load(SyncOrdering::Relaxed));
//    if deny(puzzle, &candidate, false) { return false; }
    let mut state = puzzle.initial_state(&candidate);
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
    while running && state.inters <= inters {
        if !branched {
            let ins_pointer = state.ins_pointer();
            let ins = state.current_ins(&candidate);
            let method_index = ins_pointer.get_method_index();
            let ins_index = ins_pointer.get_ins_index();
            let nop_branch = ins.is_nop();
            let release_branch = !state.current_tile().executes(ins)
                && !ins.is_released(state.current_tile().to_cond())
                && (candidate[method_index][0..puzzle.methods[method_index]].contains(&HALT)
                // TODO: check if neccessary
                || candidate[method_index][0..puzzle.methods[method_index]].contains(&NOP));
            let loosening_branch = !ins.is_debug() && !ins.is_loose()
                && !state.current_tile().executes(ins.get_cond());
            if nop_branch || release_branch || loosening_branch {
                let mut instructions: Vec<Ins> =
                    if nop_branch {
                        [HALT].iter().chain(
                            puzzle.get_ins_set(state.current_tile().to_cond(), false).iter()
                                .filter(|&ins| {
                                    !ins.is_function() || preferred[ins.source_index()]
                                })
                        ).cloned().collect()
                    } else if release_branch {
                        puzzle.get_ins_set(state.current_tile().to_cond(), false)
                            .iter().map(|i| i.as_loose()
                            .as_released(state.current_tile().to_cond())
                            .as_released(ins.get_cond())).collect()
                    } else if loosening_branch {
                        vec![ins.as_loose(), ins.get_ins().as_loose()]
                    } else { vec![] };
//                let mut rng = thread_rng();
//                instructions.shuffle(&mut rng);
                if release_branch {
                    let mut temp = candidate.clone();
                    temp[method_index][ins_index] = temp[method_index][ins_index]
                        .as_released(state.current_tile().to_cond());
                    let other_inss = if !ins.is_loose() {
                        vec![ins.as_loose().as_released(state.current_tile().to_cond()),
                             ins.get_ins().as_loose().as_released(state.current_tile().to_cond())]
                    } else {
                        vec![ins.as_released(state.current_tile().to_cond())]
                    };
                    let ins = temp[method_index][ins_index];
                    for instruction in other_inss {
                        let mut temp = temp.clone();
                        temp[method_index][ins_index] = instruction;
                        if !snip_around(puzzle, &temp, *ins_pointer, false)
                            && !deny(puzzle, &temp, false) {
                            let branch = Frame { candidate: temp.to_owned(), inters };
                            brancher(branch, nop_branch);
                        }
                    }
                    let mut temp = candidate.clone();
                    for i in ((ins_index + 1)..puzzle.methods[method_index]).rev() {
                        temp[method_index][i] = temp[method_index][i - 1];
                    }
                    for &instruction in instructions.iter().rev() {
                        temp[method_index][ins_index] = instruction;
                        if !snip_around(puzzle, &temp, *ins_pointer, false)
                            && !deny(puzzle, &temp, false) {
                            let branch = Frame { candidate: temp.to_owned(), inters };
                            brancher(branch, nop_branch);
                        }
                    }
                } else {
                    for &instruction in instructions.iter().rev() {
                        let mut temp = candidate.clone();
                        temp[method_index][ins_index] = instruction;
                        if !snip_around(puzzle, &temp, *ins_pointer, false)
                            && !deny(puzzle, &candidate, false) {
                            let branch = Frame { candidate: temp.to_owned(), inters };
                            brancher(branch, nop_branch);
                        }
                    }
                }
                return state.stars == 0;
            }
        }
        running = state.step(&candidate, puzzle);
    }
    return state.stars == 0;
}

fn snip_around(puzzle: &Puzzle, temp: &Source, ins_pointer: InsPtr, show: bool) -> bool {
    let m = ins_pointer.get_method_index();
    let i = ins_pointer.get_ins_index();
    let mut result = false;
    for j in max(i, 1)..min(i + 1, puzzle.methods[m]) {
        let a = temp[m][j - 1];
        let b = temp[m][j];
        result |= banned_pair(puzzle, a, b, show);
        if show && result {
            println!("banned pair {}", j);
            return true;
        }
    }
    for j in max(i, 2)..min(i + 2, puzzle.methods[m]) {
        let a = temp[m][j - 2];
        let b = temp[m][j - 1];
        let c = temp[m][j];
        result |= banned_trio(puzzle, a, b, c, show);
        if show && result {
            println!("banned trio {}", j);
            return true;
        }
    }
    return result;
}

pub(crate) fn deny(puzzle: &Puzzle, program: &Source, show: bool) -> bool {
    let mut denied = false;
    let mut only_cond = [HALT, NOP, NOP, NOP, NOP, ];
    let mut invoked = [1, 0, 0, 0, 0];
    for m in 0..5 {
        let mut halt_count = 0;
        for i in 0..puzzle.methods[m] {
            let ins = program[m][i];
            if ins.is_function() {
                invoked[ins.source_index()] += 1;
                if only_cond[ins.source_index()] == NOP {
                    only_cond[ins.source_index()] = ins.get_cond() | ins.with_loose(true);
                } else if only_cond[ins.source_index()] != (ins.get_cond() | ins.with_loose(true)) {
                    only_cond[ins.source_index()] = HALT;
                }
                let called = program[ins.source_index()];
                denied |= called == [HALT; 10];
                if show && denied {
                    println!("only halt {}", ins.source_index());
                    return true;
                }
                let mut trivial = true;
                for j in 1..puzzle.methods[ins.source_index()] {
                    trivial &= called[j] == HALT;
                }
//                denied |= trivial && (ins.get_cond() == called[0].get_cond() || called[0].is_gray());
            }
//            halt_count += ins.is_halt() as usize;
        }
//        denied |= halt_count == puzzle.methods[m];
    }
    for m in 2..5 {
        let a = program[m - 1];
        let b = program[m];
        let mut acount = 0;
        let mut bcount = 0;
        for i in 0..10 {
            acount += !a[i].is_halt() as i32;
            bcount += !b[i].is_halt() as i32;
        }
        denied |= acount < bcount;
        if show && denied {
            println!("function lengths {} < {}", acount, bcount);
            return true;
        }
//        let mut same = true;
//        for i in 0..10 {
//            same &= a[i].is_halt() == b[i].is_halt() && a[i].is_nop() == b[i].is_nop();
//        }
//        if same {
//            denied |= a > b;
//        }
    }
//    denied |= !has_forward;
    for m in 0..5 {
        let meth = program[m];
        denied |= meth[0].is_function() && meth[0].source_index() == m;
        denied |= !program[m][0].is_halt() && program[m][1].is_halt();
        if show && denied {
            println!("ghal");
            return true;
        }
        for i in 1..puzzle.methods[m] {
            let a = meth[i - 1];
            let b = meth[i];
            denied |= !b.is_halt() && a.is_function() && a.is_gray() && a.source_index() == m;
            if show && denied { println!("de0"); }
//            denied |= banned_pair(puzzle, a, b, show);
//            if show && denied { println!("de1"); }
            if a.is_function() && invoked[a.source_index()] == 1 && only_cond[a.source_index()].is_gray() {
                denied |= banned_pair(puzzle, program[a.source_index()][puzzle.methods[a.source_index()] - 1], b, show);
                if show && denied { println!("de3"); }
            }
            if b.is_function() && invoked[b.source_index()] == 1 && only_cond[b.source_index()].is_gray() {
                denied |= banned_pair(puzzle, a, program[b.source_index()][0], show);
                if show && denied { println!("de5"); }
            }
        }
        if !only_cond[m].is_nop() && !only_cond[m].is_halt() {
            for i in 0..puzzle.methods[m] {
                denied |= !meth[i].is_cond(only_cond[m].get_cond())
                    && (meth[i].is_loose() == only_cond[m].is_loose());
                if !meth[i].is_turn() {
                    break;
                }
            }
            if show && denied { println!("de6"); }
        }
//        if show && denied { println!("de11"); }
    }
    return denied;
}

pub fn banned_pair(puzzle: &Puzzle, a: Ins, b: Ins, show: bool) -> bool {
    if b.is_halt() { return false; }
    let mut banned = false;
    banned |= a.is_halt() && !b.is_halt();
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

//pub fn reject(state: &State, puzzle: &Puzzle, program: &Source) -> bool {
//    if state.stack.len() > 1 {
//        let conditions = state.stack[0].get_cond() == state.stack[1].get_cond();
//        let wiggles = (state.stack[0].get_ins() == LEFT && state.stack[1].get_ins() == RIGHT) || (state.stack[0].get_ins() == RIGHT && state.stack[1].get_ins() == LEFT);
//        let marks = state.stack[0].is_mark() && state.stack[1].is_mark();
//        return conditions && (wiggles || marks);
//    } else {
//        return false;
//    }
//}

fn fibonacci(n: usize) -> usize {
    let mut a = 0;
    let mut b = 1;
    for i in 0..n {
        a += b;
        mem::swap(&mut a, &mut b);
    }
    return a;
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
