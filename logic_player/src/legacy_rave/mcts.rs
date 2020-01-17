use super::piranhas::{MinimalState, Piranhas};
use game_sdk::Move;
use game_sdk::PlayerColor;
use hashbrown::HashMap;
use std::f32;
use std::i32;

use crate::LogicBasedPlayer;

/*fn varianced_playout(initial: &Piranhas, color: &PlayerColor) -> f32 {
    let mut game = initial.clone();
    while !game.is_finished() {
        // let mut potential_moves = game.allowed_actions().into_iter();
        if let Some(action) = LogicBasedPlayer::on_state(&game.state) {
            game.make_move(&action);
        } else {
            if *color == game.get_color() {
                return 0.0;
            }
            return 1.0;
        }
    }
    return game.reward(color);
}*/

fn varianced_playout(
    initial: &Piranhas,
    color: &PlayerColor,
    rave_table: &mut HashMap<Move, Value>,
) -> f32 {
    let mut game = initial.clone();
    if game.is_finished() {
        return game.reward(color);
    }
    if let Some(action) = LogicBasedPlayer::on_state(&game.state) {
        game.make_move(&action);
        let val = varianced_playout(&game, color, rave_table);
        let mut rave = rave_table.remove(&action).unwrap_or(Value::new());
        rave.n += 1.;
        rave.q += if initial.get_color() != *color {
            val
        } else {
            1. - val
        };
        rave_table.insert(action.clone(), rave);
        return val;
    } else {
        if *color == game.get_color() {
            return 0.0;
        }
        return 1.0;
    }
}

#[derive(Copy, Clone, Debug)]
struct ChildEdge {
    pub index: MinimalState,
    pub action: Move,
    pub added: bool,
}

