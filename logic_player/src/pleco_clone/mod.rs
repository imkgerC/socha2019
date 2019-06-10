mod board;
mod consts;
mod move_list;
mod movepick;
mod player;
mod root_moves_list;
mod score;
pub mod search;
mod tables;
mod tt;
pub use self::player::PlecoPlayer;
use game_sdk::{Direction, PlayerColor, Move};

/// Seed for the Zobrist's pseudo-random number generator.
/// Chosen through 32 fair coin flips
// const ZOBRIST_SEED: u64 = 23_081;
const ZOBRIST_SEED: u64 = 0b1101_0111__1111_0111__0000_1101__1001_1001__0110_1001__1100_0101__1010_0100__1111_1110;

/// Zobrist key for each piece on each square.
static mut ZOBRIST_PIECE_SQUARE: [[u64; consts::PLAYER_CNT]; consts::SQ_CNT] =
    [[0; consts::PLAYER_CNT]; consts::SQ_CNT];

/// Zobrist key for each possible en-passant capturable file.
const MAX_PLY: usize = consts::MAX_PLY as usize;
static mut ZOBRIST_PLY: [u64; MAX_PLY] = [0; MAX_PLY];

/// initialize the zobrist hash
#[cold]
pub fn init_zobrist() {
    let mut rng = PRNG::init(ZOBRIST_SEED);

    unsafe {
        for i in 0..consts::SQ_CNT {
            ZOBRIST_PIECE_SQUARE[i][0] = rng.rand();
            ZOBRIST_PIECE_SQUARE[i][1] = rng.rand();
        }

        for i in 0..MAX_PLY {
            ZOBRIST_PLY[i] = rng.rand()
        }
    }
}

#[inline(always)]
pub fn z_square(sq: SQ, color: PlayerColor) -> u64 {
    unsafe {
        *(*ZOBRIST_PIECE_SQUARE.get_unchecked(sq.0 as usize)).get_unchecked(color as usize)
    }
}

#[inline(always)]
pub fn z_turn(turn: u8) -> u64 {
    unsafe { *ZOBRIST_PLY.get_unchecked(turn as usize) }
}

pub struct PRNG {
    seed: u64,
}

impl PRNG {
    /// Creates PRNG from a seed.
    ///
    /// # Panics
    ///
    /// Undefined behavior if the seed is zero
    #[inline(always)]
    pub fn init(s: u64) -> PRNG {
        PRNG { seed: s }
    }

    /// Returns a pseudo-random number.
    #[allow(dead_code)]
    pub fn rand(&mut self) -> u64 {
        self.rand_change()
    }

    /// Randomizes the current seed and returns a random value.
    fn rand_change(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        self.seed.wrapping_mul(2685_8216_5773_6338_717)
    }
}

#[derive(Copy, Clone, Default, Hash, PartialEq, PartialOrd, Eq, Debug)]
#[repr(transparent)]
pub struct SQ(pub u8);

/// `SQ` representing no square available. Used internally to represent
/// the lack of an available en-passant square.
pub const NO_SQ: SQ = SQ(100);

impl SQ {
    /// A square that isn't on the board. Basically equivilant to `Option<SQ>` where the value is
    /// `None`.
    pub const NONE: SQ = NO_SQ;

    /// Returns if a `SQ` is within the legal bounds of a square,
    /// which is inclusively between 0 - 99.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::SQ;
    /// let sq_ok = SQ(5);
    /// let no_sq = SQ(100);
    ///
    /// assert!(sq_ok.is_okay());
    /// assert!(!no_sq.is_okay());
    /// ```
    #[inline(always)]
    pub const fn is_okay(self) -> bool {
        self.0 < 100
    }
}

// data is laid out as 7 bits each in that order: x + 10 * y, dest_x + 10 * dest_y
// direction is not saved and can't be regained from the move, beware
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct BitMove {
    data: u16,
}
const NULL_IDENTIFIER: u16 = 0b1000_0000_0000_0000;
impl BitMove {
    pub fn from_move(m: &Move) -> BitMove {
        let from = m.x as u16 + m.y as u16 * 10;
        let to = m.dest_x as u16 + m.dest_y as u16 * 10;
        let data = from | (to << 7) | NULL_IDENTIFIER;
        return BitMove { data };
    }

    pub fn from_indices(x: u8, y: u8, dest_x: u8, dest_y: u8) -> BitMove {
        let from = x as u16 + y as u16 * 10;
        let to = dest_x as u16 + dest_y as u16 * 10;
        let data = from | (to << 7) | NULL_IDENTIFIER;
        return BitMove { data };
    }

    pub fn to_partial_move(&self) -> Move {
        let data = self.data & !NULL_IDENTIFIER;
        let from = (data & 0b111_1111) as u8;
        let to = (data >> 7) as u8;
        let x = from % 10;
        let y = from / 10;
        let dest_x = to % 10;
        let dest_y = to / 10;
        return Move {
            x,
            y,
            dest_x,
            dest_y,
            direction: Direction::Up,
        };
    }

