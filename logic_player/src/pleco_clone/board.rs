use super::move_list::MoveList;
use super::ScoringMove;
use super::{z_square, z_turn};
use super::{BitMove, SQ};
use game_sdk::bitboard::constants::*;
use game_sdk::iterators::BitboardIter;
use game_sdk::{gamerules, Bitboard, Direction, FieldType, GameState, PlayerColor};
use std::ptr;

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    pub state: GameState,
    zobrist: u64,
}

impl Board {
    pub fn from_state(state: &GameState) -> Board {
        let zob = Board::full_zobrist(state);
        return Board {
            state: state.clone(),
            zobrist: zob,
        };
    }

    pub fn is_finished(&self) -> bool {
        return gamerules::is_finished(&self.state);
    }

    pub fn winner(&self) -> Option<PlayerColor> {
        // Unsafe to call when game is not finished
        return gamerules::get_winner(&self.state);
    }

    fn full_zobrist(state: &GameState) -> u64 {
        let mut zob = z_turn(state.turn);
        for index in game_sdk::iterators::BitboardIndexIter::new(state.board.red_fields.bits) {
            zob ^= z_square(SQ(index), PlayerColor::Red);
        }
        for index in game_sdk::iterators::BitboardIndexIter::new(state.board.blue_fields.bits) {
            zob ^= z_square(SQ(index), PlayerColor::Blue);
        }
        return zob;
    }

    pub fn apply_move(&mut self, mov: &BitMove, color: &PlayerColor) -> bool {
        self.zobrist ^= z_turn(self.state.turn);
        self.state.turn = self.state.turn + 1;
        self.zobrist ^= z_turn(self.state.turn);
        let pre_index = mov.data & 0b111_1111;
        let dest_index = (mov.data & !0b1000_0000_0000_0000) >> 7;
        let pre_bit = 0b1_u128 << pre_index;
        let dest_bit = 0b1_u128 << dest_index;
        let (own_bits, other_bits) = match color {
            PlayerColor::Red => (
                &mut self.state.board.red_fields,
                &mut self.state.board.blue_fields,
            ),
            PlayerColor::Blue => (
                &mut self.state.board.blue_fields,
                &mut self.state.board.red_fields,
            ),
        };

        let killing = other_bits.bits & dest_bit > 0;

        // remove from previous part
        own_bits.bits &= !pre_bit;
        self.zobrist ^= z_square(SQ(pre_index as u8), *color);

        // set at destination
        own_bits.bits |= dest_bit;
        self.zobrist ^= z_square(SQ(dest_index as u8), *color);

        // clear if killed
        if killing {
            other_bits.bits &= !dest_bit;
            self.zobrist ^= z_square(SQ(dest_index as u8), color.get_opponent_color());
        }

        return killing;
    }

    pub fn undo_move(&mut self, mov: &BitMove, killing: bool, color: &PlayerColor) {
        self.zobrist ^= z_turn(self.state.turn);
        self.state.turn = self.state.turn - 1;
        self.zobrist ^= z_turn(self.state.turn);
        let pre_index = mov.data & 0b111_1111;
        let dest_index = (mov.data & !0b1000_0000_0000_0000) >> 7;
        let pre_bit = 0b1 << pre_index;
        let dest_bit = 0b1 << dest_index;
        let (own_bits, other_bits) = match color {
            PlayerColor::Red => (
                &mut self.state.board.red_fields,
                &mut self.state.board.blue_fields,
            ),
            PlayerColor::Blue => (
                &mut self.state.board.blue_fields,
                &mut self.state.board.red_fields,
            ),
        };

        // remove from destination
        own_bits.bits &= !dest_bit;
        self.zobrist ^= z_square(SQ(dest_index as u8), *color);

        // set at previous
        own_bits.bits |= pre_bit;
        self.zobrist ^= z_square(SQ(pre_index as u8), *color);

        // set if killed
        if killing {
            other_bits.bits |= dest_bit;
            self.zobrist ^= z_square(SQ(dest_index as u8), color.get_opponent_color());
        }
    }

