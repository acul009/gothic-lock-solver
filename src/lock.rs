use std::fmt::Display;

mod brute_force;

#[derive(Clone)]
pub struct Lock {
    pub slices: Vec<u8>,
    pub links: Links,
}

impl Lock {
    pub fn new(slice_count: usize) -> Lock {
        Lock {
            slices: vec![SLICE_MIDDLE; slice_count],
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

#[derive(Clone)]
pub struct Slice {
    pub start: u8,
}

impl Slice {
    pub fn new() -> Slice {
        Slice { start: 1 }
    }
}

#[derive(Clone)]
pub struct Links {
    size: usize,
    links: Vec<Link>,
}
impl Links {
    fn new(slice_count: usize) -> Self {
        Self {
            size: slice_count,
            links: vec![Link::None; slice_count * slice_count],
        }
    }

    pub fn links_from(&self, index: usize) -> Vec<(usize, Link)> {
        let start = index * self.size;
        self.links[start..start + self.size]
            .iter()
            .cloned()
            .enumerate()
            .collect()
    }

    pub fn links_to(&self, index: usize) -> Vec<(usize, Link)> {
        let mut links = Vec::with_capacity(self.size);
        for i in 0..self.size {
            links.push(self.links[i * self.size + index]);
        }
        links.into_iter().enumerate().collect()
    }

    pub fn link(&self, from: usize, to: usize) -> Link {
        self.links[from * self.size + to]
    }

    pub fn cycle_link(&mut self, from: usize, to: usize) {
        self.links[from * self.size + to].cycle();
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Link {
    None,
    Same,
    Opposite,
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
    pub slice: usize,
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
