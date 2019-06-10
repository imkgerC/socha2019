// TODO: documentation
use crate::iterators::BitboardIter;
use std::ops;

#[derive(Clone, Eq, PartialEq, Copy, Debug)]
pub struct Bitboard {
    pub bits: u128,
}

pub mod constants {
    pub static HORIZONTAL_BITMASKS: [u128; 10] = [0, 1, 3, 7, 15, 31, 63, 127, 255, 511];

    lazy_static! {
        pub static ref HORIZONTAL_DISTANCE_MASKS: [[[u128; 10]; 10]; 10] = get_horizontal_masks();
    }
    fn get_horizontal_masks() -> [[[u128; 10]; 10]; 10] {
        let mut result = [[[0_u128; 10]; 10]; 10];
        for x in 0..10 {
            for y in 0..10 {
                for distance in 1..10 {
                    result[x][y][distance] = HORIZONTAL_BITMASKS[distance - 1] << (x + y * 10 + 1);
                }
            }
        }
        return result;
    }

    pub static VERTICAL_BITMASKS: [u128; 10] = [
        0, // distance: 0
        1024,
        1049600,
        1074791424, // distance: 3
        1100586419200,
        1127000493261824, // distance: 5
        1154048505100108800,
        1181745669222511412224, // distance: 7
        1210107565283851686118400,
        1239150146850664126585242624,
    ];
    lazy_static! {
        pub static ref VERTICAL_DISTANCE_MASKS: [[[u128; 10]; 10]; 10] = get_vertical_masks();
    }
    fn get_vertical_masks() -> [[[u128; 10]; 10]; 10] {
        let mut result = [[[0_u128; 10]; 10]; 10];
        for x in 0..10 {
            for y in 0..10 {
                for distance in 1..10 {
                    result[x][y][distance] = VERTICAL_BITMASKS[distance - 1] << (x + y * 10);
                }
            }
        }
        return result;
    }

    pub static DIAGONAL_RIGHT_BITMASKS: [u128; 10] = [
        0, // distance: 0
        2048,
        4196352,
        8594130944, // distance: 3
        17600780175360,
        36046397799139328, // distance: 5
        73823022692637345792,
        151189550474521284184064, // distance: 7
        309636199371819590008965120,
        634134936313486520338360567808,
    ];

    lazy_static! {
        pub static ref DIAGONAL_RIGHT_DISTANCE_MASKS: [[[u128; 10]; 10]; 10] =
            get_diagonal_right_masks();
    }
    fn get_diagonal_right_masks() -> [[[u128; 10]; 10]; 10] {
        let mut result = [[[0_u128; 10]; 10]; 10];
        for x in 0..10 {
            for y in 0..10 {
                for distance in 1..10 {
                    result[x][y][distance] = DIAGONAL_RIGHT_BITMASKS[distance - 1] << (x + y * 10);
                }
            }
        }
        return result;
    }

    pub static DIAGONAL_LEFT_BITMASKS: [u128; 10] = [
        0, // distance: 0
        1024,
        1050624,
        1075843072, // distance: 3
        1101663313920,
        1128103233470464, // distance: 5
        1155177711073787904,
        1182901976139558879232, // distance: 7
        1211291623566908292464640,
        1240362622532514091484053504,
    ];
    lazy_static! {
        pub static ref DIAGONAL_LEFT_DISTANCE_MASKS: [[[u128; 10]; 10]; 10] =
            get_diagonal_left_masks();
    }
    fn get_diagonal_left_masks() -> [[[u128; 10]; 10]; 10] {
        let mut result = [[[0_u128; 10]; 10]; 10];
        for x in 0..10 {
            for y in 0..10 {
                for distance in 1..10 {
                    result[x][y][distance] =
                        DIAGONAL_LEFT_BITMASKS[distance - 1] << (x + 1 + y * 10);
                }
            }
        }
        return result;
    }

    lazy_static! {
        pub static ref SINGLE_BIT: [[u128; 10]; 10] = get_single_bits();
    }
    fn get_single_bits() -> [[u128; 10]; 10] {
        let mut result = [[0_u128; 10]; 10];
        for x in 0..10 {
            for y in 0..10 {
                result[x][y] = 0b1 << (x + y * 10);
            }
        }
        return result;
    }

