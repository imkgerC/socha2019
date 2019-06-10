use game_sdk::GameState;
use game_sdk::PlayerColor;

use std::f32;
use util::Helper;

#[allow(dead_code)]
pub fn rate_swarm(state: &GameState) -> f32 {
    let r_frac = swarm_frac(state, &PlayerColor::Red);
    let b_frac = swarm_frac(state, &PlayerColor::Blue);
    return r_frac - b_frac;
    // return state.board.red_fields.count_ones() as f32 - state.board.blue_fields.count_ones() as f32;
}

fn swarm_frac(state: &GameState, color: &PlayerColor) -> f32 {
    let field_count = state.board.get_fields_of(color).count_ones();
    let swarm = Helper::greatest_swarm_new(state, color);
    return swarm.count_ones() as f32 / field_count as f32;
}

#[allow(dead_code)]
pub fn moves_rate(state: &GameState) -> f32 {
    let mut state = state.clone();
    state.turn = 0;
    let red_len = state.get_move_list().len() as f32;
    let red_len = red_len / state.board.get_fields_of(&PlayerColor::Red).count_ones() as f32;
    state.turn = 1;
    let blue_len = state.get_move_list().len() as f32;
    let blue_len = blue_len / state.board.get_fields_of(&PlayerColor::Blue).count_ones() as f32;
    return red_len / (blue_len + red_len);
}

#[allow(dead_code)]
pub fn variance_rate(state: &GameState) -> f32 {
    let mut squared_sum_x = 0;
    let mut squared_sum_y = 0;
    let mut sum_x = 0;
    let mut sum_y = 0;
    let mut len = 0;
    for (x, y) in state.get_own_fields(&PlayerColor::Red) {
        let x = x as u32;
        let y = y as u32;
        squared_sum_x += x * x;
        squared_sum_y += y * y;
        sum_x += x;
        sum_y += y;

        len += 1;
    }
    let var_x = squared_sum_x - sum_x * sum_x / len;
    let var_y = squared_sum_y - sum_y * sum_y / len;
    let variance_r = (var_x + var_y) as f32;

    let mut squared_sum_x = 0;
    let mut squared_sum_y = 0;
    let mut sum_x = 0;
    let mut sum_y = 0;
    let mut len = 0;
    for (x, y) in state.get_own_fields(&PlayerColor::Blue) {
        let x = x as u32;
        let y = y as u32;
        squared_sum_x += x * x;
        squared_sum_y += y * y;
        sum_x += x;
        sum_y += y;

        len += 1;
    }
    let var_x = squared_sum_x - sum_x * sum_x / len;
    let var_y = squared_sum_y - sum_y * sum_y / len;
    let variance_b = (var_x + var_y) as f32;

    /*return ((1.0 - (variance_r / (variance_b + variance_r))) + (0.01 / (distance_x + distance_y)))
    / 2.0;*/
    return variance_b - variance_r;
}

fn texel_feature(phase: f32, x: f32, start: f32, end: f32) -> f32 {
    return phase * x * end + (1. - phase) * x * start;
}

pub fn texel_state_single(state: &GameState, color: &PlayerColor) -> f32 {
    let phase = state.turn as f32 / 60.;
    let mut result = 0.0;
    let swarm = Helper::greatest_swarm_new(state, color);
    let mut win: f32 = 0.;
    let mut len = 0;
    for (x, y) in state.get_own_fields(color) {
        win += 1. - (Helper::get_distance_to_swarm_new(x, y, &swarm) as f32 / 9.);
        len += 1;
    }
    //  win, "DISTANCES"
    win = win / len as f32;
    // win, "ADJ_DISTANCES"
    result += texel_feature(phase, win, 1.816, 7.583);
    // swarm.count_ones() as f32, "SWARM_COUNT"
    result += texel_feature(phase, swarm.count_ones() as f32 / 16., -3.389, 6.421);
    // swarm.count_ones() as f32 / 16., "ADJ_SWARM_COUNT",

    let fishes = state.board.get_fields_of(color);

    let mut center = fishes;
    center.mask(297799908644072875622400);
    let center_count = center.count_ones() as f32;
    // center_count, "CENTER_COUNT"
    // center_count / len as f32, "ADJ_CENTER_COUNT",
    result += texel_feature(phase, center_count / len as f32, -0.033, 3.777);

    let mut border = fishes;
    border.mask(1267033445369934637136782821375);
    let border_count = border.count_ones() as f32;
    // border_count, "BORDER_COUNT"
    // border_count / len as f32, "ADJ_BORDER_COUNT",
    result += texel_feature(phase, border_count / len as f32, -8.652, -0.648);

    // len as f32, "COUNT"
    result += texel_feature(phase, len as f32 / 16., 5.228, 4.445);
    // len as f32 / 16., "ADJ_COUNT"
    return result;
}

#[allow(dead_code)]
pub fn texel_state(state: &GameState) -> f32 {
    return texel_state_single(state, &PlayerColor::Red)
        - texel_state_single(state, &PlayerColor::Blue);
}

#[allow(dead_code)]
pub fn prob_state(state: &GameState) -> f32 {
    let r_win = prob_state_single_new(state, &PlayerColor::Red);
    let b_win = prob_state_single_new(state, &PlayerColor::Blue);
    /*let r_win = prob_state_single(state, &PlayerColor::Red);
    let b_win = prob_state_single(state, &PlayerColor::Blue);*/

    return r_win - b_win;
}

