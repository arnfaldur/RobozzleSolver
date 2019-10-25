use super::game::*;
use std::collections::HashSet;
use crate::game::instructions::*;

pub(crate) const RE: Tile = Tile(0b00001);
pub(crate) const GE: Tile = Tile(0b00010);
pub(crate) const BE: Tile = Tile(0b00100);
pub(crate) const RS: Tile = Tile(0b01001);
pub(crate) const GS: Tile = Tile(0b01010);
pub(crate) const BS: Tile = Tile(0b01100);
pub(crate) const _N: Tile = Tile(0b10000);

pub(crate) const TILE_STAR_MASK: Tile = Tile(0b00001000);
pub(crate) const TILE_COLOR_MASK: Tile = Tile(0b00010111);

pub(crate) const TILE_TOUCHED: Tile = Tile(0b00100000);

pub(crate) const NOGRAM: Source = Source([[HALT; 10]; 5]);


const TEST_SOURCE: Source = Source([
    [
        FORWARD,
        RED_FORWARD,
        GREEN_FORWARD,
        BLUE_FORWARD,
        LEFT,
        RED_LEFT,
        GREEN_LEFT,
        BLUE_LEFT,
        MARK_GREEN,
        MARK_BLUE,
    ],
    [
        RIGHT,
        RED_RIGHT,
        GREEN_RIGHT,
        BLUE_RIGHT,
        F1,
        RED_F1,
        GREEN_F1,
        BLUE_F1,
        RED_MARK_GREEN,
        RED_MARK_BLUE,
    ],
    [
        F2,
        RED_F2,
        GREEN_F2,
        BLUE_F2,
        F3,
        RED_F3,
        GREEN_F3,
        BLUE_F3,
        GREEN_MARK_GREEN,
        GREEN_MARK_BLUE,
    ],
    [
        F4,
        RED_F4,
        GREEN_F4,
        BLUE_F4,
        F5,
        RED_F5,
        GREEN_F5,
        BLUE_F5,
        BLUE_MARK_GREEN,
        BLUE_MARK_BLUE,
    ],
    [
        HALT,
        MARK_RED,
        RED_MARK_RED,
        GREEN_MARK_RED,
        BLUE_MARK_RED,
        NOP, NOP, NOP, NOP, NOP, ],
]);

