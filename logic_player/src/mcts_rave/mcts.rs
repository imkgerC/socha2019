use super::piranhas::{MinimalState, Piranhas};
use game_sdk::Move;
use game_sdk::PlayerColor;
use hashbrown::HashMap;
use std::cmp::{max, min};
use std::f32;
use std::i32;

use crate::LogicBasedPlayer;

fn varianced_playout(initial: &Piranhas, color: &PlayerColor) -> f32 {
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NodeState {
    LeafNode,
    FullyExpanded,
    Expandable,
}

impl std::fmt::Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NodeState::LeafNode => "LeafNode",
                NodeState::FullyExpanded => "FullyExpanded",
                NodeState::Expandable => "Expandable",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    index: MinimalState,
    parents: Vec<MinimalState>,
    children: Vec<(MinimalState, Move, bool)>, // next steps we investigated
    state: NodeState,                          // is this a leaf node? fully expanded?
    color: PlayerColor,
    n: f32,
    q: f32, // statistics for this game state
    lower_bound: f32,
    depth: Option<u8>,
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
            state: NodeState::Expandable,
            color: color,
            n: 0.,
            q: 0.,
            lower_bound: -2.0,
            depth: None,
        }
    }

    /// Gather some statistics about this subtree
    pub fn tree_statistics(&self, node_table: &HashMap<MinimalState, TreeNode>) -> TreeStatistics {
        let child_stats = self
            .children
            .iter()
            .map(|(c, _, _)| {
                if let Some(c) = node_table.get(&c) {
                    c.tree_statistics(node_table)
                } else {
                    TreeStatistics::empty()
                }
            })
            .collect::<Vec<_>>();
        TreeStatistics::merge(child_stats)
    }

    /// Find the best child accoring to UCT1
    pub fn best_child(
        &mut self,
        c: f32,
        node_table: &HashMap<MinimalState, TreeNode>,
    ) -> Option<(MinimalState, Move)> {
        let mut best_value: f32 = f32::NEG_INFINITY;
        let mut best_child: Option<(MinimalState, Move)> = None;
        let c_base = 19652.;
        let c = c + ((1. + self.n + c_base) / c_base).ln();

        let mut non_terminal = 0;
        for (child_idx, action, _) in &self.children {
            let child = node_table
                .get(child_idx)
                .expect("ERROR: Did not find child in UCT");

            if child.state == NodeState::LeafNode {
                let value = child.q / child.n;
                self.lower_bound = f32::max(self.lower_bound, value);
                if value > 0.5 {
                    self.state = NodeState::LeafNode;
                    self.q = (1. - value) * self.n;
                    self.depth = Some(child.depth.unwrap_or(0) + 1);
                    return Some((*child_idx, *action));
                }
            } else {
                non_terminal += 1;
                let value = child.q / child.n + c * (2. * self.n.ln() / child.n).sqrt();
                if value >= best_value || best_child == None {
                    best_value = value;
                    best_child = Some((*child_idx, *action));
                }
            }
        }
        if non_terminal == 0 {
            self.state = NodeState::LeafNode;
            best_value = self.lower_bound;
            best_child = None;
            for (child_idx, action, _) in &self.children {
                let child = node_table
                    .get(child_idx)
                    .expect("ERROR: Did not find child in UCT");
                let value = child.q / child.n;
                if value >= best_value || best_child == None {
                    best_value = value;
                    best_child = Some((*child_idx, *action));
                    self.depth = Some(child.depth.unwrap_or(0) + 1);
                }
            }
            self.q = (1. - best_value) * self.n;
        }
        return best_child;
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
            for (child, _, _) in self.children.iter_mut() {
                if let Some(mut node) = node_table.remove(child) {
                    if node.remove_parent(self.index, node_table) == false {
                        node_table.insert(child.clone(), node);
                    }
                }
            }
            return true;
        }
        return false;
    }

    pub fn expand_evaluate(
        &mut self,
        game: &mut Piranhas,
        c: f32,
        node_table: &mut HashMap<MinimalState, TreeNode>,
    ) -> (f32, f32) {
        let color = game.get_color();
        if self.children.len() == 0 {
            let allowed_actions = game.allowed_actions();
            if allowed_actions.len() == 0 {
                self.state = NodeState::LeafNode;
                self.depth = Some(0);
                return (game.reward(&color), 1.0);
            }
            for action in allowed_actions {
                let mut game_clone = game.clone();
                game_clone.make_move(&action);
                let state = MinimalState::from_state(&game_clone.state);
                self.children.push((state, action, false));
            }
        }
        let mut candidate_indices = Vec::new();
        let mut n = 0.0;
        let mut q = 0.0;
        for (i, (child_idx, _, added)) in self.children.iter_mut().enumerate() {
            if *added {
                continue;
            }
            if let Some(mut node) = node_table.get_mut(child_idx) {
                node.add_parent(self.index);
                n += node.n;
                q += node.n - node.q;
                *added = true;
            } else {
                candidate_indices.push(i);
            }
        }
        // self.backpropagate(q, n, node_table);
        if candidate_indices.len() == 1 {
            self.state = NodeState::FullyExpanded;
        }
        if candidate_indices.len() == 0 {
            self.state = NodeState::FullyExpanded;
            // Choose and recurse into child...
            let (child_idx, action) = self
                .best_child(c, node_table)
                .expect(&format!("Did not find best child in expansion"));
            let mut child = node_table
                .remove(&child_idx)
                .expect("ERROR: Did not find child in iteration");
            game.make_move(&action);
            let (delta, delta_n) = child.iteration(game, c, node_table);
            q += delta_n - delta;
            n += delta_n;
            node_table.insert(child_idx, child);
            return (q, n);
        }
        let index = *candidate_indices.get(0).expect("Should never happen, is checked");// *choose_random(&candidate_indices);
        self.children
            .get_mut(index)
            .expect("ERROR: Did not find child for flagging")
            .2 = true;
        let (state, action, _) = self
            .children
            .get(index)
            .expect("ERROR: Did not find child in expansion");
        let mut node = TreeNode::new(
            self.color.get_opponent_color(),
            state.clone(),
            Some(self.index),
        );

        game.make_move(&action);
        let delta = varianced_playout(game, &node.color);
        n += 1.0;
        q += 1.0 - delta;
        node.backpropagate(delta, 1.0, node_table);
        node_table.insert(state.clone(), node);
        return (q, n);
    }

    pub fn backpropagate(
        &mut self,
        q: f32,
        n: f32,
        node_table: &mut HashMap<MinimalState, TreeNode>,
    ) {
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
    ///
    /// XXX A non-recursive implementation would probably be faster.
    /// XXX But how to keep &mut pointers to all our parents while
    /// XXX we fiddle with our leaf node?
    pub fn iteration(
        &mut self,
        game: &mut Piranhas,
        c: f32,
        node_table: &mut HashMap<MinimalState, TreeNode>,
    ) -> (f32, f32) {
        let (delta, n) = match self.state {
            // NodeState::LeafNode => (game.reward(&self.color), 1.0),
            NodeState::LeafNode => (self.q / self.n, 1.0),
            NodeState::FullyExpanded => {
                // Choose and recurse into child...
                let (child_idx, action) = self.best_child(c, node_table).expect(&format!(
                    "Did not find best child, len of childs {}",
                    self.children.len()
                ));
                let mut child = node_table
                    .remove(&child_idx)
                    .expect("ERROR: Did not find child in iteration");
                game.make_move(&action);
                let (q, n) = child.iteration(game, c, node_table);
                node_table.insert(child_idx, child);
                (n - q, n)
            }
            NodeState::Expandable => self.expand_evaluate(game, c, node_table),
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
    pub fn empty() -> TreeStatistics {
        return TreeStatistics {
            nodes: 1,
            min_depth: 0,
            max_depth: 0,
        };
    }
    fn merge(child_stats: Vec<TreeStatistics>) -> TreeStatistics {
        if child_stats.len() == 0 {
            TreeStatistics {
                nodes: 1,
                min_depth: 0,
                max_depth: 0,
            }
        } else {
            TreeStatistics {
                nodes: child_stats.iter().fold(0, |sum, child| sum + child.nodes),
                min_depth: 1 + child_stats
                    .iter()
                    .fold(i32::MAX, |depth, child| min(depth, child.min_depth)),
                max_depth: 1 + child_stats
                    .iter()
                    .fold(0, |depth, child| max(depth, child.max_depth)),
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MCTS {
    root: MinimalState,
    game: Piranhas,
    pub iterations_per_s: f32,
    node_table: HashMap<MinimalState, TreeNode>,
}

impl MCTS {
    /// Create a new MCTS solver.
    pub fn new(game: &Piranhas) -> MCTS {
        let color = game.get_color();
        let mut node_table = HashMap::new();
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
                node.state == NodeState::Expandable;
            }
            self.root = state;
            self.node_table.insert(state, node);
        } else {
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
    pub fn tree_statistics(&self) -> TreeStatistics {
        let root = self
            .node_table
            .get(&self.root)
            .expect("ERROR: Did not find root for statistics");
        let child_stats = root.tree_statistics(&self.node_table);
        return TreeStatistics::merge(vec![child_stats]);
    }

    /// Perform n_samples MCTS iterations.
    pub fn search(&mut self, n_samples: usize, c: f32) {
        for _ in 0..n_samples {
            let mut root = self
                .node_table
                .remove(&self.root)
                .expect("ERROR: Did not find root in search");
            let mut this_game = self.game.clone();
            root.iteration(&mut this_game, c, &mut self.node_table);
            self.node_table.insert(self.root, root);
        }
    }

    /// Perform MCTS iterations for the given time budget (in s).
    #[allow(unused)]
    pub fn search_time(&mut self, budget_seconds: f32, c: f32) {
        let mut samples_total = 0;
        let t0 = time::now();

        let mut n_samples = 10; // (self.iterations_per_s * budget_seconds).max(10.).min(100.) as usize;
        while n_samples > 9 {
            self.search(n_samples, c);
            samples_total += n_samples;

            let time_spend = (time::now() - t0).num_milliseconds() as f32 / 1000.;
            self.iterations_per_s = samples_total as f32 / time_spend;

            let time_left = budget_seconds - time_spend;
            n_samples = (self.iterations_per_s * time_left).max(0.).min(10.) as usize;
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
        for (child_idx, action, _) in &root.children {
            if let Some(child) = self.node_table.get(child_idx) {
                let q = child.q;
                let n = child.n;
                let value = q / n;
                if child.state == NodeState::LeafNode {
                    if value > 0.5 {
                        return (Some(action.clone()), value, child.depth);
                    }
                }
                if value >= best_value || best_action == None {
                    best_action = Some(action.clone());
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
        for (child_idx, action, _) in &root.children {
            if let Some(child) = self.node_table.get(child_idx) {
                let value = child.n / root.n;
                ret_val.push((value, action.clone()));
            }
        }
        return ret_val;
    }
}
