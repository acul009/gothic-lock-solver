use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

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
        let graph = DependencyGraph::from_lock(self);
        brute_force::Solver::new(graph, self.clone()).solve()
    }
}

pub const SLICE_POSITIONS: u8 = 7;
pub const SLICE_MIDDLE: u8 = SLICE_POSITIONS / 2;

const MOVE_LIMIT: usize = 100;

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

pub struct DependencyGraph {
    pub solve_order: Vec<usize>,
    pub trivial: bool,
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
                    .into_iter()
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
                    .into_iter()
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

        let mut trivial = true;

        while !affected.is_empty() {
            let next = {
                let mut next = affected.iter().next().unwrap().clone();
                for possibility in affected.iter() {
                    if possibility.1.len() < next.1.len() {
                        next = possibility.clone();
                    }
                }
                if next.1.len() > 0 {
                    trivial = false;
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
            solve_order,
            trivial,
        }
    }
}

pub struct Solution {
    pub graph: DependencyGraph,
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

pub struct Solver {
    graph: DependencyGraph,
    lock: Lock,
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

impl Solver {
    fn new(graph: DependencyGraph, lock: Lock) -> Self {
        Solver { graph, lock }
    }

    fn solve(mut self) -> Solution {
        if !self.graph.trivial {
            return Solution {
                graph: self.graph,
                moves: Err(SolveError::NonTrivial),
            };
        }

        let mut moves = Vec::with_capacity(MOVE_LIMIT * self.lock.size());

        for slice in self.graph.solve_order.clone() {
            if let Err(e) = self.solve_slice(slice, &mut moves) {
                return Solution {
                    graph: self.graph,
                    moves: Err(e),
                };
            }
        }

        let mut compacted = Vec::<(Move, usize)>::new();
        for m in moves {
            if let Some(last) = compacted.last_mut() {
                if last.0 == m {
                    last.1 += 1;
                    continue;
                }
            }
            compacted.push((m, 1));
        }

        Solution {
            graph: self.graph,
            moves: Ok(compacted),
        }
    }

    fn solve_slice(&mut self, slice: usize, moves: &mut Vec<Move>) -> Result<(), SolveError> {
        println!("Solving slice {}", slice);

        while self.lock.slices[slice] != SLICE_MIDDLE {
            println!(
                "Slice {} is at {}, but should be at {}",
                slice, self.lock.slices[slice], SLICE_MIDDLE
            );
            if self.lock.slices[slice] > SLICE_MIDDLE {
                println!("Moving slice {} left to solve it", slice);
                self.move_slice(slice, Direction::Left, moves)?;
            } else {
                println!("Moving slice {} right to solve it", slice);
                self.move_slice(slice, Direction::Right, moves)?;
            }
        }

        println!("Slice {} solved", slice);

        Ok(())
    }

    fn move_slice(
        &mut self,
        slice: usize,
        direction: Direction,
        moves: &mut Vec<Move>,
    ) -> Result<(), SolveError> {
        println!("Moving slice {} {} recursively ", slice, direction);
        if !self.check_clearance(slice, direction) {
            return Err(SolveError::CascadingSliceStuck);
        }

        if moves.len() > MOVE_LIMIT {
            return Err(SolveError::ToManyMoves);
        }

        for (linked, link) in self.lock.links.links_from(slice) {
            if let Some(linked_direction) = link.apply(direction) {
                if linked == slice {
                    panic!("Slice linked to itself");
                }
                // try cascading slices. If the linked slice has no clearance, we'll try to move it the opposite way first.
                if !self.check_clearance(linked, linked_direction) {
                    println!("No clearance for linked slice, trying to compensate");
                    self.move_slice(linked, linked_direction.opposite(), moves)?;
                }

                match linked_direction {
                    Direction::Left => {
                        self.lock.slices[linked] -= 1;
                    }
                    Direction::Right => {
                        self.lock.slices[linked] += 1;
                    }
                }
            }
        }
        moves.push(Move { slice, direction });
        match direction {
            Direction::Left => {
                self.lock.slices[slice] -= 1;
            }
            Direction::Right => {
                self.lock.slices[slice] += 1;
            }
        }

        Ok(())
    }

    fn check_clearance(&self, slice: usize, direction: Direction) -> bool {
        let slice = self.lock.slices[slice];
        match direction {
            Direction::Left => slice > 0,
            Direction::Right => slice + 1 < SLICE_POSITIONS,
        }
    }
}
