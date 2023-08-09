use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Index, IndexMut};

use colored::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::constants::*;
use instructions::*;

use self::board::Board;
use self::puzzle::{make_puzzle, Puzzle};

pub mod board;
pub mod display;
pub mod instructions;
pub mod puzzle;
pub mod state;

#[cfg(test)]
mod tests;

pub type TileType = u32;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct Tile(pub TileType);

impl Tile {
    const MAX_TOUCHES: TileType = u32::MAX >> 4;
    fn touch(&mut self) {
        self.0 += TILE_TOUCHED.0;
    }
    fn untouch(&mut self) {
        self.0 -= TILE_TOUCHED.0;
    }
    pub(crate) fn touches(&self) -> usize {
        (self.0 >> 4) as usize
    }
    fn clear_star(&mut self) {
        self.0 &= !TILE_STAR_MASK.0;
    }
    pub(crate) fn has_star(&self) -> bool {
        (self.0 & TILE_STAR_MASK.0) > 0
    }
    fn color(&self) -> Tile {
        Tile(self.0 & TILE_COLOR_MASK.0)
    }
    fn is_red(&self) -> bool {
        self.0 & RE.0 > 0
    }
    fn is_green(&self) -> bool {
        self.0 & GE.0 > 0
    }
    fn is_blue(&self) -> bool {
        self.0 & BE.0 > 0
    }
    pub(crate) fn executes(&self, instruction: Ins) -> bool {
        let color = instruction.get_cond();
        return color == GRAY_COND || color.has_cond(self.to_condition());
    }
    fn mark(&mut self, instruction: Ins) {
        let color: u8 = instruction.get_mark_color().into();
        self.0 = color as TileType | (self.0 & TILE_TOUCH_MASK.0)
    }
    pub(crate) fn to_condition(&self) -> Ins {
        ((self.0 << 5) & 0b11111111).into()
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct Map(pub [[Tile; 18]; 14]);

impl Map {
    pub fn count_stars(&self) -> usize {
        self.0
            .iter()
            .map(|row| row.iter().map(|el| el.has_star() as usize).sum::<usize>())
            .sum()
    }
}

pub type Method = [Ins; 10];

#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Serialize, Deserialize)]
pub struct Source(pub [Method; 5]);

impl Source {
    fn len(&self) -> usize {
        self.0.len()
    }
    pub fn has_nop(&self) -> bool {
        let mut result = false;
        for m in self.0.iter() {
            for i in m.iter() {
                result |= i.is_nop();
            }
        }
        return result;
    }
    pub fn count_ins(&self) -> usize {
        let mut result = 0;
        for m in self.0.iter() {
            for i in m.iter() {
                result += !(i.is_nop() || i.is_halt()) as usize;
            }
        }
        return result;
    }
    pub fn count_nop(&self) -> usize {
        let mut result = 0;
        for m in self.0.iter() {
            for i in m.iter() {
                result += !(i.is_nop() || i.is_halt()) as usize;
            }
        }
        return result;
    }
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        return hasher.finish();
    }
    pub fn shade(&mut self, max_ins: usize) {
        let diff = max_ins - self.count_ins();
        for m in 0..5 {
            let mut nops = 0;
            for i in 0..10 {
                nops += self[m][i].is_nop() as usize;

                if nops > diff {
                    self[m][i] = HALT;
                }
            }
        }
    }
    pub fn sanitize(&mut self) {
        for m in 0..5 {
            let mut offset = 0;
            for i in 0..10 {
                if self[m][i].is_nop() || self[m][i].is_probe() {
                    offset += 1;
                }
                self[m][i] = if i + offset < 10 {
                    self[m][i + offset]
                } else {
                    HALT
                }
            } // ↑2↑←↑→5|||,_←↑35←↑→↑→,↑↑→↑4↑→↑←↑,←↑↑→↑←↑↑→↑,↑←↑→↑↑→↑←↑,
        }
    }
}

impl Hash for Source {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for method in self.0.iter() {
            for ins in method {
                ins.hash(state);
            }
        }
    }
}

