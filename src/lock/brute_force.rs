use std::collections::VecDeque;

use crate::lock::{
    Direction, Links, Lock, Move, SLICE_MIDDLE, SLICE_POSITIONS, Solution, SolveError,
};

#[derive(Clone, Hash, PartialEq, Eq)]
struct LockState {
    slices: Vec<u8>,
}

impl LockState {
    fn new(lock: &Lock) -> Self {
        LockState {
            slices: lock.slices.clone(),
        }
    }

    fn to_number(&self) -> usize {
        let mut mult = 1;
        let mut num = 0;
        for slice in &self.slices {
            num += (*slice as usize) * mult;
            mult *= SLICE_POSITIONS as usize;
        }
        num
    }

    fn possible_states(&self) -> usize {
        (SLICE_POSITIONS as usize).pow(self.slices.len() as u32)
    }

    fn check_clearance(&self, slice: usize, direction: Direction) -> bool {
        let slice = self.slices[slice];
        match direction {
            Direction::Left => slice > 0,
            Direction::Right => slice + 1 < SLICE_POSITIONS,
        }
    }

    fn apply_move(&self, m: &Move, links: &Links) -> Option<LockState> {
        let mut new = self.clone();
        if !new.check_clearance(m.slice, m.direction) {
            return None;
        }

        match m.direction {
            Direction::Left => {
                new.slices[m.slice] -= 1;
            }
            Direction::Right => {
                new.slices[m.slice] += 1;
            }
        }

        for linked in 0..self.slices.len() {
            if let Some(linked_direction) = links.link(m.slice, linked).apply(m.direction) {
                if linked == m.slice {
                    panic!("Slice linked to itself");
                }
                // try cascading slices. If the linked slice has no clearance, we'll try to move it the opposite way first.
                if !self.check_clearance(linked, linked_direction) {
                    return None;
                }

                match linked_direction {
                    Direction::Left => {
                        new.slices[linked] -= 1;
                    }
                    Direction::Right => {
                        new.slices[linked] += 1;
                    }
                }
            }
        }

        Some(new)
    }
}

#[derive(Clone)]
pub struct Step {
    previous: usize,
    m: Move,
}

pub struct Solver {
    lock: Lock,
}

impl Solver {
    pub(super) fn new(lock: Lock) -> Self {
        Solver { lock }
    }

    pub(super) fn solve(self) -> Solution {
        let initial_state = LockState::new(&self.lock);
        println!("initial state: {}", initial_state.to_number());

        let solved = LockState {
            slices: vec![SLICE_MIDDLE; self.lock.size()],
        }
        .to_number();
        println!("Solved state: {}", solved);

        if initial_state.to_number() == solved {
            return Solution {
                moves: Ok(Vec::new()),
            };
        }

        let mut state_map = vec![None::<Step>; initial_state.possible_states()];
        state_map[initial_state.to_number()] = Some(Step {
            previous: initial_state.to_number(),
            m: Move {
                direction: Direction::Left,
                slice: 0,
            },
        });

        let mut queue = VecDeque::new();
        queue.push_back(initial_state);

        let mut moves = Vec::new();
        for slice in 0..self.lock.size() {
            moves.push(Move {
                direction: Direction::Left,
                slice,
            });
            moves.push(Move {
                direction: Direction::Right,
                slice,
            });
        }

        let mut combinations_found = 0;

        'outer: while let Some(state) = queue.pop_front() {
            let old_state_num = state.to_number();
            for m in &moves {
                if let Some(new_state) = state.apply_move(m, &self.lock.links) {
                    let new_state_num = new_state.to_number();
                    if state_map[new_state_num].is_some() {
                        // println!("Already found state {}", new_state_num);
                        continue;
                    }
                    combinations_found += 1;

                    state_map[new_state_num] = Some(Step {
                        previous: old_state_num,
                        m: m.clone(),
                    });

                    if new_state_num == solved {
                        println!("Found path to solution!");
                        break 'outer;
                    }

                    queue.push_back(new_state);
                }
            }
        }

        println!("Done! Checked {} combinations.", combinations_found);

        println!("Looking for path to solution...");

        if state_map[solved].is_none() {
            println!("No path found!");
            return Solution {
                moves: Err(SolveError::Impossible),
            };
        }

        let mut moves = Vec::new();
        let mut state = solved;
        while let Some(step) = &state_map[state] {
            // println!("previous of {} is {}", state, step.previous);
            if state == step.previous {
                break;
            }
            moves.push(step.m);
            state = step.previous;
        }
        moves.reverse();

        println!("Found path of length {}", moves.len());

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
            moves: Ok(compacted),
        }
    }
}
