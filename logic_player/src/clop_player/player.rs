use game_sdk::ClientListener;
use game_sdk::GameState;
use game_sdk::Move;
use game_sdk::PlayerColor;

use super::algorithm::{minimax_rate_state, SearchStatistics, MATE_SCORE};
use super::transposition::TranspositionTable;

use std::f32;
use time;

static START_DEPTH: u8 = 0;

#[derive(Clone)]
pub struct MinimaxParameters {
	pub aspiration_window: f32,
	pub adj_distances_start: f32,
	pub adj_distances_end: f32,
	pub swarm_start: f32,
	pub swarm_end: f32,
	pub adj_center_start: f32,
	pub adj_center_end: f32,
	pub adj_border_start: f32,
	pub adj_border_end: f32,
	pub count_start: f32,
	pub count_end: f32,
	pub search_q: bool,
}

impl MinimaxParameters {
	pub fn empty() -> MinimaxParameters {
		MinimaxParameters {
			aspiration_window: 0.0,
			adj_distances_start: 0.0,
			adj_distances_end: 0.0,
			swarm_start: 0.0,
			swarm_end: 0.0,
			adj_center_start: 0.0,
			adj_center_end: 0.0,
			adj_border_start: 0.0,
			adj_border_end: 0.0,
			count_start: 0.0,
			count_end: 0.0,
			search_q: true,
		}
	}

	pub fn set_var_from_string(&mut self, identifier: String, val: String) {
		match &identifier[..] {
			"aspiration_window" => {
				self.aspiration_window = val.parse().expect("Got wrong val");
			}
			"adj_distances_start" => {
				self.adj_distances_start = val.parse().expect("Got wrong val");
			}
			"adj_distances_end" => {
				self.adj_distances_end = val.parse().expect("Got wrong val");
			}
			"swarm_start" => {
				self.swarm_start = val.parse().expect("Got wrong val");
			}
			"swarm_end" => {
				self.swarm_end = val.parse().expect("Got wrong val");
			}
			"adj_center_start" => {
				self.adj_center_start = val.parse().expect("Got wrong val");
			}
			"adj_center_end" => {
				self.adj_center_end = val.parse().expect("Got wrong val");
			}
			"adj_border_start" => {
				self.adj_border_start = val.parse().expect("Got wrong val");
			}
			"adj_border_end" => {
				self.adj_border_end = val.parse().expect("Got wrong val");
			}
			"count_start" => {
				self.count_start = val.parse().expect("Got wrong val");
			}
			"count_end" => {
				self.count_end = val.parse().expect("Got wrong val");
			}
			_ => {
				panic!("Wrong identifier");
			}
		};
	}
}

#[derive(Clone)]
pub struct MinimaxPlayer {
	tt: TranspositionTable,
	params: MinimaxParameters,
}
impl MinimaxPlayer {
	pub fn new(params: MinimaxParameters) -> MinimaxPlayer {
		return MinimaxPlayer {
			tt: TranspositionTable::new(),
			params,
		};
	}
}

impl ClientListener for MinimaxPlayer {
	fn on_move_request(&mut self, state: &GameState) -> Move {
		let before = time::now();
		let player_index = match state.get_current_player_color() {
			PlayerColor::Red => 1,
			PlayerColor::Blue => -1,
		};

		let mut playable_moves = state.get_move_list();
		let mut action = None;
		let mut best = -MATE_SCORE;

		let mut current_index = 0;
		let mut current_depth = START_DEPTH;
		let mut current_depth_best = -MATE_SCORE;
		let mut current_depth_best_move = None;
		let color = state.get_current_player_color();
		let mut search_stats = SearchStatistics::new();
		let max_time = 1700;

		while (time::now() - before).num_milliseconds() < max_time as i64 {
			search_stats.nodes += 1;
			if current_depth == START_DEPTH {
				let action_considered = playable_moves
					.get(current_index)
					.expect("Wrong index for move list");
				let mut state = state.clone();
				state.perform(&action_considered, &color);
				let rate = -minimax_rate_state(
					&mut search_stats,
					&state,
					-player_index,
					-MATE_SCORE,
					MATE_SCORE,
					current_depth,
					&mut self.tt,
					&before,
					max_time,
					&self.params,
				);
				if rate == f32::NAN {
					break;
				}

				if rate >= current_depth_best {
					current_depth_best = rate;
					current_depth_best_move = Some(action_considered.clone());
				}
			} else if current_index == 0 {
				let action_considered = playable_moves
					.get(current_index)
					.expect("Wrong index for move list");
				let mut state = state.clone();
				state.perform(&action_considered, &color);
				let mut rate = -minimax_rate_state(
					&mut search_stats,
					&state,
					-player_index,
					-best - self.params.aspiration_window,
					-best + self.params.aspiration_window,
					current_depth,
					&mut self.tt,
					&before,
					max_time,
					&self.params,
				);
				if rate == f32::NAN {
					break;
				}
				search_stats.aspire_probed += 1;
				if rate >= best + self.params.aspiration_window
					|| rate <= best - self.params.aspiration_window
				{
					search_stats.aspire_re += 1;
					rate = -minimax_rate_state(
						&mut search_stats,
						&state,
						-player_index,
						-MATE_SCORE,
						MATE_SCORE,
						current_depth - 1,
						&mut self.tt,
						&before,
						max_time,
						&self.params,
					);
					if rate == f32::NAN {
						break;
					}
				}
				current_depth_best = rate;
				current_depth_best_move = Some(action_considered.clone());
			} else {
				let action_considered = playable_moves
					.get(current_index)
					.expect("Wrong index for move list");
				let mut state = state.clone();
				state.perform(&action_considered, &color);
				let mut rate;
				rate = -minimax_rate_state(
					&mut search_stats,
					&state,
					-player_index,
					-current_depth_best - 1e-5,
					-current_depth_best,
					current_depth - 1,
					&mut self.tt,
					&before,
					max_time,
					&self.params,
				);
				if rate == f32::NAN {
					break;
				}
				search_stats.probed += 1;
				if rate > current_depth_best {
					search_stats.re_searched += 1;
					rate = -minimax_rate_state(
						&mut search_stats,
						&state,
						-player_index,
						-MATE_SCORE,
						MATE_SCORE,
						current_depth - 1,
						&mut self.tt,
						&before,
						max_time,
						&self.params,
					);
					if rate == f32::NAN {
						break;
					}
					current_depth_best = rate;
					current_depth_best_move = Some(action_considered.clone());
				}
			}

			current_index += 1;
			if current_index >= playable_moves.len() || current_depth_best == MATE_SCORE {
				current_depth += 1;
				current_index = 0;

				best = current_depth_best;
				action = current_depth_best_move;
				let index = playable_moves
					.iter()
					.position(|&m| m == action.expect("Did not find action for re-ordering"))
					.expect("Did not find move in same state?");
				playable_moves.remove(index);
				playable_moves.insert(0, action.expect("SHOULD NEVER HAPPEN"));

				current_depth_best = -MATE_SCORE;
				current_depth_best_move = None;
				if (best == -MATE_SCORE || best == MATE_SCORE) && action != None {
					break;
				}
			}
		}
		if current_depth_best_move != action && current_depth_best_move != None {
			if current_depth_best > -MATE_SCORE {
				action = current_depth_best_move;
			}
		}
		if let Some(action) = action {
			return action;
		} else {
			if let Some(action) = current_depth_best_move {
				return action;
			}
			panic!("No playable move found");
		}
	}
}
