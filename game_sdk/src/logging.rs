use crate::Move;
use crate::GameState;
use crate::gamerules;
use crate::PlayerColor;

pub enum Data {
    Step(State),
    End(EndState),
}

pub struct EndState {
    pub id: u32,
    pub winner: Winner,
    pub color: Option<PlayerColor>,
}

impl EndState {
    pub fn get_end(state: &GameState, id: u32) -> EndState {
        let winner;
        let mut color = None;
        if let Some(c) = gamerules::get_winner(state) {
            winner = Winner::get_winner(&c, id);
            color = Some(c);
        } else {
            winner = Winner::Draw;
        }
        return EndState { id, winner, color };
    }
}

#[derive(PartialEq, Eq)]
pub enum Winner {
    One,
    Two,
    Draw,
}

impl std::fmt::Display for Winner {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Winner::One => "ONE ",
                Winner::Two => "TWO ",
                Winner::Draw => "DRAW",
            }
        )
    }
}

impl Winner {
    pub fn get_winner(color: &PlayerColor, index: u32) -> Winner {
        if color == &PlayerColor::Blue {
            match index % 2 {
                0 => return Winner::Two,
                _ => return Winner::One,
            };
        }
        match index % 2 {
            0 => return Winner::One,
            _ => return Winner::Two,
        };
    }
}

#[derive(Clone)]
pub struct State {
    pub id: u32,
    pub gamestate: GameState,
    pub moves: Vec<MoveValuePair>,
    pub data: Vec<(String,f32)>,
}

impl State {
    pub fn to_writable(&self) -> String {
        let mut string_version = "".to_string();
        string_version += self.gamestate.get_xml().as_str();
        string_version += "<moves>\n";
        for action in self.moves.iter() {
            string_version += action.get_xml().as_str();
        }
        string_version += "</moves>\n";
        return string_version;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MoveValuePair {
    pub action: Move,
    pub value: f32,
}

impl MoveValuePair {
    pub fn get_xml(&self) -> String {
        let mut string_version = "".to_string();
        string_version += &format!(
            "<move_value value={} move={} />",
            self.value,
            self.action.get_xml()
        );
        return string_version;
    }
}