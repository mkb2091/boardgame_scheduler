use crate::*;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToExplore(u64);

impl ToExplore {
    pub const fn empty() -> Self {
        Self(0)
    }
    pub fn filled() -> Self {
        let mut to_explore = 0;
        for round in ROUNDS {
            for table in TABLES {
                to_explore |= Self::encode(round, table);
            }
        }
        Self(to_explore)
    }

    const fn encode(round: Round, table: Table) -> u64 {
        1 << (((round as usize) << TO_EXPLORE_SHIFT) + table as usize)
    }

    fn decode(trailing_zeros: u32) -> (Round, Table) {
        let round = trailing_zeros >> TO_EXPLORE_SHIFT;
        let table = trailing_zeros - (round << TO_EXPLORE_SHIFT);
        let round = (round as usize).try_into().unwrap();
        let table = (table as usize).try_into().unwrap();
        (round, table)
    }

    pub fn pop(&mut self) -> Option<(Round, Table)> {
        if self.0 == 0 {
            None
        } else {
            let trailing_zeros = self.0.trailing_zeros();
            self.0 &= !(1 << trailing_zeros);
            let (round, table) = Self::decode(trailing_zeros);
            if table as usize >= TABLE_COUNT || round as usize >= ROUND_COUNT {
                unreachable!();
            }
            Some((round, table))
        }
    }
    pub fn remove(&mut self, round: Round, table: Table) {
        self.0 &= !Self::encode(round, table);
    }
    pub fn add(&mut self, round: Round, table: Table) {
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
