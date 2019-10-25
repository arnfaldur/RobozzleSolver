use crate::game::*;
use crate::constants::*;
use crate::carlo::{score_cmp, carlo};
use crate::game::won;
use std::fmt::{Display, Formatter, Error};
use std::collections::{HashSet};
use once_cell::sync::OnceCell;

const BACKTRACK_STACK_SIZE: usize = 2200; // 44 *

static REJECTS_2: OnceCell<HashSet<[Ins; 2]>> = OnceCell::new();
static REJECTS_3: OnceCell<HashSet<[Ins; 3]>> = OnceCell::new();

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
    fn add_ordeal<'a>(&mut self, puzzle: &Puzzle, source: &Source, i: usize, j: usize, new_instructions: impl DoubleEndedIterator<Item=&'a Ins>, state: &State) -> u64 {
//        let last_pointer = self.pointer;
        let mut denies = 0;
        for &instruction in new_instructions.rev() {
            let mut temp = source.clone();
            temp[i][j] = instruction;
            if (j > 0 && banned_pair(puzzle, temp[i][j - 1], temp[i][j], false)) || (j > 1 && banned_trio(puzzle, temp[i][j], temp[i][j - 1], temp[i][j], false)) {
                denies += 1;
                continue;
            }
            self.push(temp.to_owned());
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
//    let rejects_2 = get_rejects_2();

//    let mut tested = HashSet::new();
//    for thread in 0..THREADS {
//        thread::spawn(move || {
    let mut stack: Stack = Stack { pointer: 0, data: [NOGRAM; BACKTRACK_STACK_SIZE] };
    stack.push(puzzle.empty_source());
//    stack.push(PUZZLE_1337_SOLUTION);
    let mut considered: u64 = 0;
    let mut executed: u64 = 0;
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
            if stack_top == NOP {
                let i = state.stack_frame().source_index();
                let j = state.instruction_number(puzzle);
                if top[i][j] == NOP {
//                    branched = true;
                    let mut temp = top.clone();
                    for k in j..puzzle.functions[i] {
                        temp[i][k] = HALT;
                    }
                    stack.push(temp);
                    snips += stack.add_ordeal(puzzle, &top, i, j,
                                     puzzle.get_instruction_set(GRAY_COND, true).iter()
                                         .filter(|&ins| {
                                             !ins.is_function() || preferred[ins.source_index()]
                                         }).chain(
                                         state.current_tile().get_probes(puzzle.get_color_mask()).iter()
                                     ), &state);
                    executed -= 1;
                }
                break;
            } else if stack_top.is_probe() {
                let i = state.stack_frame().source_index();
                let j = state.instruction_number(puzzle);
                if state.current_tile().clone().executes(stack_top) {
                    snips += stack.add_ordeal(puzzle, &top, i, j,
                                     puzzle.get_instruction_set(state.current_tile().to_condition(), false).iter(), &state);
                    executed -= 1;
                    break;
                }
            }
            state.step(&top, &puzzle);
        }
        if state.stars == 0 {
            println!("done! considered: {}, executed: {}, denies: {}, snips: {}, duplicates: {} and {}", considered, executed, denies, snips, duplicates, stack);
            return Some(top);
        }
        if considered % 10000000 == 0 {
            println!("considered: {}, executed: {}, denies: {}, snips: {}, duplicates: {} and {}\n then {}", considered, executed, denies, snips, duplicates, stack, state);
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
            if b == HALT || b == NOP { break; }
//            denied |= banned_pair(puzzle, a, b, show);
//            if show && denied { println!("de1"); }
            if a.is_function() && invoked[a.source_index()] == 1 && conditioned[a.source_index()] == GRAY_COND {
                denied |= banned_pair(puzzle, program.0[a.source_index()][puzzle.functions[a.source_index()] - 1], b, show);
                if show && denied { println!("de3"); }
            }
            if b.is_function() && invoked[b.source_index()] == 1 && conditioned[b.source_index()] == GRAY_COND {
                denied |= banned_pair(puzzle, a, program.0[b.source_index()][0], show);
                if show && denied { println!("de5"); }
            }
        }
        if conditioned[method] != NOP && conditioned[method] != HALT {
            for i in 0..puzzle.functions[method] {
                denied |= !program.0[method][i].is_color(GRAY_COND);
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
////            denied |= banned_trio(puzzle, a, b, c, show);
////            if a.is_color(GRAY_COND) as usize + b.is_color(GRAY_COND) as usize + b.is_color(GRAY_COND) as usize == 2 {
////
////            }
//        }
//        if show && denied { println!("de11"); }
    }
    return denied;
}

fn banned_pair(puzzle: &Puzzle, a: Ins, b: Ins, show: bool) -> bool {
    let mut banned = false;
    if a.get_condition() == b.get_condition() {
        banned |= a.is_order_invariant() && b.is_order_invariant() && a > b;
        if show && banned { println!("conds1"); }
        banned |= a.is_turn() && b.is_instruction(RIGHT);
        if show && banned { println!("conds2"); }
        banned |= a.is_mark() && !a.is_color(GRAY_COND);
        if show && banned { println!("conds2"); }
    }
    if a.is_turn() && b.is_turn() {
        banned |= a > b; // only let a series of turns have one color order
        if show && banned { println!("turns"); }
    }
    if a.is_mark() && b.is_mark() {
        banned |= b.is_color(GRAY_COND);
        banned |= a.get_instruction() == b.get_instruction() && a > b;
        banned |= a.get_condition() == b.get_condition();
        if show && banned { println!("marks"); }
    }
    banned |= a.is_color(GRAY_COND) && a.is_mark() && !b.is_color(GRAY_COND);
    if (a.is_turn() && a.is_color(GRAY_COND) && b.is_mark()) || (a.is_mark() && b.is_turn() && b.is_color(GRAY_COND)) {
        banned |= a > b;
        if show && banned { println!("five"); }
    }
    if (puzzle.red as i32 + puzzle.green as i32 + puzzle.blue as i32) == 3 {
        banned |= a.is_color(GRAY_COND) && !b.is_color(GRAY_COND) && a.is_turn() && b.is_instruction(a.get_instruction().other_turn());
        if show && banned { println!("six"); }
    } else if puzzle.red as i32 + puzzle.green as i32 + puzzle.blue as i32 == 2 {
        banned |= a.is_color(GRAY_COND) && !b.is_color(GRAY_COND) && a.is_turn() && b.is_instruction(a.get_instruction().other_turn());
        if show && banned { println!("seven"); }
    }
    return banned || get_rejects_2().contains(&[a, b]);
}

fn banned_trio(puzzle: &Puzzle, a: Ins, b: Ins, c: Ins, show: bool) -> bool {
    let mut banned = false;
    if a.get_condition() == b.get_condition() && a.get_condition() == c.get_condition() {
        banned |= a.is_turn() && a == b && a == c;
    }
    if a.is_turn() && a.is_color(GRAY_COND) && b.is_mark() && c.is_turn() && c.is_color(GRAY_COND) {
        banned |= !a.is_instruction(LEFT) || !c.is_instruction(LEFT);
    }
    return banned || get_rejects_3().contains(&[a, b, c]);
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

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Stack: ({},\n", self.pointer)?;
        for i in (0..self.pointer).rev() {
            write!(f, "{},\n", self.data[i])?;
        }
        write!(f, ")")
    }
}

fn get_rejects_2() -> &'static HashSet<[Ins; 2]> {
    return REJECTS_2.get().unwrap_or_else(|| {
        init_rejects_2();
        return REJECTS_2.get().unwrap();
    });
}

