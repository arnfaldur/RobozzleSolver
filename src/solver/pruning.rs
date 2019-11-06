use std::cmp::{max, min};

use once_cell::sync::OnceCell;

use crate::game::{*, instructions::*};
use std::collections::HashSet;
use crate::constants::{init_rejects_2, init_rejects_3, init_rejects_4};

pub(crate) fn snip_around(puzzle: &Puzzle, temp: &Source, ins_pointer: InsPtr, show: bool) -> bool {
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
//    let mut has_forward = false;
//    let starting_tile = *puzzle.initial_state(program).current_tile();
    for m in 0..5 {
        let mut halt_count = 0;
        for i in 0..puzzle.methods[m] {
            let ins = program[m][i];
            if ins.is_function() {
                invoked[ins.source_index()] += 1;
                if only_cond[ins.source_index()] == NOP {
                    only_cond[ins.source_index()] = ins.get_cond() | ins.with_loosened(true);
                } else if only_cond[ins.source_index()] != (ins.get_cond() | ins.with_loosened(true)) {
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
    }
//    denied |= !has_forward;
    for m in 0..5 {
        let meth = program[m];
        denied |= !program[m][0].is_halt() && program[m][1].is_halt();
        if show && denied {
            println!("ghal");
            return true;
        }
        for i in 1..puzzle.methods[m] {
            let a = meth[i - 1];
            let b = meth[i];
            if b.is_halt() {
                if show && denied { println!("b is halt and bad"); }
                return denied;
            }
            denied |= a.is_function() && a.is_gray() && a.source_index() == m;
            if show && denied { println!("de0"); }
            if b.is_nop() { break; }
            denied |= banned_pair(puzzle, a, b, show);
            if show && denied { println!("de1"); }
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
                    && (meth[i].is_loosened() == only_cond[m].is_loosened());
                if !meth[i].is_turn() {
                    break;
                }
            }
            if show && denied { println!("de6"); }
        }
//        for other in (m + 1)..5 {
//            let mother = program.0[other];
//            let mut same = true;
//            for i in 0..10 {
//                same &= meth[i].is_debug() == mother[i].is_debug();
//            }
//            if same {
//                denied |= meth > mother;
//            }
//        }
//        for i in 2..puzzle.functions[m] {
//            let a = meth[i - 2];
//            let b = meth[i - 1];
//            let c = meth[i];
//            if c == HALT || c == NOP { break; }
//            denied |= banned_trio(puzzle, a, b, c, show);
//        }
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

pub fn banned_quartet(puzzle: &Puzzle, a: Ins, b: Ins, c: Ins, d: Ins, show: bool) -> bool {
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

pub(crate) static REJECTS_2: OnceCell<HashSet<[Ins; 2]>> = OnceCell::new();
pub(crate) static REJECTS_3: OnceCell<HashSet<[Ins; 3]>> = OnceCell::new();
pub(crate) static REJECTS_4: OnceCell<HashSet<[Ins; 4]>> = OnceCell::new();

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
