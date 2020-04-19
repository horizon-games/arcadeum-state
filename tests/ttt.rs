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
#![cfg_attr(not(feature = "std"), feature(alloc_prelude))]

use arcadeum::{
    crypto::{sign, Addressable, SecretKey},
    Player, PlayerAction, Proof, ProofAction, ProofState, RootProof, State,
};

#[cfg(feature = "std")]
use arcadeum::utils::hex;

use rand::RngCore;

#[cfg(feature = "std")]
use std::{convert::TryInto, mem::size_of};

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use {
    alloc::{format, prelude::v1::*, vec},
    core::{convert::TryInto, mem::size_of},
};

#[cfg(not(feature = "std"))]
macro_rules! println {
    () => {
        ()
    };
    ($($arg:tt)*) => {
        ()
    };
}

#[derive(Clone, Default)]
struct TTT {
    nonce: u8,
    board: [[Option<Player>; 3]; 3],
}

impl State for TTT {
    type ID = [u8; 16];
    type Nonce = u8;
    type Action = Action;

    fn version() -> &'static [u8] {
        "TTT".as_bytes()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() != 3 * 3 {
            return Err("data.len() != 3 * 3".to_string());
        }

        let mut nonce = 0;
        let mut board = [[None; 3]; 3];

        for i in 0..3 {
            for j in 0..3 {
                board[i][j] = match data[3 * i + j] {
                    0 => None,
                    1 => {
                        nonce += 1;

                        Some(0)
                    }
                    2 => {
                        nonce += 1;

                        Some(1)
                    }
                    byte => return Err(format!("byte == {}", byte)),
                };
            }
        }

