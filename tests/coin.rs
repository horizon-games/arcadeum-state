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
#![feature(async_await)]

use arcadeum::{
    crypto, forbid, log,
    store::{Context, State, StoreState},
    Action, Player, PlayerAction, Proof, ProofAction, ProofState, RootProof, ID,
};

use libsecp256k1_rand::Rng;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{
    cell::RefCell, collections::VecDeque, convert::TryInto, future::Future, mem::size_of, pin::Pin,
    rc::Rc,
};

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use {
    alloc::{collections::VecDeque, format, prelude::v1::*, rc::Rc, vec},
    core::{cell::RefCell, convert::TryInto, future::Future, mem::size_of, pin::Pin},
};

#[cfg(not(feature = "std"))]
macro_rules! println {
    () => {};
    ($($arg:tt)*) => {};
}

#[cfg(feature = "bindings")]
arcadeum::bind!(Coin);

#[derive(Serialize, Clone, Default)]
pub struct Coin {
    nonce: u8,
    score: (u8, u8),
}

impl State for Coin {
    type ID = CoinID;
    type Nonce = u8;
    type Action = CoinAction;
    type Secret = ();

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        forbid!(data.len() != 1 + 1 + 1);

        Ok(Self {
            nonce: data[0],
            score: (data[1], data[2]),
        })
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        Some(vec![self.nonce, self.score.0, self.score.1])
    }

    fn verify(&self, player: Option<crate::Player>, _action: &Self::Action) -> Result<(), String> {
        forbid!(player != Some(self.nonce % 2));

        Ok(())
    }

    fn apply(
        mut self,
        player: Option<crate::Player>,
        action: Self::Action,
        mut context: Context,
    ) -> Pin<Box<dyn Future<Output = (Self, Context)>>> {
        Box::pin(async move {
            let random = context.random().await.next_u32();

            log!(context, random);

            if action.0 == (random % 2 == 1) {
                match player {
                    Some(0) => self.score.0 += 1,
                    Some(1) => self.score.1 += 1,
                    _ => unreachable!(),
                }
            }

            self.nonce += 1;

            (self, context)
        })
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CoinID([u8; 16]);

impl ID for CoinID {
    fn deserialize(data: &mut &[u8]) -> Result<Self, String> {
        let mut id = [0; size_of::<CoinID>()];

        forbid!(data.len() < size_of::<CoinID>());
        id.copy_from_slice(&data[..size_of::<CoinID>()]);
        *data = &data[size_of::<CoinID>()..];

        Ok(Self(id))
    }

    fn serialize(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CoinAction(bool);

impl Action for CoinAction {
    fn deserialize(data: &[u8]) -> Result<Self, String> {
        forbid!(data.len() != 1);

        Ok(CoinAction(match data[0] {
            0 => false,
            1 => true,
            byte => return Err(format!("byte == {}", byte)),
        }))
    }

    fn serialize(&self) -> Vec<u8> {
        vec![if self.0 { 1 } else { 0 }]
    }
}

#[test]
fn test_coin_async() {
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

    let mut id = [0; size_of::<CoinID>()];
    random.fill_bytes(&mut id);

    let mut players = [[0; size_of::<crypto::Address>()]; 2];

    players.copy_from_slice(
        &secrets
            .iter()
            .map(|secret| crypto::address(&secp256k1::PublicKey::from_secret_key(secret)))
            .collect::<Vec<_>>(),
    );

    let state = ProofState::<StoreState<Coin>>::new(
        CoinID(id),
        players,
        StoreState::Ready {
            state: Default::default(),
            log: None,
        },
    )
    .unwrap();

    let root = RootProof::new(state, Vec::new(), &mut |message| {
        crypto::sign(message, &owner)
    })
    .unwrap();

    println!("{}", hex(&root.serialize()));

    assert_eq!(
        root.serialize(),
        RootProof::<StoreState<Coin>>::deserialize(&root.serialize())
            .unwrap()
            .serialize()
    );

    let mut proof = Proof::new(root.clone());

    println!("{}", hex(&proof.serialize()));

    let data = proof.serialize();

    assert_eq!(data, {
        let mut proof = Proof::new(root.clone());
        proof.deserialize(&data).unwrap();
        proof.serialize()
    });

    let queue1 = Rc::new(RefCell::new(VecDeque::new()));
    let queue2 = Rc::new(RefCell::new(VecDeque::new()));

    let mut store1 = {
        let subkey = subkeys[0].clone();
        let opponent_queue = queue2.clone();

        arcadeum::store::Store::<Coin>::new(
            0,
            &root.serialize(),
            Box::new(|| {
                println!("0: ready");
            }),
            Box::new(move |message| crypto::sign(message, &subkey)),
            Box::new(move |diff| {
                opponent_queue
                    .try_borrow_mut()
                    .unwrap()
                    .push_back(diff.clone());
            }),
            Box::new(|message| {
                println!("0: {:?}", message);
            }),
            Box::new(rand::thread_rng()),
        )
        .unwrap()
    };

    let mut store2 = {
        let subkey = subkeys[1].clone();
        let opponent_queue = queue1.clone();

        arcadeum::store::Store::<Coin>::new(
            1,
            &root.serialize(),
            Box::new(|| {
                println!("1: ready");
            }),
            Box::new(move |message| crypto::sign(message, &subkey)),
            Box::new(move |diff| {
                opponent_queue
                    .try_borrow_mut()
                    .unwrap()
                    .push_back(diff.clone());
            }),
            Box::new(|message| {
                println!("1: {:?}", message);
            }),
            Box::new(rand::thread_rng()),
        )
        .unwrap()
    };

    for (i, secret) in secrets.iter().enumerate() {
        let address = crypto::address(&secp256k1::PublicKey::from_secret_key(&subkeys[i]));

        let action = ProofAction {
            player: Some(i.try_into().unwrap()),
            action: PlayerAction::Certify {
                address,
                signature: crypto::sign(Coin::certificate(&address).as_bytes(), secret).unwrap(),
            },
        };

        let diff = proof
            .diff(vec![action], &mut |message| crypto::sign(message, secret))
            .unwrap();

        proof.apply(&diff).unwrap();
        store1.apply(&diff).unwrap();
        store2.apply(&diff).unwrap();

        println!("{}", hex(&proof.serialize()));

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
                crypto::sign(message, &subkeys[usize::from(player)])
            })
            .unwrap();

        proof.apply(&diff).unwrap();
        store1.apply(&diff).unwrap();
        store2.apply(&diff).unwrap();

        loop {
            while let Some(diff) = queue1.try_borrow_mut().unwrap().pop_front() {
                store1.apply(&diff).unwrap();
                proof.apply(&diff).unwrap();
            }

            while let Some(diff) = queue2.try_borrow_mut().unwrap().pop_front() {
                store2.apply(&diff).unwrap();
                proof.apply(&diff).unwrap();
            }

            if queue1.try_borrow().unwrap().is_empty() && queue2.try_borrow().unwrap().is_empty() {
                break;
            }
        }

        println!("{}", hex(&proof.serialize()));

        let data = proof.serialize();

        assert_eq!(data, {
            let mut proof = Proof::new(root.clone());
            proof.deserialize(&data).unwrap();
            proof.serialize()
        });
    };

    apply(0, arcadeum::store::StoreAction::Action(CoinAction(true)));
    apply(1, arcadeum::store::StoreAction::Action(CoinAction(true)));
    apply(0, arcadeum::store::StoreAction::Action(CoinAction(true)));
    apply(1, arcadeum::store::StoreAction::Action(CoinAction(true)));
    apply(0, arcadeum::store::StoreAction::Action(CoinAction(true)));
    apply(1, arcadeum::store::StoreAction::Action(CoinAction(true)));
    apply(0, arcadeum::store::StoreAction::Action(CoinAction(true)));

    println!("{:?}", proof.serialize());
}

fn hex(data: &[u8]) -> String {
    let mut hex = String::with_capacity("0x".len() + 2 * data.len());

    hex += "0x";
    hex.extend(data.iter().map(|byte| format!("{:02x}", byte)));

    hex
}
