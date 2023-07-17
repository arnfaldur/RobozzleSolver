use std::cmp::max;
use std::cmp::min;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

use crate::solver::backtrack::Frame;

use super::*;

impl Display for Source {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{")?;
        for i in 0..self.len() {
            let mut separate = false;
            for &instruction in self[i].iter() {
                if instruction == HALT {
                    write!(f, "|")?;
                //                    break;
                } else if instruction == NOP {
                    write!(f, "_")?;
                } else if instruction != NOP {
                    write!(f, "{}", instruction)?;
                }
                separate = true;
            }
            if separate {
                write!(f, ",")?;
            }
        }
        write!(f, "}}")
    }
}
impl Debug for Source {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{")?;
        for i in 0..self.len() {
            let mut separate = false;
            for &instruction in self[i].iter() {
                if instruction == HALT {
                    write!(f, "|")?;
                //                    break;
                } else if instruction == NOP {
                    write!(f, "_")?;
                } else if instruction != NOP {
                    write!(f, "{}", instruction)?;
                }
                separate = true;
            }
            if separate {
                write!(f, ",")?;
            }
        }
        write!(f, "}}")
    }
}

impl Display for Puzzle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "stars: {}", self.stars,)?;
        write!(f, "{{")?;
        for &me in self.methods.iter() {
            if me > 0 {
                for i in 0..me {
                    write!(f, "_")?;
                }
                write!(f, ",")?;
            }
        }
        write!(f, "}}\nconditions: ")?;
        if self.red {
            write!(f, "{}", RED_PROBE)?;
        }
        if self.green {
            write!(f, "{}", GREEN_PROBE)?;
        }
        if self.blue {
            write!(f, "{}", BLUE_PROBE)?;
        }
        write!(f, "\nmarks: ")?;
        if self.marks[0] {
            write!(f, "{}", MARK_RED)?;
        }
        if self.marks[1] {
            write!(f, "{}", MARK_GREEN)?;
        }
        if self.marks[2] {
            write!(f, "{}", MARK_BLUE)?;
        }
        writeln!(f, "\nmap:")?;
        let (mut miny, mut minx, mut maxy, mut maxx) = (14, 18, 0, 0);
        for y in 1..13 {
            for x in 1..17 {
                if self.map.0[y][x] != _N {
                    miny = min(miny, y);
                    minx = min(minx, x);
                    maxy = max(maxy, y + 1);
                    maxx = max(maxx, x + 1);
                }
            }
        }
        for y in miny..maxy {
            for x in minx..maxx {
                let tile = self.map.0[y][x];
                let string = if self.y == y && self.x == x {
                    match self.direction {
                        Direction::Up => "↑",
                        Direction::Left => "←",
                        Direction::Down => "↓",
                        Direction::Right => "→",
                    }
                } else {
                    "★"
                };
                let background = match tile.color() {
                    RE => Color::Red,
                    GE => Color::Green,
                    BE => Color::Blue,
                    _ => Color::Black,
                };
                let foreground = if self.y == y && self.x == x {
                    Color::BrightWhite
                } else if tile.has_star() {
                    Color::Yellow
                } else {
                    background
                };
                write!(f, "{}", string.color(foreground).on_color(background))?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self)
    }
}
impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{{")?;
        writeln!(f, "\nmap:")?;
        let (mut miny, mut minx, mut maxy, mut maxx) = (14, 18, 0, 0);
        for y in 1..13 {
            for x in 1..17 {
                if self.0[y][x] != _N {
                    miny = min(miny, y);
                    minx = min(minx, x);
                    maxy = max(maxy, y + 1);
                    maxx = max(maxx, x + 1);
                }
            }
        }
        for y in miny..maxy {
            for x in minx..maxx {
                let tile = self.0[y][x];
                let string = "★";
                let background = match tile.color() {
                    RE => Color::Red,
                    GE => Color::Green,
                    BE => Color::Blue,
                    _ => Color::Black,
                };
                let foreground = if tile.has_star() {
                    Color::Yellow
                } else {
                    background
                };
                write!(f, "{}", string.color(foreground).on_color(background))?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "At ({}, {}), stars: {}, Running: {}\nMap:\n",
            self.x,
            self.y,
            self.stars,
            self.running()
        )?;
        let (mut miny, mut minx, mut maxy, mut maxx) = (14, 18, 0, 0);
        for y in 1..13 {
            for x in 1..17 {
                if self.map.0[y][x] != _N {
                    miny = min(miny, y);
                    minx = min(minx, x);
                    maxy = max(maxy, y + 1);
                    maxx = max(maxx, x + 1);
                }
            }
        }
        for y in miny..maxy {
            for x in minx..maxx {
                let tile = self.map.0[y][x];
                let string = if self.y == y && self.x == x {
                    match self.direction {
                        Direction::Up => "↑",
                        Direction::Left => "←",
                        Direction::Down => "↓",
                        Direction::Right => "→",
                    }
                } else {
                    "★"
                };
                let background = match tile.color() {
                    RE => Color::Red,
                    GE => Color::Green,
                    BE => Color::Blue,
                    _ => Color::Black,
                };
                let foreground = if self.y == y && self.x == x {
                    Color::BrightWhite
                } else if tile.has_star() {
                    Color::Yellow
                } else {
                    background
                };
                write!(f, "{}", string.color(foreground).on_color(background))?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self)
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Frame:");
        writeln!(f, "Candidate: {}", self.candidate);
        write!(f, "State: {}", self.state);
        write!(f, "Score: {}, Max score: {}", self.score, self.max_score)
        //write!(f, "{}[2J", 27 as char) // control character to clear screen
    }
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("candidate", &self.candidate)
            .field("state", &self.state)
            .field("score", &self.score)
            .field("max_score", &self.max_score)
            .finish()
    }
}
