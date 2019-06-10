extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

pub mod bitboard;
mod board;
pub mod gamerules;
mod gamestate;
pub mod iterators;
pub mod logging;
mod states;

pub use self::bitboard::Bitboard;
pub use self::board::Board;
pub use self::gamestate::GameState;
pub use self::states::Direction;
pub use self::states::Field;
pub use self::states::FieldType;
pub use self::states::Joined;
pub use self::states::Memento;
pub use self::states::Move;
pub use self::states::PlayerColor;
pub use self::states::Room;
pub use self::states::WelcomeMessage;

/// Trait that needs to be implemented for every Player
/// The "on"-methods are called on the events
pub trait ClientListener {
	/// This function is called whenever a memento message is received from the
	/// server. It is given a gamestate-struct
	fn on_update_state(&mut self, _state: &GameState) {}

	/// On connection with the server gets our PlayerColor inside the WelcomeMessage
	fn on_welcome_message(&mut self, _welcome_message: &WelcomeMessage) {}

	/// On every Request a Move is requested. Needs to be implemented by every client.
	/// Implements most of the game-playing logic inside this method in a typical client
	fn on_move_request(&mut self, state: &GameState) -> Move;

	/// Is called with a room number, not really interesting for a client implementation
	fn on_join(&mut self, _room: &Room) {}

	/// Is called when the TCPListener is idling. Should do some work but take no longer than 1ms
	fn on_idle(&mut self) {}
}
