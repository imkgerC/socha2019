mod mcts;
mod piranhas;
mod enemy_pool;
pub use self::enemy_pool::EnemyPool;

mod player;
mod algorithm;
mod evaluation;
mod transposition;

mod new_minimax;
pub use self::new_minimax::NewMinimaxPlayer as ToClop;
pub use self::new_minimax::NewMinimaxParameters as ClopParameters;

/*pub use self::player::MinimaxPlayer as ToClop;
pub use self::player::MinimaxParameters as ClopParameters;*/

mod rave_player;
/*pub use self::rave_player::RavePlayer as ToClop;
pub use self::rave_player::RaveParameters as ClopParameters;*/