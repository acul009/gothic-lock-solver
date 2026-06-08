use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
};

pub struct Lock {
    pub slices: Vec<Slice>,
}

impl Lock {
    pub fn new(slice_count: usize) -> Lock {
        Lock {
            slices: vec![Slice::new(); slice_count],
        }
    }

    pub fn size(&self) -> usize {
        self.slices.len()
    }
}

pub const SLICE_POSITIONS: u8 = 7;

#[derive(Clone)]
pub struct Slice {
    pub start: u8,
    pub target: u8,
    pub linked: BTreeMap<usize, Direction>,
}

impl Slice {
    pub fn new() -> Slice {
        Slice {
            start: 1,
            target: 1,
            linked: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Same,
    Opposite,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Same => write!(f, "Same direction"),
            Direction::Opposite => write!(f, "Opposite direction"),
        }
    }
}