    lazy_static! {
        pub static ref RIGHT_DIAGONAL_FULL_MASKS: [u128; 19] = get_right_diagonal_masks();
    }
    fn get_right_diagonal_masks() -> [u128; 19] {
        let mut result = [0_u128; 19];
        for y in 0..9 {
            let mut y = y;
            let mut x = 0;
            let mut inner = 0;
            while y <= 9 && x <= 9 {
                inner |= 0b1 << (x + y * 10);
                x += 1;
                y += 1;
            }
            result[(x - y + 9) as usize] = inner;
        }
        for x in 0..9 {
            let mut y = 0;
            let mut x = x;
            let mut inner = 0;
            while y <= 9 && x <= 9 {
                inner |= 0b1 << (x + y * 10);
                x += 1;
                y += 1;
            }
            result[(x - y + 9) as usize] = inner;
        }
        let mut y = 0;
        let mut x = 0;
        let mut inner = 0;
        while y <= 9 && x <= 9 {
            inner |= 0b1 << (x + y * 10);
            x += 1;
            y += 1;
        }
        result[(x - y + 9) as usize] = inner;

        return result;
    }

    lazy_static! {
        pub static ref LEFT_DIAGONAL_FULL_MASKS: [u128; 19] = get_left_diagonal_masks();
    }
    fn get_left_diagonal_masks() -> [u128; 19] {
        let mut result = [0_u128; 19];
        for y in 0..10 {
            let mut y = y;
            let mut x = 0;
            let mut inner = 0;
            while y >= 0 && x <= 9 {
                inner |= 0b1 << (x + y * 10);
                x += 1;
                y -= 1;
            }
            result[(x + y) as usize] = inner;
        }
        for x in 0..10 {
            let mut y = 9;
            let mut x = x;
            let mut inner = 0;
            while y >= 0 && x <= 9 {
                inner |= 0b1 << (x + y * 10);
                x += 1;
                y -= 1;
            }
            result[(x + y) as usize] = inner;
        }

        return result;
    }
    lazy_static! {
        pub static ref HORIZONTAL_FULL_MASKS: [u128; 10] = get_horizontal_full_masks();
    }
    fn get_horizontal_full_masks() -> [u128; 10] {
        let mut result = [0_u128; 10];
        for y in 0..10 {
            let mut inner = 0;
            for x in 0..10 {
                inner |= 0b1 << (x + y * 10);
            }
            result[y] = inner;
        }
        return result;
    }

    lazy_static! {
        pub static ref VERTICAL_FULL_MASKS: [u128; 10] = get_vertical_full_masks();
    }
    fn get_vertical_full_masks() -> [u128; 10] {
        let mut result = [0_u128; 10];
        for x in 0..10 {
            let mut inner = 0;
            for y in 0..10 {
                inner |= 0b1 << (x + y * 10);
            }
            result[x] = inner;
        }
        return result;
    }
}

impl Bitboard {
    pub fn new() -> Bitboard {
        return Bitboard { bits: 0 };
    }

    pub fn index_from_coordinates(x: u8, y: u8) -> u8 {
        return y * 10 + x;
    }

    pub fn set_field(&mut self, x: u8, y: u8) {
        self.set_bit(Bitboard::index_from_coordinates(x, y));
    }

    pub fn set_bit(&mut self, index: u8) {
        self.bits |= 0b1 << index;
    }

    pub fn set_bits(&mut self, bits: u128) {
        self.bits |= bits;
    }

    pub fn clear_bit(&mut self, index: u8) {
        self.bits &= !(0b1 << index);
    }

    pub fn clear_bits(&mut self, bits: u128) {
        self.bits &= !(bits);
    }

    pub fn is_bit_set(&self, index: u8) -> bool {
        return self.bits & (0b1 << index) > 0;
    }

    pub fn is_field_set(&self, x: u8, y: u8) -> bool {
        return self.is_bit_set(Bitboard::index_from_coordinates(x, y));
    }

    pub fn clear_field(&mut self, x: u8, y: u8) {
        self.clear_bit(Bitboard::index_from_coordinates(x, y));
    }

    pub fn are_bits_set(&self, bitfield: u128) -> bool {
        return self.bits & bitfield > 0;
    }

    pub fn count_ones(&self) -> u32 {
        return self.bits.count_ones();
    }

    pub fn coordinates_from_index(index: u8) -> (u8, u8) {
        let x = index % 10;
        let y = index / 10;
        return (x, y);
    }

    pub fn mask(&mut self, mask: u128) {
        self.bits = self.bits & mask;
    }

