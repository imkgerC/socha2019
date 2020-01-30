use super::mcts::MCTS;
use super::piranhas::Piranhas;
use game_sdk::gamerules;
use game_sdk::logging::{Data, MoveValuePair, State};
use game_sdk::ClientListener;
use game_sdk::{GameState, Move};
use std::sync::mpsc;

use time;

#[derive(Clone)]
pub struct RavePlayer {
    tx: Option<mpsc::Sender<Data>>,
    id: i64,
    mcts: Option<MCTS>,
}

impl RavePlayer {
    pub fn new(tx: Option<mpsc::Sender<Data>>, id: i64) -> RavePlayer {
        return RavePlayer { tx, id, mcts: None };
    }

    pub fn move_with_id(&mut self, state: &GameState, id: i64) -> Move {
        let before = time::now();
        let game = Piranhas::from_state(state);
        if let Some(ref mut mcts) = self.mcts {
            mcts.set_root(&game);
        // *mcts = MCTS::new(&game); // to deactivate taking knowledge over from last turn
        } else {
            self.mcts = Some(MCTS::new(&game));
        }
        if let Some(ref mut mcts) = self.mcts {
            let before_samples = mcts.get_root_samples();
            let c = 0.038;
            // mcts.search(1000, c);
            let budget_seconds = 0.1 - ((time::now() - before).num_milliseconds() as f32 / 1000.);
            mcts.search_time(budget_seconds, c);
            if let (Some(action), value, depth) = mcts.best_action() {
                if let Some(ref tx) = self.tx {
                    let moves = mcts    
                        .get_pairs()
                        .iter()
                        .map(|x| MoveValuePair {
                            action: x.1,
                            value: x.0,
                        })
                        .collect();
                    let send_state = State {
                        id: id as u32,
                        gamestate: state.clone(),
                        moves,
                        data: Vec::new(),
                    };
                    tx.send(Data::Step(send_state)).unwrap();
                } else if id < 0 {
                    let stats = mcts.tree_statistics();
                    if let Some(depth) = depth {
                        print!("end in {}; ", depth);
                    }
                    println!(
                        "|{}| reused {} samples; {}ms | {} nodes | {}-{} depth | val {:.3} | {} table|{:.0}it/s)",
                        state.turn,
                        before_samples,
                        (time::now() - before).num_milliseconds(),
                        stats.nodes,
                        stats.min_depth,
                        stats.max_depth,
                        value,
                        mcts.table_size(),
                        mcts.iterations_per_s,
                    );
                    // println!("{}", action);
                }
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

impl ClientListener for RavePlayer {
    fn on_move_request(&mut self, state: &GameState) -> Move {
        let id = self.id;
        return self.move_with_id(state, id);
    }
}
