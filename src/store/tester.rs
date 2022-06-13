/*
 * Copyright 2019 Horizon Blockchain Games Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Store tester

use crate::store::SecretKnowledge;

use {
    alloc::{
        boxed::Box,
        collections::VecDeque,
        format,
        rc::Rc,
        string::{String, ToString},
        vec,
        vec::Vec,
    },
    core::{cell::RefCell, column, file, line, mem::size_of, ops::Deref},
};

/// Store tester
pub struct Tester<S: crate::store::State>
where
    S::ID: Default,
{
    proof: crate::Proof<crate::store::StoreState<S>>,
    stores: [crate::store::Store<S>; 3],
    queues: [Rc<RefCell<VecDeque<Vec<u8>>>>; 3],
}

impl<S: crate::store::State> Tester<S>
where
    S::ID: Default,
{
    /// Constructs a new store tester.
    pub fn new(
        state: S,
        [secret1, secret2]: [S::Secret; 2],
        actions: Vec<crate::ProofAction<crate::store::StoreState<S>>>,
        ready: impl FnMut(Option<crate::Player>, &S, [Option<&S::Secret>; 2]) + 'static,
        log: impl FnMut(Option<crate::Player>, Option<crate::Player>, S::Event) + 'static,
    ) -> Result<Self, String> {
        let mut randoms = {
            const SIZE: usize = size_of::<
                <libsecp256k1_rand::rngs::StdRng as libsecp256k1_rand::SeedableRng>::Seed,
            >();

            [
                <libsecp256k1_rand::rngs::StdRng as libsecp256k1_rand::SeedableRng>::from_seed(
                    [1; SIZE],
                ),
                <libsecp256k1_rand::rngs::StdRng as libsecp256k1_rand::SeedableRng>::from_seed(
                    [2; SIZE],
                ),
                <libsecp256k1_rand::rngs::StdRng as libsecp256k1_rand::SeedableRng>::from_seed(
                    [3; SIZE],
                ),
            ]
        };

        let (keys, subkeys) = generate_keys_and_subkeys(&mut randoms)?;

        let randoms = {
            const SIZE: usize =
                size_of::<<rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed>();

            [
                <rand_xorshift::XorShiftRng as rand::SeedableRng>::from_seed([1; SIZE]),
                <rand_xorshift::XorShiftRng as rand::SeedableRng>::from_seed([2; SIZE]),
                <rand_xorshift::XorShiftRng as rand::SeedableRng>::from_seed([3; SIZE]),
            ]
        };

        let certificates = if cfg!(feature = "test-approvals") {
            [
                {
                    let player = crate::crypto::Addressable::address(&keys[1]);
                    let subkey = crate::crypto::Addressable::address(&subkeys[0]);

                    crate::ProofAction {
                        player: None,
                        action: crate::PlayerAction::Approve {
                            player,
                            subkey,
                            signature: crate::crypto::sign(
                                <crate::store::StoreState<S> as crate::State>::approval(
                                    &player, &subkey,
                                )
                                .as_bytes(),
                                &keys[0],
                            ),
                        },
                    }
                },
                {
                    let player = crate::crypto::Addressable::address(&keys[2]);
                    let subkey = crate::crypto::Addressable::address(&subkeys[1]);

                    crate::ProofAction {
                        player: None,
                        action: crate::PlayerAction::Approve {
                            player,
                            subkey,
                            signature: crate::crypto::sign(
                                <crate::store::StoreState<S> as crate::State>::approval(
                                    &player, &subkey,
                                )
                                .as_bytes(),
                                &keys[0],
                            ),
                        },
                    }
                },
            ]
        } else {
            [
                {
                    let address = crate::crypto::Addressable::address(&subkeys[0]);

                    crate::ProofAction {
                        player: Some(0),
                        action: crate::PlayerAction::Certify {
                            address,
                            signature: crate::crypto::sign(
                                <crate::store::StoreState<S> as crate::State>::challenge(&address)
                                    .as_bytes(),
                                &keys[1],
                            ),
                        },
                    }
                },
                {
                    let address = crate::crypto::Addressable::address(&subkeys[1]);

                    crate::ProofAction {
                        player: Some(1),
                        action: crate::PlayerAction::Certify {
                            address,
                            signature: crate::crypto::sign(
                                <crate::store::StoreState<S> as crate::State>::challenge(&address)
                                    .as_bytes(),
                                &keys[2],
                            ),
                        },
                    }
                },
            ]
        };

        let proof = crate::Proof::new(crate::RootProof::new(
            crate::ProofState::new(
                Default::default(),
                [
                    crate::crypto::Addressable::address(&keys[1]),
                    crate::crypto::Addressable::address(&keys[2]),
                ],
                crate::store::StoreState::new(state, Default::default(), |_, _| ()),
            )?,
            [&certificates[..], &actions].concat(),
            &mut |message| Ok(crate::crypto::sign(message, &keys[0])),
        )?);

        let queues = [
            Rc::new(RefCell::new(VecDeque::new())),
            Rc::new(RefCell::new(VecDeque::new())),
            Rc::new(RefCell::new(VecDeque::new())),
        ];

        let root = proof.root.serialize();

        let stores = {
            let [random0, random1, random2] = randoms;
            let [subkey1, subkey2] = subkeys;
            let ready = Rc::new(RefCell::new(ready));
            let log = Rc::new(RefCell::new(log));

            [
                {
                    let mut store = crate::store::Store::new(
                        None,
                        &root,
                        [
                            Some((secret1.clone(), [1; 16])),
                            Some((secret2.clone(), [2; 16])),
                        ],
                        false,
                        {
                            let ready = ready.clone();

                            move |state, secrets| {
                                (ready.try_borrow_mut().unwrap())(None, state, secrets)
                            }
                        },
                        move |message| Ok(crate::crypto::sign(message, &keys[0])),
                        {
                            let queue = queues[0].clone();

                            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.serialize())
                        },
                        {
                            let log = log.clone();

                            move |target, event| {
                                (log.try_borrow_mut().unwrap())(None, target, event)
                            }
                        },
                        random0,
                    )?;

                    store.flush()?;

                    store
                },
                {
                    let mut store = crate::store::Store::new(
                        Some(0),
                        &root,
                        [Some((secret1, [1; 16])), None],
                        false,
                        {
                            let ready = ready.clone();

                            move |state, secrets| {
                                (ready.try_borrow_mut().unwrap())(Some(0), state, secrets)
                            }
                        },
                        move |message| Ok(crate::crypto::sign(message, &subkey1)),
                        {
                            let queue = queues[1].clone();

                            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.serialize())
                        },
                        {
                            let log = log.clone();

                            move |target, event| {
                                (log.try_borrow_mut().unwrap())(Some(0), target, event)
                            }
                        },
                        random1,
                    )?;

                    store.flush()?;

                    store
                },
                {
                    let mut store = crate::store::Store::new(
                        Some(1),
                        &root,
                        [None, Some((secret2, [2; 16]))],
                        false,
                        move |state, secrets| {
                            (ready.try_borrow_mut().unwrap())(Some(1), state, secrets)
                        },
                        move |message| Ok(crate::crypto::sign(message, &subkey2)),
                        {
                            let queue = queues[2].clone();

                            move |diff| queue.try_borrow_mut().unwrap().push_back(diff.serialize())
                        },
                        move |target, event| {
                            (log.try_borrow_mut().unwrap())(Some(1), target, event)
                        },
                        random2,
                    )?;

                    store.flush()?;

                    store
                },
            ]
        };

        let mut tester = Self {
            proof,
            stores,
            queues,
        };

        tester.flush()?;

        tester.check()?;

        Ok(tester)
    }

    /// Gets the state of the tester.
    pub fn state(&self) -> &S {
        self.stores[0].state().state().state().unwrap()
    }

    /// Gets a player's secret information.
    pub fn secret<'a>(&'a self, player: crate::Player) -> Box<dyn Deref<Target = S::Secret> + 'a> {
        self.stores[1 + usize::from(player)]
            .state()
            .state()
            .secret(player)
            .unwrap()
    }

    /// Applies an action by a given player (or the owner) to the tester.
    ///
    /// Returns a [Vec] of actions that were automatically dispatched as a result.
    pub fn apply(
        &mut self,
        player: Option<crate::Player>,
        action: &S::Action,
    ) -> Result<Vec<crate::ProofAction<crate::store::StoreState<S>>>, String> {
        let diff = self.stores[if let Some(player) = player {
            1 + usize::from(player)
        } else {
            0
        }]
        .diff(vec![crate::ProofAction {
            player,
            action: crate::PlayerAction::Play(crate::store::StoreAction::new(action.clone())),
        }])?;

        self.proof.apply(&diff)?;

        for store in &mut self.stores {
            store.apply(&diff)?;
        }

        let reveals = self.flush()?;

        self.check().map(|_| reveals)
    }

    fn check(&self) -> Result<(), String> {
        crate::forbid!({
            let data = self.proof.root.serialize();

            deserialize_root_proof::<S>(&data)?.serialize() != data
        });

        crate::forbid!({
            let data = self.proof.serialize();

            deserialize_proof(&data, self.proof.root.clone())?.serialize() != data
        });

        for store in &self.stores {
            crate::forbid!(store.proof.serialize() != self.proof.serialize());

            crate::forbid!({
                let data = store.serialize(SecretKnowledge::Both);

                deserialize_store::<S>(&data)?.serialize(SecretKnowledge::Both) != data
            });
        }

        crate::forbid!(
            self.stores[1].serialize(SecretKnowledge::Both)
                != self.stores[1].serialize(SecretKnowledge::Some(0))
        );
        crate::forbid!(
            self.stores[2].serialize(SecretKnowledge::Both)
                != self.stores[2].serialize(SecretKnowledge::Some(1))
        );
        crate::forbid!(
            self.stores[0].serialize(SecretKnowledge::Some(0))
                != self.stores[1].serialize(SecretKnowledge::Some(0))
        );
        crate::forbid!(
            self.stores[0].serialize(SecretKnowledge::Some(1))
                != self.stores[2].serialize(SecretKnowledge::Some(1))
        );

        crate::forbid!(self.stores[0].state().state().secret(0).is_none());
        crate::forbid!(self.stores[0].state().state().secret(1).is_none());
        crate::forbid!(self.stores[1].state().state().secret(0).is_none());
        crate::forbid!(self.stores[1].state().state().secret(1).is_some());
        crate::forbid!(self.stores[2].state().state().secret(0).is_some());
        crate::forbid!(self.stores[2].state().state().secret(1).is_none());

        let store = deserialize_store::<S>(&self.stores[0].serialize(SecretKnowledge::Some(0)))?;
        crate::forbid!(store.state().state().secret(0).is_none());
        crate::forbid!(store.state().state().secret(1).is_some());

        let store = deserialize_store::<S>(&self.stores[0].serialize(SecretKnowledge::Some(1)))?;
        crate::forbid!(store.state().state().secret(0).is_some());
        crate::forbid!(store.state().state().secret(1).is_none());

        let store = deserialize_store::<S>(&self.stores[1].serialize(SecretKnowledge::Some(0)))?;
        crate::forbid!(store.state().state().secret(0).is_none());
        crate::forbid!(store.state().state().secret(1).is_some());

        let store = deserialize_store::<S>(&self.stores[1].serialize(SecretKnowledge::Some(1)))?;
        crate::forbid!(store.state().state().secret(0).is_some());
        crate::forbid!(store.state().state().secret(1).is_some());

        let store = deserialize_store::<S>(&self.stores[2].serialize(SecretKnowledge::Some(0)))?;
        crate::forbid!(store.state().state().secret(0).is_some());
        crate::forbid!(store.state().state().secret(1).is_some());

        let store = deserialize_store::<S>(&self.stores[2].serialize(SecretKnowledge::Some(1)))?;
        crate::forbid!(store.state().state().secret(0).is_some());
        crate::forbid!(store.state().state().secret(1).is_none());

        Ok(())
    }

    fn flush(&mut self) -> Result<Vec<crate::ProofAction<crate::store::StoreState<S>>>, String> {
        let mut reveals = Vec::new();

        loop {
            let mut repeat = false;

            if let Some(diff) = self.queues[0]
                .try_borrow_mut()
                .map_err(|error| error.to_string())?
                .pop_front()
            {
                let diff = crate::Diff::deserialize(&diff)?;

                reveals.extend(diff.actions.clone());

                self.proof.apply(&diff)?;
                self.stores[1].apply(&diff)?;
                self.stores[2].apply(&diff)?;

                repeat = true;
            }

            if let Some(diff) = self.queues[1]
                .try_borrow_mut()
                .map_err(|error| error.to_string())?
                .pop_front()
            {
                let diff = crate::Diff::deserialize(&diff)?;

                reveals.extend(diff.actions.clone());

                self.proof.apply(&diff)?;
                self.stores[0].apply(&diff)?;
                self.stores[2].apply(&diff)?;

                repeat = true;
            }

            if let Some(diff) = self.queues[2]
                .try_borrow_mut()
                .map_err(|error| error.to_string())?
                .pop_front()
            {
                let diff = crate::Diff::deserialize(&diff)?;

                reveals.extend(diff.actions.clone());

                self.proof.apply(&diff)?;
                self.stores[0].apply(&diff)?;
                self.stores[1].apply(&diff)?;

                repeat = true;
            }

            if !repeat {
                break;
            }
        }

        Ok(reveals)
    }
}

#[cfg(not(feature = "no-crypto"))]
#[allow(clippy::unnecessary_wraps)]
fn generate_keys_and_subkeys<R: libsecp256k1_rand::RngCore>(
    randoms: &mut [R; 3],
) -> Result<([crate::crypto::SecretKey; 3], [crate::crypto::SecretKey; 2]), String> {
    Ok((
        [
            crate::crypto::SecretKey::random(&mut randoms[0]),
            crate::crypto::SecretKey::random(&mut randoms[1]),
            crate::crypto::SecretKey::random(&mut randoms[2]),
        ],
        [
            crate::crypto::SecretKey::random(&mut randoms[1]),
            crate::crypto::SecretKey::random(&mut randoms[2]),
        ],
    ))
}

#[cfg(feature = "no-crypto")]
fn generate_keys_and_subkeys<R: libsecp256k1_rand::RngCore>(
    randoms: &mut [R; 3],
) -> Result<([crate::crypto::SecretKey; 3], [crate::crypto::SecretKey; 2]), String> {
    Ok((
        [
            {
                let mut key = crate::crypto::SecretKey::default();

                randoms[0]
                    .try_fill_bytes(&mut key)
                    .map_err(|error| error.to_string())?;

                key
            },
            {
                let mut key = crate::crypto::SecretKey::default();

                randoms[1]
                    .try_fill_bytes(&mut key)
                    .map_err(|error| error.to_string())?;

                key
            },
            {
                let mut key = crate::crypto::SecretKey::default();

                randoms[2]
                    .try_fill_bytes(&mut key)
                    .map_err(|error| error.to_string())?;

                key
            },
        ],
        [
            {
                let mut subkey = crate::crypto::SecretKey::default();

                randoms[1]
                    .try_fill_bytes(&mut subkey)
                    .map_err(|error| error.to_string())?;

                subkey
            },
            {
                let mut subkey = crate::crypto::SecretKey::default();

                randoms[2]
                    .try_fill_bytes(&mut subkey)
                    .map_err(|error| error.to_string())?;

                subkey
            },
        ],
    ))
}

fn deserialize_store<S: crate::store::State>(
    data: &[u8],
) -> Result<crate::store::Store<S>, String> {
    let mut store = crate::store::Store::deserialize(
        data,
        false,
        |_, _| (),
        |_| unreachable!("{}:{}:{}", file!(), line!(), column!()),
        |_| (),
        |_, _| (),
        UnreachableRng,
    )?;

    store.flush()?;

    Ok(store)
}

fn deserialize_proof<S: crate::store::State>(
    data: &[u8],
    root: crate::RootProof<crate::store::StoreState<S>>,
) -> Result<crate::Proof<crate::store::StoreState<S>>, String> {
    let mut proof = crate::Proof::new(root);

    proof.deserialize(data)?;

    Ok(proof)
}

fn deserialize_root_proof<S: crate::store::State>(
    data: &[u8],
) -> Result<crate::RootProof<crate::store::StoreState<S>>, String> {
    crate::RootProof::deserialize(data)
}

struct UnreachableRng;

impl rand::RngCore for UnreachableRng {
    fn next_u32(&mut self) -> u32 {
        unreachable!("{}:{}:{}", file!(), line!(), column!());
    }

    fn next_u64(&mut self) -> u64 {
        unreachable!("{}:{}:{}", file!(), line!(), column!());
    }

    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        unreachable!("{}:{}:{}", file!(), line!(), column!());
    }

    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
        unreachable!("{}:{}:{}", file!(), line!(), column!());
    }
}
