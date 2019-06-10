use board::Board;

use bitboard::Bitboard;
use iterators;
use states::Direction;
use states::Field;
#[cfg(test)]
use states::FieldType;
use states::Move;
use states::PlayerColor;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct GameState {
    pub turn: u8,
    pub board: Board,
}

impl GameState {
    pub fn new(board: Board, turn: u8) -> GameState {
        return GameState { board, turn };
    }

    pub fn get_move_list(&self) -> Vec<Move> {
        return iterators::get_possible_moves(&self);
    }

    pub fn get_attack_board(&self, color: &PlayerColor) -> u128 {
        return iterators::get_attack_board(&self, color);
    }

    pub fn get_captures(&self) -> Vec<Move> {
        return iterators::get_captures(&self);
    }

    pub fn get_destination(&self, action: &Move) -> Option<Field> {
        return self.board.get_field(action.dest_x, action.dest_y);
    }

    pub fn get_destination_coordinates(&self, x: u8, y: u8, direction: Direction) -> (i8, i8) {
        let distance = self.board.get_distance(x, y, direction);
        let multipliers = direction.get_multipliers();
        let x_n = x as i8 + distance * multipliers.0;
        let y_n = y as i8 + distance * multipliers.1;
        return (x_n, y_n);
    }

    pub fn perform(&mut self, action: &Move, color: &PlayerColor) {
        self.turn = self.turn + 1;
        let pre_bit = 0b1 << (action.x + action.y * 10);
        self.board.red_fields.bits &= !pre_bit;
        self.board.blue_fields.bits &= !pre_bit;

        let dest_bit = 0b1 << (action.dest_x + action.dest_y * 10);
        match color {
            PlayerColor::Red => {
                self.board.red_fields.bits |= dest_bit;
                self.board.blue_fields.bits &= !dest_bit;
            }
            PlayerColor::Blue => {
                self.board.blue_fields.bits |= dest_bit;
                self.board.red_fields.bits &= !dest_bit;
            }
        }
    }

    pub fn is_connected(&self, color: &PlayerColor) -> bool {
        let mut fields = self.board.get_fields_of(color);
        let (x, y) = fields.get_first();
        let _ = self.get_swarm_size(&mut fields, x, y);
        if fields.count_ones() > 0 {
            return false;
        }
        return true;
    }

    pub fn greatest_swarm_size(&self, color: &PlayerColor) -> u8 {
        let mut current_max: u8 = 0;
        let mut fields = self.board.get_fields_of(color);
        while current_max < fields.count_ones() as u8 {
            let (x, y) = fields.get_first();
            let size = self.get_swarm_size(&mut fields, x, y);
            if size > current_max {
                current_max = size;
            }
        }
        return current_max;
    }

    fn get_swarm_size(&self, fields: &mut Bitboard, x: u8, y: u8) -> u8 {
        let mut swarm = Bitboard::new();
        swarm.set_field(x, y);
        let mut current_size = 1;
        let mut before_size = 0;
        while current_size > before_size {
            before_size = current_size;
            GameState::extend(&mut swarm, fields);
            current_size = swarm.count_ones() as u8;
        }
        fields.clear_bits(swarm.bits);
        return current_size;
    }

    fn extend(to_extend: &mut Bitboard, fields: &Bitboard) {
        let mut bits = to_extend.bits;
        let not_left_row = !bits & 1268889750375080065623288448001; 
        let not_right_row = !bits & 649671552192040993599123685376512;
        let shift_left = (bits << 1) & !not_left_row;
        let shift_right = (bits >> 1) & !not_right_row;
        bits = bits | shift_left | shift_right;
        let shift_up = bits << 10;
        let shift_down = bits >> 10;
        bits = bits | shift_down | shift_up;
        bits = bits & fields.bits;
        to_extend.bits = bits;
    }

    pub fn get_own_fields(&self, color: &PlayerColor) -> impl Iterator<Item = (u8, u8)> {
        return iterators::BitboardIter::new(self.board.get_fields_of(color).bits);
    }

    pub fn get_own_indices(&self, color: &PlayerColor) -> impl Iterator<Item = u8> {
        return iterators::BitboardIndexIter::new(self.board.get_fields_of(color).bits);
    }

    pub fn get_current_player_color(&self) -> PlayerColor {
        if self.turn % 2 == 0 {
            PlayerColor::Red
        } else {
            PlayerColor::Blue
        }
    }

    pub fn get_xml(&self) -> String {
        let mut string_version = "".to_string();
        string_version += format!(
            "<state class=\"sc.plugin2019.GameState\" startPlayerColor=\"RED\" currentPlayerColor=\"{}\" turn=\"{}\">\n", 
            self.get_current_player_color(), self.turn).as_str();
        string_version += format!("<red displayName=\"{}\" color=\"RED\"/>\n", "RED").as_str();
        string_version += format!("<blue displayName=\"{}\" color=\"BLUE\"/>\n", "BLUE").as_str();
        string_version += format!("{}", self.board.get_xml()).as_str();
        string_version += "</state>\n";

        return string_version;
    }

