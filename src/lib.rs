mod to_explore;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use to_explore::ToExplore;

const ROUND_COUNT: usize = 6;
const TABLE_COUNT: usize = 6;
const PLAYERS_PER_TABLE: usize = 4;
const PLAYER_COUNT: usize = TABLE_COUNT * PLAYERS_PER_TABLE;
const PLAYER_MASK: u32 = (1 << PLAYER_COUNT) - 1;

const TABLES: [Table; 6] = [
    Table::Zero,
    Table::One,
    Table::Two,
    Table::Three,
    Table::Four,
    Table::Five,
];

const ROUNDS: [Round; 6] = [
    Round::Zero,
    Round::One,
    Round::Two,
    Round::Three,
    Round::Four,
    Round::Five,
];

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum Table {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

impl TryFrom<usize> for Table {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum Round {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

impl TryFrom<usize> for Round {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DF2 {
    players_placed: u16,
    round: Round,
    table: Table,
    player_number: usize,
    played_in_round: [u32; ROUND_COUNT],
    played_on_table_total: [u32; TABLE_COUNT],
    schedule: [[[u8; 4]; TABLE_COUNT]; ROUND_COUNT],
    removed: [[u32; TABLE_COUNT]; ROUND_COUNT],
    players_played_with: [u32; 32],
}

impl DF2 {
    pub fn new() -> Self {
        let mut new = Self {
            players_placed: 0,
            round: Round::Zero,
            table: Table::Zero,
            player_number: 0,
            played_in_round: [0; ROUND_COUNT],
            played_on_table_total: [0; ROUND_COUNT],
            schedule: [[[PLAYER_COUNT as u8; 4]; ROUND_COUNT]; TABLE_COUNT],
            removed: [[0; TABLE_COUNT]; ROUND_COUNT],
            players_played_with: [0; 32],
        };
        for player in 0..PLAYER_COUNT as u8 {
            new.apply_player(player);
            new.increment().unwrap();
        }
        new.decrement().unwrap();

        new
    }
    pub fn from_slice(players: &[u8]) -> Result<Self, ()> {
        let mut df = Self::new();
        for player in players.iter() {
            df.increment()?;
            if df.get_mask(df.round, df.table) & (1 << player) == 0 {
                return Err(());
            }
            df.apply_player(*player);
        }

        Ok(df)
    }
    fn toggle_player(&mut self, player: u8) {
        assert!(player < PLAYER_COUNT as u8);
        let round = self.round;
        let table = self.table;
        let player_mask: u32 = 1 << player;
        self.played_in_round[round as usize] ^= player_mask;
        self.played_on_table_total[table as usize] ^= player_mask;
        let players_in_round = self.schedule[round as usize][table as usize];
        self.players_played_with[players_in_round[0] as usize & 31] ^= player_mask;
        self.players_played_with[players_in_round[1] as usize & 31] ^= player_mask;
        self.players_played_with[players_in_round[2] as usize & 31] ^= player_mask;
        self.players_played_with[players_in_round[3] as usize & 31] ^= player_mask;
        self.players_played_with[PLAYER_COUNT] = 0;
    }
    fn apply_player(&mut self, player: u8) {
        assert!(player < PLAYER_COUNT as u8);
        debug_assert_ne!(self.get_mask(self.round, self.table) & (1 << player), 0);
        self.removed[self.round as usize][self.table as usize] |= 1 << player; // Only change that is not removed by remove_player
        self.schedule[self.round as usize][self.table as usize][self.player_number as usize] = player as u8;
        log::trace!("placing player {} into {:?}", player, self.schedule[self.round as usize][self.table as usize]);
        self.toggle_player(player);
    }
    fn last_player(&self) -> u8 {
        self.schedule[self.round as usize][self.table as usize][self.player_number]
    }
    fn remove_last_player(&mut self) {
        let player = self.last_player();
        assert!(player < PLAYER_COUNT as u8);
        self.schedule[self.round as usize][self.table as usize][self.player_number] =
            PLAYER_COUNT as u8;
        self.toggle_player(player);
    }
    const fn get_mask(&self, round: Round, table: Table) -> u32 {
        /* TODO
        - First person of each round must be greater than last, otherwise waste time on multiple identical solutions
        */

        let players_in_round = self.schedule[round as usize][table as usize];

        PLAYER_MASK
            & !self.played_in_round[round as usize]
            & !self.played_on_table_total[table as usize]
            & !self.removed[round as usize][table as usize]
            & !self.players_played_with[players_in_round[0] as usize & 31]
            & !self.players_played_with[players_in_round[1] as usize & 31]
            & !self.players_played_with[players_in_round[2] as usize & 31]
            & !self.players_played_with[players_in_round[3] as usize & 31]
    }
    pub const fn get_players_placed(&self) -> u16 {
        self.players_placed
    }
    fn increment(&mut self) -> Result<(), ()> {
        self.players_placed += 1;
        self.player_number += 1;
        if self.player_number >= PLAYERS_PER_TABLE {
            if let Ok(table) = Table::try_from(self.table as usize + 1) {
                self.table = table;
            } else if let Ok(round) = Round::try_from(self.round as usize + 1) {
                self.round = round;
                self.table = Table::Zero;
            } else {
                return Err(());
            }
            self.player_number = 0;
        }
        Ok(())
    }
    fn decrement(&mut self) -> Result<(), ()> {
        self.players_placed -= 1;
        if self.player_number == 0 {
            self.removed[self.round as usize][self.table as usize] = 0;
            if let Ok(table) = Table::try_from((self.table as usize).wrapping_sub(1)) {
                self.table = table;
            } else if let Ok(round) = Round::try_from((self.round as usize).wrapping_sub(1)) {
                self.round = round;
                self.table = Table::Five;
            } else {
                return Err(());
            }
            self.player_number = PLAYERS_PER_TABLE - 1;
        } else {
            self.player_number -= 1
        }
        Ok(())
    }
    pub fn step(&mut self) -> Result<(), ()> {
        self.increment()?;

        let mut mask = self.get_mask(self.round, self.table);
        while mask == 0 {
            assert_eq!(self.last_player(), PLAYER_COUNT as u8);
            
            self.decrement()?;
            assert_ne!(self.last_player(), PLAYER_COUNT as u8);
            self.remove_last_player();
            mask = self.get_mask(self.round, self.table);
        }
        let player = mask.trailing_zeros() as u8;
        self.apply_player(player);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .is_test(true)
            .try_init();
    }

    fn expand_bitvec(mut x: u32) -> Vec<u8> {
        let mut out = Vec::new();
        while x != 0 {
            let zeros = x.trailing_zeros() as u8;
            x ^= 1 << zeros;
            out.push(zeros);
        }

        out
    }

    #[test]
    fn test() {
        let state = DF2::new();
        let mask = expand_bitvec(state.get_mask(Round::One, Table::Zero));
        assert_eq!(
            mask,
            vec![4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
        );
    }
    #[test]
    fn test2() {
        let state = DF2::from_slice(&[4, 8, 12, 16]).unwrap();
        let mask = expand_bitvec(state.get_mask(Round::One, Table::One));
        assert_eq!(
            mask,
            vec![0, 1, 2, 3, 9, 10, 11, 13, 14, 15, 17, 18, 19, 20, 21, 22, 23]
        );
    }
    #[test]
    fn test3() {
        let state = DF2::from_slice(&[4, 8, 12, 16, 0, 9, 13, 17]).unwrap();
        let mask = expand_bitvec(state.get_mask(Round::One, Table::Two));
        assert_eq!(mask, vec![1, 2, 3, 5, 6, 7, 14, 15, 18, 19, 20, 21, 22, 23]);
    }
    #[test]
    fn test4() {
        let state = DF2::from_slice(&[4, 8, 12, 16, 0, 9, 13, 17, 1, 5, 14, 18, 20]).unwrap();
        let mask = expand_bitvec(state.get_mask(Round::One, Table::Two));
        assert_eq!(mask, vec![21, 22, 23]);
    }
    #[test]
    fn run_successful() {
        init();
        let mut state = DF2::from_slice(&[
            4, 8, 12, 16, 0, 9, 13, 20, 1, 5, 17, 21, 2, 6, 18, 22, 3, 10, 14, 23, 7, 11, 15, 19,
            5, 9, 14, 18, 3, 15, 16, 22, 2, 7, 12, 20, 1, 8, 19, 23, 6, 11, 13, 21, 0, 4, 10, 17,
            6, 10, 19, 20, 2, 11, 14, 17, 4, 15, 18, 23, 0, 7, 16, 21, 1, 9, 12, 22, 3, 5, 8, 13,
            7, 13, 17, 23, 10, 12, 18, 21, 0, 14, 19, 22, 3, 4, 11,
        ])
        .unwrap();
        let mask = expand_bitvec(state.get_mask(state.round, state.table));
        assert_eq!(mask, vec![9, 20]);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct State {
    tables_to_explore: ToExplore,
    players_played_count: u8,
    empty_table_count: u8,
    players_played_with: [u32; PLAYER_COUNT],
    played_in_round: [u32; ROUND_COUNT],
    played_on_table: [[u32; TABLE_COUNT]; ROUND_COUNT],
    potential_on_table: [[u32; TABLE_COUNT]; ROUND_COUNT],
    played_on_table_total: [u32; TABLE_COUNT],
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_schedule(f)
    }
}

impl State {
    pub fn new() -> Self {
        let potential_on_table = [[PLAYER_MASK; TABLE_COUNT]; ROUND_COUNT];
        let mut state = Self {
            tables_to_explore: ToExplore::filled(),
            players_played_count: 0,
            empty_table_count: (ROUND_COUNT * TABLE_COUNT) as u8,
            players_played_with: [0; PLAYER_COUNT],
            played_in_round: [0; ROUND_COUNT],
            played_on_table: [[0; TABLE_COUNT]; ROUND_COUNT],
            potential_on_table,
            played_on_table_total: [0; TABLE_COUNT],
        };
        let mut player = 0;
        for table in TABLES {
            for _ in 0..PLAYERS_PER_TABLE {
                state.apply_player(Round::Zero, table, player);
                player += 1;
            }
        }
        //state.apply_player(Round::One, Table::Zero, 4);
        //state.apply_player(Round::One, Table::Zero, 8);
        //state.apply_player(Round::One, Table::Zero, 12);
        //state.apply_player(Round::One, Table::Zero, 16);
        state
    }

    fn can_play_with_players_in_game(&self, round: Round, table: Table, player: usize) -> bool {
        self.players_played_with[player] & self.played_on_table[round as usize][table as usize] == 0
    }

    fn game_full(&mut self, round: Round, table: Table) {
        self.empty_table_count = self.empty_table_count.checked_sub(1).unwrap();
        self.tables_to_explore.remove(round, table);
        self.potential_on_table[round as usize][table as usize] =
            self.played_on_table[round as usize][table as usize];
        if table == Table::Zero {
            let lowest_player =
                self.played_on_table[round as usize][table as usize].trailing_ones();
            let mask = !((1 << lowest_player) - 1);
            for round in (round as usize + 1)..ROUND_COUNT {
                self.potential_on_table[round as usize][table as usize] &= mask;
            }
        }
    }

    fn apply_player(&mut self, round: Round, table: Table, player: usize) {
        if round as usize >= ROUND_COUNT || table as usize >= TABLE_COUNT || player >= PLAYER_COUNT
        {
            unreachable!();
        }

        debug_assert!(self.can_play_with_players_in_game(round, table, player));

        self.players_played_count += 1;
        let player_mask: u32 = 1 << player;
        let remove_player_mask: u32 = !player_mask;
        for ptr in self.potential_on_table.iter_mut() {
            // Remove player from the table in other rounds
            ptr[table as usize] &= remove_player_mask;
        }
        for ptr in self.potential_on_table[round as usize].iter_mut() {
            // Remove player from other tables in the same round
            *ptr &= remove_player_mask;
        }

        // Add player to played in round
        debug_assert_eq!(self.played_in_round[round as usize] & player_mask, 0);
        self.played_in_round[round as usize] |= player_mask;
        // Add player to played on table
        debug_assert_eq!(self.played_on_table_total[table as usize] & player_mask, 0);
        self.played_on_table_total[table as usize] |= player_mask;

        let mut other_players = self.played_on_table[round as usize][table as usize];
        debug_assert_eq!(other_players & player_mask, 0);
        // Remove players current player has previously played with from tables potential
        self.potential_on_table[round as usize][table as usize] &=
            !self.players_played_with[player];
        // Add other players on table to current players played with list
        self.players_played_with[player] |= other_players;
        while other_players != 0 {
            let other_player = other_players.trailing_zeros() as usize;
            other_players &= !(1 << other_player);
            // Add current player to each other players played with list
            self.players_played_with[other_player] |= player_mask;
        }

        debug_assert_eq!(
            self.potential_on_table[round as usize][table as usize] & player_mask,
            0
        );
        self.potential_on_table[round as usize][table as usize] |= player_mask;
        debug_assert_eq!(
            self.played_on_table[round as usize][table as usize] & player_mask,
            0
        );
        self.played_on_table[round as usize][table as usize] |= player_mask;
        if self.played_on_table[round as usize][table as usize].count_ones()
            == PLAYERS_PER_TABLE as u32
        {
            self.game_full(round, table);
        }
        debug_assert!(
            self.played_on_table[round as usize][table as usize].count_ones()
                <= PLAYERS_PER_TABLE as u32
        );
    }

    pub fn get_available_count(&self) -> u32 {
        let mut total = 0;
        for round in ROUNDS {
            for table in TABLES {
                total += self.potential_on_table[round as usize][table as usize].count_ones();
            }
        }
        total
    }

    pub fn get_players_played_count(&self) -> u8 {
        self.players_played_count
    }

    pub fn find_hidden_singles(&mut self) -> Result<(), ()> {
        for round in ROUNDS {
            let mut potential_in_row = !self.played_in_round[round as usize];
            'loop_bits_round: while potential_in_row != 0 {
                let player = potential_in_row.trailing_zeros() as usize;
                if player >= PLAYER_COUNT {
                    break;
                }
                let player_bit: u32 = 1 << player;
                potential_in_row &= !player_bit;
                let mut only_position = None;
                for table in TABLES {
                    if self.potential_on_table[round as usize][table as usize] & player_bit != 0 {
                        if self.can_play_with_players_in_game(round, table, player) {
                            if only_position.is_none() {
                                only_position = Some(table);
                            } else {
                                continue 'loop_bits_round;
                            }
                        } else {
                            self.potential_on_table[round as usize][table as usize] &= !player_bit;
                        }
                    }
                }
                if let Some(table) = only_position {
                    self.apply_player(round, table, player);
                } else {
                    // No game in round can have player
                    return Err(());
                }
            }
        }

        for table in TABLES {
            let mut potential_in_column = !self.played_on_table_total[table as usize];
            'loop_bits_table: while potential_in_column != 0 {
                let player = potential_in_column.trailing_zeros() as usize;
                if player >= PLAYER_COUNT {
                    break;
                }
                let player_bit = 1 << player;
                potential_in_column &= !player_bit;
                let mut only_position = None;
                for round in ROUNDS {
                    if self.potential_on_table[round as usize][table as usize] & player_bit != 0 {
                        if self.can_play_with_players_in_game(round, table, player) {
                            if only_position.is_none() {
                                only_position = Some(round);
                            } else {
                                continue 'loop_bits_table;
                            }
                        } else {
                            self.potential_on_table[round as usize][table as usize] &= !player_bit;
                        }
                    }
                }
                if let Some(round) = only_position {
                    self.apply_player(round, table, player);
                } else {
                    // No game on table can have player
                    return Err(());
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub fn step(&mut self, state2: &mut Self) -> Result<Option<()>, ()> {
        //self.find_hidden_singles()?;

        let mut lowest: Option<(u8, Round, Table)> = None;
        let mut to_explore = self.tables_to_explore;
        while let Some((round, table)) = to_explore.pop() {
            let fixed_player_count =
                self.played_on_table[round as usize][table as usize].count_ones() as u8;
            match fixed_player_count.cmp(&(PLAYERS_PER_TABLE as u8)) {
                core::cmp::Ordering::Less => {
                    let potential = self.potential_on_table[round as usize][table as usize];
                    let potential_count = potential.count_ones() as u8;
                    match potential_count.cmp(&(PLAYERS_PER_TABLE as u8)) {
                        core::cmp::Ordering::Greater => {
                            lowest = Some(if let Some(lowest) = lowest {
                                if potential_count < lowest.0 {
                                    (potential_count, round, table)
                                } else {
                                    lowest
                                }
                            } else {
                                (fixed_player_count, round, table)
                            });
                        }
                        core::cmp::Ordering::Equal => {
                            let mut potential =
                                potential & !self.played_on_table[round as usize][table as usize];
                            while potential != 0 {
                                let player = potential.trailing_zeros() as usize;
                                potential &= !(1 << player);
                                if self.can_play_with_players_in_game(round, table, player) {
                                    self.apply_player(round, table, player);
                                } else {
                                    // Cannot fill game
                                    return Err(());
                                }
                            }
                        }
                        core::cmp::Ordering::Less => {
                            // Not enough potential to fill game
                            return Err(());
                        }
                    }
                }
                core::cmp::Ordering::Equal => {
                    self.game_full(round, table);
                    continue;
                }
                core::cmp::Ordering::Greater => {
                    unreachable!(); // Shouldn't be possible
                    return Err(());
                }
            }
        }
        if let Some((_, round, table)) = lowest {
            let potential = self.potential_on_table[round as usize][table as usize]
                & !self.played_on_table[round as usize][table as usize];
            let mut temp = potential;
            'played_iter: while temp != 0 {
                let player = temp.trailing_zeros() as usize;
                let player_bit = 1 << player;
                temp &= !player_bit;
                if self.can_play_with_players_in_game(round, table, player) {
                    *state2 = *self;
                    self.potential_on_table[round as usize][table as usize] &= !player_bit;
                    state2.apply_player(round, table, player);
                    return Ok(Some(()));
                } else {
                    self.potential_on_table[round as usize][table as usize] &= !player_bit;
                    continue 'played_iter;
                }
            }
            return Err(());
        }
        Ok(None)
    }

    pub fn bstep<C>(&mut self, callback: &mut C)
    where
        C: FnMut(&Self),
    {
        let mut to_explore = self.tables_to_explore;
        if let Some((round, table)) = to_explore.pop() {
            let fixed_player_count =
                self.played_on_table[round as usize][table as usize].count_ones() as u8;
            match fixed_player_count.cmp(&(PLAYERS_PER_TABLE as u8)) {
                core::cmp::Ordering::Less => {
                    let potential = self.potential_on_table[round as usize][table as usize];
                    let potential_count = potential.count_ones() as u8;
                    match potential_count.cmp(&(PLAYERS_PER_TABLE as u8)) {
                        core::cmp::Ordering::Greater => {
                            // Find players

                            let mut to_add =
                                potential & !self.played_on_table[round as usize][table as usize];
                            while to_add != 0 {
                                let player = to_add.trailing_zeros() as usize;
                                to_add &= !(1 << player);
                                if self.can_play_with_players_in_game(round, table, player) {
                                    let mut new = *self;

                                    // Make it remove all lower numbers so that lowest player is always added first
                                    // Ensures that all generated solutions are unique
                                    new.potential_on_table[round as usize][table as usize] &=
                                        !((1 << player) - 1);
                                    new.apply_player(round, table, player);
                                    callback(&new);
                                } else {
                                    unreachable!();
                                }
                            }
                        }
                        core::cmp::Ordering::Equal => {
                            let mut potential =
                                potential & !self.played_on_table[round as usize][table as usize];
                            while potential != 0 {
                                let player = potential.trailing_zeros() as usize;
                                potential &= !(1 << player);
                                if self.can_play_with_players_in_game(round, table, player) {
                                    self.apply_player(round, table, player);
                                } else {
                                    // Cannot fill game
                                    return;
                                }
                            }

                            callback(self);
                        }
                        core::cmp::Ordering::Less => {
                            // Not enough potential to fill game
                            return;
                        }
                    }
                }
                core::cmp::Ordering::Greater => {
                    unreachable!()
                }
                core::cmp::Ordering::Equal => {
                    self.game_full(round, table);
                    callback(self);
                }
            }
        } else {
            // Maybe Done, Maybe error
            unreachable!()
        }
    }

    pub fn format_schedule<W: core::fmt::Write>(&self, output: &mut W) -> core::fmt::Result {
        fn base_10_length(n: usize) -> usize {
            (1..)
                .try_fold(n, |acc, i| if acc >= 10 { Ok(acc / 10) } else { Err(i) })
                .err()
                .unwrap_or(0)
        }
        output.write_str("     ")?;
        for table in 0..TABLE_COUNT {
            let now = table + 1;
            output.write_char('|')?;
            for _ in 0..(3 - base_10_length(now)) {
                output.write_char(' ')?;
            }
            output.write_fmt(format_args!("{}", now))?;
            output.write_str("  ")?;
        }

        for round in 0..ROUND_COUNT {
            output.write_str("\n-----")?;
            for _ in 0..TABLE_COUNT {
                output.write_char('+')?;
                output.write_str("-----")?;
            }
            for i in 0..PLAYERS_PER_TABLE + 1 {
                if i == (PLAYERS_PER_TABLE + 1) / 2 {
                    output.write_char('\n')?;
                    let now = round + 1;
                    for _ in 0..(3 - base_10_length(now)) {
                        output.write_char(' ')?;
                    }
                    output.write_fmt(format_args!("{}", now))?;
                    output.write_str("  ")?;
                } else {
                    output.write_str("\n     ")?;
                }
                'table: for table in 0..TABLE_COUNT {
                    output.write_char('|')?;
                    let mut counter = 0;
                    let mut temp = self.played_on_table[round as usize][table as usize];
                    while temp != 0 {
                        let trailing_zeros = temp.trailing_zeros() as usize;
                        let player = trailing_zeros;
                        let player_bit = 1 << trailing_zeros;
                        temp &= !player_bit;
                        if counter == i {
                            let now = player;
                            for _ in 0..(3 - base_10_length(now)) {
                                output.write_char(' ')?;
                            }
                            output.write_fmt(format_args!("{}", now))?;
                            output.write_str("  ")?;
                            continue 'table;
                        }
                        counter += 1;
                    }

                    output.write_str("     ")?;
                }
            }
        }
        Ok(())
    }
}
