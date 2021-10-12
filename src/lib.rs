mod to_explore;
use to_explore::ToExplore;

use serde::{Deserialize, Serialize};

const ROUND_COUNT: usize = 6;
const TABLE_COUNT: usize = 6;
const PLAYERS_PER_TABLE: usize = 4;
const TO_EXPLORE_SHIFT: usize = 3;
const PLAYER_COUNT: usize = TABLE_COUNT * PLAYERS_PER_TABLE;

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

impl State {
    pub fn new() -> Self {
        let potential_on_table = [[(1 << 24) - 1; TABLE_COUNT]; ROUND_COUNT];
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
        for table in 0..TABLE_COUNT {
            for _ in 0..PLAYERS_PER_TABLE {
                state.apply_player(0, table, player);
                player += 1;
            }
        }
        state
    }

    fn can_play_with_players_in_game(&self, round: usize, table: usize, player: usize) -> bool {
        self.players_played_with[player] & self.played_on_table[round][table] == 0
    }

    fn game_full(&mut self, round: usize, table: usize) {
        self.tables_to_explore.remove(round, table);
        self.potential_on_table[round][table] = self.played_on_table[round][table];
    }

    fn apply_player(&mut self, round: usize, table: usize, player: usize) {
        if round >= ROUND_COUNT || table >= TABLE_COUNT || player >= PLAYER_COUNT {
            unreachable!();
        }

        debug_assert!(self.can_play_with_players_in_game(round, table, player));

        self.players_played_count += 1;
        let player_mask: u32 = 1 << player;
        let remove_player_mask: u32 = !player_mask;
        for ptr in self.potential_on_table.iter_mut() {
            // Remove player from the table in other rounds
            ptr[table] &= remove_player_mask;
        }
        for ptr in self.potential_on_table[round].iter_mut() {
            // Remove player from other tables in the same round
            *ptr &= remove_player_mask;
        }

        // Add player to played in round
        debug_assert_eq!(self.played_in_round[round] & player_mask, 0);
        self.played_in_round[round] |= player_mask;
        // Add player to played on table
        debug_assert_eq!(self.played_on_table_total[table] & player_mask, 0);
        self.played_on_table_total[table] |= player_mask;

        let mut other_players = self.played_on_table[round][table];
        debug_assert_eq!(other_players & player_mask, 0);
        // Remove players current player has previously played with from tables potential
        self.potential_on_table[round][table] &= !self.players_played_with[player];
        // Add other players on table to current players played with list
        self.players_played_with[player] |= other_players;
        while other_players != 0 {
            let other_player = other_players.trailing_zeros() as usize;
            other_players &= !(1 << other_player);
            // Add current player to each other players played with list
            self.players_played_with[other_player] |= player_mask;
        }

        debug_assert_eq!(self.potential_on_table[round][table] & player_mask, 0);
        self.potential_on_table[round][table] |= player_mask;
        debug_assert_eq!(self.played_on_table[round][table] & player_mask, 0);
        self.played_on_table[round][table] |= player_mask;
        if self.played_on_table[round][table].count_ones() == PLAYERS_PER_TABLE as u32 {
            self.game_full(round, table);
        }
        debug_assert!(self.played_on_table[round][table].count_ones() <= PLAYERS_PER_TABLE as u32);
    }

    pub fn get_players_played_count(&self) -> u8 {
        self.players_played_count
    }

    pub fn find_hidden_singles(&mut self) -> Result<(), ()> {
        for round in 0..ROUND_COUNT {
            let mut potential_in_row = !self.played_in_round[round];
            'loop_bits_round: while potential_in_row != 0 {
                let player = potential_in_row.trailing_zeros() as usize;
                if player >= PLAYER_COUNT {
                    break;
                }
                let player_bit: u32 = 1 << player;
                potential_in_row &= !player_bit;
                let mut only_position = None;
                for table in 0..TABLE_COUNT {
                    if self.potential_on_table[round][table] & player_bit != 0 {
                        if self.can_play_with_players_in_game(round, table, player) {
                            if only_position.is_none() {
                                only_position = Some(table);
                            } else {
                                continue 'loop_bits_round;
                            }
                        } else {
                            self.potential_on_table[round][table] &= !player_bit;
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

        for table in 0..TABLE_COUNT {
            let mut potential_in_column = !self.played_on_table_total[table];
            'loop_bits_table: while potential_in_column != 0 {
                let player = potential_in_column.trailing_zeros() as usize;
                if player >= PLAYER_COUNT {
                    break;
                }
                let player_bit = 1 << player;
                potential_in_column &= !player_bit;
                let mut only_position = None;
                for round in 0..ROUND_COUNT {
                    if self.potential_on_table[round][table] & player_bit != 0 {
                        if self.can_play_with_players_in_game(round, table, player) {
                            if only_position.is_none() {
                                only_position = Some(round);
                            } else {
                                continue 'loop_bits_table;
                            }
                        } else {
                            self.potential_on_table[round][table] &= !player_bit;
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

    pub fn step(&mut self) -> Result<Option<Self>, ()> {
        //self.find_hidden_singles()?;

        let mut lowest: Option<(u8, usize, usize)> = None;
        let mut to_explore = self.tables_to_explore;
        while let Some((round, table)) = to_explore.pop() {
            let fixed_player_count = self.played_on_table[round][table].count_ones() as u8;
            match fixed_player_count.cmp(&(PLAYERS_PER_TABLE as u8)) {
                core::cmp::Ordering::Less => {
                    let potential = self.potential_on_table[round][table];
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
                            let mut potential = potential & !self.played_on_table[round][table];
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
                    self.empty_table_count = self.empty_table_count.checked_sub(1).unwrap();
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
            let potential =
                self.potential_on_table[round][table] & !self.played_on_table[round][table];
            let mut temp = potential;
            'played_iter: while temp != 0 {
                let player = temp.trailing_zeros() as usize;
                let player_bit = 1 << player;
                temp &= !player_bit;
                if self.can_play_with_players_in_game(round, table, player) {
                    let mut state2 = *self;
                    self.potential_on_table[round][table] &= !player_bit;
                    state2.apply_player(round, table, player);
                    return Ok(Some(state2));
                } else {
                    self.potential_on_table[round][table] &= !player_bit;
                    continue 'played_iter;
                }
            }
            return Err(());
        }
        Ok(None)
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
                    let mut temp = self.played_on_table[round][table];
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
