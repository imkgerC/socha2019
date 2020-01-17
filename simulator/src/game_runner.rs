// random vs random: 529 380 / 122 399 it/s (multi/single) (avg over 1M iterations)
// those speeds won't ever be reproducible, as they were patched out
use game_sdk::{gamerules, ClientListener, GameState, PlayerColor};

use game_sdk::logging::{Data, EndState, Winner};
use rand::{thread_rng, Rng};
use xml_utils;

use logic_player::RavePlayer as PlayerOne;

use logic_player::LegacyRavePlayer as PlayerTwo;

use std::fs;
use std::sync::mpsc;

pub fn run_game_wo_sending(index: u32) {
    let mut player_one = PlayerOne::new(None, index as i64);
    let mut player_two = PlayerTwo::new(None, index as i64);
    let mut state = gamerules::get_random_state();
    loop {
        if state.turn % 2 == 0 {
            if gamerules::is_finished(&state) {
                break;
            }
        }

        let color = state.get_current_player_color();
        let action = match color {
            PlayerColor::Red => match index % 2 {
                0 => player_one.on_move_request(&state),
                _ => player_two.on_move_request(&state),
            },
            PlayerColor::Blue => match index % 2 {
                1 => player_one.on_move_request(&state),
                _ => player_two.on_move_request(&state),
            },
        };
        state.perform(&action, &color);
    }
}

pub fn run_single_game(
    index: u32,
    t_winner: mpsc::Sender<(Winner, Option<PlayerColor>)>,
    xml_enabled: bool,
) {
    let mut player_one = PlayerOne::new(None, index as i64);
    let mut player_two = PlayerTwo::new(None, index as i64);
    let mut string_xml = "<protocol>\n".to_string();
    let mut state = gamerules::get_random_state();
    loop {
        if xml_enabled {
            string_xml += xml_utils::get_xml_turn(&state).as_str();
        }
        if state.turn % 2 == 0 {
            if gamerules::is_finished(&state) {
                break;
            }
        }

        let color = state.get_current_player_color();
        let action = match color {
            PlayerColor::Red => match index % 2 {
                0 => player_one.on_move_request(&state),
                _ => player_two.on_move_request(&state),
            },
            PlayerColor::Blue => match index % 2 {
                1 => player_one.on_move_request(&state),
                _ => player_two.on_move_request(&state),
            },
        };
        state.perform(&action, &color);
    }
    if xml_enabled {
        string_xml += xml_utils::get_xml_result(&state).as_str();
        string_xml += "</protocol>";
        fs::write(format!("replays/game_{}.xml", index), string_xml).expect("unable to write file");
    }
    send_winner(t_winner, &state, index);
}

pub fn run_with_player<T, D>(
    index: u32,
    t_winner: mpsc::Sender<EndState>,
    mut player_one: D,
    mut player_two: T,
    xml_enabled: bool,
) where
    T: ClientListener,
    D: ClientListener,
{
    let mut string_xml = "<protocol>\n".to_string();
    let mut state = gamerules::get_random_state();
    let red_blue = thread_rng().gen::<u8>();
    loop {
        if xml_enabled {
            string_xml += xml_utils::get_xml_turn(&state).as_str();
        }
        if state.turn % 2 == 0 {
            if gamerules::is_finished(&state) {
                break;
            }
        }

        let color = state.get_current_player_color();
        let action = match color {
            PlayerColor::Red => match red_blue % 2 {
                0 => player_one.on_move_request(&state),
                _ => player_two.on_move_request(&state),
            },
            PlayerColor::Blue => match red_blue % 2 {
                1 => player_one.on_move_request(&state),
                _ => player_two.on_move_request(&state),
            },
        };
        state.perform(&action, &color);
    }
    if xml_enabled {
        string_xml += xml_utils::get_xml_result(&state).as_str();
        string_xml += "</protocol>";
        fs::write(format!("replays/game_{}.xml", index), string_xml).expect("unable to write file");
    }
    t_winner
        .send(get_end_state(&state, index, red_blue))
        .unwrap();
}

fn get_end_state(state: &GameState, id: u32, red_blue: u8) -> EndState {
    let winner;
    let mut color = None;
    if let Some(c) = gamerules::get_winner(state) {
        winner = Winner::get_winner(&c, red_blue as u32);
        color = Some(c);
    } else {
        winner = Winner::Draw;
    }
    return EndState { id, winner, color };
}

pub fn collect_selfplay(
    index: u32,
    t_log: mpsc::Sender<Data>,
    t_winner: mpsc::Sender<(Winner, Option<PlayerColor>)>,
) {
    #[cfg(windows)]
    let mut player = logic_player::RavePlayer::new(Some(t_log.clone()), index as i64);
    #[cfg(unix)]
    let mut player = logic_player::RavePlayer::new(Some(t_log.clone()), index as i64);

    let mut state = gamerules::get_random_state();

    loop {
        if state.turn % 2 == 0 {
            if gamerules::is_finished(&state) {
                break;
            }
        }

        let action = player.on_move_request(&state);
        let color = state.get_current_player_color();
        state.perform(&action, &color);
    }
    t_log
        .send(Data::End(EndState::get_end(&state, index)))
        .unwrap();
    send_winner(t_winner, &state, index);
}

pub fn collect_data(
    index: u32,
    t_log: mpsc::Sender<Data>,
    t_winner: mpsc::Sender<(Winner, Option<PlayerColor>)>,
) {
    #[cfg(windows)]
    let mut player = logic_player::MinimaxPlayer::new(Some(t_log.clone()), index as i64);
    #[cfg(unix)]
    let mut player = logic_player::MinimaxPlayer::new(Some(t_log.clone()), index as i64);
    let mut random_player = logic_player::LogicBasedPlayer::new(None, 0);
    let mut state = gamerules::get_random_state();
    loop {
        if state.turn % 2 == 0 {
            if gamerules::is_finished(&state) {
                break;
            }
        }

        let id = index * 60 + state.turn as u32;
        {
            let mut state = state.clone();
            loop {
                if gamerules::is_finished(&state) {
                    break;
                }
                // let action = inner_player.on_move_request(&state);
                let action = player.move_with_id(&state, id as i64);
                let color = state.get_current_player_color();
                state.perform(&action, &color);
            }
            t_log
                .send(Data::End(EndState::get_end(&state, id)))
                .unwrap();
        }
        let action = random_player.on_move_request(&state);
        let color = state.get_current_player_color();
        state.perform(&action, &color);
    }

    send_winner(t_winner, &state, index);
}

fn send_winner(
    t_winner: mpsc::Sender<(Winner, Option<PlayerColor>)>,
    state: &GameState,
    index: u32,
) {
    if let Some(c) = gamerules::get_winner(state) {
        t_winner
            .send((Winner::get_winner(&c, index), Some(c)))
            .unwrap();
        return;
    }
    t_winner.send((Winner::Draw, None)).unwrap();
}
