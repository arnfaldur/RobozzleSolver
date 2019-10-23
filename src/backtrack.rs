use crate::game::{Instruction, Source, Puzzle, State};
use crate::constants::*;
use std::fmt::{Display, Formatter, Error};
use std::collections::HashSet;
use crate::carlo::score;
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
    fn push_branch(&mut self, source: Source, method: usize, ip: usize, instruction: Instruction) {
        let mut temp = source.clone();
        temp[method][ip] = instruction;
        self.push(temp.to_owned());
    }
}

const THREADS: usize = 16;

pub fn backtrack(puzzle: &Puzzle) -> Option<Source> {
//    let mut tested = HashSet::new();
//    for thread in 0..THREADS {
//        thread::spawn(move || {
    let mut stack: Stack = Stack { pointer: 0, data: [NOGRAM; BACKTRACK_STACK_SIZE] };
    stack.push(puzzle.empty_source());
    let mut considered: u64 = 0;
    let mut analyzed: u64 = 0;
    let mut rejects: u64 = 0;
    let mut denies: u64 = 0;

    let mut duplicates: u64 = 0;

    let mut visited: HashSet<u64> = HashSet::new();
    while !stack.empty() {
        considered += 1;
        analyzed += 1;

        let mut top = stack.pop();
        if deny(puzzle, &top) {
            denies += 1;
            analyzed -= 1;
            continue;
        }
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
//        visited.clear();
        while state.running() {
            if state.steps > 64 && !visited.insert(state.get_hash()) {
                duplicates += 1;
                break;
            }
            if reject(&state, puzzle, &top) {
                rejects += 1;
                analyzed -= 1;
                break;
            }
            let stack_top = state.stack.top().clone();
            if stack_top == NOP {
                let mut branched = false;
                let i = state.stack_frame().source_index();
                let j = state.instruction_number(puzzle);
                if top[i][j] == NOP {
                    for instruction in
                        puzzle.get_instruction_set(GRAY_COND, true).iter()
                            .filter(|&ins| {
                                !ins.is_function() || preferred[ins.source_index()]
                            }).chain(
                            state.current_tile().get_probes(puzzle.get_color_mask()).iter()
                        ).rev() {
                        stack.push_branch(top, i, j, *instruction);
                    }
                }
                break;
            } else if stack_top.is_probe() {
                let i = state.stack_frame().source_index();
                let j = state.instruction_number(puzzle);
                if state.current_tile().executes(stack_top) {
                    for instruction in puzzle.get_instruction_set(state.current_tile().to_condition(), false).iter().rev() {
                        stack.push_branch(top, i, j, *instruction);
                    }
                    break;
                }
            }
            state.step(&top, &puzzle);
        }
        if state.stars == 0 {
            println!("done! considered: {}, analyzed: {}, rejected: {}, duplicates: {} and {}", considered, analyzed, rejects, duplicates, stack);
            return Some(top);
        }
        if considered % 1000000 == 0 {
            println!("considered: {}, analyzed: {}, rejected: {}, duplicates: {} and {}\n then {}", considered, analyzed, rejects, duplicates, stack, state);
        }
    }
    return None;
//        });
//    }

//    return None;
}

fn deny(puzzle: &Puzzle, program: &Source) -> bool {
    let mut denied = false;
    for method in 0..5 {
        for i in 1..puzzle.functions[method] {
            let a = program.0[method][i - 1];
            let b = program.0[method][i];
            if a.get_condition() == b.get_condition() {
                if a.is_order_invariant() && b.is_order_invariant() && a > b {
                    return true;
                }
                if a.is_instruction(LEFT) && b.is_instruction(RIGHT) {
                    return true;
                }
                if a.is_mark() && b.is_mark() {
                    return true;
                }
            }
        }
    }
    return false;
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
    return puzzle.execute(&source, |state, _| state.stars == 0);
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
