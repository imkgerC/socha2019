use states::Direction;
use std;

/// Basic move struct with all needed information saved in it. Should only be passed around
/// if was checked for specific state
#[derive(Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Move {
    pub x: u8,
    pub y: u8,
    pub dest_x: u8,
    pub dest_y: u8,
    pub direction: Direction,
}

impl Move {
    pub fn new(x: u8, y: u8, dest_x: u8, dest_y: u8, direction: Direction) -> Move {
        return Move {
            x,
            y,
            dest_x,
            dest_y,
            direction,
        };
    }

    /// Constructs XML for sending to java server for decoding into sc-plugin2019
    pub fn get_xml(&self) -> String {
        let direction = match self.direction {
            Direction::Up => "UP",
            Direction::UpRight => "UP_RIGHT",
            Direction::UpLeft => "UP_LEFT",
            Direction::Left => "LEFT",
            Direction::Right => "RIGHT",
            Direction::Down => "DOWN",
            Direction::DownLeft => "DOWN_LEFT",
            Direction::DownRight => "DOWN_RIGHT",
        };
        return format!(
            "<data class=\"move\" x=\"{}\" y=\"{}\" direction=\"{}\"></data>",
            self.x, self.y, direction
        );
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "x:{} y:{} {} x_n:{} y_n:{}",
            self.x, self.y, self.direction, self.dest_x, self.dest_y
        )
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "x:{} y:{} {} x_n:{} y_n:{}",
            self.x, self.y, self.direction, self.dest_x, self.dest_y
        )
    }
}
