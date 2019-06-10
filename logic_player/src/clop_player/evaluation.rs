use game_sdk::GameState;
use game_sdk::PlayerColor;

use super::player::MinimaxParameters;

use std::f32;
use util::Helper;

#[allow(dead_code)]
pub fn prob_state(state: &GameState) -> f32 {
    let r_win = prob_state_single_new(state, &PlayerColor::Red);
    let b_win = prob_state_single_new(state, &PlayerColor::Blue);

    return r_win - b_win;
}

fn texel_feature(phase: f32, x: f32, start: f32, end: f32) -> f32 {
    return phase * x * end + (1. - phase) * x * start;
}

pub fn texel_state_single(
    state: &GameState,
    color: &PlayerColor,
    params: &MinimaxParameters,
) -> f32 {
    let phase = state.turn as f32 / 60.;
    let mut result = 0.0;

    let swarm = Helper::greatest_swarm_new(state, color);
    let mut win: f32 = 0.;
    let mut len = 0;
    for (x, y) in state.get_own_fields(color) {
        win += 1. - (Helper::get_distance_to_swarm_new(x, y, &swarm) as f32 / 9.);
        len += 1;
    }
    // "ADJ_DISTANCES"
    result += texel_feature(
        phase,
        win / len as f32,
        params.adj_distances_start,
        params.adj_distances_end,
    );

    // swarm.count_ones() as f32, "SWARM_COUNT"
    result += texel_feature(
        phase,
        swarm.count_ones() as f32 / 16.,
        params.swarm_start,
        params.swarm_end,
    );

    let fishes = state.board.get_fields_of(color);
    let mut center = fishes;
    center.mask(297799908644072875622400);
    let center_count = center.count_ones() as f32;
    // center_count / len as f32, "ADJ_CENTER_COUNT",
    result += texel_feature(
        phase,
        center_count / len as f32,
        params.adj_center_start,
        params.adj_center_end,
    );

    let mut border = fishes;
    border.mask(1267033445369934637136782821375);
    let border_count = border.count_ones() as f32;
    // border_count / len as f32, "ADJ_BORDER_COUNT",
    result += texel_feature(
        phase,
        border_count / len as f32,
        params.adj_border_start,
        params.adj_border_end,
    );

    // len as f32, "ADJ_COUNT"
    result += texel_feature(phase, len as f32 / 16., params.count_start, params.count_end);

    return result;
}

#[allow(dead_code)]
pub fn texel_state(state: &GameState, params: &MinimaxParameters) -> f32 {
    return texel_state_single(state, &PlayerColor::Red, &params)
        - texel_state_single(state, &PlayerColor::Blue, &params);
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
