use std::mem;
use std::ops::{Index,IndexMut};

use super::{StatBoard,NumStatCube};
use super::super::{SQ};
use super::super::consts::*;
use game_sdk::FieldType;


/// PieceToBoards are addressed by a move's [piece]][to] information
#[derive(Clone)]
pub struct PieceToHistory {
    a: [[i16; SQ_CNT]; PIECE_CNT]
}

// [Us][Our Piece][To SQ]
#[allow(non_camel_case_types)]
type PTH_idx = (FieldType, SQ);

impl Index<PTH_idx> for PieceToHistory {
    type Output = i16;

    #[inline(always)]
    fn index(&self, idx: PTH_idx) -> &Self::Output {
        unsafe {
            self.a.get_unchecked(idx.0 as usize)      // [Piece moved]
                .get_unchecked((idx.1).0 as usize)  // [To SQ]
        }
    }
}

impl IndexMut<PTH_idx> for PieceToHistory {
    #[inline(always)]
    fn index_mut(&mut self, idx: PTH_idx) -> &mut Self::Output {
        /*unsafe {
            self.a.get_unchecked_mut(idx.0 as usize)    // [Piece moved]
                .get_unchecked_mut((idx.1).0 as usize)  // [To SQ]
        }*/
        &mut self.a[idx.0 as usize][(idx.1).0 as usize]
    }
}

impl StatBoard<i16, PTH_idx> for PieceToHistory {
    const FILL: i16 = 0;
}

impl NumStatCube<PTH_idx> for PieceToHistory {
    const D: i32 = 936;
    const W: i32 = 32;
}

/// ContinuationHistory is the history of a given pair of moves, usually the
/// current one given a previous one. History table is based on PieceToBoards
/// instead of ButterflyBoards.
#[derive(Clone)]
pub struct ContinuationHistory {
    a: [[PieceToHistory; SQ_CNT]; PIECE_CNT]
}

impl ContinuationHistory {
    pub fn new() -> Self {
        unsafe {mem::zeroed()}
    }

    pub fn clear(&mut self) {
        *self = unsafe {mem::zeroed()};
    }
}

// [player][Our Moved Piece][To SQ]
#[allow(non_camel_case_types)]
type CH_idx = (FieldType, SQ);

impl Index<CH_idx> for ContinuationHistory {
    type Output = PieceToHistory;

    #[inline(always)]
    fn index(&self, idx: CH_idx) -> &Self::Output {
        unsafe {
            self.a.get_unchecked(idx.0 as usize)    // [moved piece]
                .get_unchecked((idx.1).0 as usize)  // [To SQ]
        }
    }
}

impl IndexMut<CH_idx> for ContinuationHistory {
    #[inline(always)]
    fn index_mut(&mut self, idx: CH_idx) -> &mut Self::Output {
        /*unsafe {
            self.a.get_unchecked_mut(idx.0 as usize)    // [moved Piece]
                .get_unchecked_mut((idx.1).0 as usize)  // [To SQ]
        }*/
        &mut self.a[idx.0 as usize][(idx.1).0 as usize]
    }
}