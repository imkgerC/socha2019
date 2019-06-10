use super::player::{MinimaxParameters, MinimaxPlayer};
use super::rave_player::{RaveParameters, RavePlayer};
use game_sdk::{ClientListener, GameState, Move};

const MINIMAX: MinimaxParameters = MinimaxParameters {
    aspiration_window: 3e-1,
    adj_distances_start: 0.183,
    adj_distances_end: 6.382,
    swarm_start: -1.292,
    swarm_end: 6.78,
    adj_center_start: 4.029,
    adj_center_end: 2.314,
    adj_border_start: -8.799,
    adj_border_end: -4.382,
    count_start: 6.681,
    count_end: 2.931,
    search_q: false,
};

const RAVE: RaveParameters = RaveParameters {
    c: 0.0,
    c_base: 19652,
};

pub enum Enemy {
    RaveEnemy(RavePlayer),
    MinimaxEnemy(MinimaxPlayer),
}

impl Enemy {
    pub fn on_move_request(&mut self, state: &GameState) -> Move {
        return match self {
            Enemy::RaveEnemy(ref mut p) => p.on_move_request(state),
            Enemy::MinimaxEnemy(ref mut p) => p.on_move_request(state),
        };
    }
}

pub struct EnemyPool;

impl EnemyPool {
    pub fn get_new(seed: i64) -> Enemy {
        if seed % 4 < 2 {
            return Enemy::MinimaxEnemy(MinimaxPlayer::new(MINIMAX));
        }
        return Enemy::RaveEnemy(RavePlayer::new(RAVE));
    }
}
