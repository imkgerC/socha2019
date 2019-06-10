use super::mcts::MCTS;
use super::piranhas::Piranhas;
use game_sdk::gamerules;
use game_sdk::ClientListener;
use game_sdk::{GameState, Move};

use time;

#[derive(Clone)]
pub struct RaveParameters {
    pub c: f32,
    pub c_base: usize,
}

impl RaveParameters {
    pub fn empty() -> RaveParameters {
        return RaveParameters { c: 0.0, c_base: 0 };
    }

    pub fn set_var_from_string(&mut self, identifier: String, val: String) {
        match &identifier[..] {
            "c" => {
                self.c = val.parse().expect("Got wrong val");
            }
            "c_base" => {
                self.c_base = val.parse().expect("Got wrong val");
            }
            _ => panic!("wrong identifier"),
        };
    }
}

#[derive(Clone)]
pub struct RavePlayer {
    mcts: Option<MCTS>,
    params: RaveParameters,
}

impl RavePlayer {
    pub fn new(params: RaveParameters) -> RavePlayer {
        return RavePlayer { mcts: None, params };
    }
}

impl ClientListener for RavePlayer {
    fn on_move_request(&mut self, state: &GameState) -> Move {
        let before = time::now();
        let game = Piranhas::from_state(state);
        if let Some(ref mut mcts) = self.mcts {
            mcts.set_root(&game);
        // *mcts = MCTS::new(&game); // to deactivate taking knowledge over from last turn
        } else {
            self.mcts = Some(MCTS::new(&game, self.params.c, self.params.c_base));
        }
        if let Some(ref mut mcts) = self.mcts {
            // mcts.search(1_000, c);
            let budget_seconds = 1.7 - ((time::now() - before).num_milliseconds() as f32 / 1000.);
            mcts.search_time(budget_seconds);

            if let (Some(action), _, _) = mcts.best_action() {
                return action;
            } else {
                println!("{:?}", mcts.tree_statistics());
                println!("{}", game.allowed_actions().len());
                println!("{:?}", gamerules::get_winner(state));
                println!("{}", gamerules::is_finished(state));
                println!("{}", game.state.get_move_list().len());
                println!("{:?}", mcts.get_pairs());
                panic!("Did not find any move or something like that");
            }
        } else {
            panic!("Did not find mcts, should never happen");
        }
    }
}
