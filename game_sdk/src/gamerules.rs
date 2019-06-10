use board::Board;
use gamestate::GameState;
use rand::{thread_rng, Rng};
use states::FieldType;
use states::Move;
use states::PlayerColor;

pub fn is_move_legal(state: &GameState, action: &Move, color: PlayerColor) -> bool {
    let x = action.x;
    let y = action.y;
    if x > 9 || y > 9 {
        // board is 0-9 for x and y
        return false;
    }

    if !state.board.is_field(x, y, color.to_fieldtype()) {
        // unowned fields are not to be moved
        return false;
    }

    // check availability of goal field
    if action.dest_x > 9 || action.dest_y > 9 {
        return false;
    }
    if state
        .board
        .is_field(action.dest_x, action.dest_y, FieldType::Obstacle)
        || state
            .board
            .is_field(action.dest_x, action.dest_y, color.to_fieldtype())
    {
        return false;
    }
    if i8::abs(x as i8 - action.dest_x as i8) > 1 || i8::abs(y as i8 - action.dest_y as i8) > 1 {
        if state.board.is_field_between(
            x,
            y,
            action.dest_x,
            action.dest_y,
            color.get_opponent_color().to_fieldtype(),
        ) {
            // cannot jump over enemy fields
            return false;
        }
    }
    return true;
}

pub fn get_winner(state: &GameState) -> Option<PlayerColor> {
    if state.turn % 2 == 1 && state.turn < 60 {
        return None;
    }

    if state.is_connected(&PlayerColor::Red) {
        let red_size = state.greatest_swarm_size(&PlayerColor::Red);
        let blue_size = state.greatest_swarm_size(&PlayerColor::Blue);
        if blue_size
            == state
                .board
                .get_fields_of(&PlayerColor::Blue)
                .count_ones() as u8
        {
            // blue is connected, red too
            if red_size > blue_size {
                return Some(PlayerColor::Red);
            }
            if red_size == blue_size {
                return None;
            }
            return Some(PlayerColor::Blue);
        }
        return Some(PlayerColor::Red);
    }
    if state.is_connected(&PlayerColor::Blue) {
        return Some(PlayerColor::Blue);
    }
    if state.turn == 60 {
        let red_size = state.greatest_swarm_size(&PlayerColor::Red);
        let blue_size = state.greatest_swarm_size(&PlayerColor::Blue);
        if red_size > blue_size {
            return Some(PlayerColor::Red);
        }
        if red_size < blue_size {
            return Some(PlayerColor::Blue);
        }
    }

    return None;
}

pub fn is_finished(state: &GameState) -> bool {
    if state.turn % 2 == 1 {
        return false;
    }
    if state.turn >= 60 {
        return true;
    }
    if state.is_connected(&PlayerColor::Red) || state.is_connected(&PlayerColor::Blue) {
        return true;
    }
    return false;
}

struct Point(i8, i8);

pub fn get_random_state() -> GameState {
    let mut fields = [[FieldType::Free; 10]; 10];
    for i in 1..9 {
        fields[0][i] = FieldType::RedPlayer;
        fields[9][i] = FieldType::RedPlayer;
        fields[i][0] = FieldType::BluePlayer;
        fields[i][9] = FieldType::BluePlayer;
    }
    let mut blockable_fields = Vec::new();
    for i in 2..8 {
        for j in 2..8 {
            blockable_fields.push(Point(i, j));
        }
    }
    let mut rng = thread_rng();
    let obs = blockable_fields
        .get(rng.gen_range(0, blockable_fields.len()) as usize)
        .unwrap();
    fields[obs.0 as usize][obs.1 as usize] = FieldType::Obstacle;
    let blockable_fields: Vec<&Point> = blockable_fields
        .iter()
        .filter(|f| {
            !(f.0 == obs.0
                || f.1 == obs.1
                || f.0 + f.1 == obs.0 + obs.1
                || f.0 - f.1 == obs.0 - obs.1)
        })
        .collect();
    let obs = blockable_fields
        .get(rng.gen_range(0, blockable_fields.len()) as usize)
        .unwrap();
    fields[obs.0 as usize][obs.1 as usize] = FieldType::Obstacle;
    let board = Board::new(fields);
    return GameState::new(board, 0);
}
