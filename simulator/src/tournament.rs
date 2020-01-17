use game_runner::run_with_player;
use game_sdk::logging::EndState;
use game_sdk::logging::Winner;
use logic_player::Player;
#[allow(unused_imports)]
use logic_player::{
    LogicBasedPlayer, MinimaxPlayer, MultiDistancePlayer,
    SingleDistancePlayer,
};

use std::cmp::Ordering;
use std::sync::mpsc;
use threadpool::ThreadPool;

struct Participant {
    pub player: Player,
    pub wins: f32,
}

impl Participant {
    fn new(player: Player) -> Participant {
        return Participant { player, wins: 0.0 };
    }
}

pub fn run_tournament(threads: usize, n: u32, xml_enabled: bool) {
    // create logging thread
    let (t_winner, r_winner): (mpsc::Sender<EndState>, mpsc::Receiver<EndState>) = mpsc::channel();

    let pool = ThreadPool::new(threads);
    let mut players: [Participant; 4] = [
        Participant::new(Player::MinimaxPlayer(MinimaxPlayer::new(None, 0))),
        Participant::new(Player::LogicBasedPlayer(LogicBasedPlayer::new(None, 0))),
        Participant::new(Player::SingleDistancePlayer(SingleDistancePlayer::new(
            None, 0,
        ))),
        Participant::new(Player::MultiDistancePlayer(MultiDistancePlayer::new(
            None, 0,
        ))),
    ];

    let player_len = players.len() as u32;
    for i in 0..n {
        // players.sort_unstable_by(|a, b| b.wins.partial_cmp(&a.wins).unwrap_or(Ordering::Equal));
        let games = player_len / 2;
        for j in 0..games {
            let t_winner = t_winner.clone();
            let index = player_len * i + j;
            let player_one = players[(2 * j) as usize].player.clone();
            let player_two = players[(2 * j + 1) as usize].player.clone();
            pool.execute(move || {
                run_with_player(index, t_winner, player_one, player_two, xml_enabled);
            });
        }
        for _ in 0..games {
            for state in r_winner.recv() {
                match state.winner {
                    Winner::One => players[((state.id % player_len) as usize) * 2].wins += 1.0,
                    Winner::Two => players[((state.id % player_len) as usize) * 2 + 1].wins += 1.0,
                    Winner::Draw => {
                        players[((state.id % player_len) as usize) * 2 + 1].wins += 0.5;
                        players[((state.id % player_len) as usize) * 2].wins += 0.5;
                    }
                };
            }
        }
        if player_len % 2 != 0 {
            players[player_len as usize - 1].wins += 0.5;
        }
        println!("round {} finished, results:", i+1);
        players.sort_unstable_by(|a, b| b.wins.partial_cmp(&a.wins).unwrap_or(Ordering::Equal));
        for participant in players.iter() {
            println!("{}:{}", participant.wins, participant.player)
        }
        println!("");
    }
}
