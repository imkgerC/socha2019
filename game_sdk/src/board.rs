use bitboard::constants::*;
use bitboard::Bitboard;
use states::Direction;
use states::Field;
use states::FieldType;
use states::PlayerColor;
use std;

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    pub red_fields: Bitboard, // bitboards
    pub blue_fields: Bitboard,
    pub obstacle_fields: Bitboard,
}

impl Board {
    pub fn new(fields: [[FieldType; 10]; 10]) -> Board {
        let mut red_fields: Bitboard = Bitboard::new();
        let mut blue_fields: Bitboard = Bitboard::new();
        let mut obstacle_fields: Bitboard = Bitboard::new();
        for (row_index, row) in fields.iter().enumerate() {
            for (line_index, field) in row.iter().enumerate() {
                // building bitfields
                match field {
                    FieldType::RedPlayer => {
                        red_fields.set_field(row_index as u8, line_index as u8);
                    }
                    FieldType::BluePlayer => {
                        blue_fields.set_field(row_index as u8, line_index as u8);
                    }
                    FieldType::Obstacle => {
                        obstacle_fields.set_field(row_index as u8, line_index as u8);
                    }
                    _ => {}
                };
            }
        }
        return Board {
            red_fields,
            blue_fields,
            obstacle_fields,
        };
    }

    pub fn is_field_between(
        &self,
        mut x: u8,
        mut y: u8,
        mut x_n: u8,
        mut y_n: u8,
        type_searched: FieldType,
    ) -> bool {
        if y == y_n {
            // horizontal case
            if x > x_n {
                let b = x;
                x = x_n;
                x_n = b;
            }
            let fields = ((0b1 << (x_n - x - 1)) - 1) << (x + y * 10 + 1);
            return self.is_type_in(fields, type_searched);
        } else if x == x_n {
            // vertical case
            if y > y_n {
                let b = y;
                y = y_n;
                y_n = b;
            }
            let fields = VERTICAL_BITMASKS[(y_n - y - 1) as usize] << (x + y * 10);
            return self.is_type_in(fields, type_searched);
        }
        // diagonal case
        if y > y_n {
            // flipping line vertically
            // this removes two special cases to handle
            y = y_n;
            let temp = x_n;
            x_n = x;
            x = temp;
        }
        if x > x_n {
            // left case
            let distance = x - x_n - 1;
            let fields = DIAGONAL_LEFT_BITMASKS[distance as usize] << (x - distance + y * 10);
            return self.is_type_in(fields, type_searched);
        } else {
            // right case
            let fields = DIAGONAL_RIGHT_BITMASKS[(x_n - x - 1) as usize] << (x + y * 10);
            return self.is_type_in(fields, type_searched);
        }
    }

    pub fn is_field(&self, x: u8, y: u8, fieldtype: FieldType) -> bool {
        let field = 0b1 << (x + y * 10);
        return self.is_type_in(field, fieldtype);
    }

    pub fn is_type_in(&self, mask: u128, fieldtype: FieldType) -> bool {
        return match fieldtype {
            FieldType::RedPlayer => self.red_fields,
            FieldType::BluePlayer => self.blue_fields,
            FieldType::Obstacle => self.obstacle_fields,
            FieldType::Free => !(self.obstacle_fields | self.blue_fields | self.red_fields),
        }
        .are_bits_set(mask);
    }

    pub fn get_fields(&self) -> [[FieldType; 10]; 10] {
        let mut fields = [[FieldType::Free; 10]; 10];
        for x in 0..10 {
            for y in 0..10 {
                let index = y * 10 + x;
                if self.red_fields.is_bit_set(index) {
                    fields[x as usize][y as usize] = FieldType::RedPlayer;
                } else if self.blue_fields.is_bit_set(index) {
                    fields[x as usize][y as usize] = FieldType::BluePlayer;
                } else if self.obstacle_fields.is_bit_set(index) {
                    fields[x as usize][y as usize] = FieldType::Obstacle;
                }
            }
        }
        return fields;
    }

    pub fn get_distance(&self, x: u8, y: u8, direction: Direction) -> i8 {
        let fish = self.red_fields.bits | self.blue_fields.bits;
        let fish = fish
            & match direction {
                Direction::Right => HORIZONTAL_FULL_MASKS[y as usize],
                Direction::Left => HORIZONTAL_FULL_MASKS[y as usize],
                Direction::Up => VERTICAL_FULL_MASKS[x as usize],
                Direction::Down => VERTICAL_FULL_MASKS[x as usize],
                Direction::UpLeft => LEFT_DIAGONAL_FULL_MASKS[(x + y) as usize],
                Direction::DownRight => LEFT_DIAGONAL_FULL_MASKS[(x + y) as usize],
                Direction::UpRight => RIGHT_DIAGONAL_FULL_MASKS[(x as i8 - y as i8 + 9) as usize],
                Direction::DownLeft => RIGHT_DIAGONAL_FULL_MASKS[(x as i8 - y as i8 + 9) as usize],
            };
        return fish.count_ones() as i8;
    }

