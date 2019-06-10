//! This module contains the basic RootMove structures, allowing for storage of the moves from a specific position
//! alongside information about each of the moves.

use std::cmp::Ordering as CmpOrder;

use super::score::*;

use std::iter::{IntoIterator, Iterator};
use std::mem;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice;

use super::{move_list::MoveList, BitMove};

// 250 as this fits into 64 byte cache lines easily.
const MAX_MOVES: usize = 130;

/// Keeps track of information of a move for the position to be searched.
#[derive(Copy, Clone, Debug)]
pub struct RootMove {
    pub score: f32,
    pub prev_score: f32,
    pub bit_move: BitMove,
    pub depth_reached: i16,
}

impl RootMove {
    /// Creates a new `RootMove`.
    #[inline]
    pub fn new(bit_move: BitMove) -> Self {
        RootMove {
            bit_move,
            score: NEG_INFINITE,
            prev_score: NEG_INFINITE,
            depth_reached: 0,
        }
    }

    /// Places the current score into the previous_score field, and then updates
    /// the score and depth.
    #[inline]
    pub fn rollback_insert(&mut self, score: f32, depth: i16) {
        self.prev_score = self.score;
        self.score = score;
        self.depth_reached = depth;
    }

    /// Inserts a score and depth.
    #[inline]
    pub fn insert(&mut self, score: f32, depth: i16) {
        self.score = score;
        self.depth_reached = depth;
    }

    /// Places the current score in the previous score.
    #[inline]
    pub fn rollback(&mut self) {
        self.prev_score = self.score;
    }
}

// Moves with higher score for a higher depth are less
impl Ord for RootMove {
    #[inline]
    fn cmp(&self, other: &RootMove) -> CmpOrder {
        if self.depth_reached == other.depth_reached {
            let value_diff = self.score - other.score;
            if value_diff == 0. {
                let prev_value_diff = self.prev_score - other.prev_score;
                if prev_value_diff == 0. {
                    return CmpOrder::Equal;
                } else if prev_value_diff > 0. {
                    return CmpOrder::Less;
                }
            } else if value_diff > 0. {
                return CmpOrder::Less;
            }
            CmpOrder::Greater
        } else {
            return other.depth_reached.cmp(&self.depth_reached);
        }
    }
}

impl Eq for RootMove {}

impl PartialOrd for RootMove {
    fn partial_cmp(&self, other: &RootMove) -> Option<CmpOrder> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RootMove {
    fn eq(&self, other: &RootMove) -> bool {
        self.score == other.score && self.prev_score == other.prev_score
    }
}

pub struct RootMoveList {
    len: usize,
    moves: [RootMove; MAX_MOVES],
}

impl Clone for RootMoveList {
    fn clone(&self) -> Self {
        RootMoveList {
            len: self.len,
            moves: self.moves,
        }
    }
}

impl RootMoveList {
    /// Creates an empty `RootMoveList`.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            RootMoveList {
                len: 0,
                moves: [mem::uninitialized(); MAX_MOVES],
            }
        }
    }

    /// Returns the length of the list.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Replaces the current `RootMoveList` with the moves inside a `MoveList`.
    pub fn replace(&mut self, moves: &MoveList) {
        self.len = moves.len();
        for (i, mov) in moves.iter().enumerate() {
            self[i] = RootMove::new(*mov);
        }
    }

    /// Applies `RootMove::rollback()` to each `RootMove` inside.
    #[inline]
    pub fn rollback(&mut self) {
        self.iter_mut().for_each(|b| b.prev_score = b.score);
    }

    /// Returns the first `RootMove` in the list.
    ///
    /// # Safety
    ///
    /// May return a nonsense `RootMove` if the list hasn't been initalized since the start.
    #[inline]
    pub fn first(&mut self) -> &mut RootMove {
        unsafe { self.get_unchecked_mut(0) }
    }

    /// Converts to a `MoveList`.
    pub fn to_list(&self) -> MoveList {
        let vec = self.iter().map(|m| m.bit_move).collect::<Vec<BitMove>>();
        MoveList::from(vec)
    }

    /// Returns the previous best score.
    #[inline]
    pub fn prev_best_score(&self) -> f32 {
        unsafe { self.get_unchecked(0).prev_score }
    }

    #[inline]
    pub fn insert_score_depth(&mut self, index: usize, score: f32, depth: i16) {
        unsafe {
            let rm: &mut RootMove = self.get_unchecked_mut(index);
            rm.score = score;
            rm.depth_reached = depth;
        }
    }

    #[inline]
    pub fn insert_score(&mut self, index: usize, score: f32) {
        unsafe {
            let rm: &mut RootMove = self.get_unchecked_mut(index);
            rm.score = score;
        }
    }

    pub fn find(&mut self, mov: BitMove) -> Option<&mut RootMove> {
        self.iter_mut().find(|m| m.bit_move == mov)
    }
}

impl Deref for RootMoveList {
    type Target = [RootMove];

    #[inline]
    fn deref(&self) -> &[RootMove] {
        unsafe {
            let p = self.moves.as_ptr();
            slice::from_raw_parts(p, self.len())
        }
    }
}

impl DerefMut for RootMoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut [RootMove] {
        unsafe {
            let p = self.moves.as_mut_ptr();
            slice::from_raw_parts_mut(p, self.len())
        }
    }
}

impl Index<usize> for RootMoveList {
    type Output = RootMove;

    #[inline]
    fn index(&self, index: usize) -> &RootMove {
        &(**self)[index]
    }
}

impl IndexMut<usize> for RootMoveList {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut RootMove {
        &mut (**self)[index]
    }
}

pub struct MoveIter<'a> {
    movelist: &'a RootMoveList,
    idx: usize,
    len: usize,
}

impl<'a> Iterator for MoveIter<'a> {
    type Item = RootMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len - self.idx, Some(self.len - self.idx))
    }
}

impl<'a> IntoIterator for &'a RootMoveList {
    type Item = RootMove;
    type IntoIter = MoveIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        MoveIter {
            movelist: &self,
            idx: 0,
            len: self.len(),
        }
    }
}
