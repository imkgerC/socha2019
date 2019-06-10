/// Basic structs for gamelogic

mod direction;
mod action;
mod playercolor;
mod field;
mod fieldtype;

use std::string::String;
use gamestate::GameState;

pub use self::direction::Direction;
pub use self::action::Move;
pub use self::playercolor::PlayerColor;
pub use self::field::Field;
pub use self::fieldtype::FieldType;

pub struct Room {
    pub id: String,
}

pub struct WelcomeMessage {
    pub color: String,
}
pub struct Joined {
    pub id: String,
}
pub struct Memento {
    pub state: GameState,
}

// no tests for those, too simple