fn get_rejects_3() -> &'static HashSet<[Ins; 3]> {
    return REJECTS_3.get().unwrap_or_else(|| {
        init_rejects_3();
        return REJECTS_3.get().unwrap();
    });
}

fn init_rejects_2() {
    REJECTS_2.set([
        [Ins(73), Ins(137)],
        [Ins(137), Ins(42)],
        [Ins(138), Ins(73)],
        [Ins(65), Ins(44)],
        [Ins(44), Ins(76)],
        [Ins(73), Ins(42)],
        [Ins(76), Ins(137)],
        [Ins(129), Ins(73)],
        [Ins(76), Ins(34)],
        [Ins(130), Ins(73)],
        [Ins(66), Ins(44)],
        [Ins(129), Ins(42)],
        [Ins(42), Ins(138)],
        [Ins(130), Ins(42)],
        [Ins(137), Ins(66)],
        [Ins(42), Ins(76)],
        [Ins(138), Ins(76)],
        [Ins(42), Ins(73)],
        [Ins(76), Ins(33)],
        [Ins(138), Ins(34)],
        [Ins(44), Ins(137)],
        [Ins(44), Ins(138)],
        [Ins(137), Ins(44)],
        [Ins(137), Ins(65)],
        [Ins(138), Ins(33)],
        [Ins(76), Ins(138)],
        [Ins(73), Ins(44)]
    ].iter().cloned().collect()).ok();
}

