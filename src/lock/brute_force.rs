use std::{collections::VecDeque, u32};

use crate::lock::{Direction, Links, Lock, MAX_SLICES, Move, SLICE_MIDDLE, Solution, SolveError};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct LockState(u32);

const SLICE_BIT_WIDTH: usize = 3;

impl LockState {
    fn new(lock: &Lock) -> Self {
        let mut num = Self::SOLVED.0;
        for (index, slice) in lock.slices.iter().enumerate() {
            num &= (0b111 << (index * SLICE_BIT_WIDTH)) ^ u32::MAX;
            num |= (*slice as u32) << (index * SLICE_BIT_WIDTH);
        }

        LockState(num)
    }

    const SOLVED: Self = {
        let mut num: u32 = 0;
        let mut index = 0;
        while index < MAX_SLICES as usize {
            num += (SLICE_MIDDLE as u32) << (index * SLICE_BIT_WIDTH);
            index += 1;
        }
        LockState(num)
    };

    fn possible_states(slices: usize) -> usize {
        1 << (slices * SLICE_BIT_WIDTH + 1)
    }

    fn index(&self, slices: usize) -> usize {
        let mask = (1u32 << slices * SLICE_BIT_WIDTH) - 1;
        (self.0 & mask) as usize
    }

    const CHECK_ALL_BOUNDS_MASK: u32 = {
        let mut num: u32 = 0;
        let mut index = 0;
        while index < MAX_SLICES as usize {
            num += (0b001 as u32) << (index * SLICE_BIT_WIDTH);
            index += 1;
        }
        num
    };

    const CHECK_UNDERFLOW_MASK: u32 = {
        let mut num: u32 = 0;
        let mut index = 0;
        while index <= MAX_SLICES as usize {
            num += (0b001 as u32) << (index * SLICE_BIT_WIDTH);
            index += 1;
        }
        num
    };

