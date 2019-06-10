use game_sdk::{gamerules, ClientListener, PlayerColor};
use logic_player::{ClopParameters, EnemyPool, ToClop};

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0); // remove program name
    args.remove(0); // remove processor id
    let seed: i64 = args.remove(0).parse().expect("Did not get valid seed");
    let mut key = None;
    let mut params = ClopParameters::empty();
    for arg in args {
        if let Some(id) = key {
            params.set_var_from_string(id, arg);
            key = None;
        } else {
            key = Some(arg);
        }
    }

    let mut state = gamerules::get_random_state();
    let mut own = ToClop::new(params);
    let mut other = EnemyPool::get_new(seed);

    loop {
        if state.turn % 2 == 0 {
            if gamerules::is_finished(&state) {
                break;
            }
        }
        let color = state.get_current_player_color();
        if state.get_move_list().len() == 0 {
            match color {
                PlayerColor::Red => match seed % 2 {
                    0 => {
                        println!("L");
                        return;
                    }
                    _ => {
                        println!("W");
                        return;
                    }
                },
                PlayerColor::Blue => match seed % 2 {
                    1 => {
                        println!("L");
                        return;
                    }
                    _ => {
                        println!("W");
                        return;
                    }
                },
            };
        }

        let action = match color {
            PlayerColor::Red => match seed % 2 {
                0 => own.on_move_request(&state),
                _ => other.on_move_request(&state),
            },
            PlayerColor::Blue => match seed % 2 {
                1 => own.on_move_request(&state),
                _ => other.on_move_request(&state),
            },
        };
        state.perform(&action, &color);
    }

    if let Some(color) = gamerules::get_winner(&state) {
        match color {
            PlayerColor::Red => match seed % 2 {
                0 => println!("W"),
                _ => println!("L"),
            },
            PlayerColor::Blue => match seed % 2 {
                1 => println!("W"),
                _ => println!("L"),
            },
        };
    } else {
        // Draw
        println!("D");
        return;
    }
}