        Ok(Self { nonce, board })
    }

    fn is_serializable(&self) -> bool {
        true
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        let byte = |player| match player {
            None => 0,
            Some(player) => 1 + player,
        };

        Some(vec![
            byte(self.board[0][0]),
            byte(self.board[0][1]),
            byte(self.board[0][2]),
            byte(self.board[1][0]),
            byte(self.board[1][1]),
            byte(self.board[1][2]),
            byte(self.board[2][0]),
            byte(self.board[2][1]),
            byte(self.board[2][2]),
        ])
    }

    fn apply(&mut self, player: Option<Player>, action: &Self::Action) -> Result<(), String> {
        if self.board[action.0][action.1].is_some() {
            return Err("self.board[action.0][action.1].is_some()".to_string());
        }

        match self.nonce {
            0 | 2 | 4 | 6 | 8 => {
                if player != Some(0) {
                    return Err("player != Some(0)".to_string());
                }

                self.board[action.0][action.1] = Some(0);
                self.nonce += 1;
            }
            1 | 3 | 5 | 7 => {
                if player != Some(1) {
                    return Err("player != Some(1)".to_string());
                }

                self.board[action.0][action.1] = Some(1);
                self.nonce += 1;
            }
            nonce => return Err(format!("nonce == {}", nonce)),
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Action(usize, usize);

impl arcadeum::Action for Action {
    fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() != 2 {
            return Err("data.len() != 2".to_string());
        }

        if data[0] >= 3 {
            return Err("data[0] >= 3".to_string());
        }

        if data[1] >= 3 {
            return Err("data[1] >= 3".to_string());
        }

        Ok(Self(usize::from(data[0]), usize::from(data[1])))
    }

    fn serialize(&self) -> Vec<u8> {
        vec![self.0.try_into().unwrap(), self.1.try_into().unwrap()]
    }
}

#[cfg(not(feature = "no-crypto"))]
fn generate_keys_and_subkeys<R: rand::Rng>(
    randoms: &mut [R; 3],
) -> Result<([SecretKey; 3], [SecretKey; 2]), String> {
    Ok((
        [
            SecretKey::random(&mut randoms[0]),
            SecretKey::random(&mut randoms[1]),
            SecretKey::random(&mut randoms[2]),
        ],
        [
            SecretKey::random(&mut randoms[1]),
            SecretKey::random(&mut randoms[2]),
        ],
    ))
}

#[cfg(feature = "no-crypto")]
fn generate_keys_and_subkeys<R: rand::Rng>(
    randoms: &mut [R; 3],
) -> Result<([SecretKey; 3], [SecretKey; 2]), String> {
    Ok((
        [
            {
                let mut key = SecretKey::default();

                randoms[0]
                    .try_fill_bytes(&mut key)
                    .map_err(|error| error.to_string())?;

                key
            },
            {
                let mut key = SecretKey::default();

                randoms[1]
                    .try_fill_bytes(&mut key)
                    .map_err(|error| error.to_string())?;

                key
            },
            {
                let mut key = SecretKey::default();

                randoms[2]
                    .try_fill_bytes(&mut key)
                    .map_err(|error| error.to_string())?;

                key
            },
        ],
        [
            {
                let mut subkey = SecretKey::default();

                randoms[1]
                    .try_fill_bytes(&mut subkey)
                    .map_err(|error| error.to_string())?;

                subkey
            },
            {
                let mut subkey = SecretKey::default();

                randoms[2]
                    .try_fill_bytes(&mut subkey)
                    .map_err(|error| error.to_string())?;

                subkey
            },
        ],
    ))
}

#[test]
fn test_ttt() {
    let mut randoms = {
        const SIZE: usize = size_of::<<rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed>();

        [
            <rand_xorshift::XorShiftRng as rand::SeedableRng>::from_seed([0; SIZE]),
            <rand_xorshift::XorShiftRng as rand::SeedableRng>::from_seed([1; SIZE]),
            <rand_xorshift::XorShiftRng as rand::SeedableRng>::from_seed([2; SIZE]),
        ]
    };

    let (keys, subkeys) = generate_keys_and_subkeys(&mut randoms).unwrap();

    let mut id = <TTT as State>::ID::default();
    randoms[0].fill_bytes(&mut id);

    let players = keys[1..]
        .iter()
        .map(Addressable::address)
        .collect::<Vec<_>>()
        .as_slice()
        .try_into()
        .unwrap();

    let state = ProofState::<Box<TTT>>::new(id, players, Default::default()).unwrap();

    let root = RootProof::new(state, Vec::new(), &mut |message| {
        Ok(sign(message, &keys[0]))
    })
    .unwrap();

    println!("root = {}\n", hex(&root.serialize()));

    assert_eq!(
        root.serialize(),
        RootProof::<Box<TTT>>::deserialize(&root.serialize())
            .unwrap()
            .serialize()
    );

    let mut proof = Proof::new(root.clone());

    println!("proof = {}\n", hex(&proof.serialize()));

    let data = proof.serialize();

    assert_eq!(data, {
        let mut proof = Proof::new(root.clone());
        proof.deserialize(&data).unwrap();
        proof.serialize()
    });

    for (i, key) in keys[1..].iter().enumerate() {
        let address = subkeys[i].address();

        let action = ProofAction {
            player: Some(i.try_into().unwrap()),
            action: PlayerAction::Certify {
                address,
                signature: sign(TTT::challenge(&address).as_bytes(), key),
            },
        };

        let diff = proof
            .diff(vec![action], &mut |message| Ok(sign(message, key)))
            .unwrap();

        proof.apply(&diff).unwrap();

        println!("diff = {:?}\nproof = {}\n", diff, hex(&proof.serialize()));

        let data = proof.serialize();

        assert_eq!(data, {
            let mut proof = Proof::new(root.clone());
            proof.deserialize(&data).unwrap();
            proof.serialize()
        });
    }

    let mut apply = |player, action| {
        let action = ProofAction {
            player: Some(player),
            action: PlayerAction::Play(action),
        };

        let diff = proof
            .diff(vec![action], &mut |message| {
                Ok(sign(message, &subkeys[usize::from(player)]))
            })
            .unwrap();

        proof.apply(&diff).unwrap();

        println!("diff = {:?}\nproof = {}\n", diff, hex(&proof.serialize()));

        let data = proof.serialize();

        assert_eq!(data, {
            let mut proof = Proof::new(root.clone());
            proof.deserialize(&data).unwrap();
            proof.serialize()
        });
    };

    apply(0, Action(0, 0));
    apply(1, Action(1, 1));
    apply(0, Action(2, 2));
    apply(1, Action(0, 2));
    apply(0, Action(2, 0));
    apply(1, Action(1, 0));
    apply(0, Action(2, 1));
}
