use std::fmt;
use std::fmt::{Formatter, Error, Display};
use colored::*;

type instype = u32;

#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
pub struct Ins(instype);

impl Ins {
    pub(crate) fn condition_to_color(self) -> Ins { Ins(self.un_branched().0 >> 5) }
    pub(crate) fn source_index(self) -> usize { (self.get_instruction().0 - F1.0) as usize }
    pub(crate) fn get_condition(self) -> Ins { self & INS_COLOR_MASK }
    pub(crate) fn get_instruction(self) -> Ins { self & INS_MASK }
    fn get_fun_number(self) -> u8 { (self.get_instruction().0 - F1.0 + 1) as u8 }
    pub(crate) fn get_mark_color(self) -> Ins { self & MARK_MASK }
    pub(crate) fn get_mark_as_condition(self) -> Ins { self.get_mark_color().color_to_condition() }
    pub(crate) fn color_to_condition(self) -> Ins { Ins(self.0 << 5).un_branched() }
    pub(crate) fn get_marker(self) -> Ins { (self.get_instruction() | NOP).un_branched() }
    pub(crate) fn from_marker(self) -> Ins { Ins(self.0 & !NOP.0).un_branched() }
    pub(crate) fn is_condition(self, condition: Ins) -> bool { self.get_condition() == condition }
    pub(crate) fn has_condition(self, condition: Ins) -> bool { self & condition == condition }
    pub(crate) fn is_gray(self) -> bool { self.is_condition(GRAY_COND) }
    pub(crate) fn is_mark(self) -> bool { (self & MARK_GRAY) == MARK_GRAY }
    pub(crate) fn is_function(self) -> bool { self.get_instruction() >= F1 && self.get_instruction() <= F5 }
    pub(crate) fn is_instruction(self, instruction: Ins) -> bool { self.get_instruction() == instruction }
    pub(crate) fn is_order_invariant(self) -> bool { self.is_mark() || self.is_turn() }
    pub(crate) fn is_turn(self) -> bool { self.is_instruction(LEFT) || self.is_instruction(RIGHT) }
    pub(crate) fn to_probe(self) -> Ins { self.get_condition() | NOP }
    pub(crate) fn is_probe(self) -> bool { !self.is_gray() && self.get_instruction() == NOP }
    pub(crate) fn other_turn(self) -> Ins {
        if self.is_instruction(LEFT) { RIGHT } else if self.is_instruction(RIGHT) { LEFT } else { HALT }
    }
    pub(crate) fn is_debug(self) -> bool { (self & NOP) == NOP }
    pub(crate) fn get_probes(self, excluded: Ins) -> Vec<Ins> {
        let mask = self.to_probe();
        return PROBES.iter().filter(|&ins| (*ins & mask) == *ins && *ins != excluded.to_probe()).cloned().collect();
    }
    pub(crate) fn un_branched(self) -> Ins { self & VANILLA_MASK }
    pub(crate) fn is_branched(self) -> bool { self & BRANCH_MASK == self }
    pub(crate) fn as_branched(self) -> Ins { self | BRANCH_MASK }
}

pub(crate) fn with_conditions(red: bool, green: bool, blue: bool) -> Ins {
    Ins(((red as u8) << 5 | (green as u8) << 6 | (blue as u8) << 7) as instype)
}

const BRANCH_MASK: Ins = Ins(0b100000000);
const VANILLA_MASK: Ins = Ins(0b11111111);

impl From<Ins> for u8 {
    fn from(ins: Ins) -> Self {
        ins.0 as u8
    }
}

impl From<u8> for Ins {
    fn from(val: u8) -> Self {
        Ins(val as instype)
    }
}

impl std::ops::BitOr for Ins {
    type Output = Ins;
    fn bitor(self, rhs: Self) -> Self::Output {
        Ins(self.0.bitor(rhs.0))
    }
}

impl std::ops::BitAnd for Ins {
    type Output = Ins;
    fn bitand(self, rhs: Self) -> Self::Output {
        Ins(self.0.bitand(rhs.0))
    }
}

