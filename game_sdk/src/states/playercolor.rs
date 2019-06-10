use states::FieldType;
use std;
#[derive(Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerColor {
    Red = 0b0,
    Blue = 0b1,
}
impl PlayerColor {
    pub fn to_fieldtype(&self) -> FieldType {
        match self {
            PlayerColor::Red => FieldType::RedPlayer,
            PlayerColor::Blue => FieldType::BluePlayer,
        }
    }

    pub fn get_opponent_color(&self) -> PlayerColor {
        match self {
            PlayerColor::Red => PlayerColor::Blue,
            PlayerColor::Blue => PlayerColor::Red,
        }
    }
}

impl std::fmt::Display for PlayerColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlayerColor::Red => "RED",
                PlayerColor::Blue => "BLUE",
            }
        )
    }
}

impl std::fmt::Debug for PlayerColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlayerColor::Red => "RED",
                PlayerColor::Blue => "BLUE",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_opponent_color() {
        assert_eq!(PlayerColor::Red, PlayerColor::Blue.get_opponent_color());
        assert_eq!(PlayerColor::Blue, PlayerColor::Red.get_opponent_color());
    }

    #[test]
    fn to_field_type() {
        assert_eq!(FieldType::RedPlayer, PlayerColor::Red.to_fieldtype());
        assert_eq!(FieldType::BluePlayer, PlayerColor::Blue.to_fieldtype());
    }
}
