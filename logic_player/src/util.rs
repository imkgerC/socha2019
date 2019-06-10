use game_sdk::Bitboard;
use game_sdk::GameState;
use game_sdk::PlayerColor;

pub struct Helper;

impl Helper {
    pub fn get_distance_to_swarm(x: u8, y: u8, swarm: &Vec<(u8, u8)>) -> u32 {
        let mut minimal_distance = 200;
        for (ix, iy) in swarm {
            let ix = *ix;
            let iy = *iy;
            if ix == x && iy == y {
                return 0;
            }
            let distance = ((ix as i32 - x as i32) * (ix as i32 - x as i32)
                + (iy as i32 - y as i32) * (iy as i32 - y as i32))
                as u32;
            if distance < minimal_distance {
                minimal_distance = distance;
            }
        }
        return minimal_distance;
    }

    pub fn greatest_swarm(state: &GameState, color: &PlayerColor) -> Vec<(u8, u8)> {
        let mut greatest_size: u8 = 0;
        let mut greatest_swarm = Vec::new();
        let mut fields = state.board.get_fields_of(color);
        while greatest_size < fields.count_ones() as u8 {
            let (x, y) = fields.get_first();
            // fields.clear_field(x, y);
            let pair = Helper::get_swarm_pair(state, &mut fields, x, y);
            if pair.0 > greatest_size {
                greatest_size = pair.0;
                greatest_swarm = pair.1;
            }
        }
        return greatest_swarm;
    }

    #[allow(dead_code)]
    pub fn get_distance_to_swarm_new(x: u8, y: u8, swarm: &Bitboard) -> i8 {
        let mut minimal_distance = 0;
        let mut bits = Bitboard::new();
        bits.set_field(x, y);
        let mut bits = bits.bits;
        while minimal_distance < 9 {
            if bits & swarm.bits > 0 {
                return minimal_distance;
            }
            let not_left_row = !bits & 1268889750375080065623288448001; // vertical row where x = 0
            let not_right_row = !bits & 649671552192040993599123685376512; // vertical row where x = 9;
            let shift_left = (bits << 1) & !not_left_row;
            let shift_right = (bits >> 1) & !not_right_row;
            bits = bits | shift_left | shift_right;
            let shift_up = bits << 10;
            let shift_down = bits >> 10;
            bits = bits | shift_down | shift_up;
            minimal_distance += 1;
        }
        return minimal_distance;
    }

    #[allow(dead_code)]
    pub fn get_distance_to_swarm_alt(index: u8, swarm: &Bitboard, enemy: &Bitboard) -> i8 {
        let mut minimal_distance = 0;
        let mut bits = 0b1_u128 << index;
        let mut before = 0;
        while bits > before {
            if bits & swarm.bits > 0 {
                return minimal_distance;
            }
            before = bits;
            let not_left_row = !bits & 1268889750375080065623288448001; // vertical row where x = 0
            let not_right_row = !bits & 649671552192040993599123685376512; // vertical row where x = 9;
            let shift_left = (bits << 1) & !not_left_row;
            let shift_right = (bits >> 1) & !not_right_row;
            bits = bits | shift_left | shift_right;
            let shift_up = bits << 10;
            let shift_down = bits >> 10;
            bits = bits | shift_down | shift_up;
            bits &= !enemy.bits;
            minimal_distance += 1;
        }
        return 55;
    }

    #[allow(dead_code)]
    pub fn greatest_swarm_new(state: &GameState, color: &PlayerColor) -> Bitboard {
        let mut greatest_size: u8 = 0;
        let mut greatest_swarm = Bitboard::new();
        let mut fields = state.board.get_fields_of(color);
        while greatest_size < fields.count_ones() as u8 {
            // let (x, y) = fields.get_first();
            let index = fields.get_first_index();
            let (size, swarm, _steps) = Helper::get_swarm_pair_new(state, &mut fields, index);
            if size > greatest_size {
                greatest_size = size;
                greatest_swarm = swarm;
            }
        }
        return greatest_swarm;
    }

    #[allow(dead_code)]
    pub fn greatest_swarm_other(state: &GameState, color: &PlayerColor) -> (Bitboard, u8) {
        let mut greatest_size: u8 = 0;
        let mut greatest_steps = 0;
        let mut greatest_swarm = Bitboard::new();
        let mut fields = state.board.get_fields_of(color);
        while greatest_size < fields.count_ones() as u8 {
            // let (x, y) = fields.get_first();
            let index = fields.get_first_index();
            let (size, swarm, steps) = Helper::get_swarm_pair_new(state, &mut fields, index);
            if size > greatest_size {
                greatest_size = size;
                greatest_swarm = swarm;
                greatest_steps = steps;
            }
        }
        return (greatest_swarm, greatest_steps);
    }

    fn get_swarm_pair_new(
        _state: &GameState,
        fields: &mut Bitboard,
        index: u8,
    ) -> (u8, Bitboard, u8) {
        let mut swarm = Bitboard::from_bits(0b1_u128 << index);
        let mut current_size = 1;
        let mut before_size = 0;
        let mut steps = 1;
        while current_size > before_size {
            before_size = current_size;
            Helper::extend(&mut swarm, fields);
            current_size = swarm.count_ones() as u8;
            steps += 1
        }
        fields.clear_bits(swarm.bits);
        return (current_size, swarm, steps);
    }

    #[allow(unused)]
    pub fn greatest_swarm_pair(state: &GameState, color: &PlayerColor) -> (u8, Vec<(u8, u8)>) {
        let mut greatest_pair = (0, Vec::new());
        let mut fields = state.board.get_fields_of(color);
        while greatest_pair.0 < fields.count_ones() as u8 {
            let (x, y) = fields.get_first();
            // fields.clear_field(x, y);
            let pair = Helper::get_swarm_pair(state, &mut fields, x, y);
            if pair.0 >= greatest_pair.0 {
                greatest_pair = pair;
            }
        }
        return greatest_pair;
    }

    pub fn get_swarms(state: &GameState, color: &PlayerColor) -> Vec<(u8, Vec<(u8, u8)>)> {
        let mut fields = state.board.get_fields_of(color);
        let mut res = Vec::with_capacity(4);
        while fields.count_ones() > 0 {
            let (x, y) = fields.get_first();
            let pair = Helper::get_swarm_pair(state, &mut fields, x, y);
            res.push(pair);
        }
        return res;
    }

    fn get_swarm_pair(
        _state: &GameState,
        fields: &mut Bitboard,
        x: u8,
        y: u8,
    ) -> (u8, Vec<(u8, u8)>) {
        let mut swarm = Bitboard::new();
        swarm.set_field(x, y);
        let mut current_size = 1;
        let mut before_size = 0;
        while current_size > before_size {
            before_size = current_size;
            Helper::extend(&mut swarm, fields);
            current_size = swarm.count_ones() as u8;
        }
        fields.clear_bits(swarm.bits);
        return (current_size, swarm.get_fields());
    }

    fn extend(to_extend: &mut Bitboard, fields: &Bitboard) {
        let mut bits = to_extend.bits;
        let not_left_row = !bits & 1268889750375080065623288448001; // vertical row where x = 0
        let not_right_row = !bits & 649671552192040993599123685376512; // vertical row where x = 9;
        let shift_left = (bits << 1) & !not_left_row;
        let shift_right = (bits >> 1) & !not_right_row;
        bits = bits | shift_left | shift_right;
        let shift_up = bits << 10;
        let shift_down = bits >> 10;
        bits = bits | shift_down | shift_up;
        bits = bits & fields.bits;
        to_extend.bits = bits;
    }
}
