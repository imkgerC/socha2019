extern crate argparse;
extern crate game_sdk;
extern crate logic_player;
extern crate time;

use game_sdk::gamerules;
use game_sdk::Board;
use game_sdk::FieldType;
use game_sdk::GameState;

use logic_player::MinimaxPlayer;
// use logic_player::Searcher;

use argparse::{ArgumentParser, Store, StoreTrue};

fn perft(depth: u8, state: &GameState) -> u64 {
    if depth == 0 {
        return 1;
    }
    if gamerules::is_finished(state) {
        return 1;
    }
    let mut nodes = 0;
    let color = state.get_current_player_color();
    for action in state.get_move_list() {
        let mut state = state.clone();
        state.perform(&action, &color);
        let local_nodes = perft(depth - 1, &state);
        nodes += local_nodes;
    }
    return nodes;
}

fn from_condensed(condensed: [[&str; 10]; 10], turn: u8) -> GameState {
    let mut fields: [[FieldType; 10]; 10] = [[FieldType::Free; 10]; 10];
    for ix in 0..10 {
        for iy in 0..10 {
            fields[iy][ix] = match condensed[ix][iy] {
                "B" => FieldType::BluePlayer,
                "R" => FieldType::RedPlayer,
                "O" => FieldType::Obstacle,
                "E" => FieldType::Free,
                " " => FieldType::Free,
                _ => panic!("Condensed representation has wrong entry, must be one of B, R, O, E"),
            };
        }
    }
    return GameState::new(Board::new(fields), turn);
}

fn main() {
    /*let condensed = [
        ["E", "R", "E", "E", "E", "E", "E", "E", "E", "E"],
        ["E", "E", "E", "E", "E", "B", "E", "E", "E", "E"],
        ["E", "E", "R", "B", "E", "E", "E", "E", "E", "E"],
        ["E", "E", "E", "E", "R", "O", "E", "R", "E", "E"],
        ["E", "E", "R", "B", "E", "B", "B", "R", "E", "E"],
        ["E", "E", "E", "E", "B", "E", "R", "R", "E", "E"],
        ["E", "E", "E", "E", "B", "E", "B", "R", "E", "R"],
        ["E", "E", "E", "O", "E", "E", "E", "E", "E", "E"],
        ["E", "E", "E", "E", "B", "E", "E", "E", "E", "E"],
        ["E", "E", "E", "E", "E", "B", "B", "E", "E", "E"],
    ];*/
    // checked
    /*let condensed = [
        [" ", " ", "R", "B", "B", "B", "B", " ", " ", " "],
        [" ", "B", " ", " ", "B", "B", "B", " ", " ", " "],
        [" ", "R", " ", " ", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", " ", "R", " ", " ", " ", " ", "R"],
        ["R", " ", "R", "R", "R", "O", " ", " ", " ", "R"],
        ["R", " ", " ", " ", " ", " ", " ", " ", " ", "R"],
        ["R", " ", " ", " ", " ", " ", "O", " ", " ", "R"],
        [" ", " ", " ", " ", " ", " ", " ", " ", " ", "R"],
        [" ", " ", " ", " ", " ", " ", " ", " ", " ", "R"],
        [" ", " ", " ", " ", " ", " ", " ", " ", " ", " "],
    ];*/
    /*let condensed = [
        [" ", "B", "B", "B", "B", "B", "B", "B", "B", "E"], // perft4: 5 131 516
        ["R", " ", " ", " ", " ", " ", " ", " ", " ", "R"], // perft5: 250 613 480
        ["R", " ", " ", " ", " ", " ", " ", " ", " ", "R"],
        ["R", " ", " ", " ", " ", "O", " ", " ", " ", "R"],
        ["R", " ", " ", " ", " ", " ", " ", " ", " ", "R"],
        ["R", " ", " ", " ", " ", " ", " ", " ", " ", "R"],
        ["R", " ", " ", " ", "O", " ", " ", " ", " ", "R"],
        ["R", " ", " ", " ", " ", " ", " ", " ", " ", "R"],
        ["R", " ", " ", " ", " ", " ", " ", " ", " ", "R"],
        [" ", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
    ];*/
    let condensed = [
        [" ", " ", " ", " ", "B", "B", "B", "B", " ", " "],
        ["R", " ", " ", " ", "R", " ", "R", " ", " ", " "],
        [" ", " ", " ", " ", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", " ", "R", "B", " ", " ", " "],
        ["R", " ", "O", " ", " ", " ", " ", " ", " ", " "],
        ["R", " ", " ", " ", " ", " ", "R", " ", " ", " "],
        ["R", " ", " ", " ", "R", " ", "O", " ", " ", "R"],
        [" ", " ", " ", " ", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", " ", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", " ", " ", " ", " ", " ", " ", " "],
    ];
    let state = from_condensed(condensed, 0);
    let mut depth = 0;
    let mut test = false;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut depth)
            .add_option(&["-n", "--depth"], Store, "depth to search in");
        ap.refer(&mut test)
            .add_option(&["-a", "--minimax"], StoreTrue, "try it w/ a/b-engine");
        ap.parse_args_or_exit();
    }

    if !test {
        let before = time::now();
        let nodes = perft(depth, &state);
        println!("{}", nodes);
        let needed = (time::now() - before).num_milliseconds();
        println!("Needed {}ms", needed);
        let rate = (nodes as f32 / needed as f32) * 1000. / 1_000_000.;
        println!("Avg speed: {:.2}MN/s", rate);
    } else {
        /*let mut searcher = Searcher::new(0);
        searcher.main_thread_go(&state);*/
        let (action, rate, _moves) = MinimaxPlayer::get_move_and_rate(&state, depth);
        println!("{} {}", action, rate);
        /*for action in moves {
            println!("{:?}", action);
        }
        for action in &searcher.root_moves {
            println!("{:?}", action);
        }*/
    }
}
