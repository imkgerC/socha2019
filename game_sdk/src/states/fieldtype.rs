use states::PlayerColor;
use std;
#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum FieldType {
    RedPlayer = 2,
    BluePlayer = 1,
    Obstacle = 0,
    Free = 3,
}
impl FieldType {
    pub fn to_player_color(&self) -> Option<PlayerColor> {
        match self {
            FieldType::RedPlayer => Some(PlayerColor::Red),
            FieldType::BluePlayer => Some(PlayerColor::Blue),
            _ => None,
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string_version = match self {
            FieldType::BluePlayer => "BLUE",
            FieldType::RedPlayer => "RED",
            FieldType::Obstacle => "OBSTRUCTED",
            FieldType::Free => "EMPTY",
        };
        write!(f, "{}", string_version)
    }
}

impl std::fmt::Debug for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string_version = match self {
            FieldType::BluePlayer => "BLUE",
            FieldType::RedPlayer => "RED",
            FieldType::Obstacle => "OBSTRUCTED",
            FieldType::Free => "EMPTY",
        };
        write!(f, "{}", string_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_player_color() {
        assert_eq!(
            Some(PlayerColor::Red),
            FieldType::RedPlayer.to_player_color()
        );
        assert_eq!(
            Some(PlayerColor::Blue),
            FieldType::BluePlayer.to_player_color()
        );
        assert_eq!(None, FieldType::Obstacle.to_player_color());
        assert_eq!(None, FieldType::Free.to_player_color());
    }
}
