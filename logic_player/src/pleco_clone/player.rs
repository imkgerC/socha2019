use super::search::Searcher;
use game_sdk::logging::Data;
use game_sdk::{ClientListener, GameState, Move};
use std::sync::mpsc;

#[derive(Clone)]
pub struct PlecoPlayer {
    searcher: Searcher,
}

impl PlecoPlayer {
    pub fn new(_tx: Option<mpsc::Sender<Data>>, id: i64) -> PlecoPlayer {
        let searcher;
        if id >= 0 {
            searcher = Searcher::new(id as usize + 1);
        } else {
            searcher = Searcher::new(0);
        }
        return PlecoPlayer { searcher };
    }

    pub fn move_with_id(&mut self, state: &GameState, id: i64) -> Move {
        let moves = state.get_move_list();
        self.searcher.id = id as usize;
        self.searcher.main_thread_go(state);
        let bm = self.searcher.best_move.to_partial_move();
        if let Some(i) = moves.iter().position(|&m| {
            m.x == bm.x && m.y == bm.y && m.dest_x == bm.dest_x && m.dest_y == bm.dest_y
        }) {
            return *moves.get(i).expect("Did not find move just found?");
        }
        println!("Did not find best move in possible moves?");
        return *moves.get(0).expect("Did not find any move in state");
    }
}

impl ClientListener for PlecoPlayer {
    fn on_move_request(&mut self, state: &GameState) -> Move {
        let moves = state.get_move_list();
        self.searcher.main_thread_go(state);
        let bm = self.searcher.best_move.to_partial_move();
        if let Some(i) = moves.iter().position(|&m| {
            m.x == bm.x && m.y == bm.y && m.dest_x == bm.dest_x && m.dest_y == bm.dest_y
        }) {
            return *moves.get(i).expect("Did not find move just found?");
        }
        println!("Did not find best move in possible moves?");
        return *moves.get(0).expect("Did not find any move in state");
    }
}
