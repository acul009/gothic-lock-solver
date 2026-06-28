use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

mod brute_force;

#[derive(Clone, Serialize, Deserialize)]
pub struct Lock {
    pub slices: Vec<u8>,
    pub links: Links,
}

impl Lock {
    pub fn new(slice_count: u8) -> Lock {
        Lock {
            slices: vec![SLICE_MIDDLE; slice_count as usize],
            links: Links::new(slice_count),
        }
    }

    pub fn size(&self) -> usize {
        self.slices.len()
    }

    pub fn solve(&self) -> Solution {
        brute_force::Solver::new(self.clone()).solve()
    }
}

pub const SLICE_POSITIONS: u8 = 7;
pub const SLICE_MIDDLE: u8 = SLICE_POSITIONS / 2;
pub const MAX_SLICES: u8 = 7;

#[derive(Clone)]
pub struct Slice {
    pub start: u8,
}

impl Slice {
    pub fn new() -> Slice {
        Slice { start: 1 }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Links {
    size: u8,
    links: Vec<Link>,
}
impl Links {
    fn new(slice_count: u8) -> Self {
        let mut links = vec![Link::None; slice_count as usize * slice_count as usize];
        for i in 0..slice_count {
            links[i as usize * slice_count as usize + i as usize] = Link::Same;
        }
        Self {
            size: slice_count,
            links,
        }
    }

    pub fn links_from(&self, index: u8) -> Vec<(usize, Link)> {
        let start = index * self.size;
        self.links[start as usize..start as usize + self.size as usize]
            .iter()
            .cloned()
            .enumerate()
            .collect()
    }

    pub fn links_to(&self, index: u8) -> Vec<(usize, Link)> {
        let mut links = Vec::with_capacity(self.size as usize);
        for i in 0..self.size {
            links.push(self.links[i as usize * self.size as usize + index as usize]);
        }
        links.into_iter().enumerate().collect()
    }

    pub fn link(&self, from: u8, to: u8) -> Link {
        self.links[from as usize * self.size as usize + to as usize]
    }

    pub fn cycle_link(&mut self, from: u8, to: u8) {
        self.links[from as usize * self.size as usize + to as usize].cycle();
    }

    pub fn size(&self) -> u8 {
        self.size
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum Link {
    None = 0,
    Same = 1,
    Opposite = -1,
}

impl Link {
    fn cycle(&mut self) {
        match self {
            Self::None => *self = Self::Same,
            Self::Same => *self = Self::Opposite,
            Self::Opposite => *self = Self::None,
        }
    }

    fn apply(&self, direction: Direction) -> Option<Direction> {
        match self {
            Link::None => None,
            Link::Same => Some(direction),
            Link::Opposite => Some(direction.opposite()),
        }
    }
}

pub struct Solution {
    pub moves: Result<Vec<(Move, usize)>, SolveError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
        }
    }
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub slice: u8,
    pub direction: Direction,
}

pub enum SolveError {
    NonTrivial,
    CascadingSliceStuck,
    ToManyMoves,
    Impossible,
}

impl Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolveError::NonTrivial => write!(f, "Non-trivial lock"),
            SolveError::CascadingSliceStuck => write!(f, "Cascading slice stuck"),
            SolveError::ToManyMoves => write!(f, "To many moves, probably unsolvable"),
            SolveError::Impossible => write!(f, "Impossible to solve"),
        }
    }
}
