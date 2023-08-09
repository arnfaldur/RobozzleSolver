use std::{
    collections::{HashSet, VecDeque},
    fmt::{Debug, Display},
};

use colored::{Color, Colorize};
use serde::{Deserialize, Serialize};

use crate::constants::*;

use super::{instructions::Ins, Direction, Map, Tile};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct Board {
    pub map: Map,
    pub direction: Direction,
    pub x: usize,
    pub y: usize,
}

impl Board {
    pub(crate) fn clear_star(&mut self) {
        self.map.0[self.y][self.x].clear_star();
    }
    pub(crate) fn touch(&mut self) {
        self.map.0[self.y][self.x].touch();
    }
    pub(crate) fn untouch(&mut self) {
        self.map.0[self.y][self.x].untouch();
    }
    pub(crate) fn touches(&self) -> usize {
        self.map.0[self.y][self.x].touches()
    }
    pub fn max_touches(&self) -> usize {
        self.map
            .0
            .iter()
            .map(|r| r.iter().map(|e| e.touches()).max())
            .max()
            .flatten()
            .expect("the map should have elements")
    }
    pub(crate) fn mark(&mut self, ins: Ins) {
        self.map.0[self.y][self.x].mark(ins)
    }
    pub(crate) fn current_tile(&self) -> &Tile {
        &self.map.0[self.y][self.x]
    }
    pub(crate) fn current_tile_mut(&mut self) -> &mut Tile {
        &mut self.map.0[self.y][self.x]
    }
    pub fn count_tiles(&self) -> usize {
        let mut tiles = 0;
        // test which colors are reachable
        let mut frontier = VecDeque::new();
        frontier.push_front((self.x, self.y));
        let mut visited = HashSet::new();
        while let Some((x, y)) = frontier.pop_back() {
            for (dx, dy) in &[(1, 0), (0, 1), (-1, 0), (0, -1)] {
                let (nx, ny) = ((x as isize + dx) as usize, (y as isize + dy) as usize);
                if self.map.0[ny][nx] != _N && !visited.contains(&(nx, ny)) {
                    visited.insert((nx, ny));
                    frontier.push_front((nx, ny));
                    tiles += 1;
                }
            }
        }
        return tiles;
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            map: Map([[_N; 18]; 14]),
            direction: Direction::Up,
            x: 1,
            y: 1,
        }
    }
}

#[allow(non_snake_case)]
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Map:")?;
        let (mut miny, mut minx, mut maxy, mut maxx) = (14, 18, 0, 0);
        for y in 1..13 {
            for x in 1..17 {
                if self.map.0[y][x] != _N {
                    miny = miny.min(y);
                    minx = minx.min(x);
                    maxy = maxy.max(y + 1);
                    maxx = maxx.max(x + 1);
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

impl Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(
        //     f,
        //     "At ({}, {})\nMap:\n",
        //     self.x,
        //     self.y,
        // )?;
        f.debug_struct("Board")
            .field("map", &self.map)
            .field("direction", &self.direction)
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
