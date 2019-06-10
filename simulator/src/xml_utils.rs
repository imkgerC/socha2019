use game_sdk::gamerules;
use game_sdk::GameState;
use game_sdk::PlayerColor;

pub fn get_xml_turn(state: &GameState) -> String {
    let mut string_version = "".to_string();
    string_version += "<room roomId=\"12\">\n";
    string_version += "<data class=\"memento\">\n";
    string_version += state.get_xml().as_str();
    string_version += "</data>\n";
    string_version += "</room>";
    return string_version;
}

pub fn get_xml_result(state: &GameState) -> String {
    let red_points;
    let blue_points;
    match gamerules::get_winner(&state) {
        None => {
            red_points = 1;
            blue_points = 1;
        }
        Some(s) => match s {
            PlayerColor::Blue => {
                blue_points = 2;
                red_points = 0;
            }
            _ => {
                blue_points = 0;
                red_points = 2;
            }
        },
    };
    let mut string_version = "".to_string();
    string_version += "<room roomId=\"12\">\n";
    string_version += "<data class=\"result\">\n";
    string_version += get_xml_result_definition().as_str();

    string_version += "<score cause=\"REGULAR\" reason=\"Irgendwas ist passiert\">\n";
    string_version += format!("<part>{}</part>", red_points).as_str();
    string_version += format!(
        "<part>{}</part>",
        state.greatest_swarm_size(&PlayerColor::Red)
    ).as_str();
    string_version += "</score>\n";
    string_version += "<score cause=\"REGULAR\" reason=\"Irgendwas ist passiert\">\n";
    string_version += format!("<part>{}</part>", blue_points).as_str();
    string_version += format!(
        "<part>{}</part>",
        state.greatest_swarm_size(&PlayerColor::Blue)
    ).as_str();
    string_version += "</score>\n";

    string_version += match gamerules::get_winner(&state) {
        None => "",
        Some(s) => match s {
            PlayerColor::Blue => {
                "<winner class=\"player\" displayName=\"Unknown\" color=\"BLUE\"/>"
            }
            _ => "<winner class=\"player\" displayName=\"Unknown\" color=\"RED\"/>",
        },
    };

    string_version += "</data>\n";
    string_version += "</room>";
    return string_version;
}

fn get_xml_result_definition() -> String {
    let mut string_version = "".to_string();

    string_version += "<definition>\n";
    string_version += "<fragment name=\"Gewinner\">\n";
    string_version += "<aggregation>SUM</aggregation>\n";
    string_version += "<relevantForRanking>true</relevantForRanking>\n";
    string_version += "</fragment>\n";
    string_version += "<fragment name=\"? Schwarm\">\n";
    string_version += "<aggregation>AVERAGE</aggregation>\n";
    string_version += "<relevantForRanking>true</relevantForRanking>\n";
    string_version += "</fragment>\n";
    string_version += "</definition>\n";

    return string_version;
}
