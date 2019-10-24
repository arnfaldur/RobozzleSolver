use crate::game::{Instruction, Source, Puzzle, State};
use crate::constants::*;
use crate::carlo::{score_cmp, carlo};
use crate::game::won;
use std::fmt::{Display, Formatter, Error};
use std::collections::{HashSet, HashMap};
use std::hash::Hash;
use std::borrow::{BorrowMut, Borrow};

const BACKTRACK_STACK_SIZE: usize = 2200; // 44 *

struct Stack {
    pointer: usize,
    data: [Source; BACKTRACK_STACK_SIZE],
}

impl Stack {
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
    fn push_branch(&mut self, source: &Source, method: usize, ip: usize, instruction: Instruction) {
        let mut temp = source.clone();
        temp[method][ip] = instruction;
        self.push(temp.to_owned());
    }
    fn add_ordeal<'a>(&mut self, puzzle: &Puzzle, top: &Source, i: usize, j: usize, new_instructions: impl DoubleEndedIterator<Item=&'a Instruction>, state: &State) {
//        let last_pointer = self.pointer;
        for instruction in new_instructions.rev() {
            self.push_branch(top, i, j, *instruction);
        }
//        self.data.get_mut(last_pointer..self.pointer).unwrap().sort_by_cached_key(|prog| {
//            let mut temp = state.clone();
//            while temp.running() {
//                temp.step(prog, puzzle);
//            }
//            return score_cmp(&temp, puzzle);
//        });
    }
}

const THREADS: usize = 16;

pub fn backtrack(puzzle: &Puzzle) -> Option<Source> {
//    let mut tested = HashSet::new();
//    for thread in 0..THREADS {
//        thread::spawn(move || {
    let mut stack: Stack = Stack { pointer: 0, data: [NOGRAM; BACKTRACK_STACK_SIZE] };
    stack.push(puzzle.empty_source());
//    stack.push(PUZZLE_1337_SOLUTION);
    let mut considered: u64 = 0;
    let mut executed: u64 = 0;
    let mut denies: u64 = 0;

    let mut duplicates: u64 = 0;

    let mut visited: HashSet<u64> = HashSet::new();
    while !stack.empty() {
        considered += 1;

        let top = stack.pop();
        if deny(puzzle, &top) {
            denies += 1;
            continue;
        }
        executed += 1;
        let mut preferred = [true; 5];
        for i in 0..top.0.len() {
            for j in i + 1..top.0.len() {
                if i != j && top.0[i] == top.0[j] {
                    preferred[j] = false;
                }
            }
        }
        let mut state = puzzle.initial_state();
        state.stack.push(F1);
        let mut branched = false;
        while state.running() {
            if state.steps > 64 && !visited.insert(state.get_hash()) {
                duplicates += 1;
                break;
            }
            let stack_top = state.stack.top().clone();
            if stack_top == NOP && !branched {
                let i = state.stack_frame().source_index();
                let j = state.instruction_number(puzzle);
                if top[i][j] == NOP {
                    branched = true;
                    stack.add_ordeal(puzzle, &top, i, j,
                                     puzzle.get_instruction_set(GRAY_COND, true).iter()
                                         .filter(|&ins| {
                                             !ins.is_function() || preferred[ins.source_index()]
                                         }).chain(
                                         state.current_tile().get_probes(puzzle.get_color_mask()).iter()
                                     ), &state);
                }
                executed -= 1;
                break;
            } else if stack_top.is_probe() {
                let i = state.stack_frame().source_index();
                let j = state.instruction_number(puzzle);
                if state.current_tile().clone().executes(stack_top) {
                    stack.add_ordeal(puzzle, &top, i, j,
                                     puzzle.get_instruction_set(state.current_tile().to_condition(), false).iter(), &state);
                    executed -= 1;
                    break;
                }
            }
            state.step(&top, &puzzle);
        }
        if state.stars == 0 {
            println!("done! considered: {}, executed: {}, denies: {}, duplicates: {} and {}", considered, executed, denies, duplicates, stack);
            return Some(top);
        }
        if considered % 10000000 == 0 {
            println!("considered: {}, executed: {}, denies: {}, duplicates: {} and {}\n then {}", considered, executed, denies, duplicates, stack, state);
        }
    }
    return None;
//        });
//    }

//    return None;
}

fn deny(puzzle: &Puzzle, program: &Source) -> bool {
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
            denied |= banned_pair(a, b);
            if a.is_color(GRAY_COND) && a.is_mark() {
                if !b.is_color(GRAY_COND) {
                    return true;
                }
            }
            if a.is_function() && invoked[a.source_index()] == 1 && conditioned[a.source_index()] == GRAY_COND {
                denied |= banned_pair(program.0[a.source_index()][puzzle.functions[a.source_index()] - 1], b);
            }
            if b.is_function() && invoked[b.source_index()] == 1 && conditioned[b.source_index()] == GRAY_COND {
                denied |= banned_pair(a, program.0[b.source_index()][0]);
            }
        }
        if conditioned[method] != NOP && conditioned[method] != HALT {
//            denied |= !program.0[method][0].is_color(GRAY_COND);
            for i in 0..puzzle.functions[method] {
                denied |= !program.0[method][i].is_color(GRAY_COND);
                if !program.0[method][i].is_turn() {
                    break;
                }
            }
        }
        for i in 2..puzzle.functions[method] {
            let a = program.0[method][i - 2];
            let b = program.0[method][i - 1];
            let c = program.0[method][i];
            if a.get_condition() == b.get_condition() && a.get_condition() == c.get_condition() {
                denied |= a.is_turn() && a == b && a == c;
            }
        }
    }
    return denied;
}

fn banned_pair(a: Instruction, b: Instruction) -> bool {
    let mut banned = false;
    if a.get_condition() == b.get_condition() {
        banned |= a.is_order_invariant() && b.is_order_invariant() && a > b;
        banned |= a.is_instruction(LEFT) && b.is_instruction(RIGHT);
        banned |= a.is_mark() && b.is_mark();
    }
    if a.is_turn() && b.is_turn() {
        banned |= a > b; // only let a series of turns have one color order
    }
    return banned;
}

fn reject(state: &State, puzzle: &Puzzle, program: &Source) -> bool {
    if state.stack.len() > 1 {
        let conditions = state.stack[0].get_condition() == state.stack[1].get_condition();
        let wiggles = (state.stack[0].get_instruction() == LEFT && state.stack[1].get_instruction() == RIGHT) || (state.stack[0].get_instruction() == RIGHT && state.stack[1].get_instruction() == LEFT);
        let marks = state.stack[0].is_mark() && state.stack[1].is_mark();
        return conditions && (wiggles || marks);
    } else {
        return false;
    }
}

pub(crate) fn accept(puzzle: &Puzzle, source: &Source) -> bool {
    return puzzle.execute(&source, false, won);
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
