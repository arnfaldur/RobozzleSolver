use std::fmt::{Formatter, Error, Display, Debug};

use colored::*;
use serde::{Serialize, Deserialize};
type InsType = u32;

#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct Ins(InsType);

impl Ins {
    pub(crate) fn condition_to_color(self) -> Ins { Ins(self.as_vanilla().0 >> 5) }
    pub(crate) fn source_index(self) -> usize { (self.from_marker().get_ins().0 - F1.0) as usize }
    pub(crate) fn get_cond(self) -> Ins { self & INS_COLOR_MASK }
    pub(crate) fn get_ins(self) -> Ins { self & INS_MASK }
    fn get_fun_number(self) -> u8 { (self.get_ins().0 - F1.0 + 1) as u8 }
    pub(crate) fn get_mark_color(self) -> Ins { self & MARK_MASK }
    pub(crate) fn get_mark_as_cond(self) -> Ins { self.get_mark_color().color_to_cond() }
    pub(crate) fn color_to_cond(self) -> Ins { Ins(self.0 << 5).as_vanilla() }
    pub(crate) fn get_marker(self) -> Ins { (self.get_ins() | NOP).as_vanilla() }
    pub(crate) fn from_marker(self) -> Ins { Ins(self.0 & !NOP.0).as_vanilla() }
    pub(crate) fn is_cond(self, cond: Ins) -> bool { self.get_cond() == cond }
    pub(crate) fn has_cond(self, cond: Ins) -> bool { self & cond == cond }
    pub(crate) fn with_cond(self, cond: Ins) -> Ins { self.get_ins() | cond }
    pub(crate) fn is_gray(self) -> bool { self.is_cond(GRAY_COND) }
    pub(crate) fn is_mark(self) -> bool { (self & MARK_GRAY) == MARK_GRAY }
    pub(crate) fn is_function(self) -> bool { self.get_ins() >= F1 && self.get_ins() <= F5 }
    pub(crate) fn is_ins(self, ins: Ins) -> bool { self.get_ins() == ins }
    pub(crate) fn is_order_invariant(self) -> bool { self.is_mark() || self.is_turn() }
    pub(crate) fn is_turn(self) -> bool { self.is_ins(LEFT) || self.is_ins(RIGHT) }
    pub(crate) fn to_probe(self) -> Ins { self.get_cond() | NOP }
    pub(crate) fn is_probe(self) -> bool { !self.is_gray() && self.get_ins() == NOP }
    pub(crate) fn is_nop(self) -> bool { self.as_vanilla() == NOP }
    pub(crate) fn is_halt(self) -> bool { self.as_vanilla() == HALT }
    pub fn is_function_marker(self) -> bool { (self & NOP).is_nop() && self.from_marker().is_function() }
    pub(crate) fn other_turn(self) -> Ins {
        if self.is_ins(LEFT) { RIGHT } else if self.is_ins(RIGHT) { LEFT } else { HALT }
    }
    pub(crate) fn is_debug(self) -> bool { (self.as_vanilla() & NOP) == NOP }
    pub(crate) fn get_probes(self, excluded: Ins) -> Vec<Ins> {
        let mask = self.to_probe();
        return PROBES.iter()
            .filter(|&ins| (*ins & mask) == *ins && *ins != excluded.to_probe())
            .map(|ins| ins.as_loosened()).collect();
    }
    pub(crate) fn as_vanilla(self) -> Ins { self & VANILLA_MASK }
    pub(crate) fn is_loosened(self) -> bool { self & LOOSE_MASK == LOOSE_MASK }
    pub(crate) fn as_loosened(self) -> Ins { self | LOOSE_MASK }
    pub(crate) fn with_loosened(self, loosened: bool) -> Ins { if loosened { self.as_loosened() } else { self } }
    pub fn with_ins_index(self, number: usize) -> Ins { self | Ins((number << 9) as InsType) }
    pub fn get_ins_index(self) -> usize { ((self & INS_NUMBER_MASK).0 >> 9) as usize }
    pub fn with_method_index(self, number: usize) -> Ins { self | Ins((number << 13) as InsType) }
    pub fn get_method_index(self) -> usize { ((self & INS_METHOD_MASK).0 >> 13) as usize }
}

pub(crate) fn with_conds(red: bool, green: bool, blue: bool) -> Ins {
    Ins(((red as u8) << 5 | (green as u8) << 6 | (blue as u8) << 7) as InsType)
}

const INS_METHOD_MASK: Ins = Ins(0b1110000000000000);
const INS_NUMBER_MASK: Ins = Ins(0b0001111000000000);
const LOOSE_MASK: Ins = Ins(0b0000000100000000);
const VANILLA_MASK: Ins = Ins(0b0000000011111111);

impl From<Ins> for u8 {
    fn from(ins: Ins) -> Self {
        ins.0 as u8
    }
}

impl From<u8> for Ins {
    fn from(val: u8) -> Self {
        Ins(val as InsType)
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
        let background = match self.get_cond() {
            GRAY_COND => Color::BrightBlack,
            RED_COND => Color::Red,
            GREEN_COND => Color::Green,
            BLUE_COND => Color::Blue,
            _ => Color::Black,
        };
        let ins = self.get_ins();
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
            NOP => match self.as_vanilla() {
                RED_PROBE | GREEN_PROBE | BLUE_PROBE => "_".to_string(),
                _ => " ".to_string(),
            }
            _ => " ".to_string(),
        };
        write!(f, "{}", string.color(foreground).on_color(background))
    }
}

impl Debug for Ins {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
//        write!(f, "{:016b} ", self.0)?;
//        write!(f, "{}{}", self.get_method_number(), self.get_ins_index())?;
//        if self.is_branched() { write!(f, "V")?; }
//        write!(f, "_")?;
        let ins = self.as_vanilla();
        if ins == NOP {
            write!(f, "NOP")
        } else if ins == HALT {
            write!(f, "HALT")
        } else {
            write!(f, "{}", match ins.get_cond() {
                RED_COND => "RED_",
                GREEN_COND => "GREEN_",
                BLUE_COND => "BLUE_",
                _ => "",
            })?;
            write!(f, "{}", match ins.get_ins() {
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