impl Display for Ins {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let background = match self.get_condition() {
            GRAY_COND => Color::BrightBlack,
            RED_COND => Color::Red,
            GREEN_COND => Color::Green,
            BLUE_COND => Color::Blue,
            _ => Color::Black,
        };
        let ins = self.get_instruction();
        let foreground = match ins {
            MARK_RED => Color::Red,
            MARK_GREEN => Color::Green,
            MARK_BLUE => Color::Blue,
            _ => Color::BrightWhite,
        };
        let string = match ins {
            FORWARD => "↑".to_string(),
            LEFT => "←".to_string(),
            RIGHT => "→".to_string(),
            F1 | F2 | F3 | F4 | F5 => ins.get_fun_number().to_string(),
            MARK_RED | MARK_GREEN | MARK_BLUE => "●".to_string(),
            NOP => match *self {
                RED_PROBE | GREEN_PROBE | BLUE_PROBE => "_".to_string(),
                _ => " ".to_string(),
            }
            _ => " ".to_string(),
        };
        write!(f, "{}", string.color(foreground).on_color(background))
    }
}

impl fmt::Debug for Ins {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if *self == NOP {
            write!(f, "NOP")
        } else if *self == HALT {
            write!(f, "HALT")
        } else {
            write!(f, "{}", match self.get_condition() {
                RED_COND => "RED_",
                GREEN_COND => "GREEN_",
                BLUE_COND => "BLUE_",
                _ => "",
            })?;
            write!(f, "{}", match self.get_instruction() {
                FORWARD => "FORWARD",
                LEFT => "LEFT",
                RIGHT => "RIGHT",
                F1 => "F1",
                F2 => "F2",
                F3 => "F3",
                F4 => "F4",
                F5 => "F5",
                MARK_RED => "MARK_RED",
                MARK_GREEN => "MARK_GREEN",
                MARK_BLUE => "MARK_BLUE",
                F1_MARKER => "F1_MARKER",
                F2_MARKER => "F2_MARKER",
                F3_MARKER => "F3_MARKER",
                F4_MARKER => "F4_MARKER",
                F5_MARKER => "F5_MARKER",
                NOP => "PROBE",
                _ => "",
            })
        }
    }
}

pub(crate) const FORWARD: Ins = Ins(0);
pub(crate) const LEFT: Ins = Ins(1);
pub(crate) const RIGHT: Ins = Ins(2);
pub(crate) const F1: Ins = Ins(3);
pub(crate) const F2: Ins = Ins(4);
pub(crate) const F3: Ins = Ins(5);
pub(crate) const F4: Ins = Ins(6);
pub(crate) const F5: Ins = Ins(7);
// This isn't actually in the game
pub(crate) const MARK_GRAY: Ins = Ins(0b00001000);
pub(crate) const MARK_RED: Ins = Ins(0b00001001);
pub(crate) const MARK_GREEN: Ins = Ins(0b00001010);
pub(crate) const MARK_BLUE: Ins = Ins(0b00001100);

pub(crate) const NOP: Ins = Ins(0b00010000);
pub(crate) const HALT: Ins = Ins(0b11110111);

pub(crate) const GRAY_COND: Ins = Ins(0b00000000);
pub(crate) const RED_COND: Ins = Ins(0b00100000);
pub(crate) const GREEN_COND: Ins = Ins(0b01000000);
pub(crate) const BLUE_COND: Ins = Ins(0b10000000);

// masks for isolating instruction parts
pub(crate) const MARK_MASK: Ins = Ins(0b00000111);
pub(crate) const INS_MASK: Ins = Ins(0b00011111);
pub(crate) const INS_COLOR_MASK: Ins = Ins(0b11100000);

// iterable lists of constants
pub(crate) const INSTRUCTIONS: [Ins; 11] = [
    FORWARD,
    LEFT,
    RIGHT,
    F1,
    F2,
    F3,
    F4,
    F5,
    MARK_RED,
    MARK_GREEN,
    MARK_BLUE,
];
pub(crate) const MOVES: [Ins; 3] = [
    FORWARD,
    LEFT,
    RIGHT,
];
pub(crate) const FUNCTIONS: [Ins; 5] = [
    F1,
    F2,
    F3,
    F4,
    F5,
];
pub(crate) const MARKS: [Ins; 3] = [
    MARK_RED,
    MARK_GREEN,
    MARK_BLUE,
];
pub(crate) const CONDITIONS: [Ins; 4] = [
    GRAY_COND,
    RED_COND,
    GREEN_COND,
    BLUE_COND
];

