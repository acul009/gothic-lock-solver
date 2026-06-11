use std::collections::VecDeque;

use crate::lock::{
    Direction, Links, Lock, Move, SLICE_MIDDLE, SLICE_POSITIONS, Solution, SolveError,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct LockState(u32);

impl LockState {
    fn new(lock: &Lock) -> Self {
        let mut mult = 1;
        let mut num = 0;
        for slice in &lock.slices {
            num += (*slice as u32) * mult;
            mult *= SLICE_POSITIONS as u32;
        }

        LockState(num)
    }

    fn solved(slices: usize) -> Self {
        let mut mult: u32 = 1;
        let mut num: u32 = 0;
        for _ in 0..slices {
            num += SLICE_MIDDLE as u32 * mult;
            mult *= SLICE_POSITIONS as u32;
        }
        LockState(num)
    }

    fn possible_states(slices: usize) -> usize {
        (SLICE_POSITIONS as usize).pow(slices as u32)
    }

    fn apply_move(&self, m: &Move, links: &Links) -> Option<LockState> {
        let mut new = self.clone();

        if new.move_without_linked(m).is_err() {
            return None;
        }

        for linked in 0..links.size() {
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
        let divider = (SLICE_POSITIONS as u32).pow(m.slice as u32);
        let remainder = self.0 % (divider * SLICE_POSITIONS as u32);
        let slice = remainder / divider;
        match m.direction {
            Direction::Left => {
                if slice == 0 {
                    return Err(());
                }
                self.0 -= divider;
            }
            Direction::Right => {
                if slice >= SLICE_POSITIONS as u32 - 1 {
                    return Err(());
                }
                self.0 += divider;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Step {
    previous: LockState,
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

        let solved = LockState::solved(self.lock.size());

        if initial_state == solved {
            return Solution {
                moves: Ok(Vec::new()),
            };
        }

        let mut state_map = vec![None::<Step>; LockState::possible_states(self.lock.size())];
        state_map[initial_state.0 as usize] = Some(Step {
            previous: initial_state,
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
            for m in &moves {
                if let Some(new_state) = state.apply_move(m, &self.lock.links) {
                    if state_map[new_state.0 as usize].is_some() {
                        // println!("Already found state {}", new_state_num);
                        continue;
                    }
                    combinations_found += 1;

                    state_map[new_state.0 as usize] = Some(Step {
                        previous: state,
                        m: m.clone(),
                    });

                    if new_state == solved {
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

        if state_map[solved.0 as usize].is_none() {
            println!("No path found!");
            return Solution {
                moves: Err(SolveError::Impossible),
            };
        }

        let mut moves = Vec::new();
        let mut state = solved;
        while let Some(step) = &state_map[state.0 as usize] {
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
