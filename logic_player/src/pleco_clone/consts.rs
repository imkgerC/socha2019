//! Constant values and static structures.
use std::mem;
use std::ptr;
use std::sync::{ONCE_INIT,Once};

use super::tt::TranspositionTable;
use super::init_zobrist;

pub const MAX_PLY: u16 = 61;
pub const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;

pub const DEFAULT_TT_SIZE: usize = 128;

pub const SQ_CNT: usize = 100;
pub const PLAYER_CNT: usize = 2;
pub const PIECE_CNT: usize = 4;

const TT_ALLOC_SIZE: usize = mem::size_of::<TranspositionTable>();

// A object that is the same size as a transposition table
type DummyTranspositionTable = [u8; TT_ALLOC_SIZE];

static INITALIZED: Once = ONCE_INIT;

/// Global Transposition Table
static mut TT_TABLE: DummyTranspositionTable = [0; TT_ALLOC_SIZE];

#[cold]
pub fn init_globals() {
    INITALIZED.call_once(|| {
        init_zobrist();   // Initialize static tables
        init_tt();                 // Transposition Table
    });
}

// Initializes the transposition table
#[cold]
fn init_tt() {
    unsafe {
        let tt = &mut TT_TABLE as *mut DummyTranspositionTable as *mut TranspositionTable;
        ptr::write(tt, TranspositionTable::new(DEFAULT_TT_SIZE));
    }
}

/// Returns access to the global transposition table
#[inline(always)]
pub fn tt() -> &'static TranspositionTable {
    unsafe {
        &*(&TT_TABLE as *const DummyTranspositionTable as *const TranspositionTable)
    }
}


pub trait PVNode {
    fn is_pv() -> bool;
}

pub struct PV {}
pub struct NonPV {}

impl PVNode for PV {
    fn is_pv() -> bool {
        true
    }
}

impl PVNode for NonPV {
    fn is_pv() -> bool {
        false
    }
}