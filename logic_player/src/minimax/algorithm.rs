use game_sdk::{gamerules, GameState, Move, PlayerColor};

use super::evaluation::clop_state as eval_state;
use super::transposition::{EntryType, MinimalState, TranspositionTable};

use std::f32;

pub static MATE_SCORE: f32 = 200_000.0;
pub static MAX_MATE_PENALTY: f32 = 32.;
static FUTILITY_MARGIN: f32 = 38.641;

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

pub fn rate_mate(state: &GameState) -> f32 {
    let red_size = state.greatest_swarm_size(&PlayerColor::Red);
    let blue_size = state.greatest_swarm_size(&PlayerColor::Blue);

    let red_count = state.board.red_fields.count_ones() as u8;
    let blue_count = state.board.blue_fields.count_ones() as u8;

    let swarm_bonus = red_size as f32 - blue_size as f32;

    if red_size == red_count {
        if blue_size == blue_count {
            // both are connected
            if red_size < blue_size {
                // blue is bigger and won
                return -MATE_SCORE + swarm_bonus - 16.;
            }
            if red_size == blue_size {
                // draw, both connected + equal in size
                return 0.;
            }
            // red is bigger, go to default
        }
        return MATE_SCORE + swarm_bonus + 16.;
    }
    if blue_size == blue_count {
        // red is not connected
        return -MATE_SCORE + swarm_bonus - 16.;
    }

    // no one is connected. We assume, this function is called, only when game is finished
    // so turn60-adjudication can proceed
    if red_size > blue_size {
        return MATE_SCORE + swarm_bonus + 16.;
    }
    if red_size < blue_size {
        return -MATE_SCORE + swarm_bonus - 16.;
    }
    // they are equal in size, draw
    return 0.;
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
    );
}

#[allow(dead_code)]
pub fn q_search(state: &GameState, mut alpha: f32, beta: f32, player_index: i8) -> f32 {
    let color = state.get_current_player_color();
    let standing_pat = player_index as f32 * eval_state(state);
    if standing_pat + FUTILITY_MARGIN < alpha {
        return standing_pat;
    }
    alpha = f32::max(alpha, standing_pat);
    let mut best = standing_pat;
    if alpha > beta {
        return alpha;
    }
    for action in state.get_captures() {
        let mut state = state.clone();
        state.perform(&action, &color);
        let rate;
        if gamerules::is_finished(&state) {
            rate = 0.9 * rate_mate(&state) * player_index as f32;
        } else {
            rate = -q_search(&state, -beta, -alpha, -player_index);
        }
        alpha = f32::max(alpha, rate);
        best = f32::max(best, rate);
        if alpha > beta {
            return alpha;
        }
    }
    // return alpha;
    return best;
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
) -> f32 {
    stats.nodes += 1;
    let hash = MinimalState::from_state(&state);
    let start_alpha = alpha;
    let mut moves: Vec<Move>;
    let move_len;
    let mut found_move = None;
    if let Some(data) = tt.lookup(&hash) {
        // data = (value, depth, move)
        if data.depth >= depth {
            match data.entry {
                EntryType::Exact => return data.value,
                EntryType::UpperBound => beta = f32::min(beta, data.value),
                EntryType::LowerBound => alpha = f32::max(alpha, data.value),
            }
            if alpha > beta {
                stats.aspire_probed += 1;
                stats.aspire_re += 1;
                return alpha;
            }
        }
        found_move = Some(data.action);
    } else {
        if gamerules::is_finished(state) {
            return rate_mate(state) * player_index as f32;
        }
    }
    /*if depth == 0 {
        return player_index as f32 * eval_state(state);
    }*/
    if depth == 0 {
        return q_search(state, alpha, beta, player_index);
    }

    if let Some(action) = found_move {
        let rate = -minimax_rate(
            stats,
            &state,
            &action,
            -player_index,
            -beta,
            -alpha,
            depth - 1,
            tt,
            start_time,
        );
        if rate.is_nan() {
            return f32::NAN;
        }
        if rate > alpha {
            if rate > beta {
                stats.aspire_probed += 1;
                stats.aspire_re += 1;
                tt.insert(&hash, rate, depth, &action, EntryType::LowerBound);
                return rate;
            }
            alpha = rate;
        }
    }

    if depth > 2 && (time::now() - *start_time).num_milliseconds() > 1700 {
        return f32::NAN;
    }

    moves = state.get_move_list();
    move_len = moves.len();
    if move_len <= 0 {
        return -MATE_SCORE * player_index as f32;
    }

    let mut drain = moves.drain(0..move_len);
    let mut best_move = None; // drain.next().expect("Did not find first move");
    let mut best = f32::NEG_INFINITY;
    let color = state.get_current_player_color();
    while let Some(action_considered) = drain.next() {
        let mut state = state.clone();
        state.perform(&action_considered, &color);
        let mut rate;
        if best_move != None && (beta - alpha < 1e-3) && depth >= 2 {
            let static_eval = eval_state(&state) * player_index as f32;
            if static_eval < alpha {
                rate = -minimax_rate_state(
                    stats,
                    &state,
                    -player_index,
                    -alpha - 1e-5,
                    -alpha,
                    depth - 2,
                    tt,
                    start_time,
                );
                if rate < best {
                    continue;
                }
            } else if static_eval - FUTILITY_MARGIN > beta {
                rate = -minimax_rate_state(
                    stats,
                    &state,
                    -player_index,
                    -alpha - 1e-5,
                    -alpha,
                    depth - 2,
                    tt,
                    start_time,
                );
                if rate - FUTILITY_MARGIN > beta {
                    return rate;
                }
            }
        }
        rate = -minimax_rate_state(
            stats,
            &state,
            -player_index,
            -alpha - 1e-5,
            -alpha,
            depth - 1,
            tt,
            start_time,
        );
        if rate.is_nan() {
            return f32::NAN;
        }
        if alpha < rate && rate <= beta {
            rate = -minimax_rate_state(
                stats,
                &state,
                -player_index,
                -beta,
                -alpha,
                depth - 1,
                tt,
                start_time,
            );
            if rate.is_nan() {
                return f32::NAN;
            }
        }
        if rate > best {
            alpha = f32::max(rate, alpha);
            best = rate;
            best_move = Some(action_considered);
            if alpha > beta {
                stats.aspire_probed += 1;
                break;
            }
        }
    }
    if best <= start_alpha {
        tt.insert(
            &hash,
            best,
            depth,
            &best_move.expect("Did not find any move after checking"),
            EntryType::UpperBound,
        );
    } else if best > beta {
        tt.insert(
            &hash,
            best,
            depth,
            &best_move.expect("Did not find any move after checking"),
            EntryType::LowerBound,
        );
    } else {
        tt.insert(
            &hash,
            best,
            depth,
            &best_move.expect("Did not find any move after checking"),
            EntryType::Exact,
        );
    }

    return best;
}
