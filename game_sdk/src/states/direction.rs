use std;

/// Direction into all 8 directions (2-dimensional)
#[derive(Copy, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl Direction {
    pub fn get_multipliers(&self) -> (i8, i8) {
        match self {
            Direction::Up => (0, 1),
            Direction::UpRight => (1, 1),
            Direction::UpLeft => (-1, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Down => (0, -1),
            Direction::DownRight => (1, -1),
            Direction::DownLeft => (-1, -1),
        }
    }

    pub fn variants() -> Vec<Direction> {
        [
            Direction::Up,
            Direction::UpRight,
            Direction::Right,
            Direction::DownRight,
            Direction::Down,
            Direction::DownLeft,
            Direction::Left,
            Direction::UpLeft,
        ]
        .to_vec()
    }

    pub fn get_index(&self) -> usize {
        return match self {
            Direction::Up => 0,
            Direction::UpRight => 1,
            Direction::Right => 2,
            Direction::DownRight => 3,
            Direction::Down => 4,
            Direction::DownLeft => 5,
            Direction::Left => 6,
            Direction::UpLeft => 7,
        };
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string_version = match self {
            Direction::Up => "UP",
            Direction::UpLeft => "UP_LEFT",
            Direction::UpRight => "UP_RIGHT",
            Direction::Right => "RIGHT",
            Direction::Left => "LEFT",
            Direction::Down => "DOWN",
            Direction::DownLeft => "DOWN_LEFT",
            Direction::DownRight => "DOWN_RIGHT",
        };
        write!(f, "{}", string_version)
    }
}

impl std::fmt::Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string_version = match self {
            Direction::Up => "UP",
            Direction::UpLeft => "UP_LEFT",
            Direction::UpRight => "UP_RIGHT",
            Direction::Right => "RIGHT",
            Direction::Left => "LEFT",
            Direction::Down => "DOWN",
            Direction::DownLeft => "DOWN_LEFT",
            Direction::DownRight => "DOWN_RIGHT",
        };
        write!(f, "{}", string_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_multipliers() {
        assert_eq!(Direction::Up.get_multipliers(), (0, 1));
        assert_eq!(Direction::UpRight.get_multipliers(), (1, 1));
        assert_eq!(Direction::UpLeft.get_multipliers(), (-1, 1));
        assert_eq!(Direction::Left.get_multipliers(), (-1, 0));
        assert_eq!(Direction::Right.get_multipliers(), (1, 0));
        assert_eq!(Direction::Down.get_multipliers(), (0, -1));
        assert_eq!(Direction::DownRight.get_multipliers(), (1, -1));
        assert_eq!(Direction::DownLeft.get_multipliers(), (-1, -1));
    }
}
