use std::fmt;

use game_sdk::gamerules;
use game_sdk::GameState;
use game_sdk::Move;
use game_sdk::PlayerColor;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct MinimalState {
    pub red_fields: u128,
    pub blue_fields: u128,
    pub turn: u8,
}

impl MinimalState {
    pub fn from_state(state: &GameState) -> MinimalState {
        return MinimalState {
            red_fields: state.board.red_fields.bits,
            blue_fields: state.board.blue_fields.bits,
            turn: state.turn,
        };
    }

    pub fn empty() -> MinimalState {
        return MinimalState {
            red_fields: 0,
            blue_fields: 0,
            turn: 255,
        };
    }
}

#[derive(Clone, Debug)]
pub struct Piranhas {
    pub state: GameState,
    reward: Option<Option<f32>>,
    initial_color: Option<PlayerColor>,
}

impl Piranhas {
    // Create a new game with two random two's in it.
    pub fn from_state(state_in: &GameState) -> Piranhas {
        let game = Piranhas {
            state: state_in.clone(),
            reward: None,
            initial_color: None,
        };
        game
    }

    fn get_points_for(state: &GameState, color: &PlayerColor) -> Option<f32> {
        if state.turn % 2 == 1 {
            return None;
        }
        let own_size = state.greatest_swarm_size(color);
        let own_len = state
            .board
            .get_fields_of(color)
            .count_ones() as u8;
        let other_size = state.greatest_swarm_size(&color.get_opponent_color());
        let other_len = state
            .board
            .get_fields_of(&color.get_opponent_color())
            .count_ones() as u8;

        let rate = (own_size as f32 - other_size as f32) / 32.;
        if own_size == own_len {
            if other_size == other_len {
                // other is connected, we too
                if own_size > other_size {
                    return Some(1.0 + rate);
                }
                if own_size == other_size {
                    return Some(0.5);
                }
                return Some(0.0 + rate);
            }
            // only we are connected
            return Some(1.0 + rate);
        }
        if other_size == other_len {
            return Some(0.0 + rate);
        }
        if state.turn == 60 {
            if own_size > other_size {
                return Some(1.0 + rate);
            }
            if own_size == other_size {
                return Some(0.5);
            }
            return Some(0.0 + rate);
        }

        return None;
    }

    /// Return a list with all allowed actions given the current game state.
    pub fn allowed_actions(&self) -> Vec<Move> {
        match gamerules::is_finished(&self.state) {
            true => return Vec::new(),
            false => return self.state.get_move_list(),
        };
    }
    
    pub fn is_finished(&self) -> bool {
        return gamerules::is_finished(&self.state);
    }

    pub fn get_color(&self) -> PlayerColor {
        return self.state.get_current_player_color();
    }

    /// Change the current game state according to the given action.
    pub fn make_move(&mut self, action: &Move) {
        let color = self.state.get_current_player_color();
        self.state.perform(action, &color);
    }

    /// Reward for the player when reaching the current game state.
    pub fn reward(&mut self, color: &PlayerColor) -> f32 {
        match self.reward {
            Some(_) => {}
            None => {
                self.reward = Some(Piranhas::get_points_for(&self.state, color));
            }
        };
        if let Some(Some(points)) = self.reward {
            if let Some(c) = self.initial_color {
                if c == *color {
                    return points;
                }
            }
            return 1.0 - points;
        } else {
            return 0.0;
        }
    }
}

impl fmt::Display for Piranhas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}\n{:?}",
            self.state.get_current_player_color(),
            self.state.turn,
            self.state.board
        )
    }
}
