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
    crypto,
    store::{Context, State, StoreState},
    Player, PlayerAction, Proof, ProofAction, ProofState, RootProof,
};

#[cfg(feature = "std")]
use arcadeum::utils::hex;

use rand_core::{RngCore, SeedableRng};
use serde::Serialize;

#[cfg(feature = "std")]
use std::{
    cell::RefCell, collections::VecDeque, convert::TryInto, future::Future, pin::Pin, rc::Rc,
};

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use {
    alloc::{collections::VecDeque, format, prelude::v1::*, rc::Rc, vec},
    core::{cell::RefCell, convert::TryInto, future::Future, pin::Pin},
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

#[cfg(feature = "bindings")]
arcadeum::bind!(Coin);

#[derive(Serialize, Clone, Debug, Default)]
struct Coin {
    nonce: u8,
    score: [u8; 2],
}

impl State for Coin {
    type ID = [u8; 16];
    type Nonce = u8;
    type Action = bool;
    type Secret = ();

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() != 1 + 2 {
            return Err("data.len() != 1 + 2".to_string());
        }

        Ok(Self {
            nonce: data[0],
            score: [data[1], data[2]],
        })
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        Some(vec![self.nonce, self.score[0], self.score[1]])
    }

    fn verify(&self, player: Option<crate::Player>, _action: &Self::Action) -> Result<(), String> {
        if player != Some(self.nonce % 2) {
            return Err("player != Some(self.nonce % 2)".to_string());
        }

        Ok(())
    }

    fn apply(
        mut self,
        player: Option<crate::Player>,
        action: &Self::Action,
        mut context: Context<Self>,
    ) -> Pin<Box<dyn Future<Output = (Self, Context<Self>)>>> {
        Box::pin({
            let action = *action;

            async move {
                let random: u32 = context.random().await.next_u32();

                context.log(&random);

                if action == (random % 2 != 0) {
                    self.score[usize::from(player.unwrap())] += 1;
                }

                self.nonce += 1;

                (self, context)
            }
        })
    }
}

#[test]
fn test_coin() {
    let mut random = libsecp256k1_rand::thread_rng();

    let owner = secp256k1::SecretKey::random(&mut random);

    let keys = [
        secp256k1::SecretKey::random(&mut random),
        secp256k1::SecretKey::random(&mut random),
    ];

    let subkeys = [
        secp256k1::SecretKey::random(&mut random),
        secp256k1::SecretKey::random(&mut random),
    ];

    let mut random = rand::rngs::StdRng::from_seed([0; 32]);

    let mut id = <Coin as State>::ID::default();
    random.fill_bytes(&mut id);

    let players = keys
        .iter()
        .map(|key| crypto::address(&secp256k1::PublicKey::from_secret_key(key)))
        .collect::<Vec<_>>()
        .as_slice()
        .try_into()
        .unwrap();

    let state =
        ProofState::<StoreState<Coin>>::new(id, players, StoreState::new(Default::default()))
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

    let queues = [
        Rc::new(RefCell::new(VecDeque::new())),
        Rc::new(RefCell::new(VecDeque::new())),
        Rc::new(RefCell::new(VecDeque::new())),
    ];

    let mut store0 = {
        let queue = queues[0].clone();

        arcadeum::store::Store::<Coin>::new(
            None,
            &root.serialize(),
            [Some(()), Some(())],
            |state| println!("0: ready: {:?}", state),
            move |message| crypto::sign(message, &owner),
            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.clone()),
            |event| println!("0: {:?}", event),
            Box::new(rand::rngs::StdRng::from_seed([0; 32])),
        )
        .unwrap()
    };

    let mut store1 = {
        let subkey = subkeys[0].clone();
        let queue = queues[1].clone();

        arcadeum::store::Store::<Coin>::new(
            Some(0),
            &root.serialize(),
            [Some(()), None],
            |state| println!("1: ready: {:?}", state),
            move |message| crypto::sign(message, &subkey),
            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.clone()),
            |event| println!("1: {:?}", event),
            Box::new(rand::rngs::StdRng::from_seed([1; 32])),
        )
        .unwrap()
    };

    let mut store2 = {
        let subkey = subkeys[1].clone();
        let queue = queues[2].clone();

        arcadeum::store::Store::<Coin>::new(
            Some(1),
            &root.serialize(),
            [None, Some(())],
            |state| println!("2: ready: {:?}", state),
            move |message| crypto::sign(message, &subkey),
            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.clone()),
            |event| println!("2: {:?}", event),
            Box::new(rand::rngs::StdRng::from_seed([2; 32])),
        )
        .unwrap()
    };

    for (i, key) in keys.iter().enumerate() {
        let address = crypto::address(&secp256k1::PublicKey::from_secret_key(&subkeys[i]));

        let action = ProofAction {
            player: Some(i.try_into().unwrap()),
            action: PlayerAction::Certify {
                address,
                signature: crypto::sign(Coin::certificate(&address).as_bytes(), key).unwrap(),
            },
        };

        let diff = proof
            .diff(vec![action], &mut |message| crypto::sign(message, key))
            .unwrap();

        proof.apply(&diff).unwrap();
        store0.apply(&diff).unwrap();
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
        store0.apply(&diff).unwrap();
        store1.apply(&diff).unwrap();
        store2.apply(&diff).unwrap();

        loop {
            while let Some(diff) = queues[1].try_borrow_mut().unwrap().pop_front() {
                store0.apply(&diff).unwrap();
                store2.apply(&diff).unwrap();
                proof.apply(&diff).unwrap();

                while let Some(diff) = queues[0].try_borrow_mut().unwrap().pop_front() {
                    store1.apply(&diff).unwrap();
                    store2.apply(&diff).unwrap();
                    proof.apply(&diff).unwrap();
                }
            }

            while let Some(diff) = queues[2].try_borrow_mut().unwrap().pop_front() {
                store0.apply(&diff).unwrap();
                store1.apply(&diff).unwrap();
                proof.apply(&diff).unwrap();

                while let Some(diff) = queues[0].try_borrow_mut().unwrap().pop_front() {
                    store1.apply(&diff).unwrap();
                    store2.apply(&diff).unwrap();
                    proof.apply(&diff).unwrap();
                }
            }

            while let Some(diff) = queues[0].try_borrow_mut().unwrap().pop_front() {
                store1.apply(&diff).unwrap();
                store2.apply(&diff).unwrap();
                proof.apply(&diff).unwrap();
            }

            if queues[1].try_borrow().unwrap().is_empty()
                && queues[2].try_borrow().unwrap().is_empty()
            {
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

    apply(0, arcadeum::store::StoreAction::Action(true));
    apply(1, arcadeum::store::StoreAction::Action(true));
    apply(0, arcadeum::store::StoreAction::Action(true));
    apply(1, arcadeum::store::StoreAction::Action(true));
    apply(0, arcadeum::store::StoreAction::Action(true));
    apply(1, arcadeum::store::StoreAction::Action(true));
    apply(0, arcadeum::store::StoreAction::Action(true));

    println!("{:?}", proof.serialize());
}
