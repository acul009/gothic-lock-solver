use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Display,
};

#[derive(Clone)]
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

    pub fn solve(&self) -> Solution {
        Solution {
            graph: DependencyGraph::from_lock(self),
        }
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

pub struct DependencyGraph {
    affected_by: BTreeMap<usize, BTreeSet<usize>>,
    affects: BTreeMap<usize, BTreeSet<usize>>,
    pub solve_order: Vec<usize>,
}

impl DependencyGraph {
    fn from_lock(lock: &Lock) -> DependencyGraph {
        let mut affected_by = BTreeMap::<usize, BTreeSet<usize>>::new();
        let mut affects = BTreeMap::<usize, BTreeSet<usize>>::new();

        for index in 0..lock.size() {
            affected_by.insert(index, BTreeSet::new());
            affects.insert(index, BTreeSet::new());
        }

        for (index, slice) in lock.slices.iter().enumerate() {
            for linked_index in slice.linked.keys() {
                affected_by.entry(*linked_index).or_default().insert(index);
                affects.entry(index).or_default().insert(*linked_index);
            }
        }

        let mut solve_order = Vec::new();
        let mut affected = affected_by.clone();

        for (index, deps) in affected_by.iter() {
            println!("{} affected by {:?}", index, deps);
        }

        while !affected.is_empty() {
            let next = {
                let mut next = affected.iter().next().unwrap().clone();
                for possibility in affected.iter() {
                    if possibility.1.len() < next.1.len() {
                        next = possibility.clone();
                    }
                }
                next.0.clone()
            };

            solve_order.push(next);
            affected.remove(&next);
            for deps in affected.iter_mut() {
                deps.1.remove(&next);
            }
        }

        DependencyGraph {
            affected_by,
            affects,
            solve_order,
        }
    }
}

pub struct Solution {
    pub graph: DependencyGraph,
}