    pub fn get_captures(&self) -> MoveList {
        let mut result = MoveList::default();
        let color = self.state.get_current_player_color();
        let (own_bits, other_bits) = match color {
            PlayerColor::Red => (
                self.state.board.red_fields.bits,
                self.state.board.blue_fields.bits,
            ),
            PlayerColor::Blue => (
                self.state.board.blue_fields.bits,
                self.state.board.red_fields.bits,
            ),
        };
        let fields = BitboardIter::new(own_bits);
        let non_target_bits = own_bits | self.state.board.obstacle_fields.bits;
        let occupied_fields = self.state.board.blue_fields.bits | self.state.board.red_fields.bits;
        for field in fields {
            let x = field.0;
            let y = field.1;

            // LEFT
            let distance = (HORIZONTAL_FULL_MASKS[y as usize] & occupied_fields).count_ones() as i8;
            let x_n = x as i8 - distance;
            if x_n >= 0 {
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 && dest_bit & other_bits > 0 {
                    let fields =
                        HORIZONTAL_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y));
                    }
                }
            }

            // RIGHT
            // distance is same as left
            let x_n = x + distance as u8;
            if x_n < 10 {
                let dest_bit = SINGLE_BIT[x_n as usize][y as usize];
                // let dest_bit = 0b1 << (x_n + y*10);
                if non_target_bits & dest_bit == 0 && dest_bit & other_bits > 0 {
                    let fields =
                        HORIZONTAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y));
                    }
                }
            }

            // UP
            let distance = (occupied_fields & VERTICAL_FULL_MASKS[x as usize]).count_ones() as i8;
            let y_n = y + distance as u8;
            if y_n < 10 {
                let dest_bit = SINGLE_BIT[x as usize][y_n as usize];
                // let dest_bit = 0b1 << (x + y_n*10);
                if non_target_bits & dest_bit == 0 && dest_bit & other_bits > 0 {
                    let fields = VERTICAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x, y_n));
                    }
                }
            }

            // DOWN
            let y_n = y as i8 - distance;
            if y_n >= 0 {
                let y_n = y_n as u8;
                let dest_bit = SINGLE_BIT[x as usize][y_n as usize];
                // let dest_bit = 0b1 << (x + y_n*10);
                if non_target_bits & dest_bit == 0 && dest_bit & other_bits > 0 {
                    let fields =
                        VERTICAL_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x, y_n));
                    }
                }
            }

            // UPRIGHT
            let distance = (occupied_fields
                & RIGHT_DIAGONAL_FULL_MASKS[(x as i8 - y as i8 + 9) as usize])
                .count_ones() as i8;
            let x_n = x + distance as u8;
            let y_n = y + distance as u8;
            if x_n < 10 && y_n < 10 {
                let y_n = y_n as u8;
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 && dest_bit & other_bits > 0 {
                    let fields =
                        DIAGONAL_RIGHT_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y_n));
                    }
                }
            }

            // DOWNLEFT
            let x_n = x as i8 - distance;
            let y_n = y as i8 - distance;
            if x_n >= 0 && y_n >= 0 {
                let y_n = y_n as u8;
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 && dest_bit & other_bits > 0 {
                    let fields = DIAGONAL_RIGHT_DISTANCE_MASKS[x_n as usize][y_n as usize]
                        [distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y_n));
                    }
                }
            }

            // DOWNRIGHT
            let distance =
                (occupied_fields & LEFT_DIAGONAL_FULL_MASKS[(x + y) as usize]).count_ones() as i8;
            let x_n = x + distance as u8;
            let y_n = y as i8 - distance;
            if x_n < 10 && y_n >= 0 {
                let y_n = y_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 && dest_bit & other_bits > 0 {
                    let fields =
                        DIAGONAL_LEFT_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y_n));
                    }
                }
            }

            // UPLEFT
            let x_n = x as i8 - distance;
            let y_n = y + distance as u8;
            if x_n >= 0 && y_n < 10 {
                let x_n = x_n as u8;
                // let dest_bit = 0b1 << (x_n + y_n*10);
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                if non_target_bits & dest_bit == 0 && dest_bit & other_bits > 0 {
                    let fields =
                        DIAGONAL_LEFT_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y_n));
                    }
                }
            }
        }
        return result;
    }

    pub fn get_root_moves(&self) -> MoveList {
        let mut result = MoveList::default();
        let color = self.state.get_current_player_color();
        let (own_bits, other_bits) = match color {
            PlayerColor::Red => (
                self.state.board.red_fields.bits,
                self.state.board.blue_fields.bits,
            ),
            PlayerColor::Blue => (
                self.state.board.blue_fields.bits,
                self.state.board.red_fields.bits,
            ),
        };
        let fields = BitboardIter::new(own_bits);
        let non_target_bits = own_bits | self.state.board.obstacle_fields.bits;
        let occupied_fields = self.state.board.blue_fields.bits | self.state.board.red_fields.bits;
        for field in fields {
            let x = field.0;
            let y = field.1;

            // LEFT
            let distance = (HORIZONTAL_FULL_MASKS[y as usize] & occupied_fields).count_ones() as i8;
            let x_n = x as i8 - distance;
            if x_n >= 0 {
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        HORIZONTAL_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y));
                    }
                }
            }

            // RIGHT
            // distance is same as left
            let x_n = x + distance as u8;
            if x_n < 10 {
                let dest_bit = SINGLE_BIT[x_n as usize][y as usize];
                // let dest_bit = 0b1 << (x_n + y*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        HORIZONTAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y));
                    }
                }
            }

            // UP
            let distance = (occupied_fields & VERTICAL_FULL_MASKS[x as usize]).count_ones() as i8;
            let y_n = y + distance as u8;
            if y_n < 10 {
                let dest_bit = SINGLE_BIT[x as usize][y_n as usize];
                // let dest_bit = 0b1 << (x + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields = VERTICAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x, y_n));
                    }
                }
            }

            // DOWN
            let y_n = y as i8 - distance;
            if y_n >= 0 {
                let y_n = y_n as u8;
                let dest_bit = SINGLE_BIT[x as usize][y_n as usize];
                // let dest_bit = 0b1 << (x + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        VERTICAL_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x, y_n));
                    }
                }
            }

            // UPRIGHT
            let distance = (occupied_fields
                & RIGHT_DIAGONAL_FULL_MASKS[(x as i8 - y as i8 + 9) as usize])
                .count_ones() as i8;
            let x_n = x + distance as u8;
            let y_n = y + distance as u8;
            if x_n < 10 && y_n < 10 {
                let y_n = y_n as u8;
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        DIAGONAL_RIGHT_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y_n));
                    }
                }
            }

            // DOWNLEFT
            let x_n = x as i8 - distance;
            let y_n = y as i8 - distance;
            if x_n >= 0 && y_n >= 0 {
                let y_n = y_n as u8;
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields = DIAGONAL_RIGHT_DISTANCE_MASKS[x_n as usize][y_n as usize]
                        [distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y_n));
                    }
                }
            }

            // DOWNRIGHT
            let distance =
                (occupied_fields & LEFT_DIAGONAL_FULL_MASKS[(x + y) as usize]).count_ones() as i8;
            let x_n = x + distance as u8;
            let y_n = y as i8 - distance;
            if x_n < 10 && y_n >= 0 {
                let y_n = y_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        DIAGONAL_LEFT_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y_n));
                    }
                }
            }

            // UPLEFT
            let x_n = x as i8 - distance;
            let y_n = y + distance as u8;
            if x_n >= 0 && y_n < 10 {
                let x_n = x_n as u8;
                // let dest_bit = 0b1 << (x_n + y_n*10);
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        DIAGONAL_LEFT_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        result.push(BitMove::from_indices(x, y, x_n, y_n));
                    }
                }
            }
        }
        return result;
    }

    pub fn is_legal(&self, action: &BitMove) -> bool {
        if action.is_null() {
            return false;
        }
        let color = self.state.get_current_player_color();
        let field_of_color = color.to_fieldtype();
        let enemy_fields = match color {
            PlayerColor::Red => self.state.board.blue_fields.bits,
            PlayerColor::Blue => self.state.board.red_fields.bits,
        };

        let data = action.data & !0b1000_0000_0000_0000;
        let from = (data & 0b111_1111) as u8;
        let to = (data >> 7) as u8;
        let mut x = from % 10;
        let mut y = from / 10;
        let mut dest_x = to % 10;
        let mut dest_y = to / 10;
        if x > 9 || y > 9 || dest_x > 9 || dest_y > 9 {
            return false;
        }

        if !self.state.board.is_field(x, y, field_of_color) {
            // unowned fields are not to be moved
            return false;
        }

        if self
            .state
            .board
            .is_field(dest_x, dest_y, FieldType::Obstacle)
            || self.state.board.is_field(dest_x, dest_y, field_of_color)
        {
            return false;
        }
        if y == dest_y {
            // horizontal case
            if x > dest_x {
                let b = x;
                x = dest_x;
                dest_x = b;
            }
            let distance = (dest_x - x) as i8;
            if self.state.board.get_distance(x, y, Direction::Left) != distance {
                return false;
            }
            let fields = ((0b1 << (dest_x - x - 1)) - 1) << (x + y * 10 + 1);
            return fields & enemy_fields == 0;
        } else if x == dest_x {
            // vertical case
            if y > dest_y {
                let b = y;
                y = dest_y;
                dest_y = b;
            }
            let distance = (dest_y - y) as i8;
            if self.state.board.get_distance(x, y, Direction::Up) != distance {
                return false;
            }
            let fields = VERTICAL_BITMASKS[(dest_y - y - 1) as usize] << (x + y * 10);
            return fields & enemy_fields == 0;
        } else {
            // diagonal case
            if y > dest_y {
                // flipping line vertically
                // this removes two special cases to handle
                y = dest_y;
                let temp = dest_x;
                dest_x = x;
                x = temp;
            }
            if x > dest_x {
                // left case
                let distance = (x - dest_x) as i8;
                if self.state.board.get_distance(dest_x, y, Direction::UpLeft) != distance {
                    return false;
                }
                let distance = x - dest_x - 1;
                let fields = DIAGONAL_LEFT_BITMASKS[distance as usize] << (x - distance + y * 10);
                return fields & enemy_fields == 0;
            } else {
                // right case
                let distance = (dest_x - x) as i8;
                if self.state.board.get_distance(x, y, Direction::UpRight) != distance {
                    return false;
                }
                let fields = DIAGONAL_RIGHT_BITMASKS[(dest_x - x - 1) as usize] << (x + y * 10);
                return fields & enemy_fields == 0;
            }
        }
    }

    pub fn extend_from_ptr(&self, cur_ptr: *mut ScoringMove) -> *mut ScoringMove {
        let mut cur_ptr = cur_ptr;
        let color = self.state.get_current_player_color();
        let (own_bits, other_bits) = match color {
            PlayerColor::Red => (
                self.state.board.red_fields.bits,
                self.state.board.blue_fields.bits,
            ),
            PlayerColor::Blue => (
                self.state.board.blue_fields.bits,
                self.state.board.red_fields.bits,
            ),
        };
        let fields = BitboardIter::new(own_bits);
        let non_target_bits = own_bits | self.state.board.obstacle_fields.bits;
        let occupied_fields = self.state.board.blue_fields.bits | self.state.board.red_fields.bits;
        for field in fields {
            let x = field.0;
            let y = field.1;

            // LEFT
            let distance = (HORIZONTAL_FULL_MASKS[y as usize] & occupied_fields).count_ones() as i8;
            let x_n = x as i8 - distance;
            if x_n >= 0 {
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        HORIZONTAL_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        let action = ScoringMove::new(BitMove::from_indices(x, y, x_n, y));
                        unsafe {
                            ptr::write(cur_ptr, action);
                            cur_ptr = cur_ptr.add(1);
                        }
                    }
                }
            }

            // RIGHT
            // distance is same as left
            let x_n = x + distance as u8;
            if x_n < 10 {
                let dest_bit = SINGLE_BIT[x_n as usize][y as usize];
                // let dest_bit = 0b1 << (x_n + y*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        HORIZONTAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        let action = ScoringMove::new(BitMove::from_indices(x, y, x_n, y));
                        unsafe {
                            ptr::write(cur_ptr, action);
                            cur_ptr = cur_ptr.add(1);
                        }
                    }
                }
            }

            // UP
            let distance = (occupied_fields & VERTICAL_FULL_MASKS[x as usize]).count_ones() as i8;
            let y_n = y + distance as u8;
            if y_n < 10 {
                let dest_bit = SINGLE_BIT[x as usize][y_n as usize];
                // let dest_bit = 0b1 << (x + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields = VERTICAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        let action = ScoringMove::new(BitMove::from_indices(x, y, x, y_n));
                        unsafe {
                            ptr::write(cur_ptr, action);
                            cur_ptr = cur_ptr.add(1);
                        }
                    }
                }
            }

            // DOWN
            let y_n = y as i8 - distance;
            if y_n >= 0 {
                let y_n = y_n as u8;
                let dest_bit = SINGLE_BIT[x as usize][y_n as usize];
                // let dest_bit = 0b1 << (x + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        VERTICAL_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                    if other_bits & fields == 0 {
                        let action = ScoringMove::new(BitMove::from_indices(x, y, x, y_n));
                        unsafe {
                            ptr::write(cur_ptr, action);
                            cur_ptr = cur_ptr.add(1);
                        }
                    }
                }
            }

            // UPRIGHT
            let distance = (occupied_fields
                & RIGHT_DIAGONAL_FULL_MASKS[(x as i8 - y as i8 + 9) as usize])
                .count_ones() as i8;
            let x_n = x + distance as u8;
            let y_n = y + distance as u8;
            if x_n < 10 && y_n < 10 {
                let y_n = y_n as u8;
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        DIAGONAL_RIGHT_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        let action = ScoringMove::new(BitMove::from_indices(x, y, x_n, y_n));
                        unsafe {
                            ptr::write(cur_ptr, action);
                            cur_ptr = cur_ptr.add(1);
                        }
                    }
                }
            }

            // DOWNLEFT
            let x_n = x as i8 - distance;
            let y_n = y as i8 - distance;
            if x_n >= 0 && y_n >= 0 {
                let y_n = y_n as u8;
                let x_n = x_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields = DIAGONAL_RIGHT_DISTANCE_MASKS[x_n as usize][y_n as usize]
                        [distance as usize];
                    if other_bits & fields == 0 {
                        let action = ScoringMove::new(BitMove::from_indices(x, y, x_n, y_n));
                        unsafe {
                            ptr::write(cur_ptr, action);
                            cur_ptr = cur_ptr.add(1);
                        }
                    }
                }
            }

            // DOWNRIGHT
            let distance =
                (occupied_fields & LEFT_DIAGONAL_FULL_MASKS[(x + y) as usize]).count_ones() as i8;
            let x_n = x + distance as u8;
            let y_n = y as i8 - distance;
            if x_n < 10 && y_n >= 0 {
                let y_n = y_n as u8;
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                // let dest_bit = 0b1 << (x_n + y_n*10);
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        DIAGONAL_LEFT_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                    if other_bits & fields == 0 {
                        let action = ScoringMove::new(BitMove::from_indices(x, y, x_n, y_n));
                        unsafe {
                            ptr::write(cur_ptr, action);
                            cur_ptr = cur_ptr.add(1);
                        }
                    }
                }
            }

            // UPLEFT
            let x_n = x as i8 - distance;
            let y_n = y + distance as u8;
            if x_n >= 0 && y_n < 10 {
                let x_n = x_n as u8;
                // let dest_bit = 0b1 << (x_n + y_n*10);
                let dest_bit = SINGLE_BIT[x_n as usize][y_n as usize];
                if non_target_bits & dest_bit == 0 {
                    let fields =
                        DIAGONAL_LEFT_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                    if other_bits & fields == 0 {
                        let action = ScoringMove::new(BitMove::from_indices(x, y, x_n, y_n));
                        unsafe {
                            ptr::write(cur_ptr, action);
                            cur_ptr = cur_ptr.add(1);
                        }
                    }
                }
            }
        }
        return cur_ptr;
    }

    pub fn zobrist(&self) -> u64 {
        return self.zobrist;
    }

    pub fn null() -> Board {
        let bits = Bitboard { bits: 0 };
        return Board {
            zobrist: 0,
            state: GameState {
                turn: 244,
                board: game_sdk::Board {
                    red_fields: bits,
                    blue_fields: bits,
                    obstacle_fields: bits,
                },
            },
        };
    }
}
