use std::collections::VecDeque;

use crate::lock::{
    Direction, Links, Lock, Move, SLICE_MIDDLE, SLICE_POSITIONS, Solution, SolveError,
};

#[derive(Clone, Hash, PartialEq, Eq)]
struct LockState {
    num: usize,
    slices: usize,
}

impl LockState {
    fn new(lock: &Lock) -> Self {
        let mut mult = 1;
        let mut num = 0;
        for slice in &lock.slices {
            num += (*slice as usize) * mult;
            mult *= SLICE_POSITIONS as usize;
        }

        LockState {
            num,
            slices: lock.size(),
        }
    }

    fn solved(slices: usize) -> Self {
        let mut mult: usize = 1;
        let mut num: usize = 0;
        for _ in 0..slices {
            num += SLICE_MIDDLE as usize * mult;
            mult *= SLICE_POSITIONS as usize;
        }
        LockState { num, slices }
    }

    fn to_number(&self) -> usize {
        self.num
    }

    fn possible_states(&self) -> usize {
        (SLICE_POSITIONS as usize).pow(self.slices as u32)
    }

    fn apply_move(&self, m: &Move, links: &Links) -> Option<LockState> {
        let mut new = self.clone();

        if new.move_without_linked(m).is_err() {
            return None;
        }

        for linked in 0..self.slices {
            if let Some(linked_direction) = links.link(m.slice, linked).apply(m.direction) {
                let linked_move = Move {
                    direction: linked_direction,
                    slice: linked,
                };
                if new.move_without_linked(&linked_move).is_err() {
                    return None;
                }
            }
        }

        Some(new)
    }

    fn move_without_linked(&mut self, m: &Move) -> Result<(), ()> {
        let divider = (SLICE_POSITIONS as usize).pow(m.slice as u32);
        let remainder = self.num % (divider * SLICE_POSITIONS as usize);
        let slice = remainder / divider;
        match m.direction {
            Direction::Left => {
                if slice == 0 {
                    return Err(());
                }
                self.num -= divider;
            }
            Direction::Right => {
                if slice >= SLICE_POSITIONS as usize - 1 {
                    return Err(());
                }
                self.num += divider;
            }
        }
        Ok(())
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
        let start = std::time::Instant::now();
        let initial_state = LockState::new(&self.lock);

        let solved = LockState::solved(self.lock.size()).to_number();

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
                        break 'outer;
                    }

                    queue.push_back(new_state);
                }
            }
        }

        let time = std::time::Instant::now().duration_since(start);
        println!(
            "Done! Checked {} combinations in {:?}.",
            combinations_found, time
        );

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
