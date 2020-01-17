use super::mcts::{MCTS, RaveParameters};
use super::piranhas::Piranhas;
use game_sdk::gamerules;
use game_sdk::ClientListener;
use game_sdk::{GameState, Move};

use time;

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
            self.mcts = Some(MCTS::new(&game, self.params.clone()));
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
