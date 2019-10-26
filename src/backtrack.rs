use crate::game::{Source, Puzzle, State, won};
use crate::game::instructions::*;
use crate::constants::*;
use crate::carlo::{score_cmp, carlo};
use std::fmt::{Display, Formatter, Error};
use std::collections::HashSet;
use once_cell::sync::OnceCell;
use std::borrow::{Borrow, BorrowMut};

const BACKTRACK_STACK_SIZE: usize = 2200; // 44 *

pub(crate) static REJECTS_2: OnceCell<HashSet<[Ins; 2]>> = OnceCell::new();
pub(crate) static REJECTS_3: OnceCell<HashSet<[Ins; 3]>> = OnceCell::new();
pub(crate) static REJECTS_4: OnceCell<HashSet<[Ins; 4]>> = OnceCell::new();

struct Stack {
    pointer: usize,
    data: [Source; BACKTRACK_STACK_SIZE],
}

impl Stack {
    fn new() -> Stack {
        Stack { pointer: 0, data: [NOGRAM; BACKTRACK_STACK_SIZE] }
    }
    fn push(&mut self, element: Source) {
        self.data[self.pointer] = element;
        self.pointer += 1;
    }
    fn pop(&mut self) -> Source {
        let result = self.top();
        self.pointer -= 1;
        return result;
    }
    fn top(&self) -> Source { self.data[self.pointer - 1] }
    fn len(&self) -> usize { self.pointer }
    fn empty(&self) -> bool { self.pointer == 0 }
    fn push_iterator<'a>(&mut self, puzzle: &Puzzle, source: &Source, i: usize, j: usize, new_instructions: impl DoubleEndedIterator<Item=&'a Ins>, state: &State, branched: bool) -> u64 {
//        let last_pointer = self.pointer;
        let mut denies = 0;
        for &instruction in new_instructions.rev() {
            let mut temp = source.clone();
            temp[i][j] = instruction.with_branched(branched);
            if (j > 0 && banned_pair(puzzle, temp[i][j - 1], temp[i][j], false))
                || (j < puzzle.functions[i] - 1 && banned_pair(puzzle, temp[i][j], temp[i][j + 1], false))
                || (j > 1 && banned_trio(puzzle, temp[i][j - 2], temp[i][j - 1], temp[i][j], false)) {
                denies += 1;
            } else {
                self.push(temp.to_owned());
            }
        }
//        self.data.get_mut(last_pointer..self.pointer).unwrap().sort_by_cached_key(|prog| {
//            let mut temp = state.clone();
//            while temp.running() {
//                temp.step(prog, puzzle);
//            }
//            return score_cmp(&temp, puzzle);
//        });
        return denies;
    }
}

const THREADS: usize = 16;

