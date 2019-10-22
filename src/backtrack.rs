use crate::game::{Source, Puzzle};
use crate::constants::*;
use std::fmt::{Display, Formatter, Error};
use std::hash::Hash;
use crate::game::Direction::Right;

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
}

pub fn backtrack(puzzle: &Puzzle) -> Option<Source> {
    let mut stack: Stack = Stack { pointer: 0, data: [NOGRAM; BACKTRACK_STACK_SIZE] };
    stack.push(puzzle.empty_source());
    let mut boi: u64 = 0;
    let mut rejects: u64 = 0;
    while !stack.empty() {
        boi += 1;
        let top = stack.pop();
//        if reject(puzzle, &top) {
//            rejects += 1;
//            continue;
//        }

        let mut state = puzzle.initial_state();
        state.stack.push(F1);
        let mut branched = false;
        while state.running() {
            if state.stack.len() > 1
                && state.stack[0].get_condition() == state.stack[1].get_condition()
                && ((state.stack[0].get_instruction() == LEFT && state.stack[1].get_instruction() == RIGHT)
                || (state.stack[0].get_instruction() == RIGHT && state.stack[1].get_instruction() == LEFT)) {
                let mut new_rejects = puzzle.get_instruction_set().len();
                rejects += 1;
                break;
            }
            if state.stack.top() == NOP && !branched {
                let i = state.stack_frame().source_index();
                for j in 0..puzzle.functions[i] {
                    if top[i][j] == HALT { break; }
                    if top[i][j] == NOP {
                        branched = true;
                        for instruction in puzzle.get_instruction_set().iter().rev() {
                            let mut temp = top.clone();
                            temp[i][j] = *instruction;
                            stack.push(temp.to_owned());
                        }
                        break;
                    }
                }
            }
            state.step(&top, &puzzle);
        }
        if state.stars == 0 {
            println!("n! boi: {}, rej: {} and {}", boi, rejects, stack);
            return Some(top);
        }
//        println!("len: {}, ", stack.len());
        if boi % 100000 == 0 {
            println!("hey man! boi: {}, rej: {} and {}", boi, rejects, stack);
        }
//        if boi > 100 { break; }
    }
    return None;
}

fn reject(puzzle: &Puzzle, program: &Source) -> bool {
//    let mut used = [true, false, false, false, false, ];
//    print!("{}\nscore: {}", state, score(&state));
    let mut moves = false;
    for met in 0..5 {
        for ins in program[met].iter() {
            moves |= ins.get_instruction() == FORWARD;
        }
    }
    return false;
}

pub(crate) fn accept(puzzle: &Puzzle, source: &Source) -> bool {
    return puzzle.execute(&source, |state, _| state.stars == 0);
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Stack: ({},\n", self.pointer)?;
        let mut count = 0;
        for i in (0..self.pointer).rev() {
            write!(f, "{},\n", self.data[i])?;
            count += 1;
            if count == 20 {
                write!(f, "...")?;
                break;
            }
        }
        write!(f, ")")
        //        write!(f, "{}]", self.data[self.data.len() - 1])
    }
}
