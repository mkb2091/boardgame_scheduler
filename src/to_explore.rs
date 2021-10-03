use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToExplore(u64);

impl ToExplore {
    pub const fn empty() -> Self {
        Self(0)
    }
    pub fn filled() -> Self {
        let mut to_explore = 0;
        for round in 0..ROUND_COUNT {
            for table in 0..TABLE_COUNT {
                to_explore |= Self::encode(round, table);
            }
        }
        Self(to_explore)
    }

    const fn encode(round: usize, table: usize) -> u64 {
        1 << ((round << TO_EXPLORE_SHIFT) + table)
    }

    const fn decode(trailing_zeros: u32) -> (usize, usize) {
        let round = trailing_zeros >> TO_EXPLORE_SHIFT;
        let table = trailing_zeros - (round << TO_EXPLORE_SHIFT);
        let round = round as usize;
        let table = table as usize;
        (round, table)
    }

    pub fn pop(&mut self) -> Option<(usize, usize)> {
        if self.0 == 0 {
            None
        } else {
            let trailing_zeros = self.0.trailing_zeros();
            self.0 &= !(1 << trailing_zeros);
            let (round, table) = Self::decode(trailing_zeros);
            if table >= TABLE_COUNT || round >= ROUND_COUNT {
                unreachable!();
            }
            Some((round, table))
        }
    }
    pub fn remove(&mut self, round: usize, table: usize) {
        self.0 &= !Self::encode(round, table);
    }
    pub fn add(&mut self, round: usize, table: usize) {
        self.0 |= Self::encode(round, table);
    }
}

impl core::ops::BitAnd for ToExplore {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for ToExplore {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl core::ops::BitOr for ToExplore {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for ToExplore {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl core::ops::Not for ToExplore {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
