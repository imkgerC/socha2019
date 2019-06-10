use super::GameState;
use bitboard::Bitboard;
use states::{Direction, Move, PlayerColor};
use std::iter::Iterator;

use bitboard::constants::*;

#[derive(Debug)]
pub struct BitboardIter {
    pub bits: u128,
    pub index: u8,
}

impl BitboardIter {
    pub fn new(bits: u128) -> BitboardIter {
        return BitboardIter { bits, index: 0 };
    }
}

impl Iterator for BitboardIter {
    type Item = (u8, u8);
    fn next(&mut self) -> Option<Self::Item> {
        while self.bits > 0 {
            if self.bits & 0b1 > 0 {
                let before = self.index;
                self.bits = self.bits >> 1;
                self.index = (self.index + 1) & 0b0111_1111; // mod 128
                return Some(Bitboard::coordinates_from_index(before));
            }
            let zeros = self.bits.trailing_zeros();
            self.bits = self.bits >> zeros;
            // mod 128, anything over 127 is wraparound in a 128 bit word
            self.index = (self.index + zeros as u8) & 0b0111_1111;
        }
        return None;
    }
}

#[derive(Debug)]
pub struct BitboardIndexIter {
    pub bits: u128,
    pub index: u8,
}

impl BitboardIndexIter {
    pub fn new(bits: u128) -> BitboardIndexIter {
        return BitboardIndexIter { bits, index: 0 };
    }
}

impl Iterator for BitboardIndexIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        while self.bits > 0 {
            if self.bits & 0b1 > 0 {
                let before = self.index;
                self.bits = self.bits >> 1;
                self.index = (self.index + 1) & 0b0111_1111; // mod 128
                return Some(before);
            }
            let zeros = self.bits.trailing_zeros();
            self.bits = self.bits >> zeros;
            // mod 128, anything over 127 is wraparound in a 128 bit word
            self.index = (self.index + zeros as u8) & 0b0111_1111;
        }
        return None;
    }
}