fn init_rejects_3() {
    REJECTS_3.set([
        [Ins(73), Ins(130), Ins(42)],
        [Ins(76), Ins(32), Ins(138)],
        [Ins(42), Ins(138), Ins(33)],
        [Ins(128), Ins(76), Ins(34)],
        [Ins(33), Ins(66), Ins(129)],
        [Ins(137), Ins(64), Ins(44)],
        [Ins(138), Ins(44), Ins(129)],
        [Ins(42), Ins(138), Ins(32)],
        [Ins(137), Ins(65), Ins(129)],
        [Ins(129), Ins(42), Ins(129)],
        [Ins(73), Ins(42), Ins(65)],
        [Ins(130), Ins(42), Ins(137)],
        [Ins(130), Ins(73), Ins(129)],
        [Ins(33), Ins(129), Ins(42)],
        [Ins(137), Ins(44), Ins(128)],
        [Ins(137), Ins(64), Ins(137)],
        [Ins(138), Ins(34), Ins(42)],
        [Ins(44), Ins(138), Ins(66)],
        [Ins(130), Ins(73), Ins(137)],
        [Ins(1), Ins(44), Ins(130)],
        [Ins(44), Ins(137), Ins(0)],
        [Ins(137), Ins(65), Ins(65)],
        [Ins(138), Ins(76), Ins(33)],
        [Ins(76), Ins(33), Ins(9)],
        [Ins(76), Ins(34), Ins(32)],
        [Ins(138), Ins(76), Ins(128)],
        [Ins(73), Ins(129), Ins(64)],
        [Ins(76), Ins(33), Ins(33)],
        [Ins(130), Ins(73), Ins(128)],
        [Ins(33), Ins(44), Ins(76)],
        [Ins(138), Ins(66), Ins(138)],
        [Ins(65), Ins(76), Ins(137)],
        [Ins(0), Ins(129), Ins(73)],
        [Ins(137), Ins(66), Ins(76)],
        [Ins(64), Ins(42), Ins(73)],
        [Ins(128), Ins(44), Ins(76)],
        [Ins(138), Ins(66), Ins(129)],
        [Ins(42), Ins(65), Ins(73)],
        [Ins(138), Ins(73), Ins(128)],
        [Ins(66), Ins(76), Ins(138)],
        [Ins(73), Ins(130), Ins(64)],
        [Ins(34), Ins(73), Ins(33)],
        [Ins(33), Ins(44), Ins(130)],
        [Ins(66), Ins(42), Ins(76)],
        [Ins(64), Ins(137), Ins(65)],
        [Ins(138), Ins(65), Ins(73)],
        [Ins(76), Ins(138), Ins(64)],
        [Ins(33), Ins(73), Ins(137)],
        [Ins(73), Ins(33), Ins(64)],
        [Ins(34), Ins(130), Ins(44)],
        [Ins(0), Ins(137), Ins(42)],
        [Ins(73), Ins(138), Ins(34)],
        [Ins(1), Ins(65), Ins(44)],
        [Ins(129), Ins(42), Ins(66)],
        [Ins(2), Ins(42), Ins(76)],
        [Ins(130), Ins(137), Ins(66)],
        [Ins(2), Ins(130), Ins(73)],
        [Ins(76), Ins(33), Ins(66)],
        [Ins(128), Ins(130), Ins(73)],
        [Ins(128), Ins(137), Ins(65)],
        [Ins(44), Ins(66), Ins(137)],
        [Ins(76), Ins(137), Ins(65)],
        [Ins(1), Ins(138), Ins(33)],
        [Ins(138), Ins(76), Ins(42)],
        [Ins(76), Ins(33), Ins(73)],
        [Ins(42), Ins(137), Ins(34)],
        [Ins(65), Ins(42), Ins(138)],
        [Ins(76), Ins(34), Ins(138)],
        [Ins(2), Ins(76), Ins(34)],
        [Ins(129), Ins(73), Ins(129)],
        [Ins(33), Ins(138), Ins(33)],
        [Ins(42), Ins(138), Ins(44)],
        [Ins(129), Ins(76), Ins(34)],
        [Ins(1), Ins(138), Ins(34)],
        [Ins(44), Ins(138), Ins(32)],
        [Ins(73), Ins(130), Ins(137)],
        [Ins(1), Ins(73), Ins(34)],
        [Ins(65), Ins(137), Ins(33)],
        [Ins(138), Ins(32), Ins(138)],
        [Ins(2), Ins(138), Ins(33)],
        [Ins(73), Ins(128), Ins(42)],
        [Ins(1), Ins(137), Ins(65)],
        [Ins(34), Ins(44), Ins(137)],
        [Ins(1), Ins(73), Ins(130)],
        [Ins(76), Ins(129), Ins(10)],
        [Ins(32), Ins(137), Ins(65)],
        [Ins(32), Ins(138), Ins(34)],
        [Ins(138), Ins(33), Ins(44)],
        [Ins(73), Ins(128), Ins(9)],
        [Ins(73), Ins(129), Ins(76)],
        [Ins(34), Ins(129), Ins(73)],
        [Ins(73), Ins(128), Ins(73)],
        [Ins(129), Ins(76), Ins(137)],
        [Ins(137), Ins(34), Ins(137)],
        [Ins(73), Ins(138), Ins(33)],
        [Ins(138), Ins(66), Ins(73)],
        [Ins(2), Ins(137), Ins(44)],
        [Ins(73), Ins(33), Ins(76)],
        [Ins(128), Ins(42), Ins(73)],
        [Ins(138), Ins(66), Ins(12)],
        [Ins(66), Ins(76), Ins(33)],
        [Ins(2), Ins(66), Ins(130)],
        [Ins(32), Ins(76), Ins(33)],
        [Ins(137), Ins(76), Ins(138)],
        [Ins(33), Ins(129), Ins(73)],
        [Ins(129), Ins(42), Ins(73)],
        [Ins(64), Ins(76), Ins(33)],
        [Ins(42), Ins(73), Ins(0)],
        [Ins(2), Ins(34), Ins(66)],
        [Ins(66), Ins(44), Ins(129)],
        [Ins(130), Ins(42), Ins(128)],
        [Ins(138), Ins(44), Ins(138)],
        [Ins(34), Ins(138), Ins(76)],
        [Ins(65), Ins(129), Ins(76)],
        [Ins(137), Ins(34), Ins(129)],
        [Ins(138), Ins(33), Ins(129)],
        [Ins(1), Ins(44), Ins(137)],
        [Ins(42), Ins(138), Ins(73)],
        [Ins(130), Ins(42), Ins(0)],
        [Ins(1), Ins(44), Ins(138)],
        [Ins(44), Ins(76), Ins(0)],
        [Ins(0), Ins(137), Ins(66)],
        [Ins(44), Ins(76), Ins(33)],
        [Ins(76), Ins(34), Ins(76)],
        [Ins(44), Ins(137), Ins(42)],
        [Ins(73), Ins(130), Ins(9)],
        [Ins(64), Ins(138), Ins(76)],
        [Ins(76), Ins(33), Ins(129)],
        [Ins(32), Ins(73), Ins(44)],
        [Ins(66), Ins(73), Ins(137)],
        [Ins(42), Ins(66), Ins(10)],
        [Ins(76), Ins(129), Ins(76)],
        [Ins(137), Ins(34), Ins(73)],
        [Ins(73), Ins(44), Ins(65)],
        [Ins(32), Ins(42), Ins(73)],
        [Ins(137), Ins(44), Ins(64)],
        [Ins(42), Ins(66), Ins(42)],
        [Ins(2), Ins(73), Ins(137)],
        [Ins(128), Ins(42), Ins(138)],
        [Ins(76), Ins(129), Ins(44)],
        [Ins(138), Ins(65), Ins(10)],
        [Ins(33), Ins(65), Ins(65)],
        [Ins(44), Ins(73), Ins(44)],
        [Ins(65), Ins(73), Ins(137)],
        [Ins(73), Ins(34), Ins(76)],
        [Ins(66), Ins(137), Ins(42)],
        [Ins(33), Ins(33), Ins(66)],
        [Ins(2), Ins(130), Ins(42)],
        [Ins(34), Ins(66), Ins(42)],
        [Ins(73), Ins(44), Ins(64)],
        [Ins(44), Ins(65), Ins(32)],
        [Ins(66), Ins(44), Ins(130)],
        [Ins(64), Ins(73), Ins(42)],
        [Ins(129), Ins(42), Ins(138)],
        [Ins(66), Ins(44), Ins(73)],
        [Ins(73), Ins(33), Ins(65)],
        [Ins(137), Ins(64), Ins(9)],
        [Ins(42), Ins(76), Ins(138)],
        [Ins(42), Ins(138), Ins(34)],
        [Ins(76), Ins(137), Ins(33)],
        [Ins(64), Ins(44), Ins(137)],
        [Ins(64), Ins(76), Ins(137)],
        [Ins(1), Ins(76), Ins(33)],
        [Ins(42), Ins(137), Ins(33)],
        [Ins(44), Ins(138), Ins(65)],
        [Ins(73), Ins(33), Ins(9)],
        [Ins(65), Ins(138), Ins(66)],
        [Ins(137), Ins(66), Ins(42)],
        [Ins(66), Ins(42), Ins(138)],
        [Ins(2), Ins(73), Ins(42)],
        [Ins(137), Ins(64), Ins(10)],
        [Ins(137), Ins(42), Ins(130)],
        [Ins(129), Ins(138), Ins(73)],
        [Ins(73), Ins(137), Ins(64)],
        [Ins(137), Ins(42), Ins(73)],
        [Ins(137), Ins(34), Ins(9)],
        [Ins(138), Ins(65), Ins(130)],
        [Ins(44), Ins(66), Ins(12)],
        [Ins(32), Ins(76), Ins(34)],
        [Ins(42), Ins(73), Ins(138)],
        [Ins(34), Ins(73), Ins(44)],
        [Ins(76), Ins(34), Ins(130)],
        [Ins(42), Ins(66), Ins(130)],
        [Ins(44), Ins(130), Ins(76)],
        [Ins(128), Ins(138), Ins(33)],
        [Ins(32), Ins(65), Ins(44)],
        [Ins(65), Ins(138), Ins(34)],
        [Ins(0), Ins(44), Ins(137)],
        [Ins(1), Ins(73), Ins(44)],
        [Ins(44), Ins(76), Ins(129)],
        [Ins(42), Ins(76), Ins(33)],
        [Ins(44), Ins(66), Ins(9)],
        [Ins(73), Ins(34), Ins(137)],
        [Ins(130), Ins(138), Ins(33)],
        [Ins(137), Ins(76), Ins(129)],
        [Ins(138), Ins(65), Ins(9)],
        [Ins(130), Ins(138), Ins(65)],
        [Ins(138), Ins(66), Ins(10)],
        [Ins(0), Ins(138), Ins(73)],
        [Ins(1), Ins(42), Ins(130)],
        [Ins(42), Ins(129), Ins(138)],
        [Ins(129), Ins(73), Ins(130)],
        [Ins(76), Ins(129), Ins(73)],
        [Ins(130), Ins(42), Ins(130)],
        [Ins(128), Ins(76), Ins(138)],
        [Ins(44), Ins(138), Ins(76)],
        [Ins(1), Ins(76), Ins(137)],
        [Ins(73), Ins(129), Ins(44)],
        [Ins(1), Ins(73), Ins(137)],
        [Ins(65), Ins(65), Ins(44)],
        [Ins(138), Ins(66), Ins(76)],
        [Ins(64), Ins(129), Ins(73)],
        [Ins(44), Ins(129), Ins(10)],
        [Ins(42), Ins(129), Ins(42)],
        [Ins(65), Ins(44), Ins(130)],
        [Ins(137), Ins(34), Ins(66)],
        [Ins(137), Ins(34), Ins(128)],
        [Ins(1), Ins(129), Ins(73)],
        [Ins(73), Ins(33), Ins(12)],
        [Ins(138), Ins(34), Ins(64)],
        [Ins(137), Ins(42), Ins(64)],
        [Ins(33), Ins(65), Ins(42)],
        [Ins(138), Ins(65), Ins(128)],
        [Ins(42), Ins(138), Ins(66)],
        [Ins(65), Ins(138), Ins(73)],
        [Ins(137), Ins(44), Ins(76)],
        [Ins(138), Ins(32), Ins(12)],
        [Ins(138), Ins(33), Ins(0)],
        [Ins(42), Ins(66), Ins(12)],
        [Ins(73), Ins(129), Ins(137)],
        [Ins(44), Ins(66), Ins(10)],
        [Ins(128), Ins(76), Ins(137)],
        [Ins(137), Ins(33), Ins(73)],
        [Ins(76), Ins(138), Ins(32)],
        [Ins(76), Ins(33), Ins(137)],
        [Ins(65), Ins(137), Ins(42)],
        [Ins(42), Ins(138), Ins(76)],
        [Ins(137), Ins(65), Ins(76)],
        [Ins(42), Ins(129), Ins(76)],
        [Ins(44), Ins(65), Ins(12)],
        [Ins(44), Ins(138), Ins(44)],
        [Ins(76), Ins(34), Ins(0)],
        [Ins(44), Ins(65), Ins(10)],
        [Ins(76), Ins(34), Ins(44)],
        [Ins(76), Ins(129), Ins(64)],
        [Ins(129), Ins(42), Ins(137)],
        [Ins(42), Ins(65), Ins(12)],
        [Ins(33), Ins(73), Ins(42)],
        [Ins(42), Ins(130), Ins(32)],
        [Ins(130), Ins(44), Ins(76)],
        [Ins(137), Ins(44), Ins(66)],
        [Ins(1), Ins(137), Ins(34)],
        [Ins(34), Ins(138), Ins(73)],
        [Ins(66), Ins(130), Ins(138)],
        [Ins(32), Ins(138), Ins(76)],
        [Ins(137), Ins(65), Ins(32)],
        [Ins(129), Ins(44), Ins(138)],
        [Ins(0), Ins(66), Ins(44)],
        [Ins(73), Ins(44), Ins(73)],
        [Ins(138), Ins(33), Ins(128)],
        [Ins(130), Ins(44), Ins(138)],
        [Ins(32), Ins(129), Ins(73)],
        [Ins(129), Ins(73), Ins(0)],
        [Ins(65), Ins(42), Ins(76)],
        [Ins(73), Ins(137), Ins(42)],
        [Ins(44), Ins(64), Ins(9)],
        [Ins(1), Ins(42), Ins(138)],
        [Ins(33), Ins(42), Ins(73)],
        [Ins(138), Ins(73), Ins(138)],
        [Ins(44), Ins(76), Ins(128)],
        [Ins(76), Ins(137), Ins(42)],
        [Ins(137), Ins(66), Ins(138)],
        [Ins(73), Ins(33), Ins(10)],
        [Ins(73), Ins(42), Ins(64)],
        [Ins(33), Ins(138), Ins(76)],
        [Ins(128), Ins(137), Ins(42)],
        [Ins(42), Ins(65), Ins(129)],
        [Ins(138), Ins(34), Ins(65)],
        [Ins(137), Ins(66), Ins(64)],
        [Ins(42), Ins(128), Ins(10)],
        [Ins(1), Ins(138), Ins(66)],
        [Ins(2), Ins(42), Ins(65)],
        [Ins(137), Ins(65), Ins(42)],
        [Ins(138), Ins(33), Ins(64)],
        [Ins(129), Ins(137), Ins(66)],
        [Ins(130), Ins(138), Ins(76)],
        [Ins(32), Ins(42), Ins(138)],
        [Ins(32), Ins(129), Ins(42)],
        [Ins(44), Ins(129), Ins(42)],
        [Ins(65), Ins(44), Ins(0)],
        [Ins(73), Ins(128), Ins(12)],
        [Ins(138), Ins(33), Ins(76)],
        [Ins(44), Ins(137), Ins(32)],
        [Ins(0), Ins(138), Ins(34)],
        [Ins(64), Ins(137), Ins(44)],
        [Ins(73), Ins(44), Ins(138)],
        [Ins(73), Ins(129), Ins(42)],
        [Ins(76), Ins(130), Ins(12)],
        [Ins(33), Ins(76), Ins(137)],
        [Ins(138), Ins(32), Ins(9)],
        [Ins(66), Ins(129), Ins(73)],
        [Ins(137), Ins(42), Ins(76)],
        [Ins(34), Ins(76), Ins(138)],
        [Ins(42), Ins(65), Ins(9)],
        [Ins(73), Ins(138), Ins(73)],
        [Ins(138), Ins(73), Ins(44)],
        [Ins(65), Ins(138), Ins(76)],
        [Ins(44), Ins(130), Ins(44)],
        [Ins(44), Ins(130), Ins(73)],
        [Ins(137), Ins(66), Ins(9)],
        [Ins(2), Ins(76), Ins(129)],
        [Ins(76), Ins(42), Ins(65)],
        [Ins(42), Ins(76), Ins(129)],
        [Ins(1), Ins(42), Ins(73)],
        [Ins(76), Ins(34), Ins(73)],
        [Ins(138), Ins(65), Ins(44)],
        [Ins(76), Ins(138), Ins(0)],
        [Ins(0), Ins(137), Ins(65)],
        [Ins(44), Ins(130), Ins(12)],
        [Ins(42), Ins(73), Ins(32)],
        [Ins(44), Ins(138), Ins(33)],
        [Ins(76), Ins(138), Ins(66)],
        [Ins(129), Ins(42), Ins(128)],
        [Ins(64), Ins(130), Ins(42)],
        [Ins(128), Ins(65), Ins(44)],
        [Ins(128), Ins(138), Ins(76)],
        [Ins(34), Ins(138), Ins(66)],
        [Ins(44), Ins(66), Ins(138)],
        [Ins(129), Ins(137), Ins(44)],
        [Ins(130), Ins(73), Ins(34)],
        [Ins(42), Ins(76), Ins(34)],
        [Ins(44), Ins(129), Ins(76)],
        [Ins(138), Ins(65), Ins(76)],
        [Ins(73), Ins(34), Ins(65)],
        [Ins(137), Ins(65), Ins(130)],
        [Ins(129), Ins(42), Ins(64)],
        [Ins(73), Ins(130), Ins(44)],
        [Ins(44), Ins(73), Ins(42)],
        [Ins(34), Ins(137), Ins(66)],
        [Ins(34), Ins(138), Ins(34)],
        [Ins(138), Ins(65), Ins(129)],
        [Ins(32), Ins(137), Ins(44)],
        [Ins(33), Ins(138), Ins(73)],
        [Ins(42), Ins(76), Ins(32)],
        [Ins(76), Ins(34), Ins(137)],
        [Ins(33), Ins(76), Ins(34)],
        [Ins(130), Ins(76), Ins(33)],
        [Ins(42), Ins(128), Ins(73)],
        [Ins(66), Ins(137), Ins(66)],
        [Ins(2), Ins(137), Ins(33)],
        [Ins(129), Ins(42), Ins(0)],
        [Ins(2), Ins(42), Ins(73)],
        [Ins(138), Ins(34), Ins(73)],
        [Ins(76), Ins(34), Ins(66)],
        [Ins(42), Ins(130), Ins(42)],
        [Ins(129), Ins(42), Ins(130)],
        [Ins(130), Ins(137), Ins(33)],
        [Ins(76), Ins(130), Ins(44)],
        [Ins(1), Ins(138), Ins(76)],
        [Ins(76), Ins(137), Ins(32)],
        [Ins(1), Ins(76), Ins(138)],
        [Ins(1), Ins(137), Ins(42)],
        [Ins(33), Ins(44), Ins(137)],
        [Ins(138), Ins(34), Ins(76)],
        [Ins(137), Ins(76), Ins(130)],
        [Ins(34), Ins(66), Ins(73)],
        [Ins(42), Ins(137), Ins(42)],
        [Ins(44), Ins(66), Ins(32)],
        [Ins(66), Ins(44), Ins(137)],
        [Ins(137), Ins(33), Ins(9)],
        [Ins(73), Ins(42), Ins(130)],
        [Ins(0), Ins(42), Ins(76)],
        [Ins(34), Ins(42), Ins(73)],
        [Ins(76), Ins(33), Ins(64)],
        [Ins(130), Ins(73), Ins(138)],
        [Ins(73), Ins(129), Ins(73)],
        [Ins(65), Ins(129), Ins(42)],
        [Ins(65), Ins(138), Ins(33)],
        [Ins(138), Ins(66), Ins(128)],
        [Ins(129), Ins(138), Ins(33)],
        [Ins(0), Ins(73), Ins(44)],
        [Ins(64), Ins(44), Ins(76)],
        [Ins(32), Ins(138), Ins(33)],
        [Ins(2), Ins(66), Ins(44)],
        [Ins(42), Ins(76), Ins(137)],
        [Ins(65), Ins(44), Ins(76)],
        [Ins(65), Ins(76), Ins(138)],
        [Ins(76), Ins(129), Ins(12)],
        [Ins(73), Ins(33), Ins(66)],
        [Ins(130), Ins(73), Ins(44)],
        [Ins(42), Ins(65), Ins(10)],
        [Ins(137), Ins(33), Ins(128)],
        [Ins(73), Ins(137), Ins(44)],
        [Ins(44), Ins(130), Ins(10)],
        [Ins(33), Ins(130), Ins(42)],
        [Ins(73), Ins(34), Ins(42)],
        [Ins(129), Ins(73), Ins(137)],
        [Ins(138), Ins(33), Ins(130)],
        [Ins(138), Ins(73), Ins(33)],
        [Ins(42), Ins(66), Ins(73)],
        [Ins(42), Ins(138), Ins(65)],
        [Ins(2), Ins(73), Ins(33)],
        [Ins(44), Ins(64), Ins(44)],
        [Ins(73), Ins(42), Ins(66)],
        [Ins(73), Ins(34), Ins(12)],
        [Ins(137), Ins(66), Ins(0)],
        [Ins(64), Ins(73), Ins(44)],
        [Ins(129), Ins(138), Ins(66)],
        [Ins(42), Ins(129), Ins(73)],
        [Ins(137), Ins(33), Ins(66)],
        [Ins(66), Ins(73), Ins(42)],
        [Ins(130), Ins(44), Ins(66)],
        [Ins(137), Ins(66), Ins(137)],
        [Ins(76), Ins(137), Ins(76)],
        [Ins(2), Ins(138), Ins(76)],
        [Ins(34), Ins(73), Ins(137)],
        [Ins(44), Ins(138), Ins(73)],
        [Ins(138), Ins(34), Ins(12)],
        [Ins(138), Ins(65), Ins(42)],
        [Ins(129), Ins(73), Ins(138)],
        [Ins(42), Ins(129), Ins(9)],
        [Ins(137), Ins(42), Ins(65)],
        [Ins(42), Ins(130), Ins(44)],
        [Ins(33), Ins(65), Ins(129)],
        [Ins(42), Ins(73), Ins(34)],
        [Ins(65), Ins(44), Ins(66)],
        [Ins(130), Ins(137), Ins(42)],
        [Ins(128), Ins(73), Ins(137)],
        [Ins(138), Ins(76), Ins(129)],
        [Ins(44), Ins(73), Ins(34)],
        [Ins(44), Ins(129), Ins(9)],
        [Ins(129), Ins(44), Ins(76)],
        [Ins(73), Ins(34), Ins(73)],
        [Ins(73), Ins(129), Ins(12)],
        [Ins(137), Ins(65), Ins(0)],
        [Ins(32), Ins(44), Ins(76)],
        [Ins(34), Ins(44), Ins(138)],
        [Ins(76), Ins(34), Ins(65)],
        [Ins(76), Ins(130), Ins(138)],
        [Ins(44), Ins(66), Ins(42)],
        [Ins(76), Ins(32), Ins(12)],
        [Ins(66), Ins(44), Ins(0)],
        [Ins(33), Ins(33), Ins(65)],
        [Ins(129), Ins(73), Ins(42)],
        [Ins(1), Ins(76), Ins(130)],
        [Ins(34), Ins(137), Ins(44)],
        [Ins(33), Ins(137), Ins(34)],
        [Ins(138), Ins(33), Ins(42)],
        [Ins(138), Ins(76), Ins(138)],
        [Ins(138), Ins(73), Ins(129)],
        [Ins(0), Ins(130), Ins(73)],
        [Ins(66), Ins(73), Ins(33)],
        [Ins(137), Ins(33), Ins(137)],
        [Ins(137), Ins(65), Ins(64)],
        [Ins(130), Ins(76), Ins(129)],
        [Ins(42), Ins(73), Ins(42)],
        [Ins(137), Ins(65), Ins(138)],
        [Ins(66), Ins(44), Ins(65)],
        [Ins(129), Ins(137), Ins(42)],
        [Ins(76), Ins(130), Ins(10)],
        [Ins(76), Ins(34), Ins(129)],
        [Ins(33), Ins(44), Ins(138)],
        [Ins(34), Ins(66), Ins(130)],
        [Ins(44), Ins(76), Ins(138)],
        [Ins(73), Ins(33), Ins(42)],
        [Ins(76), Ins(138), Ins(34)],
        [Ins(138), Ins(34), Ins(10)],
        [Ins(64), Ins(42), Ins(76)],
        [Ins(129), Ins(76), Ins(130)],
        [Ins(65), Ins(44), Ins(128)],
        [Ins(137), Ins(65), Ins(44)],
        [Ins(34), Ins(44), Ins(129)],
        [Ins(32), Ins(66), Ins(44)],
        [Ins(76), Ins(138), Ins(73)],
        [Ins(137), Ins(33), Ins(12)],
        [Ins(0), Ins(73), Ins(137)],
        [Ins(137), Ins(66), Ins(73)],
        [Ins(130), Ins(44), Ins(65)],
        [Ins(138), Ins(76), Ins(137)],
        [Ins(130), Ins(44), Ins(137)],
        [Ins(137), Ins(33), Ins(138)],
        [Ins(137), Ins(66), Ins(129)],
        [Ins(76), Ins(129), Ins(9)],
        [Ins(76), Ins(130), Ins(64)],
        [Ins(137), Ins(44), Ins(0)],
        [Ins(138), Ins(32), Ins(76)],
        [Ins(0), Ins(76), Ins(33)],
        [Ins(138), Ins(33), Ins(65)],
        [Ins(44), Ins(76), Ins(137)],
        [Ins(138), Ins(76), Ins(34)],
        [Ins(32), Ins(73), Ins(42)],
        [Ins(138), Ins(33), Ins(137)],
        [Ins(32), Ins(73), Ins(137)],
        [Ins(76), Ins(34), Ins(9)],
        [Ins(76), Ins(42), Ins(66)],
        [Ins(129), Ins(73), Ins(44)],
        [Ins(65), Ins(137), Ins(65)],
        [Ins(138), Ins(73), Ins(137)],
        [Ins(73), Ins(42), Ins(138)],
        [Ins(32), Ins(76), Ins(138)],
        [Ins(73), Ins(42), Ins(128)],
        [Ins(34), Ins(138), Ins(33)],
        [Ins(128), Ins(138), Ins(73)],
        [Ins(130), Ins(138), Ins(73)],
        [Ins(34), Ins(65), Ins(44)],
        [Ins(33), Ins(76), Ins(129)],
        [Ins(137), Ins(42), Ins(128)],
        [Ins(66), Ins(44), Ins(66)],
        [Ins(128), Ins(130), Ins(42)],
        [Ins(0), Ins(138), Ins(76)],
        [Ins(137), Ins(66), Ins(130)],
        [Ins(34), Ins(130), Ins(73)],
        [Ins(76), Ins(138), Ins(65)],
        [Ins(130), Ins(42), Ins(76)],
        [Ins(65), Ins(42), Ins(66)],
        [Ins(64), Ins(65), Ins(44)],
        [Ins(65), Ins(129), Ins(73)],
        [Ins(42), Ins(130), Ins(73)],
        [Ins(137), Ins(33), Ins(10)],
        [Ins(73), Ins(137), Ins(34)],
        [Ins(64), Ins(138), Ins(34)],
        [Ins(1), Ins(42), Ins(66)],
        [Ins(138), Ins(34), Ins(130)],
        [Ins(42), Ins(130), Ins(76)],
        [Ins(64), Ins(138), Ins(33)],
        [Ins(73), Ins(33), Ins(44)],
        [Ins(137), Ins(76), Ins(34)],
        [Ins(137), Ins(64), Ins(12)],
        [Ins(130), Ins(42), Ins(129)],
        [Ins(76), Ins(137), Ins(44)],
        [Ins(33), Ins(65), Ins(44)],
        [Ins(44), Ins(137), Ins(64)],
        [Ins(2), Ins(76), Ins(138)],
        [Ins(66), Ins(76), Ins(137)],
        [Ins(42), Ins(137), Ins(44)],
        [Ins(44), Ins(137), Ins(34)],
        [Ins(73), Ins(137), Ins(65)],
        [Ins(129), Ins(44), Ins(137)],
        [Ins(42), Ins(130), Ins(9)],
        [Ins(73), Ins(33), Ins(129)],
        [Ins(76), Ins(42), Ins(138)],
        [Ins(137), Ins(34), Ins(65)],
        [Ins(2), Ins(138), Ins(34)],
        [Ins(137), Ins(65), Ins(73)],
        [Ins(0), Ins(44), Ins(138)],
        [Ins(73), Ins(130), Ins(73)],
        [Ins(44), Ins(76), Ins(130)],
        [Ins(42), Ins(129), Ins(12)],
        [Ins(138), Ins(33), Ins(33)],
        [Ins(138), Ins(34), Ins(44)],
        [Ins(42), Ins(65), Ins(138)],
        [Ins(44), Ins(65), Ins(76)],
        [Ins(130), Ins(42), Ins(66)],
        [Ins(66), Ins(130), Ins(42)],
        [Ins(137), Ins(65), Ins(128)],
        [Ins(66), Ins(44), Ins(64)],
        [Ins(42), Ins(76), Ins(42)],
        [Ins(137), Ins(42), Ins(138)],
        [Ins(138), Ins(33), Ins(10)],
        [Ins(44), Ins(129), Ins(73)],
        [Ins(76), Ins(130), Ins(73)],
        [Ins(66), Ins(42), Ins(65)],
        [Ins(137), Ins(44), Ins(65)],
        [Ins(73), Ins(44), Ins(137)],
        [Ins(42), Ins(73), Ins(128)],
        [Ins(130), Ins(73), Ins(0)],
        [Ins(138), Ins(44), Ins(76)],
        [Ins(1), Ins(137), Ins(44)],
        [Ins(34), Ins(76), Ins(137)],
        [Ins(44), Ins(130), Ins(138)],
        [Ins(137), Ins(44), Ins(138)],
        [Ins(65), Ins(129), Ins(138)],
        [Ins(137), Ins(44), Ins(129)],
        [Ins(73), Ins(42), Ins(129)],
        [Ins(33), Ins(138), Ins(34)],
        [Ins(2), Ins(76), Ins(137)],
        [Ins(76), Ins(33), Ins(76)],
        [Ins(76), Ins(32), Ins(76)],
        [Ins(44), Ins(66), Ins(44)],
        [Ins(2), Ins(44), Ins(65)],
        [Ins(76), Ins(34), Ins(42)],
        [Ins(33), Ins(73), Ins(34)],
        [Ins(42), Ins(65), Ins(42)],
        [Ins(2), Ins(44), Ins(76)],
        [Ins(129), Ins(73), Ins(128)],
        [Ins(73), Ins(137), Ins(33)],
        [Ins(137), Ins(34), Ins(10)],
        [Ins(138), Ins(65), Ins(12)],
        [Ins(65), Ins(137), Ins(66)],
        [Ins(73), Ins(130), Ins(76)],
        [Ins(129), Ins(42), Ins(65)],
        [Ins(137), Ins(66), Ins(12)],
        [Ins(34), Ins(65), Ins(65)],
        [Ins(44), Ins(64), Ins(10)],
        [Ins(76), Ins(130), Ins(76)],
        [Ins(76), Ins(130), Ins(9)],
        [Ins(130), Ins(76), Ins(34)],
        [Ins(73), Ins(42), Ins(137)],
        [Ins(44), Ins(76), Ins(32)],
        [Ins(44), Ins(65), Ins(9)],
        [Ins(128), Ins(73), Ins(44)],
        [Ins(66), Ins(44), Ins(128)],
        [Ins(73), Ins(137), Ins(0)],
        [Ins(33), Ins(66), Ins(44)],
        [Ins(32), Ins(137), Ins(66)],
        [Ins(33), Ins(42), Ins(76)],
        [Ins(42), Ins(138), Ins(64)],
        [Ins(44), Ins(64), Ins(12)],
        [Ins(76), Ins(129), Ins(138)],
        [Ins(137), Ins(33), Ins(44)],
        [Ins(130), Ins(76), Ins(137)],
        [Ins(138), Ins(34), Ins(129)],
        [Ins(138), Ins(34), Ins(9)],
        [Ins(33), Ins(130), Ins(73)],
        [Ins(137), Ins(65), Ins(12)],
        [Ins(73), Ins(34), Ins(10)],
        [Ins(0), Ins(130), Ins(42)],
        [Ins(138), Ins(34), Ins(32)],
        [Ins(32), Ins(42), Ins(76)],
        [Ins(129), Ins(137), Ins(34)],
        [Ins(138), Ins(34), Ins(66)],
        [Ins(65), Ins(44), Ins(65)],
        [Ins(64), Ins(137), Ins(66)],
        [Ins(42), Ins(73), Ins(137)],
        [Ins(1), Ins(44), Ins(66)],
        [Ins(1), Ins(138), Ins(73)],
        [Ins(73), Ins(138), Ins(76)],
        [Ins(44), Ins(65), Ins(138)],
        [Ins(66), Ins(76), Ins(34)],
        [Ins(42), Ins(130), Ins(12)],
        [Ins(33), Ins(76), Ins(138)],
        [Ins(42), Ins(128), Ins(9)],
        [Ins(137), Ins(44), Ins(137)],
        [Ins(44), Ins(138), Ins(64)],
        [Ins(130), Ins(42), Ins(65)],
        [Ins(129), Ins(73), Ins(34)],
        [Ins(129), Ins(137), Ins(65)],
        [Ins(130), Ins(42), Ins(138)],
        [Ins(76), Ins(138), Ins(33)],
        [Ins(33), Ins(65), Ins(130)],
        [Ins(138), Ins(65), Ins(137)],
        [Ins(129), Ins(129), Ins(42)],
        [Ins(128), Ins(44), Ins(138)],
        [Ins(138), Ins(66), Ins(9)],
        [Ins(64), Ins(66), Ins(44)],
        [Ins(34), Ins(130), Ins(137)],
        [Ins(65), Ins(76), Ins(130)],
        [Ins(65), Ins(44), Ins(129)],
        [Ins(73), Ins(44), Ins(76)],
        [Ins(138), Ins(73), Ins(0)],
        [Ins(32), Ins(138), Ins(73)],
        [Ins(73), Ins(128), Ins(10)],
        [Ins(138), Ins(73), Ins(32)],
        [Ins(129), Ins(44), Ins(130)],
        [Ins(137), Ins(34), Ins(44)],
        [Ins(42), Ins(138), Ins(0)],
        [Ins(42), Ins(130), Ins(138)],
        [Ins(33), Ins(42), Ins(66)],
        [Ins(66), Ins(138), Ins(33)],
        [Ins(44), Ins(130), Ins(42)],
        [Ins(129), Ins(129), Ins(73)],
        [Ins(73), Ins(130), Ins(10)],
        [Ins(76), Ins(33), Ins(0)],
        [Ins(33), Ins(73), Ins(44)],
        [Ins(34), Ins(44), Ins(76)],
        [Ins(129), Ins(138), Ins(34)],
        [Ins(138), Ins(34), Ins(137)],
        [Ins(42), Ins(130), Ins(10)],
        [Ins(128), Ins(73), Ins(42)],
        [Ins(129), Ins(76), Ins(138)],
        [Ins(42), Ins(73), Ins(44)],
        [Ins(66), Ins(137), Ins(65)],
        [Ins(138), Ins(66), Ins(137)],
        [Ins(44), Ins(129), Ins(137)],
        [Ins(44), Ins(138), Ins(0)],
        [Ins(137), Ins(34), Ins(130)],
        [Ins(2), Ins(34), Ins(130)],
        [Ins(65), Ins(73), Ins(42)],
        [Ins(137), Ins(76), Ins(137)],
        [Ins(34), Ins(130), Ins(42)],
        [Ins(65), Ins(73), Ins(34)],
        [Ins(34), Ins(137), Ins(42)],
        [Ins(66), Ins(137), Ins(34)],
        [Ins(128), Ins(129), Ins(42)],
        [Ins(130), Ins(137), Ins(65)],
        [Ins(2), Ins(44), Ins(138)],
        [Ins(42), Ins(137), Ins(65)],
        [Ins(138), Ins(33), Ins(66)],
        [Ins(0), Ins(129), Ins(42)],
        [Ins(128), Ins(137), Ins(44)],
        [Ins(1), Ins(129), Ins(42)],
        [Ins(76), Ins(42), Ins(76)],
        [Ins(76), Ins(137), Ins(34)],
        [Ins(2), Ins(137), Ins(42)],
        [Ins(34), Ins(73), Ins(42)],
        [Ins(44), Ins(76), Ins(42)],
        [Ins(44), Ins(76), Ins(34)],
        [Ins(44), Ins(64), Ins(137)],
        [Ins(76), Ins(130), Ins(137)],
        [Ins(76), Ins(42), Ins(73)],
        [Ins(44), Ins(65), Ins(44)],
        [Ins(137), Ins(44), Ins(130)],
        [Ins(138), Ins(76), Ins(0)],
        [Ins(137), Ins(44), Ins(73)],
        [Ins(2), Ins(138), Ins(65)],
        [Ins(2), Ins(138), Ins(73)],
        [Ins(2), Ins(42), Ins(129)],
        [Ins(76), Ins(33), Ins(32)],
        [Ins(0), Ins(137), Ins(44)],
        [Ins(66), Ins(44), Ins(138)],
        [Ins(33), Ins(65), Ins(73)],
        [Ins(66), Ins(138), Ins(34)],
        [Ins(65), Ins(137), Ins(44)],
        [Ins(130), Ins(76), Ins(138)],
        [Ins(76), Ins(138), Ins(44)],
        [Ins(73), Ins(44), Ins(66)],
        [Ins(65), Ins(130), Ins(73)],
        [Ins(137), Ins(65), Ins(10)],
        [Ins(2), Ins(44), Ins(129)],
        [Ins(64), Ins(138), Ins(73)],
        [Ins(32), Ins(130), Ins(42)],
        [Ins(44), Ins(129), Ins(138)],
        [Ins(34), Ins(66), Ins(44)],
        [Ins(128), Ins(44), Ins(137)],
        [Ins(138), Ins(33), Ins(32)],
        [Ins(73), Ins(137), Ins(66)],
        [Ins(76), Ins(33), Ins(130)],
        [Ins(66), Ins(138), Ins(65)],
        [Ins(65), Ins(76), Ins(33)],
        [Ins(34), Ins(129), Ins(42)],
        [Ins(137), Ins(42), Ins(137)],
        [Ins(138), Ins(33), Ins(9)],
        [Ins(44), Ins(129), Ins(32)],
        [Ins(138), Ins(76), Ins(130)],
        [Ins(66), Ins(73), Ins(44)],
        [Ins(34), Ins(137), Ins(33)],
        [Ins(42), Ins(128), Ins(42)],
        [Ins(73), Ins(130), Ins(12)],
        [Ins(73), Ins(42), Ins(73)],
        [Ins(137), Ins(34), Ins(42)],
        [Ins(137), Ins(33), Ins(130)],
        [Ins(73), Ins(44), Ins(130)],
        [Ins(137), Ins(66), Ins(44)],
        [Ins(0), Ins(76), Ins(138)],
        [Ins(34), Ins(42), Ins(65)],
        [Ins(137), Ins(65), Ins(137)],
        [Ins(44), Ins(65), Ins(129)],
        [Ins(33), Ins(137), Ins(44)],
        [Ins(42), Ins(65), Ins(76)],
        [Ins(2), Ins(73), Ins(44)],
        [Ins(138), Ins(66), Ins(44)],
        [Ins(129), Ins(73), Ins(33)],
        [Ins(76), Ins(33), Ins(44)],
        [Ins(138), Ins(76), Ins(32)],
        [Ins(73), Ins(129), Ins(10)],
        [Ins(137), Ins(33), Ins(129)],
        [Ins(138), Ins(34), Ins(138)],
        [Ins(64), Ins(130), Ins(73)],
        [Ins(138), Ins(44), Ins(137)],
        [Ins(76), Ins(34), Ins(128)],
        [Ins(130), Ins(42), Ins(64)],
        [Ins(64), Ins(44), Ins(138)],
        [Ins(1), Ins(73), Ins(42)],
        [Ins(42), Ins(76), Ins(0)],
        [Ins(66), Ins(130), Ins(73)],
        [Ins(64), Ins(76), Ins(138)],
        [Ins(65), Ins(44), Ins(64)],
        [Ins(64), Ins(42), Ins(138)],
        [Ins(64), Ins(76), Ins(34)],
        [Ins(137), Ins(34), Ins(138)],
        [Ins(76), Ins(129), Ins(42)],
        [Ins(137), Ins(76), Ins(33)],
        [Ins(34), Ins(76), Ins(33)],
        [Ins(44), Ins(130), Ins(137)],
        [Ins(44), Ins(66), Ins(76)],
        [Ins(73), Ins(137), Ins(32)],
        [Ins(73), Ins(34), Ins(66)],
        [Ins(76), Ins(33), Ins(42)],
        [Ins(128), Ins(42), Ins(76)],
        [Ins(0), Ins(42), Ins(138)],
        [Ins(66), Ins(44), Ins(76)],
        [Ins(138), Ins(73), Ins(34)],
        [Ins(76), Ins(129), Ins(137)],
        [Ins(128), Ins(137), Ins(66)],
        [Ins(76), Ins(34), Ins(64)],
        [Ins(42), Ins(73), Ins(129)],
        [Ins(138), Ins(66), Ins(42)],
        [Ins(42), Ins(65), Ins(44)],
        [Ins(44), Ins(137), Ins(65)],
        [Ins(130), Ins(73), Ins(42)],
        [Ins(137), Ins(66), Ins(10)],
        [Ins(73), Ins(137), Ins(76)],
        [Ins(129), Ins(44), Ins(65)],
        [Ins(34), Ins(76), Ins(34)],
        [Ins(129), Ins(138), Ins(76)],
        [Ins(130), Ins(138), Ins(34)],
        [Ins(0), Ins(42), Ins(73)],
        [Ins(42), Ins(129), Ins(10)],
        [Ins(34), Ins(42), Ins(138)],
        [Ins(33), Ins(76), Ins(33)],
        [Ins(32), Ins(130), Ins(73)],
        [Ins(66), Ins(137), Ins(44)],
        [Ins(1), Ins(42), Ins(76)],
        [Ins(138), Ins(65), Ins(138)],
        [Ins(33), Ins(138), Ins(65)],
        [Ins(65), Ins(76), Ins(34)],
        [Ins(73), Ins(34), Ins(44)],
        [Ins(130), Ins(73), Ins(33)],
        [Ins(2), Ins(137), Ins(66)],
        [Ins(44), Ins(130), Ins(9)],
        [Ins(138), Ins(33), Ins(12)],
        [Ins(66), Ins(129), Ins(42)],
        [Ins(0), Ins(44), Ins(76)],
        [Ins(66), Ins(138), Ins(76)],
        [Ins(137), Ins(33), Ins(42)],
        [Ins(73), Ins(34), Ins(130)],
        [Ins(33), Ins(137), Ins(65)],
        [Ins(44), Ins(129), Ins(12)],
        [Ins(128), Ins(138), Ins(34)],
        [Ins(0), Ins(76), Ins(34)],
        [Ins(137), Ins(33), Ins(65)],
        [Ins(44), Ins(73), Ins(33)],
        [Ins(129), Ins(73), Ins(32)],
        [Ins(42), Ins(76), Ins(130)],
        [Ins(42), Ins(137), Ins(66)],
        [Ins(130), Ins(42), Ins(73)],
        [Ins(76), Ins(34), Ins(10)],
        [Ins(73), Ins(34), Ins(64)],
        [Ins(130), Ins(137), Ins(44)],
        [Ins(32), Ins(44), Ins(137)],
        [Ins(33), Ins(137), Ins(66)],
        [Ins(73), Ins(44), Ins(128)],
        [Ins(128), Ins(129), Ins(73)],
        [Ins(76), Ins(33), Ins(12)],
        [Ins(130), Ins(44), Ins(129)],
        [Ins(130), Ins(73), Ins(32)],
        [Ins(32), Ins(44), Ins(138)],
        [Ins(138), Ins(66), Ins(130)],
        [Ins(137), Ins(42), Ins(0)],
        [Ins(64), Ins(73), Ins(137)],
        [Ins(128), Ins(66), Ins(44)],
        [Ins(73), Ins(44), Ins(129)],
        [Ins(73), Ins(42), Ins(0)],
        [Ins(2), Ins(76), Ins(33)],
        [Ins(0), Ins(76), Ins(137)],
        [Ins(42), Ins(129), Ins(44)],
        [Ins(138), Ins(34), Ins(0)],
        [Ins(44), Ins(137), Ins(33)],
        [Ins(76), Ins(33), Ins(65)],
        [Ins(44), Ins(137), Ins(76)],
        [Ins(73), Ins(42), Ins(76)],
        [Ins(33), Ins(137), Ins(42)],
        [Ins(34), Ins(65), Ins(129)],
        [Ins(76), Ins(33), Ins(128)],
        [Ins(137), Ins(42), Ins(66)],
        [Ins(137), Ins(42), Ins(129)],
        [Ins(1), Ins(137), Ins(66)],
        [Ins(33), Ins(33), Ins(130)],
        [Ins(44), Ins(73), Ins(137)],
        [Ins(138), Ins(32), Ins(10)],
        [Ins(138), Ins(34), Ins(128)],
        [Ins(1), Ins(76), Ins(34)],
        [Ins(44), Ins(130), Ins(32)],
        [Ins(76), Ins(32), Ins(9)],
        [Ins(137), Ins(66), Ins(128)],
        [Ins(73), Ins(33), Ins(73)],
        [Ins(32), Ins(76), Ins(137)],
        [Ins(138), Ins(44), Ins(130)],
        [Ins(64), Ins(137), Ins(42)],
        [Ins(42), Ins(128), Ins(12)],
        [Ins(42), Ins(66), Ins(9)],
        [Ins(34), Ins(137), Ins(65)],
        [Ins(44), Ins(129), Ins(44)],
        [Ins(33), Ins(129), Ins(44)],
        [Ins(64), Ins(129), Ins(42)],
        [Ins(129), Ins(76), Ins(33)],
        [Ins(73), Ins(138), Ins(65)],
        [Ins(129), Ins(44), Ins(66)],
        [Ins(137), Ins(66), Ins(32)],
        [Ins(66), Ins(42), Ins(73)],
        [Ins(76), Ins(33), Ins(10)],
        [Ins(129), Ins(42), Ins(76)],
        [Ins(32), Ins(137), Ins(42)],
        [Ins(0), Ins(138), Ins(33)],
        [Ins(65), Ins(44), Ins(73)],
        [Ins(42), Ins(66), Ins(32)],
        [Ins(42), Ins(73), Ins(33)],
        [Ins(76), Ins(34), Ins(12)],
        [Ins(137), Ins(65), Ins(9)],
        [Ins(34), Ins(42), Ins(76)],
        [Ins(76), Ins(137), Ins(64)],
        [Ins(34), Ins(76), Ins(130)],
        [Ins(76), Ins(137), Ins(66)],
        [Ins(44), Ins(66), Ins(130)],
        [Ins(65), Ins(130), Ins(42)],
        [Ins(2), Ins(137), Ins(65)],
        [Ins(44), Ins(137), Ins(44)],
        [Ins(2), Ins(73), Ins(129)],
        [Ins(138), Ins(73), Ins(42)],
        [Ins(137), Ins(34), Ins(12)],
        [Ins(42), Ins(66), Ins(138)],
        [Ins(42), Ins(76), Ins(128)],
        [Ins(130), Ins(73), Ins(130)],
        [Ins(73), Ins(33), Ins(137)],
        [Ins(2), Ins(42), Ins(138)],
        [Ins(66), Ins(76), Ins(129)],
        [Ins(42), Ins(129), Ins(32)],
        [Ins(138), Ins(33), Ins(73)],
        [Ins(138), Ins(73), Ins(130)],
        [Ins(0), Ins(65), Ins(44)],
        [Ins(42), Ins(65), Ins(32)],
        [Ins(33), Ins(42), Ins(138)],
        [Ins(42), Ins(66), Ins(44)],
        [Ins(76), Ins(137), Ins(0)],
        [Ins(65), Ins(44), Ins(137)],
        [Ins(42), Ins(73), Ins(130)],
        [Ins(44), Ins(137), Ins(66)],
        [Ins(76), Ins(32), Ins(10)],
        [Ins(0), Ins(73), Ins(42)],
        [Ins(73), Ins(34), Ins(9)],
        [Ins(73), Ins(129), Ins(9)],
        [Ins(1), Ins(44), Ins(76)],
        [Ins(42), Ins(66), Ins(76)],
        [Ins(2), Ins(44), Ins(137)],
        [Ins(44), Ins(138), Ins(34)],
        [Ins(76), Ins(138), Ins(76)],
        [Ins(44), Ins(65), Ins(42)],
        [Ins(33), Ins(129), Ins(137)],
        [Ins(66), Ins(138), Ins(73)],
        [Ins(66), Ins(130), Ins(76)],
        [Ins(128), Ins(76), Ins(33)],
        [Ins(44), Ins(65), Ins(137)],
        [Ins(76), Ins(130), Ins(42)],
        [Ins(73), Ins(44), Ins(0)],
        [Ins(138), Ins(33), Ins(138)],
        [Ins(65), Ins(73), Ins(44)],
        [Ins(76), Ins(33), Ins(138)],
        [Ins(65), Ins(44), Ins(138)],
        [Ins(65), Ins(42), Ins(73)],
        [Ins(33), Ins(33), Ins(129)],
        [Ins(73), Ins(138), Ins(66)]
    ].iter().cloned().collect()).ok();
}