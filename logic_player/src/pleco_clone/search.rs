//! The main searching function.

use std::mem;
use std::ptr;

use super::board::Board;
use super::score::*;
use super::tt::*;
use super::{BitMove, PreFetchable, SQ};
use crate::minimax::evaluation;

use game_sdk::{FieldType, GameState, PlayerColor};
use time;

use super::consts::{MAX_PLY, THREAD_STACK_SIZE};

use super::consts::*;
use super::movepick::MovePicker;
use super::root_moves_list::RootMove;
use super::root_moves_list::RootMoveList;
use super::tables::prelude::*;

static RAZOR_MARGIN: [f32; 3] = [0., 590., 604.];

pub struct Stack {
    pv: BitMove,
    cont_history: *mut PieceToHistory,
    ply: u16,
    current_move: BitMove,
    killers: [BitMove; 2],
    static_eval: Value,
    stat_score: i32,
    move_count: u32,
}

impl Stack {
    /// Get the next ply at an offset.
    pub fn offset(&mut self, count: isize) -> &mut Stack {
        unsafe {
            let ptr: *mut Stack = self as *mut Stack;
            &mut *ptr.offset(count)
        }
    }

    /// Get the next ply's Stack.
    pub fn incr(&mut self) -> &mut Stack {
        self.offset(1)
    }
}

/// A Stack for the searcher, with information being contained per-ply.
pub struct ThreadStack {
    stack: [Stack; THREAD_STACK_SIZE],
}

impl ThreadStack {
    pub fn new() -> Self {
        unsafe { mem::zeroed() }
    }

    /// Gets a certain frame from the stack.
    ///
    /// Assumes the frame is within bounds, otherwise undefined behavior.
    pub fn get(&mut self, frame: usize) -> &mut Stack {
        debug_assert!(frame < THREAD_STACK_SIZE);
        unsafe { self.stack.get_unchecked_mut(frame) }
    }

    /// Get the ply at Zero
    pub fn ply_zero(&mut self) -> &mut Stack {
        self.get(4)
    }
}

#[derive(Clone)]
pub struct Searcher {
    // Synchronization primitives
    pub id: usize,

    // search data
    pub depth_completed: i16,
    pub start_time: time::Tm,
    pub board: Board,
    pub root_moves: RootMoveList,
    pub last_best_move: BitMove,
    pub last_best_move_depth: i16,
    pub nodes: u64,

    pub counter_moves: CounterMoveHistory,
    pub main_history: ButterflyHistory,
    pub cont_history: ContinuationHistory,

    // MainThread Information
    pub previous_score: Value,
    pub best_move: BitMove,

    reductions: [[[[i16; 64]; 64]; 2]; 2],
    futility_move_counts: [[i32; 16]; 2],
}

impl Searcher {
    /// Creates a new `Searcher` of an ID and condition to be released by.
    pub fn new(id: usize) -> Self {
        let mut reductions = [[[[0; 64]; 64]; 2]; 2];
        let mut futility_move_counts = [[0; 16]; 2];
        for d in 0..16 {
            futility_move_counts[0][d] = (2.4 + 0.74 * (d as f64).powf(1.78)) as i32;
            futility_move_counts[1][d] = (5.0 + 1.0 * (d as f64).powf(2.0)) as i32;
        }
        for imp in 0..2 {
            for d in 1..64 {
                for mc in 1..64 {
                    let r: f64 = (d as f64).log(2.0) * (mc as f64).log(2.0) / 1.95;
                    reductions[0][imp][d][mc] = r as i16;
                    reductions[1][imp][d][mc] = (reductions[0][imp][d][mc] - 1).max(1);

                    // Increase reduction for non-PV nodes when eval is not improving
                    if imp == 0 && r > 1.0 {
                        reductions[0][imp][d][mc] += 1;
                    }
                }
            }
        }
        init_globals();
        let searcher = Searcher {
            id,
            start_time: time::now(),
            depth_completed: 0,
            board: Board::null(),
            root_moves: RootMoveList::new(),
            last_best_move: BitMove::null(),
            last_best_move_depth: 0,
            nodes: 0,
            counter_moves: CounterMoveHistory::new(),
            main_history: ButterflyHistory::new(),
            cont_history: ContinuationHistory::new(),
            previous_score: 0.,
            best_move: BitMove::null(),
            reductions,
            futility_move_counts,
        };
        return searcher;
    }

