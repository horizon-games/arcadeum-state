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

use {
    arcadeum::{
        crypto::{address, sign, Address},
        utils::hex,
        Player, PlayerAction, Proof, ProofAction, ProofState, RootProof, State,
    },
    libsecp256k1_rand::Rng,
};

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
    () => {};
    ($($arg:tt)*) => {};
}

#[derive(Clone, Default)]
pub struct TTT {
    nonce: u8,
    board: [[Option<Player>; 3]; 3],
}

impl State for TTT {
    type ID = ID;
    type Nonce = u8;
    type Action = Action;

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

#[derive(Clone, PartialEq, Eq)]
pub struct ID([u8; 16]);

impl arcadeum::ID for ID {
    fn deserialize(data: &mut &[u8]) -> Result<Self, String> {
        if data.len() < size_of::<ID>() {
            return Err("data.len() < size_of::<ID>()".to_string());
        }

        let mut id = [0; size_of::<ID>()];
        id.copy_from_slice(&data[..size_of::<ID>()]);
        *data = &data[size_of::<ID>()..];

        Ok(Self(id))
    }

    fn serialize(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

#[derive(Clone, Debug)]
pub struct Action(usize, usize);

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

#[test]
fn test_ttt() {
    let mut random = libsecp256k1_rand::thread_rng();

    let owner = secp256k1::SecretKey::random(&mut random);

    let secrets = [
        secp256k1::SecretKey::random(&mut random),
        secp256k1::SecretKey::random(&mut random),
    ];

    let subkeys = [
        secp256k1::SecretKey::random(&mut random),
        secp256k1::SecretKey::random(&mut random),
    ];

    let mut id = [0; size_of::<ID>()];
    random.fill_bytes(&mut id);
    let id = ID(id);

    let mut players = [[0; size_of::<Address>()]; 2];

    players.copy_from_slice(
        &secrets
            .iter()
            .map(|secret| address(&secp256k1::PublicKey::from_secret_key(secret)))
            .collect::<Vec<_>>(),
    );

    let state = ProofState::<Box<TTT>>::new(id, players, Default::default()).unwrap();

    let root = RootProof::new(state, Vec::new(), &mut |message| sign(message, &owner)).unwrap();

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

    for (i, secret) in secrets.iter().enumerate() {
        let address = address(&secp256k1::PublicKey::from_secret_key(&subkeys[i]));

        let action = ProofAction {
            player: Some(i.try_into().unwrap()),
            action: PlayerAction::Certify {
                address,
                signature: sign(TTT::certificate(&address).as_bytes(), secret).unwrap(),
            },
        };

        let diff = proof
            .diff(vec![action], &mut |message| sign(message, secret))
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
                sign(message, &subkeys[usize::from(player)])
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