    #[inline]
    pub const fn from_to_key(&self) -> u16 {
        let data = self.data & !NULL_IDENTIFIER;
        let from = data & 0b111_1111;
        let to = data >> 7;
        return from + to * 100;
    }

    #[inline]
    pub const fn new(input: u16) -> BitMove {
        BitMove { data: input }
    }

    #[inline]
    pub const fn null() -> Self {
        BitMove { data: 0 }
    }

    /// Returns the raw number representation of the move.
    #[inline(always)]
    pub const fn get_raw(self) -> u16 {
        self.data
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.data == 0
    }

    pub fn to_string(&self) -> String {
        let data = self.data & !NULL_IDENTIFIER;
        let from = (data & 0b111_1111) as u8;
        let to = (data >> 7) as u8;
        let x = from % 10;
        let y = from / 10;
        let dest_x = to % 10;
        let dest_y = to / 10;
        return format!("{} {} to {} {}", x, y, dest_x, dest_y);
    }

    #[inline(always)]
    pub const fn get_dest(&self) -> SQ {
        let data = self.data & !NULL_IDENTIFIER;
        let to = data >> 7;
        SQ(to as u8)
    }

    #[inline(always)]
    pub const fn get_src(&self) -> SQ {
        let from = self.data & 0b111_1111;
        SQ(from as u8)
    }
}

/// Structure containing both a score (represented as a i16) and a `BitMove`.
///
/// This is useful for tracking a list of moves alongside each of their scores.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(C)]
pub struct ScoringMove {
    pub bit_move: BitMove,
    pub score: i16,
}

impl Default for ScoringMove {
    #[inline(always)]
    fn default() -> Self {
        ScoringMove {
            bit_move: BitMove::null(),
            score: 0,
        }
    }
}

impl ScoringMove {
    /// Creates a new `ScoringMove` with a default score of 0.
    #[inline(always)]
    pub fn new(m: BitMove) -> Self {
        ScoringMove {
            bit_move: m,
            score: 0,
        }
    }

    /// Creates a new `ScoringMove`.
    #[inline(always)]
    pub fn new_score(m: BitMove, score: i16) -> Self {
        ScoringMove { bit_move: m, score }
    }

    /// Returns a `ScoringMove` containing a `BitMove::null()` and a user-defined score.
    #[inline(always)]
    pub fn blank(score: i16) -> Self {
        ScoringMove {
            bit_move: BitMove::null(),
            score,
        }
    }

    /// Returns the move.
    #[inline(always)]
    pub fn bitmove(self) -> BitMove {
        self.bit_move
    }

    /// Returns the score.
    #[inline(always)]
    pub fn score(self) -> i16 {
        self.score
    }

    /// Negates the current score.
    #[inline(always)]
    pub fn negate(mut self) -> Self {
        self.score = self.score.wrapping_neg();
        self
    }

    /// Swaps the current move with another move.
    #[inline(always)]
    pub fn swap_move(mut self, mov: BitMove) -> Self {
        self.bit_move = mov;
        self
    }

    /// Returns a `ScoringMove` containing a `BitMove::null()` and a score of zero.
    #[inline(always)]
    pub const fn null() -> Self {
        ScoringMove {
            bit_move: BitMove::null(),
            score: 0,
        }
    }
}

// https://doc.rust-lang.org/core/arch/x86_64/fn._mm_prefetch.html
/// Allows an object to have it's entries pre-fetchable.
pub trait PreFetchable {
    /// Pre-fetches a particular key. This means bringing it into the cache for faster access.
    fn prefetch(&self, key: u64);
}

/// Prefetch's `ptr` to all levels of the cache.
///
/// For some platforms this may compile down to nothing, and be optimized away.
/// To prevent compiling down into nothing, compilation must be done for a
/// `x86` or `x86_64` platform with SSE instructions available. An easy way to
/// do this is to add the environmental variable `RUSTFLAGS=-C target-cpu=native`.
#[inline(always)]
pub fn prefetch_write<T>(ptr: *const T) {
    __prefetch_write::<T>(ptr);
}

#[cfg(feature = "nightly")]
#[inline(always)]
fn __prefetch_write<T>(ptr: *const T) {
    use std::intrinsics::prefetch_write_data;
    unsafe {
        prefetch_write_data::<T>(ptr, 3);
    }
}

#[cfg(all(
    not(feature = "nightly"),
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse"
))]
#[inline(always)]
fn __prefetch_write<T>(ptr: *const T) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::_mm_prefetch;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::_mm_prefetch;
    unsafe {
        _mm_prefetch(ptr as *const i8, 3);
    }
}

#[cfg(all(
    not(feature = "nightly"),
    any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            not(target_feature = "sse")
        ),
        not(any(target_arch = "x86", target_arch = "x86_64"))
    )
))]
#[inline(always)]
fn __prefetch_write<T>(ptr: *const T) {
    // Do nothing
}
