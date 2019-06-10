use std::ops::{Index,IndexMut};

use super::{StatBoard, NumStatCube};
use super::super::consts::*;
use super::super::{BitMove};
use game_sdk::PlayerColor;

/// ButterflyBoards are 2 tables (one for each color) indexed by the move's from
/// and to squares, see chessprogramming.wikispaces.com/Butterfly+Boards
#[derive(Clone)]
pub struct ButterflyHistory {
    a: [[i16; (SQ_CNT * SQ_CNT)]; PLAYER_CNT]
}

// [Us][Move], Or rather [Us][To SQ][From SQ]
#[allow(non_camel_case_types)]
type BF_idx = (PlayerColor, BitMove);

impl Index<BF_idx> for ButterflyHistory {
    type Output = i16;

    #[inline(always)]
    fn index(&self, idx: BF_idx) -> &Self::Output {
        unsafe {
            let from_to = idx.1.from_to_key() as usize;
            self.a.get_unchecked(idx.0 as usize)   // [player]
                .get_unchecked(from_to)            // [From SQ][to SQ]
        }
    }
}

impl IndexMut<BF_idx> for ButterflyHistory {
    #[inline(always)]
    fn index_mut(&mut self, idx: BF_idx) -> &mut Self::Output {
        /*unsafe {
            let from_to = idx.1.from_to_key() as usize;
            self.a.get_unchecked_mut(idx.0 as usize)   // [player]
                .get_unchecked_mut(from_to)            // [From SQ][to SQ]
        }*/
        let from_to = idx.1.from_to_key() as usize;
        &mut self.a[idx.0 as usize][from_to]
    }
}


impl StatBoard<i16, BF_idx> for ButterflyHistory {
    const FILL: i16 = 0;
}

impl NumStatCube<BF_idx> for ButterflyHistory {
    const D: i32 = 324;
    const W: i32 = 32;
}
