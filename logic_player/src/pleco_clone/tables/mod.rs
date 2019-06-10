pub mod butterfly;
pub mod continuation;
pub mod counter_move;

use std::mem;
use std::ops::*;
use std::ptr;

pub mod prelude {
    // easier exporting :)
    pub use super::butterfly::ButterflyHistory;
    pub use super::continuation::{ContinuationHistory, PieceToHistory};
    pub use super::counter_move::CounterMoveHistory;
    pub use super::{NumStatBoard, NumStatCube, StatBoard};
}

pub trait StatBoard<T, IDX>: Sized + IndexMut<IDX, Output = T>
where
    T: Copy + Clone + Sized,
{
    const FILL: T;

    fn new() -> Self {
        unsafe { mem::zeroed() }
    }

    fn clear(&mut self) {
        self.fill(Self::FILL);
    }

    fn fill(&mut self, val: T) {
        let num: usize = mem::size_of::<Self>() / mem::size_of::<T>();

        unsafe {
            let ptr: *mut T = self as *mut Self as *mut T;
            for i in 0..num {
                ptr::write(ptr.add(i), val);
            }
        }
    }
}

pub trait NumStatBoard<IDX>: StatBoard<i16, IDX> {
    const D: i16;
    fn update(&mut self, idx: IDX, bonus: i16) {
        assert!(bonus.abs() <= Self::D); // Ensure range is [-32 * D, 32 * D]
        let entry = self.index_mut(idx);
        *entry += bonus * 32 - (*entry) * bonus.abs() / Self::D;
    }
}

pub trait NumStatCube<IDX>: StatBoard<i16, IDX> {
    const D: i32;
    const W: i32;

    fn update(&mut self, idx: IDX, bonus: i32) {
        assert!(bonus.abs() <= Self::D);
        let entry = self.index_mut(idx);
        let before = (*entry) as i32;
        let new = (before + bonus * Self::W - before * bonus.abs() / Self::D) as i16;
        *entry = new;
        assert!(((*entry) as i32).abs() <= Self::D * Self::W);
    }
}
