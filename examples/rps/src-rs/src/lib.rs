/*
 * Arcadeum blockchain game framework
 * Copyright (C) 2019  Horizon Blockchain Games Inc.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 3.0 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate arcadeum_async;
#[macro_use]
extern crate arcadeum_state;

create_game!(SharedState, LocalState);

#[cfg_attr(not(feature = "bindings"), derive(Debug, Default))]
#[cfg_attr(feature = "bindings", derive(Deserialize, Serialize, Debug, Default))]
pub struct SharedState {
    score: (u32, u32),
    count: u32,
}

#[cfg_attr(not(feature = "bindings"), derive(Debug, Default))]
#[cfg_attr(feature = "bindings", derive(Deserialize, Serialize, Debug, Default))]
pub struct LocalState;

impl arcadeum_state::SharedState<LocalState> for SharedState {
    fn owner() -> Vec<u8> {
        b"\x37\x35\x13\xE3\x6c\x78\x04\x4A\x08\xA3\x5D\x23\x7C\x94\xEc\x49\xF3\x62\xe3\x72".to_vec()
    }

    fn seed(
        store: &mut arcadeum_state::Store<Self, LocalState>,
        _match_seed: &[u8],
        _public_seed_1: &[u8],
        _public_seed_2: &[u8],
    ) {
        #[asynchronous]
        fn process(store: &mut arcadeum_state::Store<SharedState, LocalState>) {
            let (store, secret_1, secret_2) = store.secret().await;

            match (secret_1[0], secret_2[0]) {
                (0, 1) | (1, 2) | (2, 0) => store.shared_state.score.1 += 1,
                (0, 2) | (1, 0) | (2, 1) => store.shared_state.score.0 += 1,
                _ => {}
            };

            store.shared_state.count += 1;

            if store.winner().is_none() {
                process(store);
            }
        }

        process(store);
    }
}

impl arcadeum_state::LocalState for LocalState {
    fn new(_secret_seed: &[u8]) -> Self {
        Default::default()
    }
}

impl arcadeum_state::State<SharedState, LocalState> for SharedState {
    fn winner(&self) -> Option<arcadeum_state::Player> {
        if self.score.0 >= 10 || self.score.1 >= 10 {
            if self.score.0 > self.score.1 + 1 {
                return Some(arcadeum_state::Player::One);
            } else if self.score.1 > self.score.0 + 1 {
                return Some(arcadeum_state::Player::Two);
            }
        }

        None
    }

    fn next_player(&self) -> Option<arcadeum_state::Player> {
        panic!("next_player");
    }

    fn verify(
        _store: &arcadeum_state::Store<Self, LocalState>,
        _player: arcadeum_state::Player,
        _action: &[u8],
    ) -> Result<(), arcadeum_state::Error> {
        panic!("verify");
    }

    fn mutate(
        _store: &mut arcadeum_state::Store<Self, LocalState>,
        _player: arcadeum_state::Player,
        _action: &[u8],
    ) {
        panic!("mutate");
    }
}
