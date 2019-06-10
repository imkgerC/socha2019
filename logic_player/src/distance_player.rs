use game_sdk::logging::Data;
use game_sdk::ClientListener;
use game_sdk::GameState;
use game_sdk::Move;
use game_sdk::PlayerColor;
use std::sync::mpsc;

use util::Helper;

#[derive(Clone)]
pub struct SingleDistancePlayer;

impl SingleDistancePlayer {
    pub fn new(_: Option<mpsc::Sender<Data>>, _: i64) -> SingleDistancePlayer {
        return SingleDistancePlayer {};
    }

    pub fn rate(action: &Move, swarm: &Vec<(u8, u8)>) -> i32 {
        return Helper::get_distance_to_swarm(action.x, action.y, swarm) as i32
            - Helper::get_distance_to_swarm(action.dest_x, action.dest_y, swarm) as i32;
    }
}

impl ClientListener for SingleDistancePlayer {
    fn on_move_request(&mut self, state: &GameState) -> Move {
        let mut moves = state.get_move_list().into_iter();
        let color = state.get_current_player_color();
        let swarm = Helper::greatest_swarm(state, &color);
        let mut action;
        let mut max_rate;
        if let Some(action_considered) = moves.next() {
            max_rate = SingleDistancePlayer::rate(&action_considered, &swarm);
            action = action_considered;
        } else {
            panic!("No move found");
        }
        while let Some(action_considered) = moves.next() {
            let rate = SingleDistancePlayer::rate(&action_considered, &swarm);
            if rate > max_rate {
                max_rate = rate;
                action = action_considered;
            }
        }
        return action;
    }
}

#[derive(Clone)]
pub struct MultiDistancePlayer;
impl MultiDistancePlayer {
    pub fn new(_: Option<mpsc::Sender<Data>>, _: i64) -> MultiDistancePlayer {
        return MultiDistancePlayer {};
    }

    fn rate_with_move(state: &GameState, action: &Move, color: &PlayerColor) -> i32 {
        let mut state_clone = state.clone();
        state_clone.perform(action, color);
        return MultiDistancePlayer::rate(&state_clone, color);
    }

    fn rate(state: &GameState, color: &PlayerColor) -> i32 {
        let swarm = Helper::greatest_swarm(state, color);
        let other_swarm = Helper::greatest_swarm(state, &color.get_opponent_color());
        let mut distance = 0;
        for (x, y) in state.get_own_fields(color) {
            distance += Helper::get_distance_to_swarm(x, y, &swarm) as i32;
        }
        for (x, y) in state.get_own_fields(&color.get_opponent_color()) {
            distance -= Helper::get_distance_to_swarm(x, y, &other_swarm) as i32;
        }
        return distance;
    }
}

impl ClientListener for MultiDistancePlayer {
    fn on_move_request(&mut self, state: &GameState) -> Move {
        let mut moves = state.get_move_list().into_iter();
        let color = state.get_current_player_color();

        let before_rate = MultiDistancePlayer::rate(state, &color);

        let mut action;
        let mut max_rate: i32;
        if let Some(action_considered) = moves.next() {
            max_rate = before_rate
                - MultiDistancePlayer::rate_with_move(&state, &action_considered, &color);
            action = action_considered;
        } else {
            panic!("No move found");
        }
        while let Some(action_considered) = moves.next() {
            let rate = before_rate
                - MultiDistancePlayer::rate_with_move(&state, &action_considered, &color);
            if rate > max_rate {
                max_rate = rate;
                action = action_considered;
            }
        }
        //println!("best rate {}", max_rate);
        return action;
    }
}
