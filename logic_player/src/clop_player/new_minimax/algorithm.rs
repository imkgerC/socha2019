use game_sdk::gamerules;
use game_sdk::GameState;
use game_sdk::Move;
use game_sdk::PlayerColor;

use super::evaluation::texel_state as eval_state;
use super::player::MinimaxParameters;
use super::transposition::{EntryType, MinimalState, TranspositionTable};

use std::f32;

pub static MATE_SCORE: f32 = 200_000.0;
pub static MAX_MATE_PENALTY: f32 = 32.;

pub fn rate_mate(state: &GameState) -> f32 {
    let red_size = state.greatest_swarm_size(&PlayerColor::Red);
    let blue_size = state.greatest_swarm_size(&PlayerColor::Blue);

    let red_count = state.board.red_fields.count_ones() as u8;
    let blue_count = state.board.blue_fields.count_ones() as u8;

    let swarm_bonus = red_size as f32 - blue_size as f32;

    if red_size == red_count {
        if blue_size == blue_count {
            // blue is connected, red too
            if red_size < blue_size {
                return -MATE_SCORE + swarm_bonus - 16.;
            }
            if red_size == blue_size {
                return 0.;
            }
        }
        return MATE_SCORE + swarm_bonus + 16.;
    }
    if blue_size == blue_count {
        return -MATE_SCORE + swarm_bonus - 16.;
    }

    if red_size > blue_size {
        return MATE_SCORE + swarm_bonus + 16.;
    }
    if red_size < blue_size {
        return -MATE_SCORE + swarm_bonus - 16.;
    }

    return 0.;
}

pub struct SearchStatistics {
    pub nodes: usize,
    pub re_searched: usize,
    pub probed: usize,
    pub aspire_probed: usize,
    pub aspire_re: usize,
}

impl SearchStatistics {
    pub fn new() -> SearchStatistics {
        return SearchStatistics {
            nodes: 0,
            re_searched: 0,
            probed: 0,
            aspire_probed: 0,
            aspire_re: 0,
        };
    }
}

pub fn minimax_rate(
    stats: &mut SearchStatistics,
    state: &GameState,
    action: &Move,
    player_index: i8,
    alpha: f32,
    beta: f32,
    depth: u8,
    tt: &mut TranspositionTable,
    start_time: &time::Tm,
    allotted_time: usize,
    params: &MinimaxParameters,
) -> f32 {
    let mut state = state.clone();
    let color = state.get_current_player_color();
    state.perform(action, &color);
    return minimax_rate_state(
        stats,
        &state,
        player_index,
        alpha,
        beta,
        depth,
        tt,
        start_time,
        allotted_time,
        params,
    );
}

#[allow(dead_code)]
pub fn q_search(
    state: &GameState,
    mut alpha: f32,
    beta: f32,
    player_index: i8,
    params: &MinimaxParameters,
) -> f32 {
    let color = state.get_current_player_color();
    let standing_pat = player_index as f32 * eval_state(state, params);
    if standing_pat + params.futility_margin < alpha {
        return standing_pat;
    }
    alpha = f32::max(alpha, standing_pat);
    if alpha >= beta {
        return beta;
    }
    for action in state.get_captures() {
        let mut state = state.clone();
        state.perform(&action, &color);
        let rate;
        if gamerules::is_finished(&state) {
            rate = rate_mate(&state) * player_index as f32;
        } else {
            rate = -q_search(&state, -beta, -alpha, -player_index, params);
        }
        alpha = f32::max(alpha, rate);
        if alpha >= beta {
            return beta;
        }
    }
    return alpha;
}

pub fn minimax_rate_state(
    stats: &mut SearchStatistics,
    state: &GameState,
    player_index: i8,
    mut alpha: f32,
    mut beta: f32,
    depth: u8,
    tt: &mut TranspositionTable,
    start_time: &time::Tm,
    allotted_time: usize,
    params: &MinimaxParameters,
) -> f32 {
    stats.nodes += 1;
    let hash = MinimalState::from_state(&state);
    let start_alpha = alpha;
    let mut moves: Vec<Move>;
    let move_len;
    if let Some(data) = tt.lookup(&hash) {
        // data = (value, depth, move)
        if data.depth == depth {
            match data.entry {
                EntryType::Exact => return data.value,
                EntryType::UpperBound => beta = f32::min(beta, data.value),
                EntryType::LowerBound => alpha = f32::max(alpha, data.value),
            }
            if alpha >= beta {
                return data.value;
            }
        }
    }
    if gamerules::is_finished(state) {
        return rate_mate(state) * player_index as f32;
    }

    if depth == 0 {
        return q_search(state, alpha, beta, player_index, params);
    }

    moves = state.get_move_list();
    move_len = moves.len();
    if move_len <= 0 {
        return -MATE_SCORE * player_index as f32;
    }

    if depth > 2 && (time::now() - *start_time).num_milliseconds() > allotted_time as i64 {
        return f32::NAN;
    }

    if let Some(data) = tt.lookup(&hash) {
        let index = moves
            .iter()
            .position(|&m| m == data.action)
            .expect("Did not find move in same state?");
        moves.remove(index);
        moves.insert(0, data.action);
    }

    let mut drain = moves.drain(0..move_len);
    let mut best_move: Move = drain.next().expect("Did not find first move");
    let mut best = -minimax_rate(
        stats,
        &state,
        &best_move,
        -player_index,
        -beta,
        -alpha,
        depth - 1,
        tt,
        start_time,
        allotted_time,
        params,
    );
    if best == f32::NAN {
        return f32::NAN;
    }
    if best > alpha {
        if best >= beta {
            if best <= start_alpha {
                tt.insert(&hash, best, depth, &best_move, EntryType::UpperBound);
            } else if best >= beta {
                tt.insert(&hash, best, depth, &best_move, EntryType::LowerBound);
            } else {
                tt.insert(&hash, best, depth, &best_move, EntryType::Exact);
            }
            return best;
        }
        alpha = best;
    }
    let color = state.get_current_player_color();
    while let Some(action_considered) = drain.next() {
        let mut state = state.clone();
        state.perform(&action_considered, &color);
        let mut rate;
        rate = -minimax_rate_state(
            stats,
            &state,
            -player_index,
            -alpha - 1e-5,
            -alpha,
            depth - 1,
            tt,
            start_time,
            allotted_time,
            params,
        );
        if rate == f32::NAN {
            return f32::NAN;
        }
        if alpha < rate && rate < beta {
            rate = -minimax_rate_state(
                stats,
                &state,
                -player_index,
                -beta,
                -alpha,
                depth - 1,
                tt,
                start_time,
                allotted_time,
                params,
            );
            if rate == f32::NAN {
                return f32::NAN;
            }
            if rate > alpha {
                alpha = rate;
            }
        }
        if rate > best {
            best = rate;
            best_move = action_considered;
            if best >= beta {
                break;
            }
        }
    }
    if best <= start_alpha {
        tt.insert(&hash, best, depth, &best_move, EntryType::UpperBound);
    } else if best >= beta {
        tt.insert(&hash, best, depth, &best_move, EntryType::LowerBound);
    } else {
        tt.insert(&hash, best, depth, &best_move, EntryType::Exact);
    }

    return best;
}
