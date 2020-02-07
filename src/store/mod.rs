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

#[cfg(feature = "std")]
use std::{
    cell::{Ref, RefCell},
    convert::TryInto,
    fmt::{Debug, Error, Formatter},
    future::Future,
    mem::size_of,
    ops::Deref,
    pin::Pin,
    ptr,
    rc::Rc,
    task,
    task::{Poll, RawWaker, RawWakerVTable, Waker},
};

#[cfg(not(feature = "std"))]
use {
    alloc::{
        fmt::{Debug, Error, Formatter},
        format,
        prelude::v1::*,
        rc::Rc,
        vec,
    },
    core::{
        cell::{Ref, RefCell},
        convert::TryInto,
        future::Future,
        mem::size_of,
        ops::Deref,
        pin::Pin,
        ptr, task,
        task::{Poll, RawWaker, RawWakerVTable, Waker},
    },
};

#[cfg(feature = "tester")]
mod tester;

#[cfg(feature = "tester")]
pub use tester::Tester;

/// Generates WebAssembly bindings for a [store::State].
#[macro_export]
macro_rules! bind {
    ($type:ty) => {
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub struct JsGame {
            store: $crate::store::Store<$type>,
            send: js_sys::Function,
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl JsGame {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new(
                player: Option<$crate::Player>,
                root: &[u8],
                secret: wasm_bindgen::JsValue,
                p2p: bool,
                ready: js_sys::Function,
                sign: js_sys::Function,
                send: js_sys::Function,
                log: js_sys::Function,
                random: js_sys::Function,
            ) -> Result<JsGame, wasm_bindgen::JsValue> {
                Ok(Self {
                    store: {
                        $crate::store::Store::new(
                            player,
                            root,
                            match player {
                                None => $crate::utils::from_js(secret)?,
                                Some(0) => [Some($crate::utils::from_js(secret)?), None],
                                Some(1) => [None, Some($crate::utils::from_js(secret)?)],
                                _ => return Err("player.is_some() && player.unwrap() >= 2".into()),
                            },
                            p2p,
                            move |state, secrets| {
                                if let Ok(state) = $crate::utils::to_js(state) {
                                    match secrets {
                                        [Some(secret1), Some(secret2)] => {
                                            drop(
                                                ready.call3(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                    &$crate::utils::to_js(secret1)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                    &$crate::utils::to_js(secret2)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                ),
                                            );
                                        }
                                        [Some(secret), None] | [None, Some(secret)] => {
                                            drop(
                                                ready.call2(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                    &$crate::utils::to_js(secret)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                ),
                                            );
                                        }
                                        [None, None] => {
                                            drop(ready.call1(&wasm_bindgen::JsValue::UNDEFINED, &state));
                                        }
                                    }
                                }
                            },
                            move |message| {
                                let data: Vec<_> = $crate::utils::from_js(
                                    sign.call1(
                                        &wasm_bindgen::JsValue::UNDEFINED,
                                        &$crate::utils::to_js(message)?,
                                    )
                                    .map_err(|error| format!("{:?}", error))?,
                                )?;

                                if data.len() != std::mem::size_of::<$crate::crypto::Signature>() {
                                    return Err(
                                        "data.len() != std::mem::size_of::<$crate::crypto::Signature>()"
                                            .to_string(),
                                    );
                                }

                                let mut signature = [0; std::mem::size_of::<$crate::crypto::Signature>()];
                                signature.copy_from_slice(&data);

                                Ok(signature)
                            },
                            {
                                let send = send.clone();

                                move |diff| {
                                    if let Ok(value) = &$crate::utils::to_js(&diff.serialize()) {
                                        drop(send.call1(&wasm_bindgen::JsValue::UNDEFINED, value));
                                    }
                                }
                            },
                            move |event| {
                                if let Ok(event) = $crate::utils::to_js(&*event) {
                                    drop(log.call1(&wasm_bindgen::JsValue::UNDEFINED, &event));
                                }
                            },
                            $crate::store::bindings::JsRng(random),
                        )
                        .map_err(wasm_bindgen::JsValue::from)?
                    },
                    send,
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn deserialize(
                data: &[u8],
                p2p: bool,
                ready: js_sys::Function,
                sign: js_sys::Function,
                send: js_sys::Function,
                log: js_sys::Function,
                random: js_sys::Function,
            ) -> Result<JsGame, wasm_bindgen::JsValue> {
                Ok(Self {
                    store: {
                        $crate::store::Store::deserialize(
                            data,
                            p2p,
                            move |state, secrets| {
                                if let Ok(state) = $crate::utils::to_js(state) {
                                    match secrets {
                                        [Some(secret1), Some(secret2)] => {
                                            drop(
                                                ready.call3(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                    &$crate::utils::to_js(secret1)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                    &$crate::utils::to_js(secret2)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                ),
                                            );
                                        }
                                        [Some(secret), None] | [None, Some(secret)] => {
                                            drop(
                                                ready.call2(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                    &$crate::utils::to_js(secret)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                ),
                                            );
                                        }
                                        [None, None] => {
                                            drop(ready.call1(&wasm_bindgen::JsValue::UNDEFINED, &state));
                                        }
                                    }
                                }
                            },
                            move |message| {
                                let data: Vec<_> = $crate::utils::from_js(
                                    sign.call1(
                                        &wasm_bindgen::JsValue::UNDEFINED,
                                        &$crate::utils::to_js(message)?,
                                    )
                                    .map_err(|error| format!("{:?}", error))?,
                                )?;

                                if data.len() != std::mem::size_of::<$crate::crypto::Signature>() {
                                    return Err(
                                        "data.len() != std::mem::size_of::<$crate::crypto::Signature>()"
                                            .to_string(),
                                    );
                                }

                                let mut signature = [0; std::mem::size_of::<$crate::crypto::Signature>()];
                                signature.copy_from_slice(&data);

                                Ok(signature)
                            },
                            {
                                let send = send.clone();

                                move |diff| {
                                    if let Ok(value) = &$crate::utils::to_js(&diff.serialize()) {
                                        drop(send.call1(&wasm_bindgen::JsValue::UNDEFINED, value));
                                    }
                                }
                            },
                            move |event| {
                                if let Ok(event) = $crate::utils::to_js(&*event) {
                                    drop(log.call1(&wasm_bindgen::JsValue::UNDEFINED, &event));
                                }
                            },
                            $crate::store::bindings::JsRng(random),
                        )
                        .map_err(wasm_bindgen::JsValue::from)?
                    },
                    send,
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn serialize(&self, player: Option<$crate::Player>) -> Vec<u8> {
                self.store.serialize(player)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn player(&self) -> Option<$crate::Player> {
                self.store.player()
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn id(&self) -> Vec<u8> {
                $crate::ID::serialize(self.store.state().id())
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn addresses(&self) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    &self
                        .store
                        .state()
                        .players()
                        .iter()
                        .map($crate::crypto::Addressable::eip55)
                        .collect::<Vec<_>>(),
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn hash(&self) -> String {
                $crate::utils::hex(self.store.hash())
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn state(&self) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    self.store
                        .state()
                        .state()
                        .state()
                        .ok_or(wasm_bindgen::JsValue::from(
                            "self.store.state().state().state().is_none()",
                        ))?,
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn secret(
                &self,
                player: $crate::Player,
            ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    &**self
                        .store
                        .state()
                        .state()
                        .secret(player)
                        .ok_or("self.store.state().state().secret(player).is_none()")?,
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter, js_name = pendingPlayer)]
            pub fn pending_player(&self) -> Result<Option<$crate::Player>, wasm_bindgen::JsValue> {
                if let $crate::store::StoreState::Pending { phase, .. } = self.store.state().state() {
                    match *phase
                        .try_borrow()
                        .map_err(|error| wasm_bindgen::JsValue::from(error.to_string()))?
                    {
                        $crate::store::Phase::RandomCommit => Ok(Some(0)),
                        $crate::store::Phase::RandomReply { .. } => Ok(Some(1)),
                        $crate::store::Phase::RandomReveal {
                            owner_hash: false, ..
                        } => Ok(Some(0)),
                        $crate::store::Phase::RandomReveal {
                            owner_hash: true, ..
                        } => Ok(None),
                        _ => unreachable!(),
                    }
                } else {
                    Err(wasm_bindgen::JsValue::from(
                        "self.store.state().state() != $crate::store::StoreState::Pending { .. }",
                    ))
                }
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn simulate(
                &self,
                player: Option<$crate::Player>,
                action: wasm_bindgen::JsValue,
            ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                let action: <$type as $crate::store::State>::Action = $crate::utils::from_js(action)?;

                Ok($crate::utils::to_js(
                    &self.store.state().state().simulate(player, &action)?,
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn flush(&mut self) -> Result<(), wasm_bindgen::JsValue> {
                Ok(self.store.flush()?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn dispatch(&mut self, action: wasm_bindgen::JsValue) -> Result<(), wasm_bindgen::JsValue> {
                let action: <$type as $crate::store::State>::Action = $crate::utils::from_js(action)?;

                let diff = self.store.diff(vec![$crate::ProofAction {
                    player: self.store.player(),
                    action: $crate::PlayerAction::Play($crate::store::StoreAction::Action(action)),
                }])?;

                self.send.call1(
                    &wasm_bindgen::JsValue::UNDEFINED,
                    &$crate::utils::to_js(&diff.serialize())?,
                )?;

                self.store.apply(&diff)?;

                Ok(())
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = dispatchTimeout)]
            pub fn dispatch_timeout(&mut self) -> Result<(), wasm_bindgen::JsValue> {
                self.store
                    .dispatch_timeout()
                    .map_err(wasm_bindgen::JsValue::from)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn apply(&mut self, diff: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
                Ok(self.store.apply(&$crate::Diff::deserialize(diff)?)?)
            }
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn certificate(address: &[u8]) -> Result<String, wasm_bindgen::JsValue> {
            Ok(<$type as $crate::store::State>::certificate(
                std::convert::TryInto::<_>::try_into(address)
                    .map_err(|error| wasm_bindgen::JsValue::from(format!("{}", error)))?,
            ))
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getRootProofPlayer)]
        pub fn root_proof_player(
            root: &[u8],
            player: &[u8],
        ) -> Result<$crate::Player, wasm_bindgen::JsValue> {
            if player.len() != std::mem::size_of::<$crate::crypto::Address>() {
                return Err("player.len() != std::mem::size_of::<$crate::crypto::Address>()".into());
            }

            $crate::RootProof::<$crate::store::StoreState<$type>>::deserialize(root)?
                .state()
                .player(std::convert::TryInto::<_>::try_into(player).map_err(|error| format!("{}", error))?)
                .ok_or("root.state().player(player).is_none()".into())
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getRootProofID)]
        pub fn root_proof_id(root: &[u8]) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
            Ok($crate::ID::serialize(
                $crate::RootProof::<$crate::store::StoreState<$type>>::deserialize(root)?
                    .state()
                    .id(),
            ))
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getDiffProof)]
        pub fn diff_proof(diff: &[u8]) -> Result<String, wasm_bindgen::JsValue> {
            Ok($crate::utils::hex($crate::Diff::<$crate::store::StoreAction<<$type as $crate::store::State>::Action>>::deserialize(diff)?.proof()))
        }
    };
}

#[cfg(feature = "bindings")]
/// WebAssembly-specific utilities
pub mod bindings;

/// Client [State] store
pub struct Store<S: State> {
    player: Option<crate::Player>,
    proof: crate::Proof<StoreState<S>>,
    p2p: bool,
    ready: Box<dyn FnMut(&S, [Option<&S::Secret>; 2])>,
    sign: Box<dyn FnMut(&[u8]) -> Result<crate::crypto::Signature, String>>,
    send: Box<dyn FnMut(&StoreDiff<S>)>,
    random: Box<dyn rand::RngCore>,
    seed: Option<Vec<u8>>,
}

impl<S: State> Store<S> {
    /// Constructs a new store for a given player.
    ///
    /// You should call [Store::flush] on the new store.
    ///
    /// `root` must have been constructed using [RootProof::serialize](crate::RootProof::serialize).
    pub fn new(
        player: Option<crate::Player>,
        root: &[u8],
        [secret1, secret2]: [Option<(S::Secret, [u8; 16])>; 2],
        p2p: bool,
        ready: impl FnMut(&S, [Option<&S::Secret>; 2]) + 'static,
        sign: impl FnMut(&[u8]) -> Result<crate::crypto::Signature, String> + 'static,
        send: impl FnMut(&StoreDiff<S>) + 'static,
        log: impl FnMut(&dyn Event) + 'static,
        random: impl rand::RngCore + 'static,
    ) -> Result<Self, String> {
        Ok(Self {
            player,
            proof: crate::Proof::new(crate::RootProof::<StoreState<S>>::deserialize_and_init(
                root,
                |state| {
                    if let StoreState::Ready { secrets, .. } = state {
                        *secrets = [
                            secret1
                                .map(|(secret, seed)| (secret, rand::SeedableRng::from_seed(seed))),
                            secret2
                                .map(|(secret, seed)| (secret, rand::SeedableRng::from_seed(seed))),
                        ];
                    } else {
                        unreachable!();
                    }

                    state.set_logger(Rc::new(RefCell::new(Logger::new(log))));
                },
            )?),
            p2p,
            ready: Box::new(ready),
            sign: Box::new(sign),
            send: Box::new(send),
            random: Box::new(random),
            seed: None,
        })
    }

    /// Constructs a store from its binary representation.
    ///
    /// You should call [Store::flush] on the new store.
    ///
    /// `data` must have been constructed using [Store::serialize].
    pub fn deserialize(
        mut data: &[u8],
        p2p: bool,
        ready: impl FnMut(&S, [Option<&S::Secret>; 2]) + 'static,
        sign: impl FnMut(&[u8]) -> Result<crate::crypto::Signature, String> + 'static,
        send: impl FnMut(&StoreDiff<S>) + 'static,
        log: impl FnMut(&dyn Event) + 'static,
        random: impl rand::RngCore + 'static,
    ) -> Result<Self, String> {
        crate::forbid!(data.len() < 1 + size_of::<u32>() + size_of::<u32>() + 1);

        let player = match crate::utils::read_u8(&mut data)? {
            0 => None,
            byte => Some(byte - 1),
        };

        let mut log = Logger::new(log);
        log.enabled = false;
        let log = Rc::new(RefCell::new(log));

        let secrets = {
            let secret1 = if crate::utils::read_u8_bool(&mut data)? {
                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                let secret = S::Secret::deserialize(&data[..size])?;
                data = &data[size..];

                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                let random = rand_xorshift::XorShiftRng::deserialize(&data[..size])?;
                data = &data[size..];

                Some((secret, random))
            } else {
                None
            };

            let secret2 = if crate::utils::read_u8_bool(&mut data)? {
                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                let secret = S::Secret::deserialize(&data[..size])?;
                data = &data[size..];

                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                let random = rand_xorshift::XorShiftRng::deserialize(&data[..size])?;
                data = &data[size..];

                Some((secret, random))
            } else {
                None
            };

            [secret1, secret2]
        };

        let size = crate::utils::read_u32_usize(&mut data)?;

        crate::forbid!(data.len() < size);

        let root =
            crate::RootProof::<StoreState<S>>::deserialize_and_init(&data[..size], |state| {
                if let StoreState::Ready {
                    secrets: state_secrets,
                    ..
                } = state
                {
                    *state_secrets = secrets;
                } else {
                    unreachable!();
                }

                state.set_logger(log.clone());
            })?;

        data = &data[size..];

        if let Some(player) = player {
            crate::forbid!(usize::from(player) >= root.state.players.len());
        }

        let secrets = {
            let secret1 = if crate::utils::read_u8_bool(&mut data)? {
                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                let secret = S::Secret::deserialize(&data[..size])?;
                data = &data[size..];

                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                let random = rand_xorshift::XorShiftRng::deserialize(&data[..size])?;
                data = &data[size..];

                Some((secret, random))
            } else {
                None
            };

            let secret2 = if crate::utils::read_u8_bool(&mut data)? {
                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                let secret = S::Secret::deserialize(&data[..size])?;
                data = &data[size..];

                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                let random = rand_xorshift::XorShiftRng::deserialize(&data[..size])?;
                data = &data[size..];

                Some((secret, random))
            } else {
                None
            };

            [secret1, secret2]
        };

        let size = crate::utils::read_u32_usize(&mut data)?;

        crate::forbid!(data.len() < size);
        let mut proof = crate::Proof::new(root);

        proof.deserialize_and_init(&data[..size], |state| {
            if let StoreState::Ready {
                secrets: state_secrets,
                ..
            } = state
            {
                *state_secrets = secrets;
            } else {
                unreachable!();
            }

            state.set_logger(log);
        })?;

        data = &data[size..];

        let seed = if crate::utils::read_u8_bool(&mut data)? {
            Some(data.to_vec())
        } else {
            crate::forbid!(!data.is_empty());

            None
        };

        Ok(Self {
            player,
            proof,
            p2p,
            ready: Box::new(ready),
            sign: Box::new(sign),
            send: Box::new(send),
            random: Box::new(random),
            seed,
        })
    }

    /// Generates a binary representation that can be used to reconstruct the store for a given
    /// player.
    ///
    /// See [Store::deserialize].
    pub fn serialize(&self, player: Option<crate::Player>) -> Vec<u8> {
        let root = self.proof.root.serialize();
        let proof = self.proof.serialize();

        let mut data = Vec::with_capacity(
            1 + 1
                + size_of::<u32>()
                + root.len()
                + 1
                + size_of::<u32>()
                + proof.len()
                + 1
                + self.seed.as_ref().map_or(0, Vec::len),
        );

        crate::utils::write_u8(
            &mut data,
            match self.player {
                None => match player {
                    None => 0,
                    Some(player) => 1 + player,
                },
                Some(player) => 1 + player,
            },
        );

        if let StoreState::Ready { secrets, .. } = &self.proof.root.state.state {
            for (i, secret) in secrets.iter().enumerate() {
                if player.is_none() || player == Some(i.try_into().unwrap()) {
                    match secret {
                        Some((secret, random)) => {
                            crate::utils::write_u8_bool(&mut data, true);

                            let secret = secret.serialize();
                            crate::utils::write_u32_usize(&mut data, secret.len()).unwrap();
                            data.extend(secret);

                            let random = random.serialize();
                            crate::utils::write_u32_usize(&mut data, random.len()).unwrap();
                            data.extend(random);
                        }
                        None => crate::utils::write_u8_bool(&mut data, false),
                    }
                } else {
                    crate::utils::write_u8_bool(&mut data, false);
                }
            }
        } else {
            unreachable!();
        }

        crate::utils::write_u32_usize(&mut data, root.len()).unwrap();
        data.extend(root);

        if let StoreState::Ready { secrets, .. } = &self
            .proof
            .proofs
            .iter()
            .find(|proof| match proof {
                Some(proof) => proof.range.start == 0,
                None => false,
            })
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .state
            .state
        {
            for (i, secret) in secrets.iter().enumerate() {
                if player.is_none() || player == Some(i.try_into().unwrap()) {
                    match secret {
                        Some((secret, random)) => {
                            crate::utils::write_u8_bool(&mut data, true);

                            let secret = secret.serialize();
                            crate::utils::write_u32_usize(&mut data, secret.len()).unwrap();
                            data.extend(secret);

                            let random = random.serialize();
                            crate::utils::write_u32_usize(&mut data, random.len()).unwrap();
                            data.extend(random);
                        }
                        None => crate::utils::write_u8_bool(&mut data, false),
                    }
                } else {
                    crate::utils::write_u8_bool(&mut data, false);
                }
            }
        } else {
            unreachable!();
        }

        crate::utils::write_u32_usize(&mut data, proof.len()).unwrap();
        data.extend(proof);

        if player.is_none() || player == Some(0) {
            if let Some(seed) = &self.seed {
                crate::utils::write_u8_bool(&mut data, true);
                data.extend(seed);
            } else {
                crate::utils::write_u8_bool(&mut data, false);
            }
        } else {
            crate::utils::write_u8_bool(&mut data, false);
        }

        data
    }

    /// Gets the player associated with the store.
    pub fn player(&self) -> Option<crate::Player> {
        self.player
    }

    /// Gets the hash of the store's proof.
    pub fn hash(&self) -> &crate::crypto::Hash {
        &self.proof.hash
    }

    /// Gets the state of the store's proof.
    pub fn state(&self) -> &crate::ProofState<StoreState<S>> {
        &self.proof.state
    }

    /// Dispatches an action that will continue a stalled commit-reveal sequence.
    /// Only call this if the pending player isn't live.
    /// Only the owner can call this.
    pub fn dispatch_timeout(&mut self) -> Result<(), String> {
        crate::forbid!(self.player.is_some());

        let action = match &self.proof.state.state {
            StoreState::Pending { phase, .. } => match &*phase.try_borrow().unwrap() {
                Phase::RandomCommit => {
                    let seed = {
                        let mut seed =
                            <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                        self.random.fill_bytes(&mut seed);
                        seed
                    };

                    self.seed = Some(seed.to_vec());

                    Some(crate::ProofAction {
                        player: None,
                        action: crate::PlayerAction::Play(StoreAction::<S::Action>::RandomCommit(
                            tiny_keccak::keccak256(&seed),
                        )),
                    })
                }
                Phase::RandomReply { .. } => {
                    let seed = {
                        let mut seed =
                            <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                        self.random.fill_bytes(&mut seed);
                        seed
                    };

                    Some(crate::ProofAction {
                        player: None,
                        action: crate::PlayerAction::Play(StoreAction::<S::Action>::RandomReply(
                            seed.to_vec(),
                        )),
                    })
                }
                Phase::RandomReveal {
                    owner_hash: false, ..
                } => {
                    let seed = {
                        let mut seed =
                            <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                        self.random.fill_bytes(&mut seed);
                        seed
                    };

                    Some(crate::ProofAction {
                        player: None,
                        action: crate::PlayerAction::Play(StoreAction::<S::Action>::RandomReveal(
                            seed.to_vec(),
                        )),
                    })
                }
                _ => None,
            },
            StoreState::Ready { state, secrets, .. } => {
                self.seed = None;

                (self.ready)(
                    state,
                    [
                        secrets[0].as_ref().map(|(secret, _)| secret),
                        secrets[1].as_ref().map(|(secret, _)| secret),
                    ],
                );

                None
            }
        };

        if let Some(action) = action {
            let diff = self.diff(vec![action])?;

            (self.send)(&diff);

            self.apply(&diff)?;
        }

        Ok(())
    }

    /// Dispatches any actions the client is required to send.
    pub fn flush(&mut self) -> Result<(), String> {
        let action = match &self.proof.state.state {
            StoreState::Pending { phase, secrets, .. } => {
                match (&*phase.try_borrow().unwrap(), self.player) {
                    (Phase::RandomCommit, Some(0)) => {
                        let seed = {
                            let mut seed =
                                <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                            self.random.fill_bytes(&mut seed);
                            seed
                        };

                        self.seed = Some(seed.to_vec());

                        Some(crate::ProofAction {
                            player: Some(0),
                            action: crate::PlayerAction::Play(
                                StoreAction::<S::Action>::RandomCommit(tiny_keccak::keccak256(
                                    &seed,
                                )),
                            ),
                        })
                    }
                    (Phase::RandomReply { .. }, Some(1)) => {
                        let seed = {
                            let mut seed =
                                <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                            self.random.fill_bytes(&mut seed);
                            seed
                        };

                        Some(crate::ProofAction {
                            player: Some(1),
                            action: crate::PlayerAction::Play(
                                StoreAction::<S::Action>::RandomReply(seed.to_vec()),
                            ),
                        })
                    }
                    (
                        Phase::RandomReveal {
                            hash,
                            owner_hash: false,
                            ..
                        },
                        Some(0),
                    )
                    | (
                        Phase::RandomReveal {
                            hash,
                            owner_hash: true,
                            ..
                        },
                        None,
                    ) => {
                        if let Some(seed) = &self.seed {
                            crate::forbid!(&tiny_keccak::keccak256(seed) != hash);

                            Some(crate::ProofAction {
                                player: self.player,
                                action: crate::PlayerAction::Play(
                                    StoreAction::<S::Action>::RandomReveal(seed.to_vec()),
                                ),
                            })
                        } else {
                            return Err("self.seed.is_none()".to_string());
                        }
                    }
                    (
                        Phase::Reveal {
                            request:
                                RevealRequest {
                                    player,
                                    reveal,
                                    verify,
                                },
                            ..
                        },
                        _,
                    ) => {
                        if self.player.is_none() && !self.p2p
                            || self.player == Some(*player) && self.p2p
                        {
                            if let Some(secret) = &secrets[usize::from(*player)] {
                                let secret = reveal(
                                    &secret.try_borrow().map_err(|error| error.to_string())?.0,
                                );

                                crate::forbid!(!verify(&secret));

                                Some(crate::ProofAction {
                                    player: self.player,
                                    action: crate::PlayerAction::Play(
                                        StoreAction::<S::Action>::Reveal(secret),
                                    ),
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            StoreState::Ready { state, secrets, .. } => {
                self.seed = None;

                (self.ready)(
                    state,
                    [
                        secrets[0].as_ref().map(|(secret, _)| secret),
                        secrets[1].as_ref().map(|(secret, _)| secret),
                    ],
                );

                None
            }
        };

        if let Some(action) = action {
            let diff = self.diff(vec![action])?;

            (self.send)(&diff);

            self.apply(&diff)?;
        }

        Ok(())
    }

    /// Verifies and applies a cryptographically constructed diff to the store.
    ///
    /// `diff` must have been constructed using [Store::diff] on a store with the same state.
    pub fn apply(&mut self, diff: &StoreDiff<S>) -> Result<(), String> {
        self.proof
            .state
            .state
            .logger()
            .try_borrow_mut()
            .map_err(|error| error.to_string())?
            .enabled = true;

        self.proof.apply(diff)?;

        self.flush()
    }

    /// Generates a diff that can be applied to a store with the same state.
    ///
    /// See [Store::apply].
    pub fn diff(
        &mut self,
        actions: Vec<crate::ProofAction<StoreAction<S::Action>>>,
    ) -> Result<StoreDiff<S>, String> {
        self.proof
            .state
            .state
            .logger()
            .try_borrow_mut()
            .map_err(|error| error.to_string())?
            .enabled = false;

        self.proof.diff(actions, &mut self.sign)
    }
}

type StoreDiff<S> = crate::Diff<StoreAction<<S as State>::Action>>;

#[doc(hidden)]
pub enum StoreState<S: State> {
    Ready {
        state: S,
        secrets: [Option<(S::Secret, rand_xorshift::XorShiftRng)>; 2],
        nonce: usize,
        logger: Rc<RefCell<Logger>>,
    },
    Pending {
        state: Pin<Box<dyn Future<Output = (S, Context<S>)>>>,
        secrets: [Option<Rc<RefCell<(S::Secret, rand_xorshift::XorShiftRng)>>>; 2],
        phase: Rc<RefCell<Phase<S>>>,
        logger: Rc<RefCell<Logger>>,
    },
}

impl<S: State> StoreState<S> {
    pub fn new(state: S) -> Self {
        Self::Ready {
            state,
            secrets: Default::default(),
            nonce: Default::default(),
            logger: Rc::new(RefCell::new(Logger::new(|_| ()))),
        }
    }

    pub fn state(&self) -> Option<&S> {
        if let Self::Ready { state, .. } = self {
            Some(state)
        } else {
            None
        }
    }

    pub fn secret<'a>(
        &'a self,
        player: crate::Player,
    ) -> Option<Box<dyn Deref<Target = S::Secret> + 'a>> {
        match self {
            Self::Ready { secrets, .. } => secrets[usize::from(player)]
                .as_ref()
                .map(|(secret, _)| Box::new(secret) as Box<dyn Deref<Target = S::Secret>>),

            Self::Pending { secrets, .. } => {
                secrets[usize::from(player)].as_ref().and_then(|secret| {
                    secret
                        .try_borrow()
                        .map(|secret| {
                            Box::new(Ref::map(secret, |(secret, _)| secret))
                                as Box<dyn Deref<Target = S::Secret>>
                        })
                        .ok()
                })
            }
        }
    }

    pub fn simulate(
        &self,
        player: Option<crate::Player>,
        action: &S::Action,
    ) -> Result<Log, String> {
        if let Self::Ready { state, secrets, .. } = self {
            let events = Rc::new(RefCell::new(Vec::new()));

            Ok({
                let mut state = Self::Ready {
                    state: state.clone(),
                    secrets: secrets.clone(),
                    nonce: Default::default(),
                    logger: Rc::new(RefCell::new(Logger::new({
                        let events = events.clone();

                        move |event| {
                            events
                                .try_borrow_mut()
                                .unwrap()
                                .push(dyn_clone::clone_box(event))
                        }
                    }))),
                };

                crate::State::apply(&mut state, player, &StoreAction::Action(action.clone()))?;

                let mut complete = true;

                while let Self::Pending {
                    state: pending,
                    secrets,
                    phase,
                    logger,
                } = state
                {
                    let (player, secret) = if let Phase::Reveal {
                        request: RevealRequest { player, reveal, .. },
                        ..
                    } = &*phase.try_borrow().unwrap()
                    {
                        (
                            *player,
                            if let Some(secret) = &secrets[usize::from(*player)] {
                                reveal(&secret.try_borrow().unwrap().0)
                            } else {
                                complete = false;

                                break;
                            },
                        )
                    } else {
                        complete = false;

                        break;
                    };

                    state = Self::Pending {
                        state: pending,
                        secrets,
                        phase,
                        logger,
                    };

                    crate::State::apply(&mut state, Some(player), &StoreAction::Reveal(secret))?;
                }

                if complete {
                    Log::Complete
                } else {
                    Log::Incomplete
                }
            }(Rc::try_unwrap(events).unwrap().into_inner()))
        } else {
            Err("self != StoreState::Ready { .. }".to_string())
        }
    }

    fn logger(&self) -> &Rc<RefCell<Logger>> {
        match self {
            Self::Ready { logger, .. } | Self::Pending { logger, .. } => logger,
        }
    }

    fn set_logger(&mut self, logger: Rc<RefCell<Logger>>) {
        match self {
            Self::Ready {
                logger: state_logger,
                ..
            }
            | Self::Pending {
                logger: state_logger,
                ..
            } => *state_logger = logger,
        }
    }
}

impl<S: State> crate::State for StoreState<S> {
    type ID = S::ID;
    type Nonce = S::Nonce;
    type Action = StoreAction<S::Action>;

    fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        crate::forbid!(data.len() < size_of::<u32>());

        Ok(Self::Ready {
            state: S::deserialize(&data[..data.len() - size_of::<u32>()])?,
            secrets: Default::default(),
            nonce: {
                data = &data[data.len() - size_of::<u32>()..];
                crate::utils::read_u32_usize(&mut data)?
            },
            logger: Rc::new(RefCell::new(Logger::new(|_| ()))),
        })
    }

    fn is_serializable(&self) -> bool {
        match self {
            Self::Ready { state, nonce, .. } => {
                TryInto::<u32>::try_into(*nonce).is_ok() && state.is_serializable()
            }
            _ => false,
        }
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        match self {
            Self::Ready { state, nonce, .. } => State::serialize(state).and_then(|mut state| {
                crate::utils::write_u32_usize(&mut state, *nonce)
                    .ok()
                    .and(Some(state))
            }),
            _ => None,
        }
    }

    fn apply(
        &mut self,
        player: Option<crate::Player>,
        action: &Self::Action,
    ) -> Result<(), String> {
        let mut handled = false;

        if let Self::Action::Action(action) = action {
            if let Self::Ready { state, .. } = self {
                state.verify(player, action)?;
            }
        }

        replace_with::replace_with_or_abort(self, |state| {
            if let Self::Action::Action(action) = action {
                if let Self::Ready {
                    state,
                    secrets: [secret1, secret2],
                    nonce,
                    logger,
                } = state
                {
                    handled = true;

                    let phase = Rc::new(RefCell::new(Phase::Idle {
                        random: None,
                        secret: None,
                    }));

                    let secrets = [
                        secret1.map(|secret| Rc::new(RefCell::new(secret))),
                        secret2.map(|secret| Rc::new(RefCell::new(secret))),
                    ];

                    Self::Pending {
                        state: state.apply(
                            player,
                            action,
                            Context {
                                phase: phase.clone(),
                                secrets: secrets.clone(),
                                nonce,
                                logger: logger.clone(),
                            },
                        ),
                        secrets,
                        phase,
                        logger,
                    }
                } else {
                    state
                }
            } else {
                state
            }
        });

        if !handled {
            match (&self, action) {
                (Self::Ready { .. }, Self::Action::Action(_)) => unreachable!(),

                (Self::Pending { phase: context, .. }, Self::Action::RandomCommit(hash)) => {
                    let phase = context.try_borrow().map_err(|error| error.to_string())?;

                    if let Phase::RandomCommit = *phase {
                        drop(phase);

                        crate::forbid!(player != None && player != Some(0));

                        context.replace(Phase::RandomReply {
                            hash: *hash,
                            owner_hash: player.is_none(),
                        });
                    } else {
                        return Err("context.try_borrow().map_err(|error| error.to_string())? != Phase::RandomCommit".to_string());
                    }
                }

                (Self::Pending { phase: context, .. }, Self::Action::RandomReply(seed)) => {
                    let phase = context.try_borrow().map_err(|error| error.to_string())?;

                    if let Phase::RandomReply { hash, owner_hash } = *phase {
                        drop(phase);

                        crate::forbid!(player != None && player != Some(1));

                        context.replace(Phase::RandomReveal {
                            hash,
                            owner_hash,
                            reply: seed.to_vec(),
                        });
                    } else {
                        return Err("context.try_borrow().map_err(|error| error.to_string())? != Phase::RandomReply { .. }".to_string());
                    }
                }

                (Self::Pending { phase: context, .. }, Self::Action::RandomReveal(seed)) => {
                    let phase = context.try_borrow().map_err(|error| error.to_string())?;

                    if let Phase::RandomReveal {
                        hash,
                        owner_hash,
                        reply,
                    } = &*phase
                    {
                        if *owner_hash {
                            crate::forbid!(player != None);
                        } else {
                            crate::forbid!(player != None && player != Some(0));
                        }

                        if player.is_some() || *owner_hash {
                            crate::forbid!(tiny_keccak::keccak256(seed) != *hash);
                        }

                        let seed = reply
                            .iter()
                            .zip(seed)
                            .map(|(x, y)| x ^ y)
                            .collect::<Vec<_>>()
                            .as_slice()
                            .try_into()
                            .map_err(|error| format!("{}", error))?;

                        drop(phase);

                        context.replace(Phase::Idle {
                            random: Some(Rc::new(RefCell::new(rand::SeedableRng::from_seed(seed)))),
                            secret: None,
                        });
                    } else {
                        return Err("context.try_borrow().map_err(|error| error.to_string())? != Phase::RandomReveal { .. }".to_string());
                    }
                }

                (Self::Pending { phase: context, .. }, Self::Action::Reveal(secret)) => {
                    let phase = context.try_borrow().map_err(|error| error.to_string())?;

                    if let Phase::Reveal {
                        random,
                        request:
                            RevealRequest {
                                player: revealer,
                                verify,
                                ..
                            },
                    } = &*phase
                    {
                        crate::forbid!(player != None && player != Some(*revealer));
                        crate::forbid!(!verify(secret));

                        let random = random.clone();

                        drop(phase);

                        context.replace(Phase::Idle {
                            random,
                            secret: Some(secret.clone()),
                        });
                    } else {
                        return Err("context.try_borrow().map_err(|error| error.to_string())? != Phase::Reveal { .. }".to_string());
                    }
                }

                (Self::Pending { .. }, Self::Action::Action(action)) => {
                    return Err(format!(
                        "StoreState::Pending can't apply StoreAction::Action({:?})",
                        action
                    ))
                }

                (Self::Ready { .. }, action) => {
                    return Err(format!("StoreState::Pending can't apply {:?}", action))
                }
            }
        }

        replace_with::replace_with_or_abort(self, |state| {
            if let Self::Pending {
                mut state,
                secrets: [secret1, secret2],
                phase,
                logger,
            } = state
            {
                if let Poll::Ready((state, context)) = state
                    .as_mut()
                    .poll(&mut task::Context::from_waker(&phantom_waker()))
                {
                    drop(context.secrets);

                    Self::Ready {
                        state,
                        secrets: [
                            secret1.map(|secret| Rc::try_unwrap(secret).ok().unwrap().into_inner()),
                            secret2.map(|secret| Rc::try_unwrap(secret).ok().unwrap().into_inner()),
                        ],
                        nonce: context.nonce,
                        logger,
                    }
                } else {
                    Self::Pending {
                        state,
                        secrets: [secret1, secret2],
                        phase,
                        logger,
                    }
                }
            } else {
                state
            }
        });

        Ok(())
    }
}

impl<S: State> Clone for StoreState<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Ready {
                state,
                secrets,
                nonce,
                logger,
            } => Self::Ready {
                state: state.clone(),
                secrets: secrets.clone(),
                nonce: *nonce,
                logger: logger.clone(),
            },
            _ => panic!("StoreState::Pending {{ .. }}.clone()"),
        }
    }
}

/// Simulation event log
#[derive(serde::Serialize)]
#[serde(tag = "status", content = "events")]
pub enum Log {
    /// A log for a complete transition.
    #[serde(rename = "complete")]
    Complete(Vec<Box<dyn Event>>),

    /// A log for an incomplete transition.
    #[serde(rename = "incomplete")]
    Incomplete(Vec<Box<dyn Event>>),
}

#[doc(hidden)]
#[derive(Clone)]
pub enum StoreAction<A: crate::Action> {
    Action(A),
    RandomCommit(crate::crypto::Hash),
    RandomReply(Vec<u8>),
    RandomReveal(Vec<u8>),
    Reveal(Vec<u8>),
}

impl<A: crate::Action> crate::Action for StoreAction<A> {
    fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        match crate::utils::read_u8(&mut data)? {
            0 => Ok(Self::Action(A::deserialize(data)?)),
            1 => Ok(Self::RandomCommit(
                data.try_into().map_err(|error| format!("{}", error))?,
            )),
            2 => Ok(Self::RandomReply(data.to_vec())),
            3 => Ok(Self::RandomReveal(data.to_vec())),
            4 => Ok(Self::Reveal(data.to_vec())),
            byte => Err(format!("byte == {}", byte)),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        match self {
            Self::Action(action) => {
                crate::utils::write_u8(&mut data, 0);
                data.extend(action.serialize());
            }
            Self::RandomCommit(hash) => {
                crate::utils::write_u8(&mut data, 1);
                data.extend(hash);
            }
            Self::RandomReply(reply) => {
                crate::utils::write_u8(&mut data, 2);
                data.extend(reply);
            }
            Self::RandomReveal(seed) => {
                crate::utils::write_u8(&mut data, 3);
                data.extend(seed);
            }
            Self::Reveal(secret) => {
                crate::utils::write_u8(&mut data, 4);
                data.extend(secret);
            }
        }

        data
    }
}

impl<A: crate::Action + Debug> Debug for StoreAction<A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Action(action) => {
                if f.alternate() {
                    write!(f, "StoreAction::Action({:#?})", action)
                } else {
                    write!(f, "StoreAction::Action({:?})", action)
                }
            }
            Self::RandomCommit(data) => {
                write!(f, "StoreAction::RandomCommit({})", crate::utils::hex(data))
            }
            Self::RandomReply(data) => {
                write!(f, "StoreAction::RandomReply({})", crate::utils::hex(data))
            }
            Self::RandomReveal(data) => {
                write!(f, "StoreAction::RandomReveal({})", crate::utils::hex(data))
            }
            Self::Reveal(data) => write!(f, "StoreAction::Reveal({})", crate::utils::hex(data)),
        }
    }
}

/// Domain-specific store state trait
pub trait State: Clone {
    /// Identifier type
    type ID: crate::ID;

    /// Nonce type
    type Nonce: crate::Nonce;

    /// Action type
    type Action: crate::Action + Debug;

    /// Secret type
    type Secret: Secret;

    /// Formats the message that must be signed in order to certify the subkey for a given address.
    fn certificate(address: &crate::crypto::Address) -> String {
        format!(
            "Sign to play! This won't cost anything.\n\n{}\n",
            crate::crypto::Addressable::eip55(address)
        )
    }

    /// Constructs a state from its binary representation.
    ///
    /// `data` must have been constructed using [State::serialize].
    fn deserialize(data: &[u8]) -> Result<Self, String>;

    /// Checks if the state has a binary representation.
    ///
    /// This should be implemented whenever possible for improved performance.
    /// The return value must agree with [State::serialize].
    fn is_serializable(&self) -> bool {
        self.serialize().is_some()
    }

    /// Generates a binary representation that can be used to reconstruct the state.
    ///
    /// See [State::deserialize] and [State::is_serializable].
    fn serialize(&self) -> Option<Vec<u8>>;

    /// Verifies if an action by a given player is valid for the state.
    fn verify(&self, player: Option<crate::Player>, action: &Self::Action) -> Result<(), String>;

    /// Applies an action by a given player to the state.
    fn apply(
        self,
        player: Option<crate::Player>,
        action: &Self::Action,
        context: Context<Self>,
    ) -> Pin<Box<dyn Future<Output = (Self, Context<Self>)>>>;
}

/// Domain-specific store state secret trait
pub trait Secret: Clone {
    /// Constructs a state secret from its binary representation.
    ///
    /// `data` must have been constructed using [Secret::serialize].
    fn deserialize(data: &[u8]) -> Result<Self, String>;

    /// Generates a binary representation that can be used to reconstruct the state secret.
    ///
    /// See [Secret::deserialize].
    fn serialize(&self) -> Vec<u8>;
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + Clone> Secret for T {
    fn deserialize(data: &[u8]) -> Result<Self, String> {
        serde_cbor::from_slice(data).map_err(|error| error.to_string())
    }

    fn serialize(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).unwrap()
    }
}

/// [State::apply] utilities
pub struct Context<S: State> {
    phase: Rc<RefCell<Phase<S>>>,
    secrets: [Option<Rc<RefCell<(S::Secret, rand_xorshift::XorShiftRng)>>>; 2],
    nonce: usize,
    logger: Rc<RefCell<Logger>>,
}

impl<S: State> Context<S> {
    /// Mutates a player's secret information.
    pub fn mutate_secret(
        &mut self,
        player: crate::Player,
        mutate: impl Fn(&mut S::Secret, &mut dyn rand::RngCore, &mut dyn FnMut(&dyn Event)),
    ) {
        self.nonce += 1;

        let mut logger = self.logger.try_borrow_mut();

        let log = if let Ok(logger) = &mut logger {
            if logger.enabled && self.nonce > logger.nonce {
                logger.nonce = self.nonce;

                true
            } else {
                false
            }
        } else {
            false
        };

        if let Some(secret) = &self.secrets[usize::from(player)] {
            let (secret, random) = &mut *secret.try_borrow_mut().unwrap();

            if log {
                mutate(secret, random, &mut logger.unwrap().log);
            } else {
                mutate(secret, random, &mut |_| ());
            }
        }
    }

    /// Requests a player's secret information.
    ///
    /// The random number generator is re-seeded after this call to prevent players from influencing the randomness of the state via trial and error.
    ///
    /// See [Context::reveal_unique] for a faster non-re-seeding version of this method.
    pub async fn reveal<T: Secret>(
        &mut self,
        player: crate::Player,
        reveal: impl Fn(&S::Secret) -> T + 'static,
        verify: impl Fn(&T) -> bool + 'static,
    ) -> T {
        self.phase.replace(Phase::Reveal {
            random: None,
            request: RevealRequest {
                player,
                reveal: Box::new(move |secret| reveal(secret).serialize()),
                verify: Box::new(move |data| {
                    if let Ok(secret) = T::deserialize(data) {
                        verify(&secret)
                    } else {
                        false
                    }
                }),
            },
        });

        let data: Vec<u8> = RevealFuture(self.phase.clone()).await;

        T::deserialize(&data).unwrap()
    }

    /// Requests a player's secret information.
    ///
    /// The random number generator is not re-seeded after this call, so care must be taken to guarantee that the verify function accepts only one possible input.
    /// Without this guarantee, players can influence the randomness of the state via trial and error.
    ///
    /// See [Context::reveal] for a slower re-seeding version of this method.
    pub async fn reveal_unique<T: Secret>(
        &mut self,
        player: crate::Player,
        reveal: impl Fn(&S::Secret) -> T + 'static,
        verify: impl Fn(&T) -> bool + 'static,
    ) -> T {
        let random = if let Phase::Idle { random, .. } = &*self.phase.try_borrow().unwrap() {
            random.clone()
        } else {
            None
        };

        self.phase.replace(Phase::Reveal {
            random,
            request: RevealRequest {
                player,
                reveal: Box::new(move |secret| reveal(secret).serialize()),
                verify: Box::new(move |data| {
                    if let Ok(secret) = T::deserialize(data) {
                        verify(&secret)
                    } else {
                        false
                    }
                }),
            },
        });

        let data: Vec<u8> = RevealFuture(self.phase.clone()).await;

        T::deserialize(&data).unwrap()
    }

    /// Constructs a random number generator via commit-reveal.
    pub fn random(&mut self) -> impl Future<Output = impl rand::Rng> {
        let phase = self.phase.try_borrow().unwrap();

        if let Phase::Idle { random: None, .. } = *phase {
            drop(phase);

            self.phase.replace(Phase::RandomCommit);
        }

        SharedXorShiftRngFuture(self.phase.clone())
    }

    /// Logs an event.
    pub fn log(&mut self, event: &impl Event) {
        if let Ok(mut logger) = self.logger.try_borrow_mut() {
            self.nonce += 1;

            logger.log(self.nonce, event);
        }
    }

    #[doc(hidden)]
    pub fn new(
        phase: Rc<RefCell<Phase<S>>>,
        secrets: [Option<Rc<RefCell<(S::Secret, rand_xorshift::XorShiftRng)>>>; 2],
        log: impl FnMut(&dyn Event) + 'static,
    ) -> Self {
        Self {
            phase,
            secrets,
            nonce: Default::default(),
            logger: Rc::new(RefCell::new(Logger::new(log))),
        }
    }
}

#[doc(hidden)]
pub struct Logger {
    log: Box<dyn FnMut(&dyn Event)>,
    nonce: usize,
    enabled: bool,
}

impl Logger {
    pub fn new(log: impl FnMut(&dyn Event) + 'static) -> Self {
        Self {
            log: Box::new(log),
            nonce: Default::default(),
            enabled: true,
        }
    }

    fn log(&mut self, nonce: usize, event: &dyn Event) {
        if self.enabled && nonce > self.nonce {
            self.nonce = nonce;

            (self.log)(event);
        }
    }
}

/// [Context::log] event trait
pub trait Event: erased_serde::Serialize + dyn_clone::DynClone + Debug + 'static {}

impl<T: serde::Serialize + Clone + Debug + 'static> Event for T {}

impl<'a> serde::Serialize for dyn Event + 'a {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        erased_serde::serialize(self, serializer)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum Phase<S: State> {
    Idle {
        random: Option<Rc<RefCell<rand_xorshift::XorShiftRng>>>,
        secret: Option<Vec<u8>>,
    },
    RandomCommit,
    RandomReply {
        hash: crate::crypto::Hash,
        owner_hash: bool,
    },
    RandomReveal {
        hash: crate::crypto::Hash,
        owner_hash: bool,
        reply: Vec<u8>,
    },
    Reveal {
        random: Option<Rc<RefCell<rand_xorshift::XorShiftRng>>>,
        request: RevealRequest<S>,
    },
}

#[doc(hidden)]
pub struct RevealRequest<S: State> {
    pub player: crate::Player,
    pub reveal: Box<dyn Fn(&S::Secret) -> Vec<u8>>,
    pub verify: Box<dyn Fn(&[u8]) -> bool>,
}

impl<S: State> Debug for RevealRequest<S> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "RevealRequest {{ player: {} }}", self.player)
    }
}

struct SharedXorShiftRngFuture<S: State>(Rc<RefCell<Phase<S>>>);

impl<S: State> Future for SharedXorShiftRngFuture<S> {
    type Output = SharedXorShiftRng;

    fn poll(self: Pin<&mut Self>, _: &mut task::Context) -> Poll<Self::Output> {
        if let Ok(phase) = self.0.try_borrow() {
            if let Phase::Idle {
                random: Some(random),
                ..
            } = &*phase
            {
                Poll::Ready(SharedXorShiftRng(random.clone()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

struct SharedXorShiftRng(Rc<RefCell<rand_xorshift::XorShiftRng>>);

impl rand::RngCore for SharedXorShiftRng {
    fn next_u32(&mut self) -> u32 {
        self.0.try_borrow_mut().unwrap().next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.try_borrow_mut().unwrap().next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.try_borrow_mut().unwrap().fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0.try_borrow_mut().unwrap().try_fill_bytes(dest)
    }
}

struct RevealFuture<S: State>(Rc<RefCell<Phase<S>>>);

impl<S: State> Future for RevealFuture<S> {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, _: &mut task::Context) -> Poll<Self::Output> {
        if let Ok(phase) = self.0.try_borrow() {
            if let Phase::Idle {
                secret: Some(secret),
                ..
            } = &*phase
            {
                Poll::Ready(secret.clone())
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

fn phantom_waker() -> Waker {
    unsafe {
        Waker::from_raw(RawWaker::new(
            ptr::null(),
            &RawWakerVTable::new(
                |_| panic!("Waker::clone"),
                |_| panic!("Waker::wake"),
                |_| panic!("Waker::wake_by_ref"),
                |_| (),
            ),
        ))
    }
}