    pub fn clear(&mut self) {
        self.previous_score = INFINITE;
        self.counter_moves.clear();
        self.main_history.clear();
        self.cont_history.clear();
    }

    /// Main thread searching function.
    pub fn main_thread_go(&mut self, state: &GameState) {
        // set the global limit
        self.start_time = time::now();
        self.board = Board::from_state(state);
        // Increment the TT search table.
        tt().new_search();

        // Search ourselves
        self.root_moves.replace(&self.board.get_root_moves());
        self.nodes = 0;
        self.search_root();

        // iterate through each thread, and find the best move available (based on score)
        self.best_move = self.root_moves.first().bit_move;
        self.previous_score = self.root_moves.first().score;

        if self.use_stdout() {
            println!("bestmove {}", self.best_move.to_string());
        }
    }

    // The per thread searching function
    fn search_root(&mut self) {
        // Early return. This shouldn't normally happen.
        if self.stop() {
            return;
        }

        // notify GUI that this thread is starting
        if self.use_stdout() {
            println!("info id {} start", self.id);
        }

        let mut stack: ThreadStack = ThreadStack::new();

        for i in [0, 1, 2, 3, 4].iter() {
            stack.get(*i).cont_history = &mut self.cont_history[(FieldType::Free, SQ(0))] as *mut _;
        }

        // If use a max_depth limit, use that as the max depth.
        let max_depth = MAX_PLY as i16 - self.board.state.turn as i16;

        // The depth to start searching at based on the thread ID.
        let start_ply: i16 = 0;
        // The number of plies to skip each iteration.
        let mut depth: i16 = start_ply + 1;

        let mut delta: f32 = NEG_INFINITE;
        #[allow(unused_assignments)]
        let mut best_value: f32 = NEG_INFINITE;
        let mut alpha: f32 = NEG_INFINITE;
        let mut beta: f32 = INFINITE;

        stack.ply_zero().ply = 0;

        // Iterative deeping. Start at the base ply (determined by thread_id), and then increment
        // by the skip size after searching that depth. If searching for depth, non-main threads
        // will ignore the max_depth and instead wait for a stop signal.
        'iterative_deepening: while !self.stop() && depth < max_depth {
            // rollback all the root moves, ala set the previous score to the current score.
            self.root_moves.rollback();

            // Delta gives a bound in the iterative loop before re-searching that position.
            // Only applicable for a depth of 5 and beyond.
            /*if depth >= 5 {
                let prev_best_score = self.root_moves.first().prev_score;
                delta = 39.474;
                alpha = f32::max(prev_best_score - delta, NEG_INFINITE);
                beta = f32::min(prev_best_score + delta, INFINITE);
            }*/

            // Loop until we find a value that is within the bounds of alpha, beta, and the delta margin.
            'aspiration_window: loop {
                // search!
                best_value = self.search::<PV>(alpha, beta, stack.ply_zero(), depth);

                self.root_moves.sort();
                if self.stop() {
                    // In case of a fail high or fail low, we do not choose to sort the moves,
                    // as the resulting scores would be incorrect
                    break 'aspiration_window;
                }

                // Sort root moves based on the scores

                // Order root moves by the score retreived post search.

                // Check for incorrect search window. If the value is less than alpha
                // or greater than beta, we need to increase the search window and re-search.
                // Otherwise, go to the next search
                if best_value <= alpha {
                    beta = (alpha + beta) / 2.;
                    alpha = f32::max(best_value - delta, NEG_INFINITE);
                } else if best_value >= beta {
                    beta = f32::min(best_value + delta, INFINITE);
                } else {
                    break 'aspiration_window;
                }
                delta += (delta / 4.) + 5.;

                assert!(alpha >= NEG_INFINITE);
                assert!(beta <= INFINITE);
            }

