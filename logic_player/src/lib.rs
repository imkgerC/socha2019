extern crate game_sdk;
extern crate hashbrown;
extern crate logger;
extern crate rand;
extern crate time;

mod distance_player;
mod hybrid;
mod legacy_minimax;
mod legacy_rave;
mod mcts_rave;
mod minimax;
mod util;

/*mod pleco_clone;
pub use pleco_clone::search::Searcher;
pub use pleco_clone::PlecoPlayer;*/

pub use distance_player::MultiDistancePlayer;
pub use distance_player::SingleDistancePlayer;
pub use hybrid::HybridPlayer;
pub use hybrid::LegacyHybridPlayer;
pub use legacy_minimax::LegacyMinimaxPlayer;
pub use legacy_rave::LegacyRavePlayer;
pub use mcts_rave::RavePlayer;
pub use minimax::MinimaxPlayer;

use game_sdk::ClientListener;
use game_sdk::GameState;
use game_sdk::{gamerules, Move};
use rand::rngs::SmallRng;
use rand::{FromEntropy, Rng};

mod clop_player;
pub use clop_player::{ClopParameters, EnemyPool, ToClop};

use game_sdk::logging::Data;
use std::sync::mpsc;

#[derive(Clone)]
pub enum Player {
	RandomPlayer(RandomPlayer),
	LogicBasedPlayer(LogicBasedPlayer),
	SingleDistancePlayer(SingleDistancePlayer),
	MultiDistancePlayer(MultiDistancePlayer),
	MinimaxPlayer(MinimaxPlayer),
}

impl std::fmt::Display for Player {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Player::RandomPlayer(_) => "RandomPlayer",
				Player::LogicBasedPlayer(_) => "LogicBasedPlayer",
				Player::SingleDistancePlayer(_) => "SingleDistancePlayer",
				Player::MultiDistancePlayer(_) => "MultiDistancePlayer",
				Player::MinimaxPlayer(_) => "MinimaxPlayer",
			}
		)
	}
}

impl ClientListener for Player {
	fn on_move_request(&mut self, state: &GameState) -> Move {
		return match self {
			Player::RandomPlayer(p) => p.on_move_request(state),
			Player::LogicBasedPlayer(p) => p.on_move_request(state),
			Player::SingleDistancePlayer(p) => p.on_move_request(state),
			Player::MultiDistancePlayer(p) => p.on_move_request(state),
			Player::MinimaxPlayer(p) => p.on_move_request(state),
		};
	}
}

#[derive(Clone)]
pub struct RandomPlayer {
	rng: SmallRng,
}

impl RandomPlayer {
	pub fn new(_: Option<mpsc::Sender<Data>>, _: i64) -> RandomPlayer {
		return RandomPlayer {
			rng: SmallRng::from_entropy(),
		};
	}
}

impl ClientListener for RandomPlayer {
	fn on_move_request(&mut self, state: &GameState) -> Move {
		let moves: Vec<Move> = state.get_move_list();
		let rng_number: usize = self.rng.gen_range(0, moves.len());
		if let Some(a) = moves.get(rng_number) {
			return *a;
		}
		println!("{:?}", moves);
		panic!("Did not find move for RandomPlayer");
	}
}

#[derive(Clone)]
pub struct LogicBasedPlayer;

impl LogicBasedPlayer {
	pub fn new(_: Option<mpsc::Sender<Data>>, _: i64) -> LogicBasedPlayer {
		return LogicBasedPlayer {};
	}

	#[allow(unused)]
	fn rate(state_in: &GameState, action: &Move) -> i32 {
		let mut state = state_in.clone();
		let color = state.get_current_player_color();
		state.perform(action, &color);
		return LogicBasedPlayer::rate_state(&state);
	}

	fn rate_state(state: &GameState) -> i32 {
		let mut squared_sum_x = 0;
		let mut squared_sum_y = 0;
		let mut sum_x = 0;
		let mut sum_y = 0;
		let mut len = 0;
		for (x, y) in state.get_own_fields(&state.get_current_player_color().get_opponent_color()) {
			let x = x as i32;
			let y = y as i32;
			squared_sum_x += x * x;
			squared_sum_y += y * y;
			sum_x += x;
			sum_y += y;

			len += 1;
		}
		let var_x = squared_sum_x - sum_x * sum_x / len;
		let var_y = squared_sum_y - sum_y * sum_y / len;
		return var_x + var_y;
	}

