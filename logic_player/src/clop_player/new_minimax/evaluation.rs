use game_sdk::GameState;
use game_sdk::PlayerColor;

use super::player::MinimaxParameters;

use std::f32;
use util::Helper;

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
    let mut dist = 0.;
    let mut len = 0;
    let enemy_bits =
        state.board.get_fields_of(&color.get_opponent_color()) | state.board.obstacle_fields;

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
    result += texel_feature(
        phase,
        dist,
        params.adj_distances_start,
        params.adj_distances_end,
    );
    result += texel_feature(
        phase,
        swarm.count_ones() as f32,
        params.swarm_start,
        params.swarm_end,
    );

    result += texel_feature(phase, len as f32, params.count_start, params.count_end);
    let var_x = squared_sum_x - sum_x * sum_x / len;
    let var_y = squared_sum_y - sum_y * sum_y / len;
    result += texel_feature(
        phase,
        -((var_x + var_y) as f32),
        params.var_start,
        params.var_end,
    );

    return result;
}

#[allow(dead_code)]
pub fn texel_state(state: &GameState, params: &MinimaxParameters) -> f32 {
    return texel_state_single(state, &PlayerColor::Red, params)
        - texel_state_single(state, &PlayerColor::Blue, params);
}
