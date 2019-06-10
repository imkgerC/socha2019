//! Primitives for determining the value / score of a specific location.
//!
//! A `Value` stores a single `i32` to represent a score. `Score` stores two `i32`s inside of it,
//! the first to determine the mid-game score, and the second to determine the end-game score.

// TODO: Why is Value an i32 now? Need some notes on why that changed.

/// Type for `i32` to determine the `Value` of an evaluation.
pub type Value = f32;

pub const ZERO: Value = 0.;
pub const DRAW: Value = 0.;
pub const MATE: Value = 31000.;
pub const INFINITE: Value = 32001.;
pub const NEG_INFINITE: Value = -32001.;
pub const NONE: Value = 32002.;

pub const MATE_IN_MAX_PLY: Value = MATE - 60.;
pub const MATED_IN_MAX_PLY: Value = -MATE + 60.;
