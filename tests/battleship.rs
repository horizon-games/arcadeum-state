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
    store::{Context, Secret, State, StoreState},
    utils::hex,
    Player, PlayerAction, Proof, ProofAction, ProofState, RootProof, ID,
};

use rand_core::{RngCore, SeedableRng};
use serde::Serialize;

#[cfg(feature = "std")]
use {
    serde::Deserialize,
    std::{
        cell::RefCell, collections::VecDeque, convert::TryInto, future::Future, mem::size_of,
        pin::Pin, rc::Rc,
    },
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
    () => {
        ()
    };
    ($($arg:tt)*) => {
        ()
    };
}

#[cfg(feature = "bindings")]
arcadeum::bind!(Battleship);

#[derive(Serialize, Clone, Debug, Default)]
struct Battleship {
    nonce: u8,
    score: [u8; 2],
    roots: [crypto::Hash; 2],
}

impl State for Battleship {
    type ID = BattleshipID;
    type Nonce = u8;
    type Action = u8;
    type Secret = BattleshipSecret;

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() != 1 + 2 + 2 * size_of::<crypto::Hash>() {
            return Err("data.len() != 1 + 2 + 2 * size_of::<crypto::Hash>()".to_string());
        }

        Ok(Self {
            nonce: data[0],
            score: [data[1], data[2]],
            roots: [
                data[3..][..size_of::<crypto::Hash>()].try_into().unwrap(),
                data[3 + size_of::<crypto::Hash>()..].try_into().unwrap(),
            ],
        })
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        let mut data = vec![self.nonce, self.score[0], self.score[1]];
        data.extend(self.roots.concat());

        Some(data)
    }

    fn verify(&self, player: Option<crate::Player>, action: &Self::Action) -> Result<(), String> {
        if player != Some(self.nonce % 2) {
            return Err("player != Some(self.nonce % 2)".to_string());
        }

        if *action > 100 {
            return Err("*action > 100".to_string());
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
                let proof = context
                    .reveal_unique(
                        1 - player.unwrap(),
                        move |secret| secret.0.proof(usize::from(action)).unwrap(),
                        {
                            let roots = self.roots;

                            move |proof: &crypto::MerkleProof<bool>| {
                                proof.index() == usize::from(action)
                                    && proof.length() == 100
                                    && *proof.root() == roots[1 - usize::from(player.unwrap())]
                            }
                        },
                    )
                    .await;

                context.log(proof.element());

                if *proof.element() {
                    self.score[usize::from(player.unwrap())] += 1;
                }

                self.nonce += 1;

                (self, context)
            }
        })
    }
}

#[derive(Clone, PartialEq, Eq)]
struct BattleshipID([u8; 16]);

impl ID for BattleshipID {
    fn deserialize(data: &mut &[u8]) -> Result<Self, String> {
        if data.len() < size_of::<BattleshipID>() {
            return Err("data.len() < size_of::<BattleshipID>()".to_string());
        }

        let id = data[..size_of::<BattleshipID>()]
            .try_into()
            .map_err(|error| format!("{}", error))?;

        *data = &data[size_of::<BattleshipID>()..];

        Ok(Self(id))
    }

    fn serialize(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Clone)]
struct BattleshipSecret(crypto::MerkleTree<bool>);

impl Secret for BattleshipSecret {
    fn deserialize(data: &[u8]) -> Result<Self, String> {
        Ok(Self(crypto::MerkleTree::deserialize(data)?))
    }

    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }
}

#[test]
fn test_battleship() {
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

    let mut id = [0; size_of::<BattleshipID>()];
    random.fill_bytes(&mut id);

    let players = keys
        .iter()
        .map(|key| crypto::address(&secp256k1::PublicKey::from_secret_key(key)))
        .collect::<Vec<_>>()
        .as_slice()
        .try_into()
        .unwrap();

    let secrets = [
        BattleshipSecret(
            crypto::MerkleTree::with_salt(
                {
                    let mut elements = [0; 100];
                    random.fill_bytes(&mut elements);
                    elements.iter().map(|element| element % 2 != 0).collect()
                },
                16,
                &mut random,
            )
            .unwrap(),
        ),
        BattleshipSecret(
            crypto::MerkleTree::with_salt(
                {
                    let mut elements = [0; 100];
                    random.fill_bytes(&mut elements);
                    elements.iter().map(|element| element % 2 != 0).collect()
                },
                16,
                &mut random,
            )
            .unwrap(),
        ),
    ];

    let state = ProofState::<StoreState<Battleship>>::new(
        BattleshipID(id),
        players,
        StoreState::new(Battleship {
            nonce: Default::default(),
            score: Default::default(),
            roots: [
                secrets[0].0.root()[..].try_into().unwrap(),
                secrets[1].0.root()[..].try_into().unwrap(),
            ],
        }),
    )
    .unwrap();

    let root = RootProof::new(state, Vec::new(), &mut |message| {
        crypto::sign(message, &owner)
    })
    .unwrap();

    println!("{}", hex(&root.serialize()));

    assert_eq!(
        root.serialize(),
        RootProof::<StoreState<Battleship>>::deserialize(&root.serialize())
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

        arcadeum::store::Store::<Battleship>::new(
            None,
            &root.serialize(),
            [Some(secrets[0].clone()), Some(secrets[1].clone())],
            |state| println!("0: ready: {:?}", state),
            move |message| crypto::sign(message, &owner),
            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.clone()),
            |message| println!("0: {:?}", message),
            Box::new(rand::rngs::StdRng::from_seed([0; 32])),
        )
        .unwrap()
    };

    let mut store1 = {
        let subkey = subkeys[0].clone();
        let queue = queues[1].clone();

        arcadeum::store::Store::<Battleship>::new(
            Some(0),
            &root.serialize(),
            [Some(secrets[0].clone()), None],
            |state| println!("1: ready: {:?}", state),
            move |message| crypto::sign(message, &subkey),
            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.clone()),
            |message| println!("1: {:?}", message),
            Box::new(rand::rngs::StdRng::from_seed([1; 32])),
        )
        .unwrap()
    };

    let mut store2 = {
        let subkey = subkeys[1].clone();
        let queue = queues[2].clone();

        arcadeum::store::Store::<Battleship>::new(
            Some(1),
            &root.serialize(),
            [None, Some(secrets[1].clone())],
            |state| println!("2: ready: {:?}", state),
            move |message| crypto::sign(message, &subkey),
            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.clone()),
            |message| println!("2: {:?}", message),
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
                signature: crypto::sign(Battleship::certificate(&address).as_bytes(), key).unwrap(),
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

    for _ in 0..20 {
        apply(
            0,
            arcadeum::store::StoreAction::Action((random.next_u32() % 100).try_into().unwrap()),
        );

        apply(
            1,
            arcadeum::store::StoreAction::Action((random.next_u32() % 100).try_into().unwrap()),
        );
    }

    println!("{:?}", proof.serialize());
}
