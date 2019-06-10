use game_sdk::GameState;
use game_sdk::Move;

use hashbrown::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Exact,
    UpperBound,
    LowerBound,
}

#[derive(Clone)]
pub struct HashEntry {
    pub value: f32,
    pub action: Move,
    pub depth: u8,
    pub entry: EntryType,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct MinimalState {
    red_fields: u128,
    blue_fields: u128,
    turn: u8,
}

impl MinimalState {
    pub fn from_state(state: &GameState) -> MinimalState {
        return MinimalState {
            red_fields: state.board.red_fields.bits,
            blue_fields: state.board.blue_fields.bits,
            turn: state.turn,
        };
    }

    #[allow(unused)]
    pub fn empty() -> MinimalState {
        return MinimalState {
            red_fields: 0,
            blue_fields: 0,
            turn: 255,
        };
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    table: HashMap<MinimalState, HashEntry>,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        return TranspositionTable {
            table: HashMap::new(),
        };
    }

    pub fn insert(
        &mut self,
        hash: &MinimalState,
        value: f32,
        depth: u8,
        action: &Move,
        entry: EntryType,
    ) {
        self.table.insert(
            *hash,
            HashEntry {
                value,
                depth,
                action: *action,
                entry,
            },
        );
    }

    pub fn lookup(&self, hash: &MinimalState) -> Option<&HashEntry> {
        return self.table.get(hash);
    }
}