	pub fn get_sums(state: &GameState) -> (i32, i32, i32, i32, i32) {
		let mut squared_sum_x = 0;
		let mut squared_sum_y = 0;
		let mut sum_x = 0;
		let mut sum_y = 0;
		let mut len = 0;
		for (x, y) in state.get_own_fields(&state.get_current_player_color()) {
			let x = x as i32;
			let y = y as i32;
			squared_sum_x += x * x;
			squared_sum_y += y * y;
			sum_x += x;
			sum_y += y;

			len += 1;
		}
		return (squared_sum_x, squared_sum_y, sum_x, sum_y, len);
	}

	pub fn easy_rate(
		squared_x: i32,
		squared_y: i32,
		x: i32,
		y: i32,
		len: i32,
		action: &Move,
	) -> i32 {
		let mut squared_x = squared_x;
		squared_x += (action.dest_x * action.dest_x) as i32;
		squared_x -= (action.x * action.x) as i32;
		let mut squared_y = squared_y;
		squared_y -= (action.y * action.y) as i32;
		squared_y += (action.dest_y * action.dest_y) as i32;
		let x = x + action.dest_x as i32 - action.x as i32;
		let y = y + action.dest_y as i32 - action.y as i32;
		let var_x = squared_x - x * x / len;
		let var_y = squared_y - y * y / len;
		return var_x + var_y;
	}

	pub fn get_rated_moves(state: &GameState) -> Vec<(f32, Move)> {
		if gamerules::is_finished(state) {
			return Vec::new();
		}
		let mut moves = state.get_move_list().into_iter();
		let mut result = Vec::with_capacity(40);
		let (squared_x, squared_y, x, y, len) = LogicBasedPlayer::get_sums(&state);
		let mut sum = 0.;
		while let Some(action_considered) = moves.next() {
			let rate =
				LogicBasedPlayer::easy_rate(squared_x, squared_y, x, y, len, &action_considered);
			sum += rate as f32;
			result.push((rate, action_considered));
		}
		let result = result
			.into_iter()
			.map(|(rate, action)| (rate as f32 / sum, action))
			.collect();
		return result;
	}

	pub fn on_state(state: &GameState) -> Option<Move> {
		let mut moves = state.get_move_list().into_iter();
		let mut action;
		let mut min_rate;
		let (squared_x, squared_y, x, y, len) = LogicBasedPlayer::get_sums(&state);
		if let Some(action_considered) = moves.next() {
			// min_rate = LogicBasedPlayer::rate(&state, &action_considered);
			min_rate =
				LogicBasedPlayer::easy_rate(squared_x, squared_y, x, y, len, &action_considered);
			action = action_considered;
		} else {
			return None;
		}
		while let Some(action_considered) = moves.next() {
			let rate =
				LogicBasedPlayer::easy_rate(squared_x, squared_y, x, y, len, &action_considered);
			if rate < min_rate {
				min_rate = rate;
				action = action_considered;
			}
		}
		return Some(action);
	}

	pub fn interesting_moves(state: &GameState) -> Vec<Move> {
		let mut moves = state.get_move_list().into_iter();
		let mut result = Vec::new();
		let (squared_x, squared_y, x, y, len) = LogicBasedPlayer::get_sums(&state);
		let base_rate = (squared_x - x * x / len) + (squared_y - y * y / len);
		let lower_bound = (base_rate as f32 * 0.9) as i32;
		while let Some(action_considered) = moves.next() {
			// let rate = LogicBasedPlayer::rate(&state, &action_considered);
			let rate =
				LogicBasedPlayer::easy_rate(squared_x, squared_y, x, y, len, &action_considered);
			if rate < lower_bound {
				result.push(action_considered);
			}
		}
		return result;
	}
}

impl ClientListener for LogicBasedPlayer {
	fn on_move_request(&mut self, state: &GameState) -> Move {
		let mut moves = state.get_move_list().into_iter();
		let mut action;
		let mut min_rate;
		let (squared_x, squared_y, x, y, len) = LogicBasedPlayer::get_sums(&state);
		if let Some(action_considered) = moves.next() {
			// min_rate = LogicBasedPlayer::rate(&state, &action_considered);
			min_rate =
				LogicBasedPlayer::easy_rate(squared_x, squared_y, x, y, len, &action_considered);
			action = action_considered;
		} else {
			panic!("No move found");
		}
		while let Some(action_considered) = moves.next() {
			// let rate = LogicBasedPlayer::rate(&state, &action_considered);
			let rate =
				LogicBasedPlayer::easy_rate(squared_x, squared_y, x, y, len, &action_considered);
			if rate < min_rate {
				min_rate = rate;
				action = action_considered;
			}
		}
		return action;
	}
}