pub fn backtrack(puzzle: &Puzzle) -> Option<Source> {
    let mut tested: HashSet<u64, _> = HashSet::new();
//    for thread in 0..THREADS {
//        thread::spawn(move || {
    let mut stack: Stack = Stack::new();
    stack.push(puzzle.empty_source());
//    stack.push(PUZZLE_1337_SOLUTION);
    let mut considered: u64 = 0;
    let mut executed: u64 = 0;
    let mut rejects: u64 = 0;
    let mut denies: u64 = 0;
    let mut snips: u64 = 0;

    let mut duplicates: u64 = 0;

    let mut visited: HashSet<u64> = HashSet::new();
    while !stack.empty() {
        considered += 1;

        let mut top = stack.pop();
        if deny(puzzle, &top, false) {
            denies += 1;
            continue;
        }
//        if !tested.insert(top.get_hash()) {
//            rejects += 1;
//            continue;
//        }
        executed += 1;
        let mut preferred = [true; 5];
        for i in 1..top.0.len() {
            for j in (i + 1)..top.0.len() {
                if top.0[i] == top.0[j] {
                    preferred[j] = false;
                }
            }
        }
//        let mut activations = [[GRAY_COND; 10]; 5];
        let mut state = puzzle.initial_state();
        state.stack.push(F1);
        state.step(&top, &puzzle);
        let mut branched = false;
        while state.running() {
            if state.steps > 256 && !visited.insert(state.get_hash()) {
                duplicates += 1;
                break;
            }
            let stack_top = state.stack.top().clone();
            let i = stack_top.get_method_number();
            let j = stack_top.get_instruction_number();
            let current_instruction = stack_top.as_vanilla();
            if stack_top.is_nop() {
                if top[i][j].is_nop() {
//                    branched = true;
                    let mut temp = top.clone();
                    for k in j..puzzle.functions[i] {
                        temp[i][k] = HALT;
                    }
                    stack.push(temp);
                    snips += stack.push_iterator(puzzle, &top, i, j,
                                                 puzzle.get_instruction_set(state.current_tile().to_condition(), false).iter()
                                                     .filter(|&ins| {
                                                         true || !ins.is_function() || preferred[ins.source_index()]
                                                     }).chain(
                                                     puzzle.get_condition_mask().get_probes(state.current_tile().to_condition()).iter()
                                                 ), &state, false);
                    executed -= 1;
                }
                break;
            } else if stack_top.is_probe() {
                if state.current_tile().clone().executes(stack_top) {
                    snips += stack.push_iterator(puzzle, &top, i, j,
                                                 puzzle.get_instruction_set(state.current_tile().to_condition(), false).iter()
                                                 , &state, true);
                    executed -= 1;
                    break;
                }
            } else if !stack_top.is_debug() && !stack_top.is_branched()
                && state.current_tile().to_condition() != stack_top.get_condition() {
//                top[i][j] = top[i][j].as_branched();
                snips += stack.push_iterator(puzzle, &top, i, j,
                                             [stack_top.as_vanilla(), stack_top.as_vanilla().get_instruction()].iter()
                                             , &state, true);
                executed -= 1;
                break;
            }
//            else if !stack_top.is_debug() && stack_top.is_gray() && !branched {
////                branched = true;
//                let i = state.stack_frame().source_index();
//                let j = state.instruction_number(puzzle);
//                if activations[i][j].is_gray() {
//                    activations[i][j] = state.current_tile().to_condition();
//                } else if activations[i][j] != state.current_tile().to_condition() && activations[i][j] != HALT {
//                    snips += stack.push_iterator(puzzle, &top, i, j,
//                                                 [stack_top | activations[i][j]].iter()
//                                                 , &state);
//                    activations[i][j] = HALT;
//                }
//            }
            state.step(&top, &puzzle);
        }
        if considered % 10000000 == 0 || state.stars == 0 {
            if state.stars == 0 { print!("done! "); }
            print!("considered: {}, executed: {}, \nrejects: {}, denies: {}, \nsnips: {}, duplicates: {}",
                   considered, executed, rejects, denies, snips, duplicates);
            print!(" and {}", stack);
            println!();
//            if considered > 10000 { return None; }
            if state.stars == 0 { return Some(top); }
        }
    }
    return None;
//        });
//    }

//    return None;
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
                    conditioned[ins.source_index()] = ins.get_condition();
                } else if conditioned[ins.source_index()] != ins.get_condition() {
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
//        if conditioned[method] != NOP && conditioned[method] != HALT {
//            for i in 0..puzzle.functions[method] {
//                denied |= !program.0[method][i].is_condition(conditioned[method].get_condition());
//                if !program.0[method][i].is_turn() {
//                    break;
//                }
//            }
//            if show && denied { println!("de6"); }
//        }
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
    if (a.is_debug()) || b.is_debug() { return false; }
    let mut banned = false;
    if a.get_condition() == b.get_condition() {
        banned |= a.is_order_invariant() && b.is_order_invariant() && a > b;
        if show && banned {
            println!("conds1 a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.is_turn() && b.is_instruction(RIGHT);
        if show && banned {
            println!("conds2 a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.is_mark() && !a.is_gray();
        if show && banned {
            println!("conds3 a: {:?} b: {:?}", a, b);
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
        banned |= b.is_gray();
        if show && banned {
            println!("marks1 a: {:?} b: {:?}", a, b);
            return true;
        }
        banned |= a.get_instruction() == b.get_instruction() && (a > b || !a.is_gray() && !b.is_gray());
        if show && banned {
            println!("marks2 a: {:?} b: {:?}", a, b);
            return true;
        }
        if puzzle.marks == [true, true, true] {
            banned |= a.get_mark_as_condition() == b.get_condition();
        }
        if show && banned {
            println!("marks5 a: {:?} b: {:?}", a, b);
            return true;
        }
    }
    banned |= a.is_gray() && a.is_mark() && !b.is_gray();
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
    if !a.is_gray() && !b.is_gray() && a.get_condition() != b.get_condition() {
        banned |= (a.is_turn() && b.is_mark() && b.get_mark_as_condition() != a.get_condition()) || (b.is_turn() && a.is_mark() && a.get_mark_as_condition() != b.get_condition());
        if show && banned {
            println!("triple color mark off a: {:?} b: {:?}", a, b);
            return true;
        }
    }
    if (puzzle.red as i32 + puzzle.green as i32 + puzzle.blue as i32) == 3 {
        banned |= a.is_gray() && !b.is_gray() && a.is_turn() && b.is_instruction(a.get_instruction().other_turn());
        if show && banned {
            println!("negation with all colors a: {:?} b: {:?}", a, b);
            return true;
        }
    } else if puzzle.red as i32 + puzzle.green as i32 + puzzle.blue as i32 == 2 {
        banned |= a.is_gray() && !b.is_gray() && a.is_turn() && b.is_instruction(a.get_instruction().other_turn());
        if show && banned {
            println!("seven a: {:?} b: {:?}", a, b);
            return true;
        }
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
    if a.get_condition() == b.get_condition() && a.get_condition() == c.get_condition() {
        banned |= a.is_turn() && a == b && a == c;
    }
    if a.get_condition() == c.get_condition() {
        banned |= a.is_mark() && b.is_turn();
    }
    if a.is_turn() && a.is_gray() && b.is_mark() && c.is_turn() && c.is_gray() {
        banned |= !a.is_instruction(LEFT) || !c.is_instruction(LEFT);
    }
    banned || query_rejects_3(&[a, b, c])
}

fn banned_quartet(puzzle: &Puzzle, a: Ins, b: Ins, c: Ins, d: Ins, show: bool) -> bool {
    if d == HALT { return banned_trio(puzzle, a, b, c, show); }
    query_rejects_4(&[a, b, c, d])
}

pub fn reject(state: &State, puzzle: &Puzzle, program: &Source) -> bool {
    if state.stack.len() > 1 {
        let conditions = state.stack[0].get_condition() == state.stack[1].get_condition();
        let wiggles = (state.stack[0].get_instruction() == LEFT && state.stack[1].get_instruction() == RIGHT) || (state.stack[0].get_instruction() == RIGHT && state.stack[1].get_instruction() == LEFT);
        let marks = state.stack[0].is_mark() && state.stack[1].is_mark();
        return conditions && (wiggles || marks);
    } else {
        return false;
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Stack: ({},\n", self.pointer)?;
        for i in (0..self.pointer).rev() {
            write!(f, "{},\n", self.data[i])?;
        }
        write!(f, ")")
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
