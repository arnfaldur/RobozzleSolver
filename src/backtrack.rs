use crate::game::{Source, Puzzle};
use crate::constants::*;
use std::fmt::{Display, Formatter, Error};
use std::collections::HashSet;
use crate::carlo::score;

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
//    let mut tested = HashSet::new();

    let mut stack: Stack = Stack { pointer: 0, data: [NOGRAM; BACKTRACK_STACK_SIZE] };
    stack.push(puzzle.empty_source());
    let mut considered: u64 = 0;
    let mut analyzed: u64 = 0;
    let mut rejects: u64 = 0;

    let mut duplicates: u64 = 0;

    while !stack.empty() {
        considered += 1;
        analyzed += 1;

        let top = stack.pop();
//        if tested.contains(&top) {
////            println!("guilty: {}", top);
//            duplicates += 1;
//            continue;
//        }
//        tested.insert(top);

        let mut state = puzzle.initial_state();
        state.stack.push(F1);
//        let mut loosened = [[false; 10]; 5];
//        let mut loosened = false;
        let mut activated = [[false; 10]; 5];
        while state.running() {
            // test if program takes pointless turns
            if state.stack.len() > 1
                && state.stack[0].get_condition() == state.stack[1].get_condition()
                && ((state.stack[0].get_instruction() == LEFT && state.stack[1].get_instruction() == RIGHT)
                || (state.stack[0].get_instruction() == RIGHT && state.stack[1].get_instruction() == LEFT)) {
                let mut new_rejects: u64 = 1;
                for method in top.0.iter() {
                    for instruction in method {
                        if *instruction == HALT { break; }
                        if *instruction == NOP { new_rejects *= (puzzle.get_instruction_set(INS_COLOR_MASK, true).len() - 1) as u64; }
                    }
                }
                rejects += new_rejects;
                analyzed -= 1;
                break;
            }
            let stack_top = state.stack.top().clone();
            if stack_top == NOP {
                let i = state.stack_frame().source_index();
                for j in 0..puzzle.functions[i] {
                    if top[i][j] == HALT { break; }
                    if top[i][j] == NOP {
                        for instruction in
                            [HALT].iter().chain(
                                puzzle.get_instruction_set(
                                    GRAY_COND,
                                    true).iter()
                            ).chain(
                                state.current_tile().get_probes(
                                    puzzle.get_color_mask()).iter()
                            ).rev() {
                            let mut temp = top.clone();
                            temp[i][j] = *instruction;
                            stack.push(temp.to_owned());
                        }
                        break;
                    }
                }
                break;
            } else if stack_top.is_probe() {
                let i = state.stack_frame().source_index();
                let j = state.instruction_number(puzzle);
                if state.current_tile().executes(stack_top) {
                    for instruction in puzzle.get_instruction_set(
                        state.current_tile().to_condition(),
                        false).iter().rev() {
                        let mut temp = top.clone();
                        temp[i][j] = *instruction;
                        stack.push(temp.to_owned());
                    }
                    break;
                } else {
                    activated[i][j] = true;
                }
            }
            state.step(&top, &puzzle);
        }
        if state.stars == 0 {
            println!("done! considered: {}, analyzed: {}, rejected: {}, duplicates: {} and {}", considered, analyzed, rejects, duplicates, stack);
            return Some(top);
        }
        if considered % 100000 == 0 {
            println!("considered: {}, analyzed: {}, rejected: {}, duplicates: {} and {}", considered, analyzed, rejects, duplicates, stack);
        }
    }
    return None;
}

//fn reject(puzzle: &Puzzle, program: &Source) -> bool {
////    let mut used = [true, false, false, false, false, ];
////    print!("{}\nscore: {}", state, score(&state));
//    let mut moves = false;
//    for met in 0..5 {
//        for ins in program[met].iter() {
//            moves |= ins.get_instruction() == FORWARD;
//        }
//    }
//    return false;
//}

pub(crate) fn accept(puzzle: &Puzzle, source: &Source) -> bool {
    return puzzle.execute(&source, |state, _| state.stars == 0);
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Stack: ({},\n", self.pointer)?;
//        let mut count = 0;
        for i in (0..self.pointer).rev() {
            write!(f, "{},\n", self.data[i])?;
//            count += 1;
//            if count == 20 {
//                write!(f, "...")?;
//                break;
//            }
        }
        write!(f, ")")
//        write!(f, "{}]", self.data[self.data.len() - 1])
    }
}