    fn apply_move(&self, m: &PrecomputedMove) -> Option<Self> {
        let raw = self.0;
        let new = raw.wrapping_add(m.delta);

        // println!("current: {:b}", raw);
        // println!("upper_bound_check: {:b}", m.upper_bound_check);
        // println!("lower_bound_check: {:b}", m.lower_bound_check);
        // println!("delta: {:b}", m.delta);

        // Newer lower bound logic
        let changed_bits = raw ^ new;
        let changed_slices = changed_bits & Self::CHECK_UNDERFLOW_MASK;
        if changed_slices != m.slice_moves {
            return None;
        }

        // let second_new = new.wrapping_add(Self::CHECK_ALL_BOUNDS_MASK);
        // let changed_bits = new ^ second_new;
        // let changed_slices = changed_bits & Self::CHECK_UNDERFLOW_MASK;
        // if changed_slices != m.slice_moves {
        //     return None;
        // }

        // Check upper bound
        let highest_bit = new & m.upper_bound_check;
        let middle_bit = (new << 1) & m.upper_bound_check;
        let lowest_bit = (new << 2) & m.upper_bound_check;
        if highest_bit & middle_bit & lowest_bit > 0 {
            // panic!("higher bound hit");
            return None;
        }

        // check lower bound
        // let highest_bit = (raw >> 2) & m.lower_bound_check;
        // let middle_bit = (raw >> 1) & m.lower_bound_check;
        // let lowest_bit = (raw) & m.lower_bound_check;
        // let non_zero_flags = highest_bit | middle_bit | lowest_bit;
        // let filler = Self::CHECK_ALL_BOUNDS_MASK ^ m.lower_bound_check;
        // let compare_to = non_zero_flags | filler;
        // // println!("highest_bit: {:b}", highest_bit);
        // // println!("middle_bit: {:b}", middle_bit);
        // // println!("lowest_bit: {:b}", lowest_bit);
        // // println!("non_zero_flags: {:b}", non_zero_flags);
        // // println!("filler: {:b}", filler);
        // // println!("compare_to:            {:b}", compare_to);
        // // println!("CHECK_ALL_BOUNDS_MASK: {:b}", Self::CHECK_ALL_BOUNDS_MASK);
        // if compare_to != Self::CHECK_ALL_BOUNDS_MASK {
        //     println!("raw:            {:b}", raw);
        //     println!("delta:          {:b}", m.delta);
        //     println!("new:            {:b}", new);
        //     println!("highest_bit:    {:b}", highest_bit);
        //     println!("middle_bit:     {:b}", middle_bit);
        //     println!("lowest_bit:     {:b}", lowest_bit);
        //     println!("non_zero_flags: {:b}", non_zero_flags);
        //     println!("filler:         {:b}", filler);
        //     println!("compare_to:     {:b}", compare_to);
        //     println!("BOUNDS_MASK:    {:b}", Self::CHECK_ALL_BOUNDS_MASK);
        //     println!("slice_moves:    {:b}", m.slice_moves);
        //     println!("lower_bound_check: {:b}", m.lower_bound_check);
        //     println!("changed_bits:   {:b}", changed_bits);
        //     println!("changed_slices: {:b}", changed_slices);
        //     println!("slice_moves:    {:b}", m.slice_moves);
        //     panic!("lower bound still hit");
        //     return None;
        // }

        Some(Self(new))
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct PackedMove(u8);

impl PackedMove {
    const EMPTY: Self = Self(0b1000_0000);
    const START: Self = Self(0b0100_0000);

    fn to_move(&self) -> Move {
        Move {
            slice: (self.0 & 0b0000_1110) >> 1,
            direction: match self.0 & 0b0000_0001 {
                0 => Direction::Left,
                _ => Direction::Right,
            },
        }
    }

    fn all(slices: u8) -> impl Iterator<Item = Self> {
        (0..(slices * 2)).map(|i| Self(i))
    }

    fn reverse(&self) -> Self {
        Self(self.0 ^ 0b0001)
    }
}

// Indexed via PackedMove
struct MoveMap([PrecomputedMove; MAX_SLICES as usize * 2]);

impl MoveMap {
    fn new(links: &Links) -> Self {
        let mut move_map = [PrecomputedMove::default(); MAX_SLICES as usize * 2];
        for packed_move in PackedMove::all(links.size()) {
            println!("generating packed move {:b}", packed_move.0);
            let m = packed_move.to_move();
            println!("{:?}", m);

            let mut delta = 0;
            let mut slice_moves = 0;
            let mut lower_bound_check = 0;
            let mut upper_bound_check = 0;

            for (target, link) in links.links_from(m.slice) {
                let operation = match link.apply(m.direction) {
                    None => continue,
                    Some(Direction::Left) => {
                        lower_bound_check |= 0b001 << (target * SLICE_BIT_WIDTH);
                        slice_moves |= 0b001 << (target * SLICE_BIT_WIDTH);
                        0u32.wrapping_sub(1)
                    }
                    Some(Direction::Right) => {
                        slice_moves |= 0b001 << (target * SLICE_BIT_WIDTH);
                        upper_bound_check |= 0b100 << (target * SLICE_BIT_WIDTH);
                        1
                    }
                };
                delta += operation << (target * SLICE_BIT_WIDTH);
            }

            move_map[packed_move.0 as usize] = PrecomputedMove {
                m: packed_move,
                delta,
                slice_moves,
                lower_bound_check,
                upper_bound_check,
            }
        }
        Self(move_map)
    }

    fn load(&self, m: &PackedMove) -> &PrecomputedMove {
        &self.0[m.0 as usize]
    }
}

#[derive(Clone, Default, Copy)]
struct PrecomputedMove {
    m: PackedMove,
    delta: u32,
    slice_moves: u32,
    // used to check for the left (lower) bound for each slice
    lower_bound_check: u32,
    // used to check for the right (upper) bound for each slice
    upper_bound_check: u32,
}

pub struct Solver {
    lock: Lock,
}

impl Solver {
    pub(super) fn new(lock: Lock) -> Self {
        Solver { lock }
    }

    pub(super) fn solve(self) -> Solution {
        println!("Lock slices: {:?}", self.lock.slices);
        let initial_state = LockState::new(&self.lock);
        println!("Initial state: {:b}", initial_state.0);
        let slices = self.lock.size();

        let solved_index = LockState::SOLVED.index(slices);

        if initial_state.index(slices) == solved_index {
            return Solution {
                moves: Ok(Vec::new()),
            };
        }

        let mut state_map = vec![PackedMove::EMPTY; LockState::possible_states(self.lock.size())];
        state_map[initial_state.index(slices)] = PackedMove::START;

        let mut queue = VecDeque::new();
        queue.push_back(initial_state);

        let move_map = MoveMap::new(&self.lock.links);

        println!("Starting state search");
        let start = std::time::Instant::now();

        let mut combinations_found = 0;

        'outer: while let Some(state) = queue.pop_front() {
            for m in move_map.0.iter().take(slices * 2) {
                match state_map[state.index(slices)] {
                    PackedMove::EMPTY => (),
                    PackedMove::START => (),
                    m => {
                        let m = &move_map.0[m.0 as usize];
                        if let Some(new_state) = state.apply_move(m) {
                            let index = new_state.index(slices);
                            if state_map[index] != PackedMove::EMPTY {
                                // println!("Already found state {}", new_state_num);
                                continue;
                            }
                            combinations_found += 1;

                            state_map[index] = m.m;

                            if index == solved_index {
                                break 'outer;
                            }

                            queue.push_back(new_state);
                        }
                    }
                }
                if let Some(new_state) = state.apply_move(m) {
                    let index = new_state.index(slices);
                    if state_map[index] != PackedMove::EMPTY {
                        // println!("Already found state {}", new_state_num);
                        continue;
                    }
                    combinations_found += 1;

                    state_map[index] = m.m;

                    if index == solved_index {
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

        if state_map[solved_index] == PackedMove::EMPTY {
            println!("No path found!");
            return Solution {
                moves: Err(SolveError::Impossible),
            };
        }

        let mut moves = Vec::new();
        let mut state = LockState::SOLVED;
        loop {
            let m = state_map[state.index(slices)];
            if m == PackedMove::START {
                break;
            }
            moves.push(m.to_move());
            state = state.apply_move(move_map.load(&m.reverse())).unwrap();
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
