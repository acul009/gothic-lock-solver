use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Display,
};

#[derive(Clone)]
pub struct Lock {
    pub slices: Vec<u8>,
    pub links: Links,
}

impl Lock {
    pub fn new(slice_count: usize) -> Lock {
        Lock {
            slices: vec![0u8; slice_count],
            links: Links::new(slice_count),
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

    pub fn links_from(&self, index: usize) -> Vec<&Link> {
        let start = index * self.size;
        self.links[start..start + self.size].iter().collect()
    }

    pub fn links_to(&self, index: usize) -> Vec<&Link> {
        let mut links = Vec::with_capacity(self.size);
        for i in 0..self.size {
            links.push(&self.links[i * self.size + index]);
        }
        links
    }

    pub fn cycle_link(&mut self, from: usize, to: usize) {
        self.links[from * self.size + to].cycle();
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
            affected_by.insert(
                index,
                lock.links
                    .links_to(index)
                    .iter()
                    .enumerate()
                    .filter_map(|(linked, link)| {
                        if let Link::None = link {
                            None
                        } else {
                            Some(linked)
                        }
                    })
                    .collect(),
            );
            affects.insert(
                index,
                lock.links
                    .links_from(index)
                    .iter()
                    .enumerate()
                    .filter_map(|(linked, link)| {
                        if let Link::None = link {
                            None
                        } else {
                            Some(linked)
                        }
                    })
                    .collect(),
            );
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