impl ChildEdge {
    pub fn new(index: MinimalState, action: Move, added: bool) -> ChildEdge {
        return ChildEdge {
            index,
            action,
            added,
        };
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Value {
    pub q: f32,
    pub n: f32,
}

impl Value {
    pub fn new() -> Value {
        return Value { n: 0., q: 0. };
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    index: MinimalState,
    parents: Vec<MinimalState>,
    children: Vec<ChildEdge>, // next steps we investigated
    is_leaf: bool,            // is this a leaf node? fully expanded?
    color: PlayerColor,
    n: f32,
    q: f32, // statistics for this game state
    n_since_last_expansion: f32,
    lower_bound: f32,
    last_update: u64,
    depth: Option<u8>,
    stats: Option<(i32, i32, i32)>,
}

impl TreeNode {
    /// Create and initialize a new TreeNode
    pub fn new(color: PlayerColor, index: MinimalState, parent: Option<MinimalState>) -> TreeNode {
        let parents;
        if let Some(s) = parent {
            parents = vec![s];
        } else {
            parents = Vec::new();
        }
        TreeNode {
            index,
            parents,
            children: Vec::new(),
            is_leaf: false,
            color: color,
            n: 0.,
            q: 0.,
            lower_bound: -2.0,
            depth: None,
            stats: None,
            n_since_last_expansion: 1.,
            last_update: 0,
        }
    }

    /// Gather some statistics about this subtree
    pub fn tree_statistics(
        &mut self,
        node_table: &mut HashMap<MinimalState, TreeNode>,
    ) -> (i32, i32, i32) {
        if let Some(stats) = self.stats {
            return stats;
        }
        let mut nodes = 1;
        let mut max_depth = 0;
        let mut min_depth = 200;
        for c in &self.children {
            if let Some(mut child) = node_table.remove(&c.index) {
                let (n, max, min) = child.tree_statistics(node_table);
                nodes += n;
                max_depth = i32::max(max, max_depth);
                min_depth = i32::min(min, min_depth);
                node_table.insert(c.index, child);
            }
        }
        if nodes == 1 {
            return (1, 0, 0);
        }
        self.stats = Some((nodes, max_depth + 1, min_depth + 1));
        return (nodes, max_depth + 1, min_depth + 1);
    }

    fn add_own_children(&mut self, game: &mut Piranhas) -> bool {
        let mut rated_actions = LogicBasedPlayer::get_rated_moves(&game.state);
        if rated_actions.len() == 0 {
            self.is_leaf = true;
            self.depth = Some(0);
            return false;
        }
        rated_actions.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        for (_, action) in rated_actions {
            let mut game_clone = game.clone();
            game_clone.make_move(&action);
            let state = MinimalState::from_state(&game_clone.state);
            self.children.push(ChildEdge::new(state, action, false));
        }
        return true;
    }

    pub fn best_child_fpu(
        &mut self,
        game: &mut Piranhas,
        c: f32,
        node_table: &mut HashMap<MinimalState, TreeNode>,
        rave_table: &mut HashMap<Move, Value>,
        is_root: bool,
    ) -> (f32, f32) {
        let color = game.get_color();
        // child generation
        if self.children.len() == 0 {
            if !self.add_own_children(game) {
                return (game.reward(&color), 1.0);
            }
        }

        let mut n = 0.0;
        let mut q = 0.0;

        let fpu_base = if is_root {1.5} else {(self.n - self.q) / self.n - 1e-2};
        let fpu_exploration = (self.n.ln() / 1.).sqrt();
        let mut best_value: f32 = f32::NEG_INFINITY;
        let mut best_child_index: Option<usize> = None;
        let c_base = 19652.;
        let b_squared = 0.35;
        let c = c + 2.2 * ((1. + self.n + c_base) / c_base).ln();
        let mut non_terminal = 0;
        for (idx, edge) in self.children.iter_mut().enumerate() {
            if !edge.added {
                if let Some(node) = node_table.get_mut(&edge.index) {
                    node.add_parent(self.index);
                    n += node.n;
                    q += node.n - node.q;
                    edge.added = true;
                } else {
                    non_terminal += 1;
                    let rave = rave_table.remove(&edge.action).unwrap_or(Value::new());
                    let beta =
                        f32::min(rave.n / (rave.n + 10. + 4. * b_squared * rave.n * 10.), 1.0);
                    let value =
                        (1. - beta) * fpu_base + c * fpu_exploration + beta * (rave.q / rave.n);
                    if value > best_value || best_child_index == None {
                        best_child_index = Some(idx);
                        best_value = value;
                    }
                    rave_table.insert(edge.action.clone(), rave);
                    continue;
                }
            }
            let node = node_table
                .get(&edge.index)
                .expect("ERROR: Did not find child in UCT");
            if node.is_leaf {
                let value = node.q / node.n;
                self.lower_bound = f32::max(self.lower_bound, value);
                if value > 0.5 {
                    self.is_leaf = true;
                    self.q = (1. - value) * self.n;
                    self.depth = Some(node.depth.unwrap_or(0) + 1);
                    return (self.q / self.n, 1.0);
                }
            } else {
                non_terminal += 1;
                let rave = rave_table.remove(&edge.action).unwrap_or(Value::new());
                let beta = f32::min(
                    rave.n / (rave.n + node.n + 4. * b_squared * rave.n * node.n),
                    1.0,
                );
                let value = (1. - beta) * (node.q / node.n)
                    + c * (2. * self.n.ln() / node.n).sqrt()
                    + beta * (rave.q / rave.n);
                if value > best_value || best_child_index == None {
                    best_value = value;
                    best_child_index = Some(idx);
                }
                rave_table.insert(edge.action.clone(), rave);
            }
        }
        if non_terminal == 0 {
            self.is_leaf = true;
            best_value = self.lower_bound;
            for c in &self.children {
                let child = node_table
                    .get(&c.index)
                    .expect("ERROR: Did not find child in UCT");
                let value = child.q / child.n;
                if value >= best_value {
                    best_value = value;
                    self.depth = Some(child.depth.unwrap_or(0) + 1);
                }
            }
            self.q = (1. - best_value) * self.n;
            return (self.q / self.n, 1.0);
        }
        if let Some(idx) = best_child_index {
            let edge = self
                .children
                .get_mut(idx)
                .expect("Should never happen, index is from iteration");
            if !edge.added {
                self.n_since_last_expansion = 1.;
                edge.added = true;
                let mut node = TreeNode::new(
                    self.color.get_opponent_color(),
                    edge.index.clone(),
                    Some(self.index),
                );
                game.make_move(&edge.action);
                let delta = varianced_playout(game, &node.color, rave_table);
                n += 1.0;
                q += 1.0 - delta;
                node.backpropagate(delta, 1.0, node_table);
                node_table.insert(edge.index.clone(), node);
                if let Some(val) = rave_table.get_mut(&edge.action) {
                    val.n += 1.;
                    val.q += delta;
                } else {
                    let mut val = Value::new();
                    val.q = delta;
                    val.n = 1.;
                    rave_table.insert(edge.action.clone(), val);
                }
                return (q, n);
            }
            let mut child = node_table
                .remove(&edge.index)
                .expect("ERROR: Did not find child in iteration");
            game.make_move(&edge.action);
            let (delta, delta_n) = child.iteration(game, c, node_table, rave_table, false);
            q += delta_n - delta;
            n += delta_n;
            node_table.insert(edge.index, child);
            if let Some(val) = rave_table.get_mut(&edge.action) {
                val.n += delta_n;
                val.q += delta;
            } else {
                let mut val = Value::new();
                val.q = delta;
                val.n = delta_n;
                rave_table.insert(edge.action.clone(), val);
            }
            self.n_since_last_expansion += 1.;
            return (q, n);
        } else {
            panic!("wut");
        }
    }

    pub fn add_parent(&mut self, index: MinimalState) {
        self.parents.push(index);
    }

    #[allow(unused)]
    pub fn remove_parent(
        &mut self,
        index: MinimalState,
        node_table: &mut HashMap<MinimalState, TreeNode>,
    ) -> bool {
        let mut found = -1;
        for (i, parent) in self.parents.iter().enumerate() {
            if *parent == index {
                found = i as i32;
                break;
            }
        }
        if found >= 0 {
            self.parents.remove(found as usize);
        } else {
            return false;
        }
        if self.parents.len() == 0 {
            for child in self.children.iter_mut() {
                if let Some(mut node) = node_table.remove(&child.index) {
                    if node.remove_parent(self.index, node_table) == false {
                        node_table.insert(child.index.clone(), node);
                    }
                }
            }
            return true;
        }
        return false;
    }

    pub fn backpropagate(
        &mut self,
        q: f32,
        n: f32,
        node_table: &mut HashMap<MinimalState, TreeNode>,
    ) {
        self.stats = None;
        let value = q / n;
        let value = f32::min(1. - self.lower_bound, value);
        self.q += value * n;
        self.n += n;
        for parent in &self.parents {
            if let Some(mut node) = node_table.remove(parent) {
                node.backpropagate(n - q, n, node_table);
                node_table.insert(parent.clone(), node);
            }
        }
    }

    /// Recursively perform an MCTS iteration.
    pub fn iteration(
        &mut self,
        game: &mut Piranhas,
        c: f32,
        node_table: &mut HashMap<MinimalState, TreeNode>,
        rave_table: &mut HashMap<Move, Value>,
        is_root: bool,
    ) -> (f32, f32) {
        self.stats = None;
        let (delta, n) = match self.is_leaf {
            true => (self.q / self.n, 1.0),
            false => self.best_child_fpu(game, c, node_table, rave_table, is_root),
        };
        self.backpropagate(delta, n, node_table);
        return (delta, n);
    }
}

#[derive(Debug, Copy, Clone)]
/// Store and process some simple statistical information about NodeTrees.
pub struct TreeStatistics {
    pub nodes: i32,
    pub min_depth: i32,
    pub max_depth: i32,
}

impl TreeStatistics {
    pub fn filled(nodes: i32, max_depth: i32, min_depth: i32) -> TreeStatistics {
        return TreeStatistics {
            nodes,
            min_depth,
            max_depth,
        };
    }
}

#[derive(Clone, Debug)]
pub struct MCTS {
    root: MinimalState,
    game: Piranhas,
    pub iterations_per_s: f32,
    node_table: HashMap<MinimalState, TreeNode>,
    rave_table: HashMap<Move, Value>,
}

impl MCTS {
    /// Create a new MCTS solver.
    pub fn new(game: &Piranhas) -> MCTS {
        let color = game.get_color();
        let mut node_table = HashMap::with_capacity(100_000);
        let state = MinimalState::from_state(&game.state);
        node_table.insert(
            state,
            TreeNode::new(color, state.clone(), Some(MinimalState::empty())),
        );
        MCTS {
            root: state,
            game: game.clone(),
            iterations_per_s: 1.,
            node_table,
            rave_table: HashMap::with_capacity(100_000),
        }
    }

    #[allow(unused)]
    pub fn get_root_samples(&self) -> f32 {
        if let Some(node) = self.node_table.get(&self.root) {
            return node.n;
        }
        return 0.0;
    }

    #[allow(unused)]
    pub fn set_root(&mut self, game: &Piranhas) {
        let state = MinimalState::from_state(&game.state);
        if state == self.root {
            return;
        }
        let previous_root = self.root;
        self.game = game.clone();
        if let Some(mut node) = self.node_table.remove(&state) {
            node.add_parent(MinimalState::empty());
            if node.children.len() == 0 {
                node.is_leaf = false;
            }
            self.root = state;
            self.node_table.insert(state, node);
        } else {
            self.node_table = HashMap::with_capacity(100_000);
            let color = game.get_color();
            self.root = state;
            self.node_table.insert(
                state,
                TreeNode::new(color, state.clone(), Some(MinimalState::empty())),
            );
        }
        if previous_root != self.root {
            if let Some(mut node) = self.node_table.remove(&previous_root) {
                node.remove_parent(MinimalState::empty(), &mut self.node_table);
            }
        }
    }

    pub fn table_size(&self) -> usize {
        return self.node_table.len();
    }

    /// Return basic statistical data about the current MCTS tree.
    pub fn tree_statistics(&mut self) -> TreeStatistics {
        let mut root = self
            .node_table
            .remove(&self.root)
            .expect("ERROR: Did not find root for statistics");
        let (nodes, max_depth, min_depth) = root.tree_statistics(&mut self.node_table);
        self.node_table.insert(self.root, root);
        return TreeStatistics::filled(nodes, max_depth, min_depth);
    }

    /// Perform n_samples MCTS iterations.
    pub fn search(&mut self, n_samples: usize, c: f32) {
        for _ in 0..n_samples {
            let mut root = self
                .node_table
                .remove(&self.root)
                .expect("ERROR: Did not find root in search");
            let mut this_game = self.game.clone();
            root.iteration(
                &mut this_game,
                c,
                &mut self.node_table,
                &mut self.rave_table,
                true,
            );
            self.node_table.insert(self.root, root);
        }
    }

    /// Perform MCTS iterations for the given time budget (in s).
    #[allow(unused)]
    pub fn search_time(&mut self, budget_seconds: f32, c: f32) {
        let mut samples_total = 0;
        let t0 = time::now();

        let mut n_samples = 20;
        while n_samples > 19 {
            self.search(n_samples, c);
            samples_total += n_samples;

            let time_spend = (time::now() - t0).num_milliseconds() as f32 / 1000.;
            self.iterations_per_s = samples_total as f32 / time_spend;

            let time_left = budget_seconds - time_spend;
            n_samples = (self.iterations_per_s * time_left).max(0.).min(20.) as usize;
        }
    }

    /// Return the best action found so far.
    pub fn best_action(&self) -> (Option<Move>, f32, Option<u8>) {
        // Find best action
        let mut best_action: Option<Move> = None;
        let mut best_value: f32 = f32::NEG_INFINITY;
        let mut depth = None;
        let root = self
            .node_table
            .get(&self.root)
            .expect("ERROR: Did not find root for best action");
        for c in &root.children {
            if let Some(child) = self.node_table.get(&c.index) {
                let q = child.q;
                let n = child.n;
                let value = q / n;
                if child.is_leaf {
                    if value > 0.5 {
                        return (Some(c.action.clone()), value, child.depth);
                    }
                }
                if value >= best_value || best_action == None {
                    best_action = Some(c.action.clone());
                    best_value = value;
                    depth = child.depth;
                }
            }
        }
        return (best_action, best_value, depth);
    }

    pub fn get_pairs(&self) -> Vec<(f32, Move)> {
        let mut ret_val = Vec::new();

        let root = self
            .node_table
            .get(&self.root)
            .expect("ERROR: Did not find root for pairs");
        for c in &root.children {
            if let Some(child) = self.node_table.get(&c.index) {
                let value = child.n / root.n;
                ret_val.push((value, c.action.clone()));
            }
        }
        return ret_val;
    }
}
