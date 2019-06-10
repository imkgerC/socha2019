use game_sdk::logging::{Data};
use game_sdk::{ClientListener, GameState, Move};

use crate::MinimaxPlayer;
use crate::RavePlayer;
use crate::LegacyRavePlayer;
use crate::LegacyMinimaxPlayer;

use std::sync::mpsc;

#[derive(Clone)]
pub struct HybridPlayer {
	id: i64,
	tx: Option<mpsc::Sender<Data>>,
	mcts: RavePlayer,
	ab: MinimaxPlayer,
}
impl HybridPlayer {
	pub fn new(tx: Option<mpsc::Sender<Data>>, id: i64) -> HybridPlayer {
		return HybridPlayer {
			id,
			tx: tx.clone(),
			mcts: RavePlayer::new(tx.clone(), id),
			ab: MinimaxPlayer::new(tx, id),
		};
	}
}

impl HybridPlayer {
	pub fn move_with_id(&mut self, state: &GameState, id: i64) -> Move {
		if state.turn > 19 {
			return self.ab.move_with_id(state, id);
		}
		return self.mcts.move_with_id(state, id);
	}
}

impl ClientListener for HybridPlayer {
	fn on_move_request(&mut self, state: &GameState) -> Move {
		let id = self.id;
		return self.move_with_id(state, id);
	}
}

#[derive(Clone)]
pub struct LegacyHybridPlayer {
	id: i64,
	tx: Option<mpsc::Sender<Data>>,
	mcts: LegacyRavePlayer,
	ab: LegacyMinimaxPlayer,
}
impl LegacyHybridPlayer {
	pub fn new(tx: Option<mpsc::Sender<Data>>, id: i64) -> LegacyHybridPlayer {
		return LegacyHybridPlayer {
			id,
			tx: tx.clone(),
			mcts: LegacyRavePlayer::new(tx.clone(), id),
			ab: LegacyMinimaxPlayer::new(tx, id),
		};
	}
}

impl LegacyHybridPlayer {
	pub fn move_with_id(&mut self, state: &GameState, id: i64) -> Move {
		if state.turn > 19 {
			return self.ab.move_with_id(state, id);
		}
		return self.mcts.move_with_id(state, id);
	}
}

impl ClientListener for LegacyHybridPlayer {
	fn on_move_request(&mut self, state: &GameState) -> Move {
		let id = self.id;
		return self.move_with_id(state, id);
	}
}
