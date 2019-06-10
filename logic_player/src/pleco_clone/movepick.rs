use std::ptr;

use super::board::Board;
use super::move_list::ScoringMoveList;
use super::{BitMove, ScoringMove, SQ};

use super::tables::prelude::*;

pub struct MovePicker {
    pick: Pick,
    board: *const Board,
    moves: ScoringMoveList,
    ttm: BitMove,
    killers: [BitMove; 2],
    cm: BitMove,
    main_hist: *const ButterflyHistory,
    cont_hist: *const [*const PieceToHistory; 4],
    cur_ptr: *mut ScoringMove,
    end_ptr: *mut ScoringMove,
}

impl MovePicker {
    /// MovePicker constructor for the main search
    pub fn main_search(
        board: &Board,
        main_hist: &ButterflyHistory,
        cont_hist: *const [*const PieceToHistory; 4],
        ttm: BitMove,
        killers: [BitMove; 2],
        counter_move: BitMove,
    ) -> Self {
        let mut pick = Pick::MainSearch;

        if ttm == BitMove::null() {
            pick = Pick::KillerOne;
        }

        let mut moves = ScoringMoveList::default();
        let first: *mut ScoringMove = moves.as_mut_ptr();

        MovePicker {
            pick,
            board: &*board,
            moves,
            ttm,
            killers,
            cm: counter_move,
            main_hist,
            cont_hist,
            cur_ptr: first,
            end_ptr: first,
        }
    }

    fn main_hist(&self) -> &ButterflyHistory {
        unsafe { &*self.main_hist }
    }

    fn cont_hist(&self, idx: usize) -> &PieceToHistory {
        unsafe { &*((&*self.cont_hist)[idx]) }
    }

    fn score_quiets(&mut self) {
        let mut ptr = self.cur_ptr;
        let color = self.board().state.get_current_player_color();
        let field_of_color = color.to_fieldtype();
        unsafe {
            while ptr < self.end_ptr {
                let mov: BitMove = (*ptr).bit_move;
                let to_sq: SQ = mov.get_dest();
                (*ptr).score = self.main_hist()[(color, mov)]
                    .saturating_add(self.cont_hist(0)[(field_of_color, to_sq)])
                    .saturating_add(self.cont_hist(1)[(field_of_color, to_sq)])
                    .saturating_add(self.cont_hist(3)[(field_of_color, to_sq)]);
                ptr = ptr.add(1);
            }
        }
    }

    fn pick_best(&self, begin: *mut ScoringMove, end: *mut ScoringMove) -> ScoringMove {
        unsafe {
            let mut best_score = begin;
            /*let mut cur = begin.add(1);
            while cur < end {
                if (*cur).score > (*best_score).score {
                    best_score = cur;
                }
                cur = cur.add(1);
            }
            ptr::swap(begin, best_score);*/
            *begin
        }
    }

    pub fn next(&mut self) -> Option<BitMove> {
        let mov = self.next_mov();
        if mov != BitMove::null() {
            Some(mov)
        } else {
            None
        }
    }

    pub fn next_mov(&mut self) -> BitMove {
        let mut mov: ScoringMove = ScoringMove::null();
        match self.pick {
            Pick::MainSearch => {
                self.pick = Pick::KillerOne;
                if self.board().is_legal(&self.ttm) {
                    return self.ttm;
                }
                return self.next_mov();
            }
            Pick::KillerOne => {
                mov.bit_move = self.killers[0];
                self.pick = Pick::KillerTwo;
                if mov.bit_move != BitMove::null()
                    && mov.bit_move != self.ttm
                    && self.board().is_legal(&mov.bit_move)
                {
                    return mov.bit_move;
                }
                return self.next_mov();
            }
            Pick::KillerTwo => {
                mov.bit_move = self.killers[1];
                self.pick = Pick::CounterMove;
                if mov.bit_move != BitMove::null()
                    && mov.bit_move != self.ttm
                    && self.board().is_legal(&mov.bit_move)
                {
                    return mov.bit_move;
                }
                return self.next_mov();
            }
            Pick::CounterMove => {
                self.pick = Pick::InitAnything;
                /*if self.cm != BitMove::null()
                    && self.cm != self.ttm
                    && self.cm != self.killers[0]
                    && self.cm != self.killers[1]
                    && self.board().is_legal(&self.cm)
                {
                    return self.cm;
                }*/
                return self.next_mov();
            }
            Pick::InitAnything => {
                self.cur_ptr = self.moves.as_mut_ptr();
                self.end_ptr = self.board().extend_from_ptr(self.cur_ptr);
                /*self.moves = ScoringMoveList::from(self.board().get_root_moves());
                self.cur_ptr = self.moves.as_mut_ptr();
                unsafe {
                    self.end_ptr = self.moves.over_bounds_ptr();
                }*/
                // self.score_quiets();
                self.pick = Pick::Anything;
                return self.next_mov();
            }
            Pick::Anything => {
                while self.cur_ptr < self.end_ptr {
                    mov = self.pick_best(self.cur_ptr, self.end_ptr);
                    unsafe {
                        self.cur_ptr = self.cur_ptr.add(1);
                    }
                    if mov.bit_move != self.ttm {
                        /*if !self.board().is_legal(&mov.bit_move) {
                            return self.next_mov();
                        }*/
                        return mov.bit_move;
                    }
                }
            }
        }
        BitMove::null()
    }

    pub fn board(&self) -> &Board {
        unsafe { &*self.board }
    }
}

#[derive(Copy, Clone)]
pub enum Pick {
    MainSearch,
    KillerOne,
    KillerTwo,
    CounterMove,
    InitAnything,
    Anything,
}