    pub fn set_field(&mut self, x: u8, y: u8, fieldtype: FieldType) {
        let index = y * 10 + x;
        let bit = 0b1 << index;

        self.red_fields.bits &= !bit;
        self.blue_fields.bits &= !bit;
        self.obstacle_fields.bits &= !bit;

        match fieldtype {
            FieldType::RedPlayer => {
                self.red_fields.set_bits(bit);
            }
            FieldType::BluePlayer => {
                self.blue_fields.set_bits(bit);
            }
            FieldType::Obstacle => {
                self.obstacle_fields.set_bits(bit);
            }
            _ => {}
        };
    }

    pub fn get_fields_of(&self, color: &PlayerColor) -> Bitboard {
        match color {
            &PlayerColor::Red => return self.red_fields,
            &PlayerColor::Blue => return self.blue_fields,
        };
    }

    pub fn get_field(&self, x: u8, y: u8) -> Option<Field> {
        if x > 9 || y > 9 {
            return None;
        }
        return Some(Field {
            x,
            y,
            fieldtype: self
                .get_fieldtype(x, y)
                .expect("ERROR: should never occur, is checked before"),
        });
    }

    pub fn get_fieldtype(&self, x: u8, y: u8) -> Option<FieldType> {
        if x > 9 || y > 9 {
            return None;
        }
        let index = y * 10 + x;
        if self.red_fields.is_bit_set(index) {
            return Some(FieldType::RedPlayer);
        }
        if self.blue_fields.is_bit_set(index) {
            return Some(FieldType::BluePlayer);
        }
        if self.obstacle_fields.is_bit_set(index) {
            return Some(FieldType::Obstacle);
        }
        return Some(FieldType::Free);
    }

    pub fn get_xml(&self) -> String {
        let mut string_version = "<board>\n".to_string();
        for ix in 0..10 {
            string_version += "<fields>\n";
            for iy in 0..10 {
                let fieldtype = self
                    .get_fieldtype(ix, iy)
                    .expect("ERROR: Did not find field; Should never occur here");
                string_version += format!(
                    "{}\n",
                    Field {
                        x: ix,
                        y: iy,
                        fieldtype,
                    }
                    .get_xml()
                )
                .as_str();
            }
            string_version += "</fields>\n";
        }
        string_version += "</board>\n";
        return string_version;
    }

    #[cfg(test)]
    pub fn from_condensed(condensed: [[&str; 10]; 10]) -> Board {
        let mut fields: [[FieldType; 10]; 10] = [[FieldType::Free; 10]; 10];
        for ix in 0..10 {
            for iy in 0..10 {
                fields[ix][iy] = match condensed[ix][iy] {
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
        return Board::new(fields);
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string_version = "  ".to_string();
        for i in 0..10 {
            string_version += &format!(" {}  ", i);
        }
        string_version += "\n";
        for i in 0..10 {
            string_version += &format!("{} ", i);
            for j in 0..10 {
                let short_field = match self.get_fieldtype(i, j).expect("ERROR: SHOULD NEVER OCCUR")
                {
                    FieldType::Free => " ",
                    FieldType::Obstacle => "O",
                    FieldType::BluePlayer => "B",
                    FieldType::RedPlayer => "R",
                };
                string_version += &format!("[{}] ", short_field);
            }
            string_version += "\n";
        }
        write!(f, "{}", string_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Functions to implement tests for:
    // - get_field
    // - set_field
    // - get_distance
    // - get_fields
    // - is_type_in

    #[test]
    fn new() {
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
        assert_eq!(board.get_fieldtype(1, 0).unwrap(), FieldType::RedPlayer);
    }

    #[test]
    fn is_field_between() {
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
        // diagonal
        assert!(board.is_field_between(0, 0, 9, 9, FieldType::Free));
        assert!(!board.is_field_between(0, 0, 9, 9, FieldType::RedPlayer));
        assert!(board.is_field_between(9, 9, 0, 0, FieldType::Free));
        assert!(!board.is_field_between(9, 9, 0, 0, FieldType::RedPlayer));
        assert!(board.is_field_between(0, 9, 9, 0, FieldType::Free));
        assert!(!board.is_field_between(0, 9, 9, 0, FieldType::RedPlayer));
        assert!(board.is_field_between(9, 0, 0, 9, FieldType::Free));
        assert!(!board.is_field_between(9, 0, 0, 9, FieldType::RedPlayer));
        // horizontal
        assert!(board.is_field_between(0, 0, 9, 0, FieldType::RedPlayer));
        assert!(!board.is_field_between(0, 0, 9, 0, FieldType::BluePlayer));
        assert!(board.is_field_between(9, 0, 0, 0, FieldType::RedPlayer));
        assert!(!board.is_field_between(9, 0, 0, 0, FieldType::BluePlayer));

        assert!(!board.is_field_between(3, 1, 0, 1, FieldType::BluePlayer));
        // vertical
        assert!(board.is_field_between(0, 8, 0, 1, FieldType::BluePlayer));
        assert!(!board.is_field_between(0, 8, 0, 1, FieldType::RedPlayer));
        assert!(board.is_field_between(0, 0, 0, 9, FieldType::BluePlayer));
        assert!(!board.is_field_between(0, 0, 0, 9, FieldType::RedPlayer));
    }

    #[test]
    fn is_field() {
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
        assert!(!board.is_field(1, 0, FieldType::BluePlayer));
        assert!(board.is_field(1, 0, FieldType::RedPlayer));
    }
}
