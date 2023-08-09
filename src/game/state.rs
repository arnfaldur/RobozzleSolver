use std::fmt::Debug;
use std::fmt::Error;
use std::intrinsics::unlikely;
use std::{
    collections::{hash_map::DefaultHasher, HashSet, VecDeque},
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

use super::board::Board;
use super::instructions::*;
use super::puzzle::Puzzle;
use super::Direction;
use super::Map;
use super::Source;
use super::Stack;
use super::Tile;
use super::TileType;
use crate::constants::*;

#[derive(Eq, PartialEq, Clone)]
pub struct State {
    pub(crate) steps: usize, // number of instructions executed
    pub(crate) stars: usize, // number of stars remaining
    pub stack: Stack,
    pub board: Board,
}

impl Default for State {
    fn default() -> Self {
        State {
            steps: 0,
            stars: 1,
            stack: Default::default(),
            board: Default::default(),
        }
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stars.hash(state);
        self.stack.hash(state);
        self.board.hash(state);
    }
}

impl State {
    pub fn initialize(&mut self, source: &Source, puzzle: &Puzzle) {
        self.invoke(source, puzzle.methods[F1.source_index()], F1.source_index());
    }
    pub fn current_tile(&self) -> &Tile {
        self.board.current_tile()
    }
    pub fn ins_pointer(&self) -> &InsPtr {
        let ins = self.stack.last();
        //        let ins = source[ins.get_method_index()][ins.get_ins_index()];
        return ins;
    }
    pub fn current_ins(&self, source: &Source) -> Ins {
        let ins = self.ins_pointer();
        let ins = source[ins.get_method_index()][ins.get_ins_index()];
        return ins;
    }
    pub(crate) fn running(&self) -> bool {
        !self.stack.is_empty()
            && self.stars > 0
            && self.board.touches() < Tile::MAX_TOUCHES as usize
            && *self.board.current_tile() != _N
        //&& self.stack.len() < StackArr::STACK_SIZE - 10
        //&& self.steps < MAX_STEPS
    }
    pub(crate) fn step(&mut self, source: &Source, puzzle: &Puzzle) -> bool {
        coz::begin!("step");
        let ins = self.current_ins(source).as_vanilla();
        self.stack.pop();
        self.steps += 1;
        if self.board.current_tile().executes(ins) {
            match ins.get_ins() {
                FORWARD => {
                    self.board.y = (self.board.y as i32
                        + [-1, 0, 1, 0][self.board.direction as usize])
                        as usize;
                    self.board.x = (self.board.x as i32
                        + [0, -1, 0, 1][self.board.direction as usize])
                        as usize;
                    if *self.board.current_tile() != _N {
                        self.stars -= self.board.current_tile().has_star() as usize;
                        self.board.clear_star();
                        self.board.touch();
                    }
                }
                LEFT => {
                    self.board.direction = self.board.direction.left();
                    self.board.touch();
                }
                RIGHT => {
                    self.board.direction = self.board.direction.right();
                    self.board.touch();
                }
                F1 | F2 | F3 | F4 | F5 => {
                    self.invoke(
                        source,
                        puzzle.methods[ins.source_index()],
                        ins.source_index(),
                    );
                    //                    self.max_stack = max(self.max_stack, self.stack.pointer);
                    self.board.touch();
                }
                MARK_GRAY | MARK_RED | MARK_GREEN | MARK_BLUE => {
                    self.board.mark(ins);
                    self.board.touch();
                }
                //HALT => {}
                _ => (),
            }
        }
        coz::end!("step");
        return self.running();
    }
    pub fn steps(
        &mut self,
        source: &Source,
        puzzle: &Puzzle,
        max_steps: usize,
        max_touches: usize,
    ) -> bool {
        for step in 0..max_steps {
            let rins = self.current_ins(source);
            let ins = rins.as_vanilla();
            let onwards = !(rins.is_nop()
                || (rins.is_probe() && self.current_tile().clone().executes(rins))
                || (true
                    && !rins.is_debug()
                    && !rins.is_loosened()
                    && !self.current_tile().to_condition().is_cond(rins.get_cond())));
            if !onwards {
                break;
            }
            self.stack.pop();
            self.steps += 1;
            if self.board.current_tile().executes(ins)
                && self.current_tile().touches() <= max_touches
            {
                match ins.get_ins() {
                    FORWARD => {
                        self.board.y = (self.board.y as i32
                            + [-1, 0, 1, 0][self.board.direction as usize])
                            as usize;
                        self.board.x = (self.board.x as i32
                            + [0, -1, 0, 1][self.board.direction as usize])
                            as usize;
                        let valid_tile = *self.board.current_tile() != _N;
                        self.stars -=
                            self.board.current_tile().has_star() as usize * valid_tile as usize;
                        self.board.current_tile_mut().0 += TILE_TOUCHED.0 * valid_tile as TileType;
                        self.board.clear_star();
                    }
                    LEFT => {
                        self.board.direction = self.board.direction.left();
                        self.board.touch();
                    }
                    RIGHT => {
                        self.board.direction = self.board.direction.right();
                        self.board.touch();
                    }
                    F1 | F2 | F3 | F4 | F5 => {
                        self.invoke(
                            source,
                            puzzle.methods[ins.source_index()],
                            ins.source_index(),
                        );
                        //                    self.max_stack = max(self.max_stack, self.stack.pointer);
                        self.board.touch();
                    }
                    MARK_GRAY | MARK_RED | MARK_GREEN | MARK_BLUE => {
                        self.board.mark(ins);
                        self.board.touch();
                    }
                    //HALT => {}
                    _ => (),
                }
            }
            if !self.running() {
                break;
            }
        }
        return self.running();
    }
    pub(crate) fn stepsj(&mut self, source: &Source, puzzle: &Puzzle, max_steps: usize) -> bool {
        next_op(self, source, self.steps + max_steps, puzzle);
        return self.running();
    }

    pub(crate) fn invoke(&mut self, source: &Source, method_length: usize, method: usize) {
        for i in (0..method_length).rev() {
            let ins = source.0[method][i];
            self.stack.push(InsPtr::new(method, i));
        }
    }
    pub(crate) fn get_hash(&self) -> u64 {
        let mut state = DefaultHasher::new();
        self.hash(&mut state);
        return state.finish();
    }
}

const JUMPS: [fn(&mut State, &Source, &Puzzle, Ins, usize); 15] = [
    forward, left, right, funcs, funcs, funcs, funcs, funcs, mark, mark, mark, nop, mark, nop, halt,
];

#[inline]
fn next_op(state: &mut State, source: &Source, max_steps: usize, puzzle: &Puzzle) {
    let onwards = state.steps <= max_steps && state.running();
    if (!onwards) {
        return;
    }
    let rins = state.current_ins(source);
    let ins = rins.as_vanilla();
    let executes = (state.board.current_tile().executes(ins));
    let onwards = !(rins.is_nop()
        || (rins.is_probe() && state.current_tile().clone().executes(rins))
        || (true
            && !rins.is_debug()
            && !rins.is_loosened()
            && !state.current_tile().to_condition().is_cond(rins.get_cond())));
    if (!onwards) {
        return;
    }
    state.stack.pop();
    state.steps += 1;
    let ins_id = (ins.get_ins().0 as usize);
    return JUMPS[ins_id.min(13) * executes as usize * onwards as usize
        + 13 * !executes as usize * onwards as usize
        + 14 * !onwards as usize](state, source, puzzle, ins, max_steps);
}
fn forward(state: &mut State, source: &Source, puzzle: &Puzzle, ins: Ins, max_steps: usize) {
    state.board.y = (state.board.y as i32 + [-1, 0, 1, 0][state.board.direction as usize]) as usize;
    state.board.x = (state.board.x as i32 + [0, -1, 0, 1][state.board.direction as usize]) as usize;
    let valid_tile = *state.board.current_tile() != _N;
    state.stars -= state.board.current_tile().has_star() as usize * valid_tile as usize;
    state.board.current_tile_mut().0 += TILE_TOUCHED.0 * valid_tile as TileType;
    state.board.clear_star();
    return next_op(state, source, max_steps, puzzle);
}
fn left(state: &mut State, source: &Source, puzzle: &Puzzle, ins: Ins, max_steps: usize) {
    state.board.direction = state.board.direction.left();
    state.board.touch();
    return next_op(state, source, max_steps, puzzle);
}
fn right(state: &mut State, source: &Source, puzzle: &Puzzle, ins: Ins, max_steps: usize) {
    state.board.direction = state.board.direction.right();
    state.board.touch();
    return next_op(state, source, max_steps, puzzle);
}
fn funcs(state: &mut State, source: &Source, puzzle: &Puzzle, ins: Ins, max_steps: usize) {
    let method_length = puzzle.methods[ins.source_index()];
    let method = ins.source_index();
    for i in (0..method_length).rev() {
        state.stack.push(InsPtr::new(method, i));
    }
    //                    self.max_stack = max(self.max_stack, self.stack.pointer);
    state.board.touch();
    return next_op(state, source, max_steps, puzzle);
}
fn mark(state: &mut State, source: &Source, puzzle: &Puzzle, ins: Ins, max_steps: usize) {
    state.board.mark(ins);
    state.board.touch();
    return next_op(state, source, max_steps, puzzle);
}
fn nop(state: &mut State, source: &Source, puzzle: &Puzzle, ins: Ins, max_steps: usize) {
    return next_op(state, source, max_steps, puzzle);
}

fn halt(state: &mut State, source: &Source, puzzle: &Puzzle, ins: Ins, max_steps: usize) {}

pub fn won(state: &State, _: &Puzzle) -> bool {
    return state.stars == 0;
}

pub fn steps(state: &State, _: &Puzzle) -> usize {
    return state.steps;
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Stars: {}, Running: {}, Steps: {}\n",
            self.stars,
            self.running(),
            self.steps
        )?;
        write!(f, "{}", self.board);
        writeln!(f, "stack: {}", self.stack.len());
        writeln!(f, "tile: {:?}", self.board.current_tile());
        write!(f, "")
    }
}
impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self)
    }
}