            if self.use_stdout() {
                /*println!("depth: {}", depth);
                for action in &self.root_moves {
                    println!("{:?}", action);
                }
                println!("");*/
                self.pv(depth, alpha, beta);
            }
            if !self.stop() {
                self.depth_completed = depth;
            } else {
                break 'iterative_deepening;
            }

            let curr_best_move = self.root_moves.first().bit_move;

            if curr_best_move != self.last_best_move {
                self.last_best_move = curr_best_move;
                self.last_best_move_depth = depth;
            }

            depth += 1;
        }
    }

    // The searching function for a specific depth.
    fn search<N: PVNode>(
        &mut self,
        mut alpha: f32,
        mut beta: f32,
        ss: &mut Stack,
        depth: i16,
    ) -> f32 {
        self.nodes += 1;
        if self.board.is_finished() {
            let winner = self.board.winner();
            if let Some(c) = winner {
                if c == self.board.state.get_current_player_color() {
                    return mate_in(ss.ply);
                }
                return mated_in(ss.ply);
            }
            return DRAW;
        }

        assert!(alpha < beta);
        if depth < 1 {
            return self.qsearch(alpha, beta, ss);
        }

        assert!(depth >= 1);
        assert!(depth < MAX_PLY as i16);
        let is_pv: bool = N::is_pv();
        let ply: u16 = ss.ply;
        let at_root: bool = ply == 0;
        let zob: u64;

        let new_depth: i16 = depth - 1;

        let mut best_move: BitMove;

        let color = self.board.state.get_current_player_color();
        let field_of_color = color.to_fieldtype();

        let mut value: Value = NEG_INFINITE;
        let mut best_value: Value = NEG_INFINITE;
        let mut moves_played: u32 = 0;
        let mut pos_eval: f32;

        ss.move_count = 0;

        let mut quiets_searched: [BitMove; 64] = [BitMove::null(); 64];
        let mut quiets_count = 0;

        let improving: bool;

        if !at_root {
            // Check for stop conditions.
            if depth > 2 && self.stop() {
                return NONE;
            }

            // Mate distance pruning. This ensures that checkmates closer to the root
            // have a higher value than otherwise.
            alpha = alpha.max(mated_in(ply));
            beta = beta.min(mate_in(ply + 1));
            if alpha >= beta {
                return alpha;
            }
        }

        // increment the next ply
        ss.incr().ply = ply + 1;
        // Set the killer moves two plies in advance to be nothing.
        ss.offset(2).killers = [BitMove::null(); 2];
        best_move = BitMove::null();
        ss.current_move = BitMove::null();
        ss.cont_history = &mut self.cont_history[(FieldType::Free, SQ(0))] as *mut _;

        // square the previous piece moved.
        let prev_sq: SQ = ss.offset(-1).current_move.get_src();

        // Initialize statScore to zero for the grandchildren of the current position.
        // So statScore is shared between all grandchildren and only the first grandchild
        // starts with statScore = 0. Later grandchildren start with the last calculated
        // statScore of the previous grandchild. This influences the reduction rules in
        // LMR which are based on the statScore of parent position.
        ss.offset(-2).stat_score = 0;

        // probe the transposition table
        zob = self.board.zobrist();
        let (tt_hit, tt_entry): (bool, &mut Entry) = tt().probe(zob);
        let tt_value: Value = if tt_hit {
            value_from_tt(tt_entry.score, ss.ply)
        } else {
            NONE
        };
        let tt_move: BitMove = if at_root {
            self.root_moves.first().bit_move
        } else if tt_hit {
            tt_entry.best_move
        } else {
            BitMove::null()
        };
        let pv_exact = is_pv && tt_entry.node_type() == NodeBound::Exact;

        // At non-PV nodes, check for a better TT value to return.
        if tt_hit && tt_entry.depth as i16 >= depth as i16 && tt_value != NONE {
            let tt_type = tt_entry.node_type();
            match tt_type {
                NodeBound::Exact => return tt_value,
                NodeBound::LowerBound => alpha = f32::max(tt_value, alpha),
                NodeBound::UpperBound => beta = f32::min(tt_value, beta),
                NodeBound::NoBound => {}
            };
            if alpha >= beta {
                return alpha;
            }
            /*if tt_move != BitMove::null() {
                if tt_value >= beta {
                    self.update_quiet_stats(tt_move, ss, &quiets_searched[0..0], stat_bonus(depth));

                    // Extra penalty for a quiet TT move in previous ply when it gets refuted
                    if ss.offset(-1).move_count == 1 {
                        self.update_continuation_histories(
                            ss.offset(-1),
                            field_of_color,
                            prev_sq,
                            -stat_bonus(depth + 1),
                        );
                    }
                } else {
                    // Penalty for a quiet ttMove that fails low
                    let penalty = -stat_bonus(depth);
                    self.main_history.update((color, tt_move), penalty);
                    self.update_continuation_histories(
                        ss,
                        field_of_color,
                        tt_move.get_dest(),
                        penalty,
                    );
                }
            }*/
        }

        /*// Get and set the position eval
        if tt_hit {
            pos_eval = if tt_entry.eval == NONE {
                self.eval()
            } else {
                tt_entry.eval
            };
            ss.static_eval = pos_eval;

            // check for tt value being a better position evaluation
            if tt_value != NONE && correct_bound(tt_value, pos_eval, tt_entry.node_type()) {
                pos_eval = tt_value;
            }
        } else {
            pos_eval = self.eval();
            ss.static_eval = pos_eval;
        }

        improving = {
            let p_ss = ss.offset(-2).static_eval;
            ss.static_eval >= p_ss || p_ss == NONE
        };

        // Razoring. At the lowest depth before qsearch, If the evaluation + a margin still
        // isn't better than alpha, go straight to qsearch.
        if !is_pv && depth < 3 && pos_eval <= alpha - RAZOR_MARGIN[depth as usize] {
            let r_alpha = alpha - ((depth >= 2) as i32) as f32 * RAZOR_MARGIN[depth as usize];
            let v = self.qsearch(r_alpha, r_alpha + 1., ss);
            if depth < 2 || v <= r_alpha {
                return v;
            }
        }

        // Futility Pruning. Disregard moves that have little chance of raising the callee's
        // alpha value. Rather, return the position evaluation as an estimate for the current
        // move's strength
        if !at_root && depth < 7 && pos_eval - futility_margin(depth, improving) >= beta {
            return pos_eval;
        }*/

        // Continuation histories of the previous moved from 1, 2, and 4 moves ago.
        let cont_hists = [
            ss.offset(-1).cont_history,
            ss.offset(-2).cont_history,
            ptr::null(),
            ss.offset(-4).cont_history,
        ];

        let counter: BitMove = self.counter_moves[(field_of_color, prev_sq)];
        let mut move_picker = MovePicker::main_search(
            &self.board,
            &self.main_history,
            &cont_hists as *const _,
            tt_move,
            ss.killers,
            counter,
        );

        while let Some(mov) = move_picker.next() {
            moves_played += 1;
            ss.move_count = moves_played;
            /*if depth < 16
                && !at_root
                && moves_played as i32
                    > self.futility_move_counts[improving as usize][depth as usize]
            {
                break;
            }


            // Pruning at a shallow depth
            if !at_root && best_value > MATED_IN_MAX_PLY {
                // Reduced depth of the next LMR search
                let lmr_depth: i16 = (new_depth
                    - self.reductions[is_pv as usize][improving as usize]
                        [(depth as usize).min(63)][(moves_played as usize).min(63)])
                .max(0);

                // Countermoves based pruning
                unsafe {
                    if lmr_depth < 3
                        && (*cont_hists[0])[(field_of_color, mov.get_dest())] < 0
                        && (*cont_hists[1])[(field_of_color, mov.get_dest())] < 0
                    {
                        continue;
                    }
                }

                // Futility pruning: parent node
                if lmr_depth < 7 && ss.static_eval + 256. + 200. * lmr_depth as f32 <= alpha {
                    continue;
                }
            }*/

            // speculative prefetch for the next key.
            // tt().prefetch(self.board.key_after(mov, color));

            ss.current_move = mov;
            ss.cont_history = &mut self.cont_history[(field_of_color, mov.get_dest())] as *mut _;

            // do the move
            let before_board = self.board.clone();
            let _killing = self.board.apply_move(&mov, &color);

            // prefetch next TT entry
            tt().prefetch(self.board.zobrist());

            // At higher depths, do a search of a lower ply to see if this move is
            // worth searching. We don't do this for capturing or promotion moves.
            let do_full_depth: bool = true; /*if moves_played > 1 && depth >= 3 {
                                                let mut r: i16 = self.reductions[is_pv as usize][improving as usize]
                                                    [(depth as usize).min(63)][(moves_played as usize).min(63)];

                                                // Decrease reduction if opponent's move count is high
                                                if ss.offset(-1).move_count > 15 {
                                                    r -= 1;
                                                }

                                                if pv_exact {
                                                    r -= 1;
                                                }

                                                ss.stat_score = unsafe {
                                                    self.main_history[(color, mov)] as i32
                                                        + (*cont_hists[0])[(field_of_color, mov.get_dest())] as i32
                                                        + (*cont_hists[1])[(field_of_color, mov.get_dest())] as i32
                                                        + (*cont_hists[3])[(field_of_color, mov.get_dest())] as i32
                                                        - 1000 //- 4000
                                                };

                                                // Decrease/increase reduction by comparing opponent's stat score
                                                if ss.stat_score >= 0 && ss.offset(-1).stat_score < 0 {
                                                    r -= 1;
                                                } else if ss.offset(-1).stat_score >= 0 && ss.stat_score < 0 {
                                                    r += 1;
                                                }
                                                r = (r - (ss.stat_score / 20000) as i16).max(0) as i16;

                                                let d = (new_depth - r).max(1);

                                                let ralpha = alpha - 38.641;
                                                value = -self.search::<NonPV>(-(ralpha + 1e-3), -ralpha, ss.incr(), d);
                                                value > ralpha && d != new_depth
                                            } else {
                                                !is_pv || moves_played > 1
                                            };*/

            // If the value is potentially better, do a full depth search.
            if do_full_depth {
                value = -self.search::<NonPV>(-alpha - 1e-3, -alpha, ss.incr(), new_depth);
            }

            // If on the PV node and the node might be a continuation, search for a full depth
            // with a PV value.
            if is_pv && (moves_played == 1 || (value > alpha && (at_root || value < beta))) {
                value = -self.search::<PV>(-beta, -alpha, ss.incr(), new_depth);
            }

            self.board = before_board;
            if value == NONE || value == -NONE {
                return NONE;
            }
            assert!(value > NEG_INFINITE);
            assert!(value < INFINITE);

            if at_root {
                /*let rm: &mut RootMove = self
                .root_moves()
                .find(mov)
                .expect("Did not find correct move at root");*/
                let mut verbose = false;
                if let Some(rm) = self.root_moves.find(mov) {
                    // Insert the score into the RootMoves list
                    rm.depth_reached = depth;
                    rm.score = value;
                    /*if moves_played == 1 || value > alpha {
                        rm.depth_reached = depth;
                        rm.score = value;
                    } else {
                        rm.score = NEG_INFINITE;
                    }*/
                } else {
                    verbose = true;
                }
                if verbose {
                    if mov == tt_move {
                        println!("counter");
                    } else if mov == ss.killers[0] || mov == ss.killers[1] {
                        println!("killer")
                    } else if mov == counter {
                        println!("counter");
                    }
                    println!("data {} len_root {}", mov.data, self.root_moves.len());
                    println!(
                        "len_checked {} played {} legal {}",
                        self.board.state.get_move_list().len(),
                        moves_played,
                        self.board.is_legal(&mov)
                    );
                    println!("{}", mov.to_string());
                    println!("");
                    println!("{:?}", self.board.state.board);
                    moves_played -= 1;
                    continue;
                }
            }

            if value > best_value {
                best_value = value;
                best_move = mov;

                if value > alpha {
                    if is_pv && !at_root {
                        ss.incr().pv = mov;
                    }

                    if is_pv && value < beta {
                        alpha = value;
                    } else {
                        break;
                    }
                }
            }

            // If best_move wasnt found, add it to the list of quiets / captures that failed
            /*if mov != best_move {
                if quiets_count < 64 {
                    quiets_searched[quiets_count] = mov;
                    quiets_count += 1;
                }
            }*/
        }

        // check for checkmate
        if moves_played == 0 {
            return mated_in(ss.ply);
        } /*else if best_move != BitMove::null() {
              // If the best move is quiet, update move heuristics
              self.update_quiet_stats(
                  best_move,
                  ss,
                  &quiets_searched[0..quiets_count],
                  stat_bonus(depth),
              );

              // penalize quiet TT move that was refuted.
              if ss.offset(-1).move_count == 1 {
                  self.update_continuation_histories(
                      ss.offset(-1),
                      field_of_color,
                      prev_sq,
                      -stat_bonus(depth + 1),
                  );
              }
          }*/
        let node_bound = if best_value >= beta {
            NodeBound::LowerBound
        } else if is_pv && !best_move.is_null() {
            NodeBound::Exact
        } else {
            NodeBound::UpperBound
        };

        tt_entry.place(
            zob,
            best_move,
            value_to_tt(best_value, ss.ply),
            ss.static_eval,
            depth as i16,
            node_bound,
            tt().time_age(),
        );
        best_value
    }

    /// Called by the main search when the depth limit has been reached. This function only traverses capturing moves
    /// and possible checking moves, unless its in check.
    ///
    /// Depth must be less than or equal to zero,
    fn qsearch(&mut self, mut alpha: f32, mut beta: f32, ss: &mut Stack) -> f32 {
        let old_alpha = alpha;
        let color = self.board.state.get_current_player_color();
        let zob = self.board.zobrist();
        let (tt_hit, tt_entry): (bool, &mut Entry) = tt().probe(zob);
        let standing_pat;
        let tt_value: Value = if tt_hit {
            value_from_tt(tt_entry.score, ss.ply)
        } else {
            NONE
        };
        let tt_move = if tt_hit {
            tt_entry.best_move
        } else {
            BitMove::null()
        };
        let mut best_move = tt_move;
        let mut best_value;

        if tt_hit && tt_entry.depth as i16 >= 0 && tt_value != NONE {
            match tt_entry.node_type() {
                NodeBound::Exact => return tt_value,
                NodeBound::LowerBound => alpha = f32::max(tt_value, alpha),
                NodeBound::UpperBound => beta = f32::min(tt_value, beta),
                NodeBound::NoBound => {}
            };
        }

        // Determine whether or not to include checking moves.
        if tt_hit {
            if tt_entry.eval == NONE {
                best_value = self.eval();
            } else {
                best_value = tt_entry.eval;
            }
            standing_pat = best_value;
            ss.static_eval = best_value;

            if tt_value != NONE && correct_bound(tt_value, best_value, tt_entry.node_type()) {
                best_value = tt_value;
            }
        } else {
            standing_pat = self.eval();
            ss.static_eval = standing_pat;
            best_value = standing_pat;
        }
        if standing_pat + 38.641 < alpha {
            return standing_pat;
        }
        ss.incr().ply = ss.ply + 1;
        alpha = f32::max(alpha, best_value);
        if alpha >= beta {
            return alpha;
        }
        {
            let before_board = self.board.clone();
            let _killing = self.board.apply_move(&tt_move, &color);
            tt().prefetch(self.board.zobrist());
            // self.nodes += 1;
            let rate;
            if self.board.is_finished() {
                let winner = self.board.winner();
                if let Some(c) = winner {
                    if c == color {
                        return 0.9 * mate_in(ss.ply + 1);
                    }
                    return 0.9 * mated_in(ss.ply + 1);
                }
                return ZERO;
            } else {
                rate = -self.qsearch(-beta, -alpha, ss.incr());
            }
            self.board = before_board;
            alpha = f32::max(alpha, rate);
            best_value = f32::max(best_value, rate);

            if alpha >= beta {
                tt_entry.place(
                    zob,
                    best_move,
                    value_to_tt(best_value, ss.ply),
                    ss.static_eval,
                    0,
                    NodeBound::LowerBound,
                    tt().time_age(),
                );
                return alpha;
            }
        }
        for mov in self.board.get_captures() {
            let before_board = self.board.clone();
            let _killing = self.board.apply_move(&mov, &color);
            tt().prefetch(self.board.zobrist());
            // self.nodes += 1;
            let rate;
            if self.board.is_finished() {
                let winner = self.board.winner();
                if let Some(c) = winner {
                    if c == color {
                        return 0.9 * mate_in(ss.ply + 1);
                    }
                    return 0.9 * mated_in(ss.ply + 1);
                }
                return ZERO;
            } else {
                rate = -self.qsearch(-beta, -alpha, ss.incr());
            }
            self.board = before_board;
            alpha = f32::max(alpha, rate);
            if rate > best_value {
                best_move = mov;
                best_value = rate;
            }

            if alpha >= beta {
                tt_entry.place(
                    zob,
                    best_move,
                    value_to_tt(best_value, ss.ply),
                    ss.static_eval,
                    0,
                    NodeBound::LowerBound,
                    tt().time_age(),
                );
                return alpha;
            }
        }

        let node_bound = if best_value > old_alpha {
            NodeBound::Exact
        } else {
            NodeBound::UpperBound
        };
        tt_entry.place(
            zob,
            best_move,
            value_to_tt(best_value, ss.ply),
            ss.static_eval,
            0,
            node_bound,
            tt().time_age(),
        );
        return best_value;
    }

    /// If a new quiet best move is found, updating sorting heuristics.
    fn update_quiet_stats(&mut self, mov: BitMove, ss: &mut Stack, quiets: &[BitMove], bonus: i32) {
        if ss.killers[0] != mov {
            ss.killers[1] = ss.killers[0];
            ss.killers[0] = mov;
        }

        let us: PlayerColor = self.board.state.get_current_player_color();
        let moved_piece = us.to_fieldtype();
        let to_sq = mov.get_dest();
        self.main_history.update((us, mov), bonus);
        self.update_continuation_histories(ss, moved_piece, to_sq, bonus);

        {
            let ss_bef: &mut Stack = ss.offset(-1);
            let prev_sq = ss_bef.current_move.get_dest();
            self.counter_moves[(moved_piece, prev_sq)] = mov;
        }

        for q_mov in quiets.iter() {
            self.main_history.update((us, *q_mov), -bonus);
            let to_sq = q_mov.get_dest();
            self.update_continuation_histories(ss, moved_piece, to_sq, -bonus);
        }
    }

    // updates histories of the move pairs formed by the current move of one, two, and four
    // moves ago
    fn update_continuation_histories(
        &mut self,
        ss: &mut Stack,
        piece: FieldType,
        to: SQ,
        bonus: i32,
    ) {
        // for i in [1, 2, 4].iter() {
        for i in [1, 2].iter() {
            let i_ss: &mut Stack = ss.offset(-i as isize);
            unsafe {
                let cont_his: &mut PieceToHistory = &mut *i_ss.cont_history;
                cont_his.update((piece, to), bonus);
            }
        }
    }

    pub fn eval(&mut self) -> Value {
        let val = evaluation::clop_state(&self.board.state);
        match self.board.state.get_current_player_color() {
            PlayerColor::Red => val,
            PlayerColor::Blue => -val,
        }
    }

    #[inline(always)]
    fn main_thread(&self) -> bool {
        self.id == 0
    }

    #[inline(always)]
    fn stop(&self) -> bool {
        if self.elapsed_time() < 1700 {
            return false;
        }
        return true;
    }

    fn elapsed_time(&self) -> i64 {
        (time::now() - self.start_time).num_milliseconds()
    }

    #[inline(always)]
    pub fn print_startup(&self) {
        if self.use_stdout() {
            println!("info id {} start", self.id);
        }
    }

    #[inline(always)]
    pub fn use_stdout(&self) -> bool {
        self.main_thread()
    }

    /// Useful information to tell to the GUI
    fn pv(&mut self, depth: i16, alpha: f32, beta: f32) {
        let elapsed = self.elapsed_time() as u64;
        let root_move: &RootMove = self.root_moves.first();
        let nodes = self.nodes;
        let score = if root_move.score == NEG_INFINITE {
            root_move.prev_score
        } else {
            root_move.score
        };

        if score == NEG_INFINITE {
            return;
        }

        let mut s = String::from("info");
        s.push_str(&format!(" depth {}", depth));
        if score.abs() < MATE - MAX_PLY as f32 {
            s.push_str(&format!(" score cp {}", score));
        } else {
            let mut mate_in = if score > 0. {
                MATE - score + 1.
            } else {
                -MATE - score
            };
            mate_in /= 2.;
            s.push_str(&format!(" score mate {}", mate_in));
        }
        if root_move.score >= beta {
            s.push_str(" lowerbound");
        } else if root_move.score <= alpha {
            s.push_str(" upperbound");
        }
        s.push_str(&format!(" nodes {}", nodes));
        if elapsed > 1000 {
            s.push_str(&format!(" nps {}", (nodes * 1000) / elapsed));
            s.push_str(&format!(" hashfull {:.2}", tt().hash_percent()));
        }
        s.push_str(&format!(" time {}", elapsed));
        s.push_str(&format!(" pv {}", root_move.bit_move.to_string()));
        println!("{}", s);
    }
}

