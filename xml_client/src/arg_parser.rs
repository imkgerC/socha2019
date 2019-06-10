use std::collections::HashMap;
use std::env;
use std::string::String;

pub struct ArgParser {
	args: HashMap<String, String>
}

impl ArgParser {
	pub fn new() -> ArgParser {
		let mut arg_map: HashMap<String, String> = HashMap::new();
		let mut current_key: Option<String> = None;
		let mut is_first: bool = false;

		for arg in env::args() {
			if !is_first {
				if let Some(key) = current_key {
					arg_map.insert(key, arg);
					current_key = None;
				} else {
					current_key = Some(arg.to_string());
				}
			} else {
				is_first = false;
			}
		}

		return ArgParser { args: arg_map };
	}
	
	pub fn get_string(&self, key: &str) -> Option<String> {
		return self.args.get(key).map(|s| s.to_string());
	}

	pub fn _get_int(&self, key: &str) -> Option<i32> {
		if let Some(value) = self.args.get(key).map(|s| s.parse::<i32>()) {
			return value.ok();
		} else {
			return None;
		}
	}
}