#[allow(dead_code)]
pub fn prob_state_single(state: &GameState, color: &PlayerColor) -> f32 {
    let swarm = Helper::greatest_swarm_new(state, color);
    let mut win: f32 = 0.;
    let mut len = 0;
    let enemy_bits =
        state.board.get_fields_of(&color.get_opponent_color()) | state.board.obstacle_fields;

    let mut squared_sum_x = 0;
    let mut squared_sum_y = 0;
    let mut sum_x = 0;
    let mut sum_y = 0;

    for index in state.get_own_indices(color) {
        win += (Helper::get_distance_to_swarm_alt(index, &swarm, &enemy_bits) as f32).powi(2);
        len += 1;

        let x = (index % 10) as u32;
        let y = (index / 10) as u32;
        squared_sum_x += x * x;
        squared_sum_y += y * y;
        sum_x += x;
        sum_y += y;
    }
    win = win / len as f32;
    win += swarm.count_ones() as f32 / 16.;

    win += len as f32 / 16.;
    let var_x = squared_sum_x - sum_x * sum_x / len;
    let var_y = squared_sum_y - sum_y * sum_y / len;
    win -= (var_x + var_y) as f32 * 0.0;

    return win;
}

#[allow(dead_code)]
pub fn experimental_eval(state: &GameState) -> f32 {
    let red_eval = experimental_eval_single(state, &PlayerColor::Red);
    let blue_eval = experimental_eval_single(state, &PlayerColor::Blue);
    return red_eval - blue_eval;
}

fn experimental_eval_single(state: &GameState, color: &PlayerColor) -> f32 {
    let mut result = 0.0;
    let phase = state.turn as f32 / 60.;

    let (swarm, steps) = Helper::greatest_swarm_other(state, color);
    let mut dist = 0.;
    let mut len = 0;
    let enemy_bits = state.board.get_fields_of(&color.get_opponent_color());

    let mut squared_sum_x = 0;
    let mut squared_sum_y = 0;
    let mut sum_x = 0;
    let mut sum_y = 0;

    for index in state.get_own_indices(color) {
        dist -= (Helper::get_distance_to_swarm_alt(index, &swarm, &enemy_bits) as f32).powi(2);
        len += 1;

        let x = (index % 10) as u32;
        let y = (index / 10) as u32;
        squared_sum_x += x * x;
        squared_sum_y += y * y;
        sum_x += x;
        sum_y += y;
    }
    dist = dist / len as f32;
    result += texel_feature(phase, dist, 0.686, 7.378);
    result += texel_feature(phase, swarm.count_ones() as f32, 4.023, 4.754);
    result += 1. / steps as f32;

    result += texel_feature(phase, len as f32, 6.599, 7.095);
    let var_x = squared_sum_x - sum_x * sum_x / len;
    let var_y = squared_sum_y - sum_y * sum_y / len;
    result += texel_feature(phase, -((var_x + var_y) as f32), 1.572, -0.196);

    return result;
}

#[allow(dead_code)]
pub fn clop_rate_colored(state: &GameState, color: &PlayerColor) -> f32 {
    let rate =
        texel_state_single(state, &color.get_opponent_color()) - texel_state_single(state, color);
    return (rate + 250.) / 500.;
}

#[allow(dead_code)]
pub fn clop_state(state: &GameState) -> f32 {
    return clop_state_single(state, &PlayerColor::Red)
        - clop_state_single(state, &PlayerColor::Blue);
}

#[allow(dead_code)]
fn clop_state_single(state: &GameState, color: &PlayerColor) -> f32 {
    let mut result = 0.0;
    let phase = state.turn as f32 / 60.;

    let swarm = Helper::greatest_swarm_new(state, color);
    let mut dist = 0.;
    let mut len = 0;
    let enemy_bits = state.board.get_fields_of(&color.get_opponent_color());

    let mut squared_sum_x = 0;
    let mut squared_sum_y = 0;
    let mut sum_x = 0;
    let mut sum_y = 0;

    for index in state.get_own_indices(color) {
        dist += -(Helper::get_distance_to_swarm_alt(index, &swarm, &enemy_bits) as f32).powi(2);
        len += 1;

        let x = (index % 10) as u32;
        let y = (index / 10) as u32;
        squared_sum_x += x * x;
        squared_sum_y += y * y;
        sum_x += x;
        sum_y += y;
    }
    dist = dist / len as f32;
    result += texel_feature(phase, dist, 0.686, 7.378);
    result += texel_feature(phase, swarm.count_ones() as f32, 4.023, 4.754);

    result += texel_feature(phase, len as f32, 6.599, 7.095);
    let var_x = squared_sum_x - sum_x * sum_x / len;
    let var_y = squared_sum_y - sum_y * sum_y / len;
    result += texel_feature(phase, -((var_x + var_y) as f32), 1.572, -0.196);

    return result;
}

#[allow(dead_code)]
pub fn prob_state_single_new(state: &GameState, color: &PlayerColor) -> f32 {
    let swarm = Helper::greatest_swarm_new(state, color);
    let mut win: f32 = 0.;
    let mut len = 0;
    for (x, y) in state.get_own_fields(color) {
        win += 1. - (Helper::get_distance_to_swarm_new(x, y, &swarm) as f32 / 9.);
        len += 1;
    }
    win = win / len as f32;
    win += swarm.count_ones() as f32 / 16.;

    let fishes = state.board.get_fields_of(color);

    let mut center = fishes;
    center.mask(297799908644072875622400);
    let center_count = center.count_ones() as f32;
    win += center_count / len as f32;

    let mut border = fishes;
    border.mask(1267033445369934637136782821375);
    let border_count = border.count_ones() as f32;
    win -= border_count / len as f32;

    win += len as f32 / 16.;

    return win;
}