impl Index<usize> for Source {
    type Output = Method;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Source {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

type Stack = StackVec;

pub(crate) const MAX_STEPS: usize = 1 << 12;
const STACK_MATCH: usize = 1 << 6;

#[derive(Clone, Debug)]
pub struct StackVec(pub SmallVec<[InsPtr; 1 << 8]>);

impl Default for StackVec {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl PartialEq for StackVec {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        } else if self.0.len() <= STACK_MATCH {
            return self.0 == other.0;
        } else if self.0.len() > STACK_MATCH {
            return self.0.get((self.0.len() - STACK_MATCH)..self.0.len())
                == other.0.get((self.0.len() - STACK_MATCH)..self.0.len());
        } else {
            return false;
        }
    }
}

impl Eq for StackVec {}

impl Hash for StackVec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0
            .get((self.0.len() - STACK_MATCH).max(0)..self.0.len())
            .hash(state);
    }
}
impl Index<usize> for StackVec {
    type Output = InsPtr;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.0.len() - index - 1]
    }
}
impl StackVec {
    fn push(&mut self, element: InsPtr) {
        self.0.push(element);
    }
    fn pop(&mut self) -> InsPtr {
        self.0
            .pop()
            .expect("pop() shouldn't be called on an empty stack")
    }
    pub fn last(&self) -> &InsPtr {
        self.0.last().unwrap_or(&INSPTR_NULL)
        //.expect("last() shouldn't be called on an empty stack")
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StackArr {
    pointer: usize,
    pub data: [InsPtr; Self::STACK_SIZE],
}

impl Default for StackArr {
    fn default() -> Self {
        Self {
            pointer: 0,
            data: [INSPTR_NULL; Self::STACK_SIZE],
        }
    }
}

impl PartialEq for StackArr {
    fn eq(&self, other: &Self) -> bool {
        if self.pointer != other.pointer {
            return false;
        } else if self.pointer <= STACK_MATCH {
            return self.data.get(0..self.pointer) == other.data.get(0..other.pointer);
        } else if self.pointer > STACK_MATCH {
            let start = self.pointer - STACK_MATCH;
            return self.data.get(start..self.pointer) == other.data.get(start..other.pointer);
        } else {
            return false;
        }
    }
}

impl Eq for StackArr {}

impl Hash for StackArr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut remaining = STACK_MATCH;
        let mut i = self.pointer;
        while remaining > 0 && i > 0 {
            i -= 1;
            //            if self.data[i] != NOP && self.data[i] != HALT {
            self.data[i].hash(state);
            remaining -= 1;
            //            }
        }
    }
}

impl Index<usize> for StackArr {
    type Output = InsPtr;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.pointer - index - 1]
    }
}

impl StackArr {
    // make the StackArr struct exactly 2^10 bytes
    const STACK_SIZE: usize = (1 << 9) - mem::size_of::<usize>();
    fn push(&mut self, element: InsPtr) {
        self.data[self.pointer] = element;
        self.pointer += 1;
    }
    fn pop(&mut self) -> InsPtr {
        let &result = self.last();
        self.pointer -= 1;
        return result;
    }
    pub fn last(&self) -> &InsPtr {
        &self.data[self.pointer - 1]
    }
    pub(crate) fn len(&self) -> usize {
        self.pointer
    }
    fn is_empty(&self) -> bool {
        self.pointer == 0
    }
    pub(crate) fn clear(&mut self) {
        self.pointer = 0;
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up = 0b00,
    Left = 0b01,
    Down = 0b10,
    Right = 0b11,
}

//#[derive()]
impl Direction {
    fn left(&self) -> Direction {
        return unsafe { std::mem::transmute((*self as u8 + 1) & 0b11) };
        // match self {
        //     Direction::Up => Direction::Left,
        //     Direction::Left => Direction::Down,
        //     Direction::Down => Direction::Right,
        //     Direction::Right => Direction::Up,
        // }
    }
    fn right(&self) -> Direction {
        return unsafe { std::mem::transmute((*self as u8 + 3) & 0b11) };
        // match self {
        //     Direction::Up => Direction::Right,
        //     Direction::Left => Direction::Up,
        //     Direction::Down => Direction::Left,
        //     Direction::Right => Direction::Down,
        // }
    }
}

pub fn genboi(ta: Tile, tb: Tile, tc: Tile) -> Puzzle {
    return make_puzzle(
        Board {
            map: Map([
                [
                    _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, ta, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tb, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, tc, _N,
                ],
                [
                    _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N,
                ],
            ]),
            direction: Direction::Right,
            x: 5,
            y: 6,
        },
        [3, 10, 0, 0, 0],
        [true, true, true],
    );
}
