use std::{collections::VecDeque, u32};

use iced::widget::operation;

use crate::lock::{
    Direction, Link, Links, Lock, MAX_SLICES, Move, SLICE_MIDDLE, SLICE_POSITIONS, Solution,
    SolveError,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct LockState(u32);

// needs to be one more than slice positions because of the move_without_linked implementation
const POWERS: [u32; SLICE_POSITIONS as usize + 1] = [
    (SLICE_POSITIONS as u32).pow(0),
    (SLICE_POSITIONS as u32).pow(1),
    (SLICE_POSITIONS as u32).pow(2),
    (SLICE_POSITIONS as u32).pow(3),
    (SLICE_POSITIONS as u32).pow(4),
    (SLICE_POSITIONS as u32).pow(5),
    (SLICE_POSITIONS as u32).pow(6),
    (SLICE_POSITIONS as u32).pow(7),
];

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

    const EMPTY: Self = LockState(1 >> 31);

    fn possible_states(slices: usize) -> usize {
        1 << (slices * SLICE_BIT_WIDTH + 1)
    }

    fn index(&self, slices: usize) -> usize {
        let mask = (1u32 << slices * SLICE_BIT_WIDTH) - 1;
        (self.0 & mask) as usize
    }

    const HAS_MOVE_BITMASK: u32 = 1 << 30;
    const MOVE_BITMASK: u32 = 0b0011_1100_0000_0000_0000_0000_0000_0000;
    const MOVE_REVERSE_BITMASK: u32 = Self::MOVE_BITMASK ^ u32::MAX;
    fn set_last_move(&mut self, m: &PackedMove) {
        let move_shifted = (m.0 as u32) << (31 - PackedMove::BIT_SIZE);
        let cleared = self.0 & Self::MOVE_REVERSE_BITMASK;
        self.0 = cleared | move_shifted | Self::HAS_MOVE_BITMASK;
    }

    fn get_last_move(&self) -> Option<PackedMove> {
        if self.0 & Self::HAS_MOVE_BITMASK == 0 {
            return None;
        }
        let m = (self.0 & Self::MOVE_BITMASK) >> (31 - PackedMove::BIT_SIZE);
        Some(PackedMove(m as u8))
    }

    const CHECK_ALL_BOUNDS_MASK: u32 = {
        let mut num: u32 = 0;
        let mut index = 0;
        while index < MAX_SLICES as usize {
            num += (0b100 as u32) << (index * SLICE_BIT_WIDTH);
            index += 1;
        }
        num
    };

    fn apply_move(&self, m: &PrecomputedMove) -> Option<Self> {
        let raw = self.0;

        // println!("current: {:b}", raw);
        // println!("upper_bound_check: {:b}", m.upper_bound_check);
        // println!("lower_bound_check: {:b}", m.lower_bound_check);
        // println!("delta: {:b}", m.delta);

        // Check upper bound
        let highest_bit = raw & m.upper_bound_check;
        let middle_bit = (raw << 1) & m.upper_bound_check;
        if highest_bit & middle_bit > 0 {
            return None;
        }

        // check lower bound
        let highest_bit = raw & m.lower_bound_check;
        let middle_bit = (raw << 1) & m.lower_bound_check;
        let lowest_bit = (raw << 2) & m.lower_bound_check;
        let non_zero_flags = highest_bit | middle_bit | lowest_bit;
        let filler = Self::CHECK_ALL_BOUNDS_MASK ^ m.lower_bound_check;
        let compare_to = non_zero_flags | filler;
        // println!("highest_bit: {:b}", highest_bit);
        // println!("middle_bit: {:b}", middle_bit);
        // println!("lowest_bit: {:b}", lowest_bit);
        // println!("non_zero_flags: {:b}", non_zero_flags);
        // println!("filler: {:b}", filler);
        // println!("compare_to:            {:b}", compare_to);
        // println!("CHECK_ALL_BOUNDS_MASK: {:b}", Self::CHECK_ALL_BOUNDS_MASK);
        if compare_to != Self::CHECK_ALL_BOUNDS_MASK {
            return None;
        }

        let mut new = Self(raw.wrapping_add(m.delta));
        new.set_last_move(&m.m);

        Some(new)
    }
}

#[derive(Clone, Copy, Default)]
struct PackedMove(u8);

impl PackedMove {
    const BIT_SIZE: usize = 4;

    fn from_move(m: &Move) -> Self {
        match m.direction {
            // Set lowest bit to 0 for left or 1 for right
            // This directions allows for densely packing all moves into the numbers 0 - 13
            Direction::Left => Self((m.slice << 1) & 0b0000_1110),
            Direction::Right => Self((m.slice << 1) | 0b0000_0001),
        }
    }

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
            let mut lower_bound_check = 0;
            let mut upper_bound_check = 0;

            for (target, link) in links.links_from(m.slice) {
                let operation = match link.apply(m.direction) {
                    None => continue,
                    Some(Direction::Left) => {
                        lower_bound_check |= 0b100 << (target * SLICE_BIT_WIDTH);
                        0u32.wrapping_sub(1)
                    }
                    Some(Direction::Right) => {
                        upper_bound_check |= 0b100 << (target * SLICE_BIT_WIDTH);
                        1
                    }
                };
                delta += operation << (target * SLICE_BIT_WIDTH);
            }

            move_map[packed_move.0 as usize] = PrecomputedMove {
                m: packed_move,
                delta,
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
        let start = std::time::Instant::now();
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

        let mut state_map = vec![128u8; LockState::possible_states(self.lock.size())];
        state_map[initial_state.index(slices)] = 64;

        let mut queue = VecDeque::new();
        queue.push_back(initial_state);

        let move_map = MoveMap::new(&self.lock.links);

        println!("Starting state search");

        let mut combinations_found = 0;

        'outer: while let Some(state) = queue.pop_front() {
            for m in move_map.0.iter().take(slices * 2) {
                if let Some(new_state) = state.apply_move(m) {
                    let index = new_state.index(slices);
                    if state_map[index] != 128 {
                        // println!("Already found state {}", new_state_num);
                        continue;
                    }
                    combinations_found += 1;

                    state_map[index] = m.m.0;

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

        if state_map[solved_index] == 128 {
            println!("No path found!");
            return Solution {
                moves: Err(SolveError::Impossible),
            };
        }

        let mut moves = Vec::new();
        let mut state = LockState::SOLVED;
        loop {
            let raw = state_map[state.index(slices)];
            if raw == 64 {
                break;
            }
            let m = PackedMove(raw);
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