fn correct_bound(tt_value: f32, val: f32, bound: NodeBound) -> bool {
    if tt_value >= val {
        bound as u8 & NodeBound::LowerBound as u8 != 0
    } else {
        bound as u8 & NodeBound::UpperBound as u8 != 0
    }
}

fn mate_in(ply: u16) -> f32 {
    MATE - ply as f32
}

fn mated_in(ply: u16) -> f32 {
    -MATE + ply as f32
}

fn value_to_tt(value: f32, ply: u16) -> f32 {
    if value >= MATE_IN_MAX_PLY {
        value + ply as f32
    } else if value <= MATED_IN_MAX_PLY {
        value - ply as f32
    } else {
        value as f32
    }
}

fn value_from_tt(value: f32, ply: u16) -> f32 {
    if value == NONE {
        NONE
    } else if value >= MATE_IN_MAX_PLY {
        value - ply as f32
    } else if value <= MATED_IN_MAX_PLY {
        value + ply as f32
    } else {
        value
    }
}

#[inline]
fn futility_margin(depth: i16, improving: bool) -> f32 {
    depth as f32 * (175. - 50. * improving as i32 as f32)
}

fn stat_bonus(depth: i16) -> i32 {
    if depth > 17 {
        0
    } else {
        let d = depth as i32;
        d * d + 2 * d - 2
    }
}