    pub fn get_first(&self) -> (u8, u8) {
        let mut bits = self.bits;
        let mut index = 0;
        while bits & 0b1 <= 0 && index < 100 {
            bits = bits >> 1;
            index += 1;
        }
        return Bitboard::coordinates_from_index(index);
        // return Bitboard::coordinates_from_index(self.bits.trailing_zeros() as u8);
    }

    pub fn get_first_index(&self) -> u8 {
        return self.bits.trailing_zeros() as u8;
    }

    pub fn iter(&self) -> impl Iterator<Item = (u8, u8)> {
        return BitboardIter::new(self.bits);
    }

    pub fn get_fields(&self) -> Vec<(u8, u8)> {
        return BitboardIter::new(self.bits).collect();
    }

    pub fn from_bits(bits: u128) -> Bitboard {
        return Bitboard { bits };
    }
}

impl ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        return Bitboard {
            bits: self.bits & rhs.bits,
        };
    }
}

impl ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        return Bitboard {
            bits: self.bits | rhs.bits,
        };
    }
}

impl ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self {
        let new_bits = !self.bits;
        return Bitboard { bits: new_bits };
    }
}

#[cfg(test)]
mod tests {
    // TODO: test more cases per function
    // TODO: is_field_set
    // TODO: clear_field
    // TODO: count_ones
    // TODO: get_first
    // TODO: coordinates_from_index
    // TODO: get_fields
    use super::*;

    #[test]
    fn are_bits_set() {
        let index = 4;
        let bits = (0b1 << index) - 1;
        let bitboard = Bitboard::from_bits(bits);
        assert!(bitboard.are_bits_set(bits));
        assert!(!bitboard.are_bits_set(0b1 << index));
    }

    #[test]
    fn is_bit_set() {
        let index = 5;
        let bitboard = Bitboard::from_bits(0b1 << index);
        assert!(bitboard.is_bit_set(index));
        assert!(!bitboard.is_bit_set(index + 1));
    }

    #[test]
    fn index_from_coordinates() {
        let x = 3;
        let y = 2;
        assert_eq!(Bitboard::index_from_coordinates(x, y), 23);
    }

    #[test]
    fn set_field() {
        let x = 5;
        let y = 3;
        let index = Bitboard::index_from_coordinates(x, y);
        let mut bitboard = Bitboard::new();
        bitboard.set_field(x, y);
        assert_eq!(bitboard, Bitboard::from_bits(0b1 << index));
    }

    #[test]
    fn clear_bit() {
        let index = 6;
        let mut bitboard_set = Bitboard::from_bits(0b1 << index);
        let bitboard = Bitboard::new();
        bitboard_set.clear_bit(index);
        assert_eq!(bitboard, bitboard_set);
    }

    #[test]
    fn set_bit() {
        let index = 6;
        let bitboard_set = Bitboard::from_bits(0b1 << index);
        let mut bitboard = Bitboard::new();
        bitboard.set_bit(index);
        assert_eq!(bitboard, bitboard_set);
    }

    #[test]
    fn not() {
        let seed = 3578234;
        let bitboard = Bitboard::from_bits(seed);
        let opposite_bitboard = Bitboard::from_bits(!seed);
        assert_eq!(bitboard, !!bitboard);
        assert_eq!(bitboard, !opposite_bitboard);
        assert_eq!(!bitboard, opposite_bitboard);
    }

    #[test]
    fn bitand() {
        let seed1 = 3578234;
        let seed2 = 5382982;
        let bitboard = Bitboard::from_bits(seed1);
        let other_bitboard = Bitboard::from_bits(seed2);
        let and_bitboard = Bitboard::from_bits(seed1 & seed2);
        assert_eq!(bitboard & other_bitboard, and_bitboard);
    }

    #[test]
    fn bitor() {
        let seed1 = 3578234;
        let seed2 = 5382982;
        let bitboard = Bitboard::from_bits(seed1);
        let other_bitboard = Bitboard::from_bits(seed2);
        let or_bitboard = Bitboard::from_bits(seed1 | seed2);
        assert_eq!(bitboard | other_bitboard, or_bitboard);
    }

    #[test]
    fn new() {
        let bitboard = Bitboard::new();
        let same_bitboard = Bitboard::from_bits(0);
        assert_eq!(bitboard, same_bitboard);
        assert_ne!(bitboard, Bitboard::from_bits(1));
    }
}