    #[cfg(test)]
    fn from_condensed_2(condensed: [[&str; 10]; 10]) -> GameState {
        let mut fields: [[FieldType; 10]; 10] = [[FieldType::Free; 10]; 10];
        for ix in 0..10 {
            for iy in 0..10 {
                fields[iy][ix] = match condensed[ix][iy] {
                    "B" => FieldType::BluePlayer,
                    "R" => FieldType::RedPlayer,
                    "O" => FieldType::Obstacle,
                    "E" => FieldType::Free,
                    _ => panic!(
                        "Condensed representation has wrong entry, must be one of B, R, O, E"
                    ),
                };
            }
        }
        return GameState::new(Board::new(fields), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Functions to test:
    // - perform
    // - get_destination
    // - get_possible_moves

    #[test]
    fn new() {
        let turn = 0;
        let condensed = [
            ["E", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "O", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "O", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["E", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
        ];
        let board = Board::from_condensed(condensed);
        let state = GameState::new(board.clone(), turn);
        assert_eq!(state.board, board);
        assert_eq!(state.turn, turn);
    }

    #[test]
    fn is_connected() {
        let turn = 0;
        let condensed = [
            ["E", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "O", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "O", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["E", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
        ];
        let board = Board::from_condensed(condensed);
        let state = GameState::new(board.clone(), turn);
        assert!(!state.is_connected(&PlayerColor::Red));
        assert!(!state.is_connected(&PlayerColor::Blue));

        let condensed = [
            ["E", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "O", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "O", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "B", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
        ];
        let board = Board::from_condensed(condensed);
        let state = GameState::new(board.clone(), turn);
        assert!(state.is_connected(&PlayerColor::Red));
        assert!(!state.is_connected(&PlayerColor::Blue));

        let condensed = [
            ["R", "R", "B", "B", "B", "B", "B", "B", "R", "R"],
            ["R", "R", "E", "E", "B", "E", "E", "E", "R", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "O", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "O", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "R", "E", "E", "E", "E", "E", "E", "R", "R"],
            ["R", "R", "E", "E", "E", "E", "E", "E", "R", "R"],
        ];
        let board = Board::from_condensed(condensed);
        let state = GameState::new(board.clone(), turn);
        assert!(!state.is_connected(&PlayerColor::Red));
        assert!(state.is_connected(&PlayerColor::Blue));

        let condensed = [
            ["E", "B", "E", "B", "B", "B", "B", "B", "B", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["B", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "O", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "O", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "R"],
            ["R", "B", "B", "B", "B", "B", "B", "B", "B", "R"],
        ];
        let state = GameState::from_condensed_2(condensed);
        assert!(!state.is_connected(&PlayerColor::Red));
        assert!(!state.is_connected(&PlayerColor::Blue));
    }

    #[test]
    fn greatest_swarm_size() {
        let turn = 0;
        let condensed = [
            ["E", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "O", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "O", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "B", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
        ];
        let board = Board::from_condensed(condensed);
        let state = GameState::new(board.clone(), turn);
        assert_eq!(state.greatest_swarm_size(&PlayerColor::Red), 8);
        assert_eq!(state.greatest_swarm_size(&PlayerColor::Blue), 8);

        let condensed = [
            ["E", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "R", "R", "E", "E"],
            ["R", "R", "R", "O", "E", "R", "E", "E", "E", "E"],
            ["R", "E", "E", "R", "R", "E", "E", "E", "E", "E"],
            ["R", "E", "O", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "B", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
        ];
        let board = Board::from_condensed(condensed);
        let state = GameState::new(board.clone(), turn);
        assert_eq!(state.greatest_swarm_size(&PlayerColor::Red), 15);
        assert_eq!(state.greatest_swarm_size(&PlayerColor::Blue), 8);
    }

    #[test]
    fn get_current_player_color() {
        let turn = 3;
        let condensed = [
            ["E", "B", "B", "B", "B", "B", "B", "B", "B", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "O", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "O", "E", "E", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "B", "E", "E", "E", "E", "E"],
            ["R", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
            ["E", "E", "E", "E", "E", "E", "E", "E", "E", "E"],
        ];
        let board = Board::from_condensed(condensed);
        let mut state = GameState::new(board.clone(), turn);
        assert_eq!(state.get_current_player_color(), PlayerColor::Blue);
        state.turn = turn + 1;
        assert_eq!(state.get_current_player_color(), PlayerColor::Red);
    }
}
