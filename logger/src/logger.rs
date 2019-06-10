use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json;
use game_sdk::logging::EndState;
use game_sdk::logging::State;
use states::WriteState;

const STATES_PER_FILE: usize = 100;

pub struct Logger {
    directory: String,
    index: u32,
    out_queue: Vec<WriteState>,
    state_map: HashMap<u32, Vec<State>>,
}

impl Logger {
    pub fn new(directory: String) -> Logger {
        fs::create_dir_all(&directory).unwrap();
        let mut index = 0;
        while Path::new(&format!("{}/{}", directory, index)).exists() {
            index += 1;
        }
        return Logger {
            directory,
            index,
            out_queue: Vec::new(),
            state_map: HashMap::new(),
        };
    }

    pub fn add_to_queue(&mut self, state: WriteState) {
        self.out_queue.push(state);

        if self.out_queue.len() > STATES_PER_FILE {
            self.write_file();
        }
    }

    pub fn add_multiple_to_queue(&mut self, states: Vec<WriteState>) {
        for state in states {
            self.add_to_queue(state);
        }
    }

    pub fn add_state(&mut self, state: &State) {
        if let Some(ref mut v) = self.state_map.get_mut(&state.id) {
            v.push(state.clone());
            return;
        }
        self.state_map.insert(state.id, vec![state.clone()]);
    }

    pub fn add_states(&mut self, states: Vec<&State>) {
        for state in states {
            self.add_state(state);
        }
    }

    pub fn end_state(&mut self, end_state: &EndState) {
        if let Some(ref mut v) = self.state_map.remove(&end_state.id) {
            if let Some(c) = end_state.color {
                let len = v.len();
                for state in v.drain(0..len) {
                    /*if state.gamestate.get_current_player_color() == c {
                        self.add_to_queue(state);
                    }*/
                    self.add_to_queue(WriteState::from_state(state, Some(c)));
                }
            } else {
                let len = v.len();
                for state in v.drain(0..len) {
                    /*if state.gamestate.get_current_player_color() == c {
                        self.add_to_queue(state);
                    }*/
                    self.add_to_queue(WriteState::from_state(state, None));
                }
            }
        } else {
            // there were no states, which could be fine, if no one is logging
            // so there is no error message thrown, as it would only clutter
            // the terminal
        }
    }

    fn write_file(&mut self) {
        /*for state in self.states.drain(0..STATES_PER_FILE) {
            string_version += "<state>\n";
            string_version += &state.to_writable();
            string_version += "</state>\n";
        }*/
        let out_queue: Vec<WriteState> = self.out_queue.drain(0..STATES_PER_FILE).collect();
        // TODO: GET BETTER STATE TO LEAVE IT IN
        let string_version = serde_json::to_string(&out_queue).unwrap();
        fs::write(self.get_path(), string_version).expect("Unable to write file");
        self.index += 1;
    }

    fn get_path(&self) -> String {
        return format!("{}/{}", self.directory, self.index);
    }
}
