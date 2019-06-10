use game_sdk::logging::MoveValuePair;
use game_sdk::logging::State;
use game_sdk::FieldType;
use game_sdk::PlayerColor;

#[derive(Serialize, Deserialize)]
pub struct WriteState {
    winner: Option<PlayerColor>,
    board: [[FieldType; 10]; 10],
    moves: Vec<MoveValuePair>,
    current_color: PlayerColor,
    turn: u8,
    data: Vec<(String, f32)>,
}

impl WriteState {
    pub fn from_state(s: State, winner: Option<PlayerColor>) -> WriteState {
        return WriteState {
            winner: winner,
            moves: s.moves,
            board: s.gamestate.board.get_fields(),
            current_color: s.gamestate.get_current_player_color(),
            turn: s.gamestate.turn,
            data: s.data,
        };
    }
}