//const PUZZLE_NULL: Puzzle = Puzzle {
//    map: [
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
//    ],
//    direction: Direction::Down,
//    x: 1,
//    y: 1,
//    stars: 1,
//    functions: [0, 0, 0, 0, 0],
//    marks: [false; 3],
//    red: false,
//    green: false,
//    blue: false,
//};
pub(crate) const PUZZLE_42: Puzzle = Puzzle {
    map: [
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BS, BS, BS, BS, BS, BS, BS, BS, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, BE, BS, BS, BS, BS, BS, BS, BS, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 5,
    y: 9,
    stars: 23,
    functions: [5, 2, 2, 2, 0],
    marks: [false; 3],
    red: false,
    green: false,
    blue: true,
};
pub(crate) const PUZZLE_42_SOLUTION: Source = Source([
    [
        F2,
        LEFT,
        F3,
        LEFT,
        F1,
        HALT, HALT, HALT, HALT, HALT, ],
    [
        F3,
        F3,
        HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT, ],
    [
        F4,
        F4,
        HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT, ],
    [
        FORWARD,
        FORWARD,
        HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT, ],
    [HALT; 10],
]);
pub(crate) const PUZZLE_536: Puzzle = Puzzle {
    map: [
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, BE, BE, BE, BE, BE, BE, BE, GE, BE, BE, BE, BE, BE, BE, BE, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, BE, _N, _N, ],
        [_N, BE, BE, BE, BE, BE, BE, RE, BE, BE, BE, BE, BE, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, BE, BE, BE, BE, GE, BE, BE, BE, BE, _N, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, BE, _N, _N, _N, _N, _N, _N, _N, RE, _N, GE, _N, RE, _N, _N, ],
        [_N, RE, _N, GE, _N, BS, BE, BE, BE, BE, BE, BE, _N, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, BE, _N, _N, _N, _N, _N, _N, _N, _N, _N, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, BE, BE, BE, BE, BE, RE, BE, BE, BE, BE, BE, _N, BE, _N, _N, ],
        [_N, BE, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, BE, _N, _N, ],
        [_N, BE, BE, BE, BE, BE, BE, BE, GE, BE, BE, BE, BE, BE, BE, BE, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 1,
    y: 1,
    stars: 1,
    functions: [3, 3, 0, 0, 0],
    marks: [false; 3],
    red: true,
    green: true,
    blue: true,
};
pub(crate) const PUZZLE_536_SOLUTION: Source = Source([
    [
        F2,
        RIGHT,
        F1,
        HALT, HALT, HALT, HALT, HALT, HALT, HALT, ],
    [
        FORWARD,
        BLUE_F2,
        FORWARD,
        HALT, HALT, HALT, HALT, HALT, HALT, HALT, ],
    [HALT; 10],
    [HALT; 10],
    [HALT; 10],
]);
pub(crate) const PUZZLE_656: Puzzle = Puzzle {
    map: [
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, RS, _N, _N, _N, RS, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, RS, _N, _N, _N, _N, ],
        [_N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, _N, _N, _N, ],
        [_N, BS, _N, _N, _N, BS, _N, _N, _N, _N, _N, _N, _N, BS, _N, RS, _N, _N, ],
        [_N, BS, _N, RS, _N, BS, _N, _N, _N, RS, _N, _N, _N, BS, _N, BS, _N, _N, ],
        [_N, BS, _N, BS, _N, BS, _N, _N, _N, BS, _N, RS, _N, BS, _N, BS, _N, _N, ],
        [_N, BS, RS, BS, _N, BS, _N, _N, _N, BS, _N, BS, _N, BS, _N, BS, _N, _N, ],
        [_N, BS, BS, BS, _N, BS, RS, _N, _N, BS, _N, BS, _N, BS, RS, BS, _N, _N, ],
        [_N, BS, BS, BS, _N, BS, BS, _N, _N, BS, RS, BS, _N, BS, BS, BS, _N, _N, ],
        [_N, BS, BS, BS, _N, BS, BS, _N, _N, BS, BS, BS, RS, BS, BS, BS, RS, _N, ],
        [_N, BS, BS, BS, RS, BS, BS, RS, RS, BS, BS, BS, BS, BS, BS, BS, BS, _N, ],
        [_N, BE, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, BS, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 1,
    y: 12,
    stars: 98,
    functions: [5, 5, 0, 0, 0],
    marks: [false; 3],
    red: true,
    green: false,
    blue: true,
};
pub(crate) const PUZZLE_656_SOLUTION: Source = Source([
    [
        LEFT,
        F2,
        LEFT,
        FORWARD,
        F1,
        HALT, HALT, HALT, HALT, HALT, ],
    [
        FORWARD,
        RED_RIGHT,
        RED_RIGHT,
        BLUE_F2,
        FORWARD,
        HALT, HALT, HALT, HALT, HALT, ],
    [HALT; 10],
    [HALT; 10],
    [HALT; 10],
]);

pub(crate) const PUZZLE_1337: Puzzle = Puzzle {
    map: [
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, BE, BE, BE, BE, BE, BE, BE, BE, BE, BE, BE, BE, _N, _N, _N, ],
        [_N, _N, _N, BE, RE, RE, RE, RE, RE, RE, RE, RE, RE, RE, BS, _N, _N, _N, ],
        [_N, _N, _N, BE, RE, GE, GE, GE, GE, GE, GE, GE, GE, RE, BE, _N, _N, _N, ],
        [_N, _N, _N, _N, RE, GE, BE, BS, BE, RE, BS, BE, GE, RE, _N, _N, _N, _N, ],
        [_N, _N, _N, GE, RE, GE, BE, BE, GE, RE, BE, BE, GE, RE, GE, _N, _N, _N, ],
        [_N, _N, _N, GE, RE, GE, _N, BE, RS, GS, BE, _N, GE, RE, GE, _N, _N, _N, ],
        [_N, _N, _N, _N, RE, GE, _N, GE, BE, GE, GE, _N, GE, RE, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, RE, GE, _N, _N, _N, _N, _N, _N, GE, RE, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, RE, GE, _N, _N, _N, _N, _N, _N, GE, RE, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, RE, GE, GE, GE, GE, GE, GE, GE, GE, RE, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, RE, RE, RE, RE, RE, RE, RE, RE, RE, RE, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, BS, BS, BS, BS, BE, BE, BS, BS, BS, BS, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 8,
    y: 7,
    stars: 13,
    functions: [6, 2, 0, 0, 0],
    marks: [true; 3],
    red: true,
    green: true,
    blue: true,
};
pub(crate) const PUZZLE_1337_SOLUTION: Source = Source([
    [
        F2,
        BLUE_MARK_RED,
        GREEN_MARK_BLUE,
        RED_MARK_GREEN,
        FORWARD,
        F1,
        HALT, HALT, HALT, HALT, ],
    [
        GREEN_RIGHT,
        BLUE_LEFT,
        HALT, HALT, HALT, HALT, HALT, HALT, HALT, HALT, ],
    [HALT; 10],
    [HALT; 10],
    [HALT; 10],
]);

pub(crate) const PUZZLE_TEST_1: Puzzle = Puzzle {
    map: [
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, BS, RE, RS, RS, GS, GS, GS, BS, BS, BS, RS, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
        [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, _N, ],
    ],
    direction: Direction::Right,
    x: 2,
    y: 1,
    stars: 10,
    functions: [10; 5],
    marks: [true; 3],
    red: true,
    green: true,
    blue: true,
};

pub(crate) const PUZZLE_TEST_1_SOLUTION: Source = Source([
    [
        RED_MARK_RED,
        RED_FORWARD,
        RED_MARK_GREEN,
        GREEN_FORWARD,
        RED_MARK_BLUE,
        BLUE_FORWARD,
        GREEN_MARK_RED,
        RED_FORWARD,
        GREEN_MARK_GREEN,
        GREEN_F2,
    ],
    [
        GREEN_FORWARD,
        GREEN_MARK_BLUE,
        BLUE_FORWARD,
        BLUE_MARK_RED,
        RED_FORWARD,
        BLUE_MARK_GREEN,
        GREEN_FORWARD,
        BLUE_MARK_BLUE,
        BLUE_FORWARD,
        F3, ],
    [RED_LEFT, RED_LEFT, F4, HALT, HALT, HALT, HALT, HALT, HALT, HALT, ],
    [RED_FORWARD, BLUE_FORWARD, GREEN_FORWARD, RED_F4, HALT, HALT, HALT, HALT, HALT, HALT, ],
    [HALT; 10],
]);