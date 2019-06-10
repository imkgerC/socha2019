extern crate game_sdk;
extern crate xml;

use self::game_sdk::*;
use self::xml::reader::{EventReader, XmlEvent};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::BufReader;
use std::net::TcpStream;
use std::vec::Vec;

#[derive(Debug)]
pub struct XMLNode {
	name: String,
	data: String,
	attribs: HashMap<String, Vec<String>>,
	childs: Vec<XMLNode>,
}

impl XMLNode {
	fn new() -> XMLNode {
		return XMLNode {
			name: String::new(),
			data: String::new(),
			attribs: HashMap::new(),
			childs: Vec::new(),
		};
	}

	pub fn read_from(xml_parser: &mut EventReader<BufReader<&TcpStream>>) -> XMLNode {
		let mut node_stack: VecDeque<XMLNode> = VecDeque::new();
		let mut has_received_first = false;
		let mut final_node: Option<XMLNode> = None;

		loop {
			match xml_parser.next() {
				Ok(XmlEvent::StartElement {
					name, attributes, ..
				}) => {
					let mut node = XMLNode::new();
					node.name = name.local_name;
					for attribute in attributes {
						let attrib_name = attribute.name.local_name;
						if !node.attribs.contains_key(&attrib_name) {
							node.attribs.insert(attrib_name.to_string(), Vec::new());
						}
						node.attribs
							.get_mut(&attrib_name)
							.unwrap()
							.push(attribute.value.to_string());
					}
					node_stack.push_back(node);
					has_received_first = true;
				}
				Ok(XmlEvent::EndElement { .. }) => {
					if node_stack.len() > 2 {
						let child = node_stack.pop_back().expect("Unexpectedly found empty XML node stack while trying to pop off new child element");
						let mut node = node_stack.pop_back().expect("Unexpectedly found empty XML node stack while trying to hook up new child element");
						node.childs.push(child);
						node_stack.push_back(node);
					} else if has_received_first {
						final_node = Some(node_stack.pop_back().expect(
							"Unexpectedly found empty XML node stack while trying to return node",
						));
					}
				}
				Ok(XmlEvent::Characters(content)) => {
					node_stack.back_mut().expect("Unexpectedly found empty XML node stack while trying to add characters").data += content.as_str();
				}
				Err(_) => {
					break;
				}
				_ => {}
			}

			// Exit condition
			if final_node.is_some() {
				break;
			}
		}

		return final_node.unwrap(); // Is guaranteed to be present due to the condition above
	}

	pub fn as_game_state(&self) -> GameState {
		let err = "Error while parsing XML node to GameState";
		return GameState::new(
			// self.get_child("red").expect(err).as_player(),
			// self.get_child("blue").expect(err).as_player(),
			self.get_child("board").expect(err).as_board(),
			self.get_attribute("turn")
				.expect(err)
				.parse::<u8>()
				.unwrap(),
		);
	}

	/* pub fn as_player(&self) -> Player {
		let err = "Error while parsing XML node to Player";
		return Player {
			display_name: self.get_attribute("displayName").expect(err).to_string(),
			color: match self.get_attribute("color").expect(err).to_string().as_str() {
				"RED" => PlayerColor::Red,
				"BLUE" => PlayerColor::Blue,
				_ => panic!("Error parsing Player"),
			},
		};
	} */

	pub fn as_room(&self) -> Room {
		let err = "Error while parsing XML node to Room";
		return Room {
			id: self.get_attribute("roomId").expect(err).to_string(),
		};
	}

	pub fn _as_joined(&self) -> Joined {
		let err = "Error while parsing XML node to Joined";
		return Joined {
			id: self.get_attribute("roomId").expect(err).to_string(),
		};
	}

	#[allow(unused)]
	pub fn winner_string(&self) -> String {
		let _err = "Error while parsing XML node to WinnerString";
		let winner_str;
		if let Some(winner) = self.get_child("winner") {
			winner_str = format!(
				"{} wins as {}",
				winner
					.get_attribute("displayName")
					.unwrap_or(&"Draw".to_string()),
				winner.get_attribute("color").unwrap_or(&"Draw".to_string())
			);
		} else {
			winner_str = format!("DRAW");
		}
		let cause_str = self
			.get_child("score")
			.expect("did not find score while parsing XML node")
			.get_attribute("reason")
			.expect("did not find reason while parsing XML node");
		return format!("{}, reason: {}", winner_str, cause_str);
	}

	pub fn as_welcome_message(&self) -> WelcomeMessage {
		let err = "Error while parsing XML node to WelcomeMessage";
		return WelcomeMessage {
			color: self.get_attribute("color").expect(err).to_string(),
		};
	}

	pub fn as_board(&self) -> Board {
		let mut fields: [[FieldType; 10]; 10] = [[FieldType::Obstacle; 10]; 10];
		for row in self.get_child_vec("fields").iter() {
			for field in row.get_child_vec("field").iter().map(|n| n.as_field()) {
				fields[field.x as usize][field.y as usize] = field.fieldtype;
			}
		}
		return Board::new(fields);
	}

	pub fn as_memento(&self) -> Memento {
		let err = "Error while parsing XML node to Memento";
		return Memento {
			state: self.get_child("state").expect(err).as_game_state(),
		};
	}

	pub fn as_field(&self) -> Field {
		let err = "Error while parsing XML node to Field";
		return Field {
			fieldtype: self.to_fieldtype(self.get_attribute("state").expect(err)),
			x: self
				.get_attribute("x")
				.expect(err)
				.parse::<u8>()
				.expect(err),
			y: self
				.get_attribute("y")
				.expect(err)
				.parse::<u8>()
				.expect(err),
		};
	}

	pub fn to_fieldtype(&self, fieldtype: &str) -> FieldType {
		match fieldtype {
			"EMPTY" => FieldType::Free,
			"RED" => FieldType::RedPlayer,
			"BLUE" => FieldType::BluePlayer,
			"OBSTRUCTED" => FieldType::Obstacle,
			_ => FieldType::Free,
		}
	}

	pub fn get_name(&self) -> &String {
		return &self.name;
	}

	pub fn get_attributes(&self) -> &HashMap<String, Vec<String>> {
		return &self.attribs;
	}

	pub fn get_attribute(&self, name: &str) -> Option<&String> {
		return self.attribs.get(name).map(|a| &a[0]);
	}

	pub fn get_child_vec(&self, name: &str) -> Vec<&XMLNode> {
		let mut result: Vec<&XMLNode> = Vec::new();

		for child in &self.childs {
			if child.name.as_str() == name {
				result.push(&child);
			}
		}

		return result;
	}

	pub fn get_children(&self) -> &Vec<XMLNode> {
		return &self.childs;
	}

	pub fn get_child(&self, name: &str) -> Option<&XMLNode> {
		for child in &self.childs {
			if child.name.as_str() == name {
				return Some(&child);
			}
		}

		return None;
	}
}

impl Clone for XMLNode {
	fn clone(&self) -> Self {
		return XMLNode {
			name: self.name.clone(),
			data: self.data.clone(),
			attribs: self.attribs.clone(),
			childs: self.childs.clone(),
		};
	}
}