pub fn get_possible_moves(state: &GameState) -> Vec<Move> {
    let mut result = Vec::with_capacity(50);
    let color = state.get_current_player_color();
    let (own_bits, other_bits) = match color {
        PlayerColor::Red => (state.board.red_fields.bits, state.board.blue_fields.bits),
        PlayerColor::Blue => (state.board.blue_fields.bits, state.board.red_fields.bits),
    };
    let fields = BitboardIter::new(own_bits);
    let non_target_bits = own_bits | state.board.obstacle_fields.bits;
    let occupied_fields = state.board.blue_fields.bits | state.board.red_fields.bits;
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
                let fields = HORIZONTAL_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                if other_bits & fields == 0 {
                    let action = Move {
                        x,
                        y,
                        direction: Direction::Left,
                        dest_x: x_n,
                        dest_y: y,
                    };
                    result.push(action);
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
                let fields = HORIZONTAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                if other_bits & fields == 0 {
                    let action = Move {
                        x,
                        y,
                        direction: Direction::Right,
                        dest_x: x_n,
                        dest_y: y,
                    };
                    result.push(action);
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
                    let action = Move {
                        x,
                        y,
                        direction: Direction::Up,
                        dest_x: x,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                let fields = VERTICAL_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                if other_bits & fields == 0 {
                    let action = Move {
                        x,
                        y,
                        direction: Direction::Down,
                        dest_x: x,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                    let action = Move {
                        x,
                        y,
                        direction: Direction::UpRight,
                        dest_x: x_n,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                let fields =
                    DIAGONAL_RIGHT_DISTANCE_MASKS[x_n as usize][y_n as usize][distance as usize];
                if other_bits & fields == 0 {
                    let action = Move {
                        x,
                        y,
                        direction: Direction::DownLeft,
                        dest_x: x_n,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                    let action = Move {
                        x,
                        y,
                        direction: Direction::DownRight,
                        dest_x: x_n,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                    let action = Move {
                        x,
                        y,
                        direction: Direction::UpLeft,
                        dest_x: x_n,
                        dest_y: y_n,
                    };
                    result.push(action);
                }
            }
        }
    }
    return result;
}

pub fn get_captures(state: &GameState) -> Vec<Move> {
    let mut result = Vec::with_capacity(50);
    let color = state.get_current_player_color();
    let (own_bits, other_bits) = match color {
        PlayerColor::Red => (state.board.red_fields.bits, state.board.blue_fields.bits),
        PlayerColor::Blue => (state.board.blue_fields.bits, state.board.red_fields.bits),
    };
    let fields = BitboardIter::new(own_bits);
    let non_target_bits = own_bits | state.board.obstacle_fields.bits;
    let occupied_fields = state.board.blue_fields.bits | state.board.red_fields.bits;
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
                let fields = HORIZONTAL_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                if other_bits & fields == 0 {
                    let action = Move {
                        x,
                        y,
                        direction: Direction::Left,
                        dest_x: x_n,
                        dest_y: y,
                    };
                    result.push(action);
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
                let fields = HORIZONTAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                if other_bits & fields == 0 && dest_bit & other_bits > 0 {
                    let action = Move {
                        x,
                        y,
                        direction: Direction::Right,
                        dest_x: x_n,
                        dest_y: y,
                    };
                    result.push(action);
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
                    let action = Move {
                        x,
                        y,
                        direction: Direction::Up,
                        dest_x: x,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                let fields = VERTICAL_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                if other_bits & fields == 0 {
                    let action = Move {
                        x,
                        y,
                        direction: Direction::Down,
                        dest_x: x,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                    let action = Move {
                        x,
                        y,
                        direction: Direction::UpRight,
                        dest_x: x_n,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                let fields =
                    DIAGONAL_RIGHT_DISTANCE_MASKS[x_n as usize][y_n as usize][distance as usize];
                if other_bits & fields == 0 {
                    let action = Move {
                        x,
                        y,
                        direction: Direction::DownLeft,
                        dest_x: x_n,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                    let action = Move {
                        x,
                        y,
                        direction: Direction::DownRight,
                        dest_x: x_n,
                        dest_y: y_n,
                    };
                    result.push(action);
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
                    let action = Move {
                        x,
                        y,
                        direction: Direction::UpLeft,
                        dest_x: x_n,
                        dest_y: y_n,
                    };
                    result.push(action);
                }
            }
        }
    }
    return result;
}

pub fn get_attack_board(state: &GameState, color: &PlayerColor) -> u128 {
    let mut result = 0b0;
    let (own_bits, other_bits) = match color {
        PlayerColor::Red => (state.board.red_fields.bits, state.board.blue_fields.bits),
        PlayerColor::Blue => (state.board.blue_fields.bits, state.board.red_fields.bits),
    };
    let fields = BitboardIter::new(own_bits);
    let non_target_bits = own_bits | state.board.obstacle_fields.bits;
    let occupied_fields = state.board.blue_fields.bits | state.board.red_fields.bits;
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
                let fields = HORIZONTAL_DISTANCE_MASKS[x_n as usize][y as usize][distance as usize];
                if other_bits & fields == 0 {
                    result |= dest_bit;
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
                let fields = HORIZONTAL_DISTANCE_MASKS[x as usize][y as usize][distance as usize];
                if other_bits & fields == 0 && dest_bit & other_bits > 0 {
                    result |= dest_bit;
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
                    result |= dest_bit;
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
                let fields = VERTICAL_DISTANCE_MASKS[x as usize][y_n as usize][distance as usize];
                if other_bits & fields == 0 {
                    result |= dest_bit;
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
                    result |= dest_bit;
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
                let fields =
                    DIAGONAL_RIGHT_DISTANCE_MASKS[x_n as usize][y_n as usize][distance as usize];
                if other_bits & fields == 0 {
                    result |= dest_bit;
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
                    result |= dest_bit;
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
                    result |= dest_bit;
                }
            }
        }
    }
    return result;
}
