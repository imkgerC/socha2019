use game_sdk::{ClientListener, GameState, Move, PlayerColor};

use crate::util::Helper;
use super::algorithm::{minimax_rate_state, SearchStatistics, MATE_SCORE, MAX_MATE_PENALTY};
use super::transposition::TranspositionTable;

use game_sdk::logging::{Data, MoveValuePair, State};
use std::f32;
use std::sync::mpsc;
use time;

static START_DEPTH: u8 = 0;
// static WINDOW_SIZE: f32 = 19.474;

#[derive(Clone)]
pub struct MinimaxPlayer {
	id: i64,
	tt: TranspositionTable,
	tx: Option<mpsc::Sender<Data>>,
}
impl MinimaxPlayer {
	pub fn new(tx: Option<mpsc::Sender<Data>>, id: i64) -> MinimaxPlayer {
		return MinimaxPlayer {
			id,
			tt: TranspositionTable::new(),
			tx,
		};
	}
}

impl MinimaxPlayer {
	pub fn move_with_id(&mut self, state: &GameState, id: i64) -> Move {
		// self.tt = TranspositionTable::new();
		let before = time::now();
		let player_index = match state.get_current_player_color() {
			PlayerColor::Red => 1,
			PlayerColor::Blue => -1,
		};

		let mut playable_moves = state.get_move_list();
		let mut action = None;
		let mut best = -MATE_SCORE - MAX_MATE_PENALTY;

		let mut current_index = 0;
		let mut current_depth = START_DEPTH;
		let mut current_depth_best = -MATE_SCORE - MAX_MATE_PENALTY;
		let mut current_depth_best_move = None;
		let color = state.get_current_player_color();
		let mut search_stats = SearchStatistics::new();

		while (time::now() - before).num_milliseconds() < 1700 {
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
					-MATE_SCORE - MAX_MATE_PENALTY,
					MATE_SCORE + MAX_MATE_PENALTY,
					current_depth,
					&mut self.tt,
					&before,
				);
				if rate.is_nan() {
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
				current_depth_best = -minimax_rate_state(
					&mut search_stats,
					&state,
					-player_index,
					-MATE_SCORE - MAX_MATE_PENALTY,
					MATE_SCORE + MAX_MATE_PENALTY,
					current_depth - 1,
					&mut self.tt,
					&before,
				);
				if current_depth_best.is_nan() {
					break;
				}
				current_depth_best_move = Some(action_considered.clone());
			/*let mut rate = -minimax_rate_state(
				&mut search_stats,
				&state,
				-player_index,
				-best - WINDOW_SIZE,
				-best + WINDOW_SIZE,
				current_depth,
				&mut self.tt,
				&before,
			);
			if rate == f32::NAN {
				break;
			}
			search_stats.aspire_probed += 1;
			if rate >= best + WINDOW_SIZE || rate <= best - WINDOW_SIZE {
				search_stats.aspire_re += 1;
				rate = -minimax_rate_state(
					&mut search_stats,
					&state,
					-player_index,
					-MATE_SCORE - MAX_MATE_PENALTY,
					MATE_SCORE + MAX_MATE_PENALTY,
					current_depth - 1,
					&mut self.tt,
					&before,
				);
				if rate == f32::NAN {
					break;
				}
			}
			current_depth_best = rate;
			current_depth_best_move = Some(action_considered.clone());*/
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
						-MATE_SCORE - MAX_MATE_PENALTY,
						MATE_SCORE + MAX_MATE_PENALTY,
						current_depth - 1,
						&mut self.tt,
						&before,
					);
					if rate == f32::NAN {
						break;
					}
					if rate >= current_depth_best {
						current_depth_best = rate;
						current_depth_best_move = Some(action_considered.clone());
					}
				}
			}

			current_index += 1;
			if current_index >= playable_moves.len() || current_depth_best >= MATE_SCORE {
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

				current_depth_best = -MATE_SCORE - MAX_MATE_PENALTY;
				current_depth_best_move = None;
				if (best <= -MATE_SCORE || best >= MATE_SCORE) && action != None {
					break;
				}
			}
		}
		if current_depth_best_move != action {
			if current_depth_best > -MATE_SCORE {
				action = current_depth_best_move;
				current_depth += 1;
			}
		}
		let moves: Vec<MoveValuePair> = Vec::new();
		let send_state = State {
			id: id as u32,
			gamestate: state.clone(),
			moves,
			data: MinimaxPlayer::get_data_vec(&state),
		};
		if let Some(ref tx) = self.tx {
			tx.send(Data::Step(send_state)).unwrap();
		} else if id < 0 {
			let ms_used = (time::now() - before).num_milliseconds();
			println!(
				"|{}| {}ms | {} nodes | val {:.3} | re-rate {:.5}/{:.2} | depth {} | {:.0} nps",
				state.turn,
				ms_used,
				search_stats.nodes,
				best,
				search_stats.re_searched as f32 / search_stats.probed as f32,
				search_stats.aspire_re as f32 / search_stats.aspire_probed as f32,
				current_depth - 1,
				search_stats.nodes as f32 / (ms_used as f32 / 1000.),
			);
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

	pub fn get_move_and_rate(state: &GameState, max_depth: u8) -> (Move, f32, Vec<(Move, f32)>) {
		let mut tt = TranspositionTable::new();
		let before = time::now();
		let player_index = match state.get_current_player_color() {
			PlayerColor::Red => 1,
			PlayerColor::Blue => -1,
		};

		let mut playable_moves = state.get_move_list();
		let mut action = None;
		let mut rated_moves = Vec::new();
		let mut current_rated_moves = Vec::new();
		let mut best = -MATE_SCORE - MAX_MATE_PENALTY;

		let mut current_index = 0;
		let mut current_depth = START_DEPTH;
		let mut current_depth_best = -MATE_SCORE - MAX_MATE_PENALTY;
		let mut current_depth_best_move = None;
		let color = state.get_current_player_color();
		let mut search_stats = SearchStatistics::new();

		while current_depth <= max_depth {
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
					-MATE_SCORE - MAX_MATE_PENALTY,
					MATE_SCORE + MAX_MATE_PENALTY,
					current_depth,
					&mut tt,
					&before,
				);
				if rate == f32::NAN {
					break;
				}

				if rate >= current_depth_best {
					current_depth_best = rate;
					current_depth_best_move = Some(action_considered.clone());
				}
				current_rated_moves.push((action_considered.clone(), rate));
			} else if current_index == 0 {
				let action_considered = playable_moves
					.get(current_index)
					.expect("Wrong index for move list");
				let mut state = state.clone();
				state.perform(&action_considered, &color);
				current_depth_best = -minimax_rate_state(
					&mut search_stats,
					&state,
					-player_index,
					-MATE_SCORE - MAX_MATE_PENALTY,
					MATE_SCORE + MAX_MATE_PENALTY,
					current_depth - 1,
					&mut tt,
					&before,
				);
				if current_depth_best == f32::NAN {
					break;
				}
				current_depth_best_move = Some(action_considered.clone());
				current_rated_moves.push((action_considered.clone(), current_depth_best));
			/*let mut rate = -minimax_rate_state(
				&mut search_stats,
				&state,
				-player_index,
				-best - WINDOW_SIZE,
				-best + WINDOW_SIZE,
				current_depth,
				&mut self.tt,
				&before,
			);
			if rate == f32::NAN {
				break;
			}
			search_stats.aspire_probed += 1;
			if rate >= best + WINDOW_SIZE || rate <= best - WINDOW_SIZE {
				search_stats.aspire_re += 1;
				rate = -minimax_rate_state(
					&mut search_stats,
					&state,
					-player_index,
					-MATE_SCORE - MAX_MATE_PENALTY,
					MATE_SCORE + MAX_MATE_PENALTY,
					current_depth - 1,
					&mut self.tt,
					&before,
				);
				if rate == f32::NAN {
					break;
				}
			}
			current_depth_best = rate;
			current_depth_best_move = Some(action_considered.clone());*/
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
					&mut tt,
					&before,
				);
				if rate.is_nan() {
					break;
				}
				search_stats.probed += 1;
				if rate > current_depth_best {
					search_stats.re_searched += 1;
					rate = -minimax_rate_state(
						&mut search_stats,
						&state,
						-player_index,
						-MATE_SCORE - MAX_MATE_PENALTY,
						MATE_SCORE + MAX_MATE_PENALTY,
						current_depth - 1,
						&mut tt,
						&before,
					);
					if rate.is_nan() {
						break;
					}
					if rate >= current_depth_best {
						current_depth_best = rate;
						current_depth_best_move = Some(action_considered.clone());
					}
				}
				current_rated_moves.push((action_considered.clone(), rate));
			}

			current_index += 1;
			if current_index >= playable_moves.len() /*|| current_depth_best >= MATE_SCORE*/ {
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

				current_depth_best = -MATE_SCORE - MAX_MATE_PENALTY;
				current_depth_best_move = None;
				rated_moves = current_rated_moves;
				current_rated_moves = Vec::new();
				if (best <= -MATE_SCORE || best >= MATE_SCORE) && action != None {
					break;
				}
			}
		}
		if current_depth_best_move != action {
			if current_depth_best > -MATE_SCORE {
				println!("re-score");
				action = current_depth_best_move;
				rated_moves = current_rated_moves;
				current_rated_moves = Vec::new();
			}
		}
		if let Some(action) = action {
			return (action, best, rated_moves);
		} else {
			if let Some(action) = current_depth_best_move {
				return (action, current_depth_best, current_rated_moves);
			}
			panic!("No playable move found");
		}
	}

	fn get_data_vec(state: &GameState) -> Vec<(String, f32)> {
		let mut result = MinimaxPlayer::get_colored_data(state, &PlayerColor::Red);
		result.extend(MinimaxPlayer::get_colored_data(state, &PlayerColor::Blue));
		return result;
	}

	fn add_to_result(
		result: &mut Vec<(String, f32)>,
		color: &PlayerColor,
		value: f32,
		description: &str,
	) {
		let string = format!("{}-{}", color, description).to_string();
		result.push((string, value));
	}

	fn get_colored_data(state: &GameState, color: &PlayerColor) -> Vec<(String, f32)> {
		let mut result = Vec::new();
		let swarm = Helper::greatest_swarm_new(state, color);
		let mut win: f32 = 0.;
		let mut len = 0;
		for (x, y) in state.get_own_fields(color) {
			win += 1. - (Helper::get_distance_to_swarm_new(x, y, &swarm) as f32 / 9.);
			len += 1;
		}
		MinimaxPlayer::add_to_result(&mut result, color, win, "DISTANCES");
		win = win / len as f32;
		MinimaxPlayer::add_to_result(&mut result, color, win, "ADJ_DISTANCES");
		MinimaxPlayer::add_to_result(&mut result, color, swarm.count_ones() as f32, "SWARM_COUNT");
		MinimaxPlayer::add_to_result(
			&mut result,
			color,
			swarm.count_ones() as f32 / 16.,
			"ADJ_SWARM_COUNT",
		);

		let fishes = state.board.get_fields_of(color);

		let mut center = fishes;
		center.mask(297799908644072875622400);
		let center_count = center.count_ones() as f32;
		MinimaxPlayer::add_to_result(&mut result, color, center_count, "CENTER_COUNT");
		MinimaxPlayer::add_to_result(
			&mut result,
			color,
			center_count / len as f32,
			"ADJ_CENTER_COUNT",
		);

		let mut border = fishes;
		border.mask(1267033445369934637136782821375);
		let border_count = border.count_ones() as f32;
		MinimaxPlayer::add_to_result(&mut result, color, border_count, "BORDER_COUNT");
		MinimaxPlayer::add_to_result(
			&mut result,
			color,
			border_count / len as f32,
			"ADJ_BORDER_COUNT",
		);

		MinimaxPlayer::add_to_result(&mut result, color, len as f32, "COUNT");
		MinimaxPlayer::add_to_result(&mut result, color, len as f32 / 16., "ADJ_COUNT");
		return result;
	}
}

impl ClientListener for MinimaxPlayer {
	fn on_move_request(&mut self, state: &GameState) -> Move {
		let id = self.id;
		return self.move_with_id(state, id);
	}
}