// constants for backtracking
pub(crate) const F1_MARKER: Ins = Ins(3 | NOP.0);
pub(crate) const F2_MARKER: Ins = Ins(4 | NOP.0);
pub(crate) const F3_MARKER: Ins = Ins(5 | NOP.0);
pub(crate) const F4_MARKER: Ins = Ins(6 | NOP.0);
pub(crate) const F5_MARKER: Ins = Ins(7 | NOP.0);
pub(crate) const FUNCTION_MARKERS: [Ins; 5] = [
    F1_MARKER,
    F2_MARKER,
    F3_MARKER,
    F4_MARKER,
    F5_MARKER,
];

pub(crate) const RED_PROBE: Ins = Ins(NOP.0 | RED_COND.0);
pub(crate) const GREEN_PROBE: Ins = Ins(NOP.0 | GREEN_COND.0);
pub(crate) const BLUE_PROBE: Ins = Ins(NOP.0 | BLUE_COND.0);
pub(crate) const PROBES: [Ins; 3] = [
    RED_PROBE,
    GREEN_PROBE,
    BLUE_PROBE,
];

// constant combinations for brevity
pub(crate) const RED_FORWARD: Ins = Ins(FORWARD.0 | RED_COND.0);
pub(crate) const RED_LEFT: Ins = Ins(LEFT.0 | RED_COND.0);
pub(crate) const RED_RIGHT: Ins = Ins(RIGHT.0 | RED_COND.0);
pub(crate) const RED_F1: Ins = Ins(F1.0 | RED_COND.0);
pub(crate) const RED_F2: Ins = Ins(F2.0 | RED_COND.0);
pub(crate) const RED_F3: Ins = Ins(F3.0 | RED_COND.0);
pub(crate) const RED_F4: Ins = Ins(F4.0 | RED_COND.0);
pub(crate) const RED_F5: Ins = Ins(F5.0 | RED_COND.0);
pub(crate) const RED_MARK_RED: Ins = Ins(MARK_RED.0 | RED_COND.0);
pub(crate) const RED_MARK_GREEN: Ins = Ins(MARK_GREEN.0 | RED_COND.0);
pub(crate) const RED_MARK_BLUE: Ins = Ins(MARK_BLUE.0 | RED_COND.0);

pub(crate) const GREEN_FORWARD: Ins = Ins(FORWARD.0 | GREEN_COND.0);
pub(crate) const GREEN_LEFT: Ins = Ins(LEFT.0 | GREEN_COND.0);
pub(crate) const GREEN_RIGHT: Ins = Ins(RIGHT.0 | GREEN_COND.0);
pub(crate) const GREEN_F1: Ins = Ins(F1.0 | GREEN_COND.0);
pub(crate) const GREEN_F2: Ins = Ins(F2.0 | GREEN_COND.0);
pub(crate) const GREEN_F3: Ins = Ins(F3.0 | GREEN_COND.0);
pub(crate) const GREEN_F4: Ins = Ins(F4.0 | GREEN_COND.0);
pub(crate) const GREEN_F5: Ins = Ins(F5.0 | GREEN_COND.0);
pub(crate) const GREEN_MARK_RED: Ins = Ins(MARK_RED.0 | GREEN_COND.0);
pub(crate) const GREEN_MARK_GREEN: Ins = Ins(MARK_GREEN.0 | GREEN_COND.0);
pub(crate) const GREEN_MARK_BLUE: Ins = Ins(MARK_BLUE.0 | GREEN_COND.0);

pub(crate) const BLUE_FORWARD: Ins = Ins(FORWARD.0 | BLUE_COND.0);
pub(crate) const BLUE_LEFT: Ins = Ins(LEFT.0 | BLUE_COND.0);
pub(crate) const BLUE_RIGHT: Ins = Ins(RIGHT.0 | BLUE_COND.0);
pub(crate) const BLUE_F1: Ins = Ins(F1.0 | BLUE_COND.0);
pub(crate) const BLUE_F2: Ins = Ins(F2.0 | BLUE_COND.0);
pub(crate) const BLUE_F3: Ins = Ins(F3.0 | BLUE_COND.0);
pub(crate) const BLUE_F4: Ins = Ins(F4.0 | BLUE_COND.0);
pub(crate) const BLUE_F5: Ins = Ins(F5.0 | BLUE_COND.0);
pub(crate) const BLUE_MARK_RED: Ins = Ins(MARK_RED.0 | BLUE_COND.0);
pub(crate) const BLUE_MARK_GREEN: Ins = Ins(MARK_GREEN.0 | BLUE_COND.0);
pub(crate) const BLUE_MARK_BLUE: Ins = Ins(MARK_BLUE.0 | BLUE_COND.0);

