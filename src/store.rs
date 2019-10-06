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

use replace_with::replace_with_or_abort;
use serde::Serialize;

#[cfg(feature = "std")]
use std::{
    cell::RefCell,
    fmt::{Debug, Error, Formatter},
    future::Future,
    mem::size_of,
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
        cell::RefCell,
        future::Future,
        mem::size_of,
        pin::Pin,
        ptr, task,
        task::{Poll, RawWaker, RawWakerVTable, Waker},
    },
};

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
                            Box::new(move |state| {
                                if let Ok(state) = wasm_bindgen::JsValue::from_serde(state) {
                                    drop(ready.call1(&wasm_bindgen::JsValue::UNDEFINED, &state));
                                }
                            }),
                            Box::new(move |message| {
                                let data: Vec<_> = sign
                                    .call1(
                                        &wasm_bindgen::JsValue::UNDEFINED,
                                        &wasm_bindgen::JsValue::from_serde(message)
                                            .map_err(|error| format!("{}", error))?,
                                    )
                                    .map_err(|error| format!("{:?}", error))?
                                    .into_serde()
                                    .map_err(|error| format!("{}", error))?;

                                if data.len() != std::mem::size_of::<$crate::crypto::Signature>() {
                                    return Err(
                                        "data.len() != std::mem::size_of::<$crate::crypto::Signature>()"
                                            .to_string(),
                                    );
                                }

                                let mut signature = [0; std::mem::size_of::<$crate::crypto::Signature>()];
                                signature.copy_from_slice(&data);

                                Ok(signature)
                            }),
                            Box::new({
                                let send = send.clone();

                                move |diff| {
                                    if let Ok(value) = &wasm_bindgen::JsValue::from_serde(&diff.serialize())
                                    {
                                        drop(send.call1(&wasm_bindgen::JsValue::UNDEFINED, value));
                                    }
                                }
                            }),
                            Box::new(move |message| {
                                drop(log.call1(&wasm_bindgen::JsValue::UNDEFINED, message));
                            }),
                            Box::new($crate::store::bindings::JsRng(random)),
                        )
                        .map_err(wasm_bindgen::JsValue::from)?
                    },
                    send,
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn deserialize(
                data: &[u8],
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
                            Box::new(move |state| {
                                if let Ok(state) = wasm_bindgen::JsValue::from_serde(state) {
                                    drop(ready.call1(&wasm_bindgen::JsValue::UNDEFINED, &state));
                                }
                            }),
                            Box::new(move |message| {
                                let data: Vec<_> = sign
                                    .call1(
                                        &wasm_bindgen::JsValue::UNDEFINED,
                                        &wasm_bindgen::JsValue::from_serde(message)
                                            .map_err(|error| format!("{}", error))?,
                                    )
                                    .map_err(|error| format!("{:?}", error))?
                                    .into_serde()
                                    .map_err(|error| format!("{}", error))?;

                                if data.len() != std::mem::size_of::<$crate::crypto::Signature>() {
                                    return Err(
                                        "data.len() != std::mem::size_of::<$crate::crypto::Signature>()"
                                            .to_string(),
                                    );
                                }

                                let mut signature = [0; std::mem::size_of::<$crate::crypto::Signature>()];
                                signature.copy_from_slice(&data);

                                Ok(signature)
                            }),
                            Box::new({
                                let send = send.clone();

                                move |diff| {
                                    if let Ok(value) = &wasm_bindgen::JsValue::from_serde(&diff.serialize())
                                    {
                                        drop(send.call1(&wasm_bindgen::JsValue::UNDEFINED, value));
                                    }
                                }
                            }),
                            Box::new(move |message| {
                                drop(log.call1(&wasm_bindgen::JsValue::UNDEFINED, message));
                            }),
                            Box::new($crate::store::bindings::JsRng(random)),
                        )
                        .map_err(wasm_bindgen::JsValue::from)?
                    },
                    send,
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn serialize(&self) -> Vec<u8> {
                self.store.serialize()
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = serializeProof)]
            pub fn serialize_proof(&self) -> Vec<u8> {
                self.store.serialize_proof()
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
            pub fn state(&self) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                wasm_bindgen::JsValue::from_serde(self.store.state().state().state().ok_or(
                    wasm_bindgen::JsValue::from("self.store.state().state().state().is_none()"),
                )?)
                .map_err(|error| wasm_bindgen::JsValue::from(format!("{}", error)))
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter, js_name = pendingPlayer)]
            pub fn pending_player(&self) -> Result<Option<$crate::Player>, wasm_bindgen::JsValue> {
                if let $crate::store::StoreState::Pending { phase, .. } = self.store.state().state() {
                    match *phase
                        .try_borrow()
                        .map_err(|error| wasm_bindgen::JsValue::from(error.to_string()))?
                    {
                        $crate::store::Phase::Commit => Ok(Some(0)),
                        $crate::store::Phase::Reply { .. } => Ok(Some(1)),
                        $crate::store::Phase::Reveal {
                            owner_hash: false, ..
                        } => Ok(Some(0)),
                        $crate::store::Phase::Reveal {
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
            pub fn dispatch(&mut self, action: wasm_bindgen::JsValue) -> Result<(), wasm_bindgen::JsValue> {
                let action: <$type as $crate::store::State>::Action =
                    action.into_serde().map_err(|err| format!("{:?}", err))?;

                let diff = self.store.diff(vec![$crate::ProofAction {
                    player: self.store.player(),
                    action: $crate::PlayerAction::Play($crate::store::StoreAction::Action(action)),
                }])?;

                self.send.call1(
                    &wasm_bindgen::JsValue::UNDEFINED,
                    &wasm_bindgen::JsValue::from_serde(&diff.serialize())
                        .map_err(|error| wasm_bindgen::JsValue::from(format!("{}", error)))?,
                )?;

                self.store
                    .apply(&diff)
                    .map_err(|err| format!("{:?}", err))?;

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
                self.store
                    .apply(&$crate::Diff::deserialize(diff).map_err(wasm_bindgen::JsValue::from)?)
                    .map_err(|err| wasm_bindgen::JsValue::from(format!("{:?}", err)))
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn reset(&mut self, proof: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
                self.store.reset(proof).map_err(wasm_bindgen::JsValue::from)
            }
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        pub struct JsProof($crate::Proof<$crate::store::StoreState<$type>>);

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl JsProof {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new(root: &[u8]) -> Result<JsProof, wasm_bindgen::JsValue> {
                Ok(Self($crate::Proof::new(
                    $crate::RootProof::deserialize(root).map_err(wasm_bindgen::JsValue::from)?,
                )))
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn serialize(&self) -> Vec<u8> {
                self.0.serialize()
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn state(&self) -> wasm_bindgen::JsValue {
                wasm_bindgen::JsValue::from_serde(&self.0.state().state().state())
                    .unwrap_or(wasm_bindgen::JsValue::null())
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn diff(
                &mut self,
                action: wasm_bindgen::JsValue,
                sign: js_sys::Function,
            ) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
                let action: <$type as $crate::store::State>::Action =
                    action.into_serde().map_err(|err| format!("{:?}", err))?;

                let diff = self
                    .0
                    .diff(
                        vec![$crate::ProofAction {
                            player: None,
                            action: $crate::PlayerAction::Play($crate::store::StoreAction::Action(action)),
                        }],
                        &mut |message| {
                            let data: Vec<_> = sign
                                .call1(
                                    &wasm_bindgen::JsValue::UNDEFINED,
                                    &wasm_bindgen::JsValue::from_serde(message)
                                        .map_err(|error| format!("{}", error))?,
                                )
                                .map_err(|error| format!("{:?}", error))?
                                .into_serde()
                                .map_err(|error| format!("{}", error))?;

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
                    )?
                    .serialize();

                Ok(diff)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn apply(&mut self, diff: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
                self.0
                    .apply(&$crate::Diff::deserialize(diff).map_err(wasm_bindgen::JsValue::from)?)
                    .map_err(|err| wasm_bindgen::JsValue::from(format!("{:?}", err)))
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

            let mut address = $crate::crypto::Address::default();
            address.copy_from_slice(player);

            $crate::RootProof::<$crate::store::StoreState<$type>>::deserialize(root)?
                .state()
                .player(&address)
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
    };
}

#[cfg(feature = "bindings")]
/// WebAssembly-specific utilities
pub mod bindings {
    use std::convert::TryInto;

    /// Random number generator using an external JavaScript function for entropy
    pub struct JsRng(pub js_sys::Function);

    impl rand::RngCore for JsRng {
        fn next_u32(&mut self) -> u32 {
            rand_core::impls::next_u32_via_fill(self)
        }

        fn next_u64(&mut self) -> u64 {
            rand_core::impls::next_u64_via_fill(self)
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.try_fill_bytes(dest).unwrap();
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            let length: u32 = dest.len().try_into().map_err(rand::Error::new)?;

            let random: Vec<u8> = self
                .0
                .call1(&wasm_bindgen::JsValue::UNDEFINED, &length.into())
                .map_err(|error| rand::Error::new(JsRngError(format!("{:?}", error))))?
                .into_serde()
                .map_err(rand::Error::new)?;

            if random.len() != dest.len() {
                return Err(rand::Error::new(JsRngError(
                    "random.len() != dest.len()".to_string(),
                )));
            }

            dest.copy_from_slice(&random);

            Ok(())
        }
    }

    #[derive(Debug)]
    struct JsRngError(String);

    impl std::error::Error for JsRngError {}

    impl std::fmt::Display for JsRngError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            std::fmt::Display::fmt(&self.0, f)
        }
    }
}

/// Client [State] store
pub struct Store<S: State + Serialize> {
    player: Option<crate::Player>,
    proof: crate::Proof<StoreState<S>>,
    ready: Box<dyn FnMut(&S)>,
    sign: Box<dyn FnMut(&[u8]) -> Result<crate::crypto::Signature, String>>,
    send: Box<dyn FnMut(&StoreDiff<S>)>,
    random: Box<dyn rand::RngCore>,
    secret: Option<Vec<u8>>,
}

impl<S: State + Serialize> Store<S> {
    /// Constructs a new store for a given player.
    ///
    /// `root` must have been constructed using [RootProof::serialize](crate::RootProof::serialize).
    pub fn new(
        player: Option<crate::Player>,
        root: &[u8],
        ready: Box<dyn FnMut(&S)>,
        sign: Box<dyn FnMut(&[u8]) -> Result<crate::crypto::Signature, String>>,
        send: Box<dyn FnMut(&StoreDiff<S>)>,
        log: Box<dyn FnMut(&Log)>,
        random: Box<dyn rand::RngCore>,
    ) -> Result<Self, String> {
        let mut store = Self {
            player,
            proof: crate::Proof::new({
                let mut root = crate::RootProof::deserialize(root)?;

                if let StoreState::Ready { log: logger, .. } = &mut root.state.state {
                    *logger = Some(Logger::new(log));
                }

                root
            }),
            ready,
            sign,
            send,
            random,
            secret: None,
        };

        store.flush()?;

        Ok(store)
    }

    /// Constructs a store from its binary representation.
    ///
    /// `data` must have been constructed using [Store::serialize].
    pub fn deserialize(
        mut data: &[u8],
        ready: Box<dyn FnMut(&S)>,
        sign: Box<dyn FnMut(&[u8]) -> Result<crate::crypto::Signature, String>>,
        send: Box<dyn FnMut(&StoreDiff<S>)>,
        log: Box<dyn FnMut(&Log)>,
        random: Box<dyn rand::RngCore>,
    ) -> Result<Self, String> {
        crate::forbid!(data.len() < 1 + size_of::<u32>() + size_of::<u32>() + 1);

        let player = match crate::utils::read_u8(&mut data)? {
            0 => None,
            byte => Some(byte - 1),
        };

        let size = crate::utils::read_u32_usize(&mut data)?;

        crate::forbid!(data.len() < size);
        let root = crate::RootProof::deserialize(&data[..size])?;
        data = &data[size..];

        if let Some(player) = player {
            crate::forbid!(usize::from(player) >= root.state.players.len());
        }

        let size = crate::utils::read_u32_usize(&mut data)?;

        crate::forbid!(data.len() < size);
        let mut proof = crate::Proof::new(root);
        proof.deserialize(&data[..size])?;
        data = &data[size..];

        let log = Logger::new(log);

        if let StoreState::Ready { log: logger, .. } = &mut proof.root.state.state {
            *logger = Some(log.clone());
        }

        if let StoreState::Ready { log: logger, .. } = &mut proof.root.latest.state {
            *logger = Some(log.clone());
        }

        for proof in &mut proof.proofs {
            if let Some(proof) = proof {
                if let StoreState::Ready { log: logger, .. } = &mut proof.state.state {
                    *logger = Some(log.clone());
                }
            }
        }

        if let StoreState::Ready { log: logger, .. } = &mut proof.state.state {
            *logger = Some(log.clone());
        }

        let secret = if crate::utils::read_u8_bool(&mut data)? {
            Some(data.to_vec())
        } else {
            crate::forbid!(!data.is_empty());

            None
        };

        let mut store = Self {
            player,
            proof,
            ready,
            sign,
            send,
            random,
            secret,
        };

        store.flush()?;

        Ok(store)
    }

    /// Generates a binary representation that can be used to reconstruct the store.
    ///
    /// See [Store::deserialize].
    pub fn serialize(&self) -> Vec<u8> {
        let root = self.proof.root.serialize();
        let proof = self.proof.serialize();

        let mut data = Vec::with_capacity(
            1 + size_of::<u32>()
                + root.len()
                + size_of::<u32>()
                + proof.len()
                + 1
                + self.secret.as_ref().map_or(0, Vec::len),
        );

        crate::utils::write_u8(
            &mut data,
            match self.player {
                None => 0,
                Some(player) => 1 + player,
            },
        );

        crate::utils::write_u32_usize(&mut data, root.len()).unwrap();
        data.extend(root);

        crate::utils::write_u32_usize(&mut data, proof.len()).unwrap();
        data.extend(proof);

        if let Some(secret) = &self.secret {
            crate::utils::write_u8_bool(&mut data, true);
            data.extend(secret);
        } else {
            crate::utils::write_u8_bool(&mut data, false);
        }

        data
    }

    /// Generates a binary representation of the proof that can be used to reset the store.
    ///
    /// See [Store::reset].
    pub fn serialize_proof(&self) -> Vec<u8> {
        self.proof.serialize()
    }

    /// Gets the player associated with the store.
    pub fn player(&self) -> Option<crate::Player> {
        self.player
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

        let diff = match &self.proof.state.state {
            StoreState::Pending { phase, .. } => {
                // XXX: do we need to lock and unlock log here?

                match &*phase.try_borrow().unwrap() {
                    Phase::Commit => {
                        let seed = {
                            let mut seed =
                                <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                            self.random.fill_bytes(&mut seed);
                            seed
                        };

                        self.secret = Some(seed.to_vec());

                        Some(self.proof.diff(
                            vec![crate::ProofAction {
                                player: None,
                                action: crate::PlayerAction::Play(
                                    StoreAction::<S::Action>::Commit(tiny_keccak::keccak256(&seed)),
                                ),
                            }],
                            &mut self.sign,
                        )?)
                    }
                    Phase::Reply { .. } => {
                        let seed = {
                            let mut seed =
                                <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                            self.random.fill_bytes(&mut seed);
                            seed
                        };

                        Some(self.proof.diff(
                            vec![crate::ProofAction {
                                player: None,
                                action: crate::PlayerAction::Play(StoreAction::<S::Action>::Reply(
                                    seed.to_vec(),
                                )),
                            }],
                            &mut self.sign,
                        )?)
                    }
                    Phase::Reveal {
                        owner_hash: false, ..
                    } => {
                        let seed = {
                            let mut seed =
                                <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                            self.random.fill_bytes(&mut seed);
                            seed
                        };

                        Some(self.proof.diff(
                            vec![crate::ProofAction {
                                player: None,
                                action: crate::PlayerAction::Play(
                                    StoreAction::<S::Action>::Reveal(seed.to_vec()),
                                ),
                            }],
                            &mut self.sign,
                        )?)
                    }
                    _ => None,
                }
            }
            StoreState::Ready { state, .. } => {
                (self.ready)(state);

                None
            }
        };

        if let Some(diff) = &diff {
            (self.send)(diff);

            self.apply(diff)?;
        }

        Ok(())
    }

    /// Verifies and applies a cryptographically constructed diff to the store.
    ///
    /// `diff` must have been constructed using [Store::diff] on a store with the same state.
    pub fn apply(&mut self, diff: &StoreDiff<S>) -> Result<(), String> {
        crate::error::check(self.proof.apply(diff))?;

        self.flush()
    }

    /// Generates a diff that can be applied to a store with the same state.
    ///
    /// See [Store::apply].
    pub fn diff(
        &mut self,
        actions: Vec<crate::ProofAction<StoreAction<S::Action>>>,
    ) -> Result<StoreDiff<S>, String> {
        let log = match &mut self.proof.state.state {
            StoreState::Ready { log, .. } => log,
            StoreState::Pending { log, .. } => log,
        }
        .take();

        let diff = self.proof.diff(actions, &mut self.sign);

        if let Some(log) = log {
            match &mut self.proof.state.state {
                StoreState::Ready { log, .. } => log,
                StoreState::Pending { log, .. } => log,
            }
            .replace(log);
        }

        diff
    }

    /// Unconditionally sets the state of the store using `proof`.
    ///
    /// `proof` must have been constructed using [Proof::serialize](crate::Proof::serialize) on a proof with the same root.
    ///
    /// It is possible to reset to a state with a lower nonce using this method.
    pub fn reset(&mut self, proof: &[u8]) -> Result<(), String> {
        // TODO XXX figure out why the logger doesn't exist on self.proof.state.state, and only in one of the player proofs.

        let mut old_logger = match &self.proof.state.state {
            StoreState::Pending { log, .. } | StoreState::Ready { log, .. } => log,
        }
        .clone();

        for player_proof in &mut self.proof.proofs {
            if let Some(proof) = player_proof {
                match &mut proof.state.state {
                    StoreState::Pending { log, .. } | StoreState::Ready { log, .. } => {
                        if let Some(current_logger) = log {
                            old_logger = Some(current_logger.clone());
                            break;
                        }
                    }
                }
            }
        }

        self.proof.deserialize(proof)?;

        for player_proof in &mut self.proof.proofs {
            if let Some(proof) = player_proof {
                match &mut proof.state.state {
                    StoreState::Pending { log, .. } | StoreState::Ready { log, .. } => {
                        if let Some(mut new_log) = old_logger.clone() {
                            new_log.nonce = new_log.state.borrow_mut().1;
                            *log = Some(new_log)
                        }
                    }
                }
            }
        }

        match &mut self.proof.state.state {
            StoreState::Pending { log, .. } | StoreState::Ready { log, .. } => {
                if let Some(mut new_log) = old_logger {
                    new_log.nonce = new_log.state.borrow_mut().1;
                    *log = Some(new_log)
                }
            }
        }

        self.flush()?;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        let diff = match &self.proof.state.state {
            StoreState::Pending { phase, .. } => {
                // XXX: do we need to lock and unlock log here?

                match (&*phase.try_borrow().unwrap(), self.player) {
                    (Phase::Commit, Some(0)) => {
                        let seed = {
                            let mut seed =
                                <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                            self.random.fill_bytes(&mut seed);
                            seed
                        };

                        self.secret = Some(seed.to_vec());

                        Some(self.proof.diff(
                            vec![crate::ProofAction {
                                player: Some(0),
                                action: crate::PlayerAction::Play(
                                    StoreAction::<S::Action>::Commit(tiny_keccak::keccak256(&seed)),
                                ),
                            }],
                            &mut self.sign,
                        )?)
                    }
                    (Phase::Reply { .. }, Some(1)) => {
                        let seed = {
                            let mut seed =
                                <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                            self.random.fill_bytes(&mut seed);
                            seed
                        };

                        Some(self.proof.diff(
                            vec![crate::ProofAction {
                                player: Some(1),
                                action: crate::PlayerAction::Play(StoreAction::<S::Action>::Reply(
                                    seed.to_vec(),
                                )),
                            }],
                            &mut self.sign,
                        )?)
                    }
                    (
                        Phase::Reveal {
                            hash,
                            owner_hash: false,
                            ..
                        },
                        Some(0),
                    )
                    | (
                        Phase::Reveal {
                            hash,
                            owner_hash: true,
                            ..
                        },
                        None,
                    ) => {
                        if let Some(secret) = &self.secret {
                            crate::forbid!(&tiny_keccak::keccak256(secret) != hash);

                            Some(self.proof.diff(
                                vec![crate::ProofAction {
                                    player: self.player,
                                    action: crate::PlayerAction::Play(
                                        StoreAction::<S::Action>::Reveal(secret.to_vec()),
                                    ),
                                }],
                                &mut self.sign,
                            )?)
                        } else {
                            return Err("self.secret.is_none()".to_string());
                        }
                    }
                    _ => None,
                }
            }
            StoreState::Ready { state, .. } => {
                (self.ready)(state);

                None
            }
        };

        if let Some(diff) = &diff {
            (self.send)(diff);

            self.apply(diff)?;
        }

        Ok(())
    }
}

type StoreDiff<S> = crate::Diff<StoreAction<<S as State>::Action>>;

#[doc(hidden)]
pub enum StoreState<S: State + Serialize> {
    Ready {
        state: S,

        log: Option<Logger>,
    },
    Pending {
        phase: Rc<RefCell<Phase>>,
        state: Pin<Box<dyn Future<Output = (S, Context)>>>,

        log: Option<Logger>,
    },
}

impl<S: State + Serialize> StoreState<S> {
    pub fn state(&self) -> Option<&S> {
        if let StoreState::Ready { state, .. } = self {
            Some(state)
        } else {
            None
        }
    }
}

impl<S: State + Serialize> crate::State for StoreState<S> {
    type ID = S::ID;
    type Nonce = S::Nonce;
    type Action = StoreAction<S::Action>;

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        Ok(Self::Ready {
            state: S::deserialize(data)?,
            log: None,
        })
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        match self {
            Self::Ready { state, .. } => <S as State>::serialize(&state),
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

        replace_with_or_abort(self, |state| {
            if let Self::Action::Action(action) = action {
                if let Self::Ready { state, log } = state {
                    let phase = Rc::new(RefCell::new(Phase::Idle));

                    handled = true;

                    Self::Pending {
                        phase: phase.clone(),
                        state: state.apply(
                            player,
                            action.clone(),
                            Context {
                                phase,
                                log: log.clone(),
                            },
                        ),
                        log,
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

                (Self::Pending { phase: context, .. }, Self::Action::Commit(hash)) => {
                    let phase = context
                        .try_borrow()
                        .map_err(|error| error.to_string())?
                        .clone();

                    if let Phase::Commit = phase {
                        crate::forbid!(player != None && player != Some(0));

                        context.replace(Phase::Reply {
                            hash: *hash,
                            owner_hash: player.is_none(),
                        });
                    } else {
                        return Err("context.try_borrow().map_err(|error| error.to_string())?.clone() != Phase::Commit".to_string());
                    }
                }

                (Self::Pending { phase: context, .. }, Self::Action::Reply(secret)) => {
                    let phase = context
                        .try_borrow()
                        .map_err(|error| error.to_string())?
                        .clone();

                    if let Phase::Reply { hash, owner_hash } = phase {
                        crate::forbid!(player != None && player != Some(1));

                        context.replace(Phase::Reveal {
                            hash,
                            owner_hash,
                            reply: secret.to_vec(),
                        });
                    } else {
                        return Err("context.try_borrow().map_err(|error| error.to_string())?.clone() != Phase::Reply { .. }".to_string());
                    }
                }

                (Self::Pending { phase: context, .. }, Self::Action::Reveal(secret)) => {
                    let phase = context
                        .try_borrow()
                        .map_err(|error| error.to_string())?
                        .clone();

                    if let Phase::Reveal {
                        hash,
                        owner_hash,
                        reply,
                    } = phase
                    {
                        if owner_hash {
                            crate::forbid!(player != None);
                        } else {
                            crate::forbid!(player != None && player != Some(0));
                        }

                        if player.is_some() || owner_hash {
                            crate::forbid!(tiny_keccak::keccak256(secret) != hash);
                        }

                        let data: Vec<_> = reply.iter().zip(secret).map(|(x, y)| x ^ y).collect();

                        let mut seed = [0; 16];
                        seed.copy_from_slice(&data);

                        let random: rand_xorshift::XorShiftRng = rand::SeedableRng::from_seed(seed);
                        context.replace(Phase::Randomized(Rc::new(RefCell::new(random))));
                    } else {
                        return Err("context.try_borrow().map_err(|error| error.to_string())?.clone() != Phase::Reveal { .. }".to_string());
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

        replace_with_or_abort(self, |state| {
            if let Self::Pending {
                mut state,
                phase,
                log,
            } = state
            {
                if let Poll::Ready((state, context)) = state
                    .as_mut()
                    .poll(&mut task::Context::from_waker(&phantom_waker()))
                {
                    Self::Ready {
                        state,
                        log: context.log,
                    }
                } else {
                    Self::Pending { state, phase, log }
                }
            } else {
                state
            }
        });

        Ok(())
    }
}

impl<S: State + Serialize> Clone for StoreState<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Ready { state, log } => Self::Ready {
                state: state.clone(),
                log: log.clone(),
            },
            _ => panic!("StoreState::Pending {{ .. }}.clone()"),
        }
    }
}

#[doc(hidden)]
#[derive(Clone)]
pub enum StoreAction<A: crate::Action> {
    Action(A),
    Commit(crate::crypto::Hash),
    Reply(Vec<u8>),
    Reveal(Vec<u8>),
}

impl<A: crate::Action> crate::Action for StoreAction<A> {
    fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        match crate::utils::read_u8(&mut data)? {
            0 => Ok(Self::Action(A::deserialize(data)?)),
            1 => {
                crate::forbid!(data.len() != size_of::<crate::crypto::Hash>());

                let mut hash = crate::crypto::Hash::default();
                hash.copy_from_slice(data);

                Ok(Self::Commit(hash))
            }
            2 => Ok(Self::Reply(data.to_vec())),
            3 => Ok(Self::Reveal(data.to_vec())),
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
            Self::Commit(hash) => {
                crate::utils::write_u8(&mut data, 1);
                data.extend(hash);
            }
            Self::Reply(reply) => {
                crate::utils::write_u8(&mut data, 2);
                data.extend(reply);
            }
            Self::Reveal(secret) => {
                crate::utils::write_u8(&mut data, 3);
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
            Self::Commit(data) => write!(f, "StoreAction::Commit({})", crate::utils::hex(data)),
            Self::Reply(data) => write!(f, "StoreAction::Reply({})", crate::utils::hex(data)),
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

    /// Formats the message that must be signed in order to certify the subkey for a given address.
    fn certificate(address: &crate::crypto::Address) -> String {
        format!(
            "Sign to play! This won't cost anything.\n\n{}\n",
            crate::crypto::eip55(address)
        )
    }

    /// Constructs a state from its binary representation.
    ///
    /// `data` must have been constructed using [State::serialize].
    fn deserialize(data: &[u8]) -> Result<Self, String>;

    /// Generates a binary representation that can be used to reconstruct the state.
    ///
    /// See [State::deserialize].
    fn serialize(&self) -> Option<Vec<u8>>;

    /// Verifies if an action by a given player is valid for the state.
    fn verify(&self, player: Option<crate::Player>, action: &Self::Action) -> Result<(), String>;

    /// Applies an action by a given player to the state.
    fn apply(
        self,
        player: Option<crate::Player>,
        action: Self::Action,
        context: Context,
    ) -> Pin<Box<dyn Future<Output = (Self, Context)>>>;
}

/// Emits an event as a side effect of a state transition.
///
/// This can only be called from within [store::State::apply] since a [Context](store::Context) is required.
///
/// If the `bindings` feature is enabled, `$message` must implement [Serialize](serde::Serialize).
/// Otherwise, `$message` must implement [Debug].
#[cfg(feature = "bindings")]
#[macro_export]
macro_rules! log {
    ($context:expr, $message:expr) => {
        if let Ok(message) = wasm_bindgen::JsValue::from_serde(&$message) {
            drop($context.log(&message));
        }
    };
}

/// Emits an event as a side effect of a state transition.
///
/// This can only be called from within [store::State::apply] since a [Context](store::Context) is required.
///
/// If the `bindings` feature is enabled, `$message` must implement [Serialize](serde::Serialize).
/// Otherwise, `$message` must implement [Debug].
#[cfg(not(feature = "bindings"))]
#[macro_export]
macro_rules! log {
    ($context:expr, $message:expr) => {
        drop($context.log(&$message));
    };
}

/// [State::apply] utilities
///
/// See [log].
pub struct Context {
    phase: Rc<RefCell<Phase>>,
    log: Option<Logger>,
}

impl Context {
    /// Construct a random number generator via commit-reveal.
    pub fn random(&mut self) -> impl Future<Output = impl rand::Rng> {
        let idle = if let Phase::Idle = &*self.phase.try_borrow().unwrap() {
            true
        } else {
            false
        };

        if idle {
            self.phase.replace(Phase::Commit);
        }

        XorShiftRngFuture(self.phase.clone())
    }

    #[doc(hidden)]
    pub fn log(&mut self, message: &Log) -> Result<(), String> {
        if let Some(log) = &mut self.log {
            log.log(message)
        } else {
            Ok(())
        }
    }

    #[doc(hidden)]
    pub fn with_phase(phase: Rc<RefCell<Phase>>) -> Self {
        Self { phase, log: None }
    }
}

#[doc(hidden)]
#[derive(Clone)]
pub struct Logger {
    state: Rc<RefCell<LoggerState>>,
    nonce: usize,
}

impl Logger {
    fn new(log: Box<dyn FnMut(&Log)>) -> Self {
        Self {
            state: Rc::new(RefCell::new((log, 0))),
            nonce: 0,
        }
    }

    fn log(&mut self, message: &Log) -> Result<(), String> {
        self.nonce += 1;

        let (log, nonce) = &mut *self
            .state
            .try_borrow_mut()
            .map_err(|error| error.to_string())?;

        if self.nonce > *nonce {
            *nonce = self.nonce;

            log(message);
        }

        Ok(())
    }
}

type LoggerState = (Box<dyn FnMut(&Log)>, usize);

#[cfg(feature = "bindings")]
type Log = wasm_bindgen::JsValue;
#[cfg(not(feature = "bindings"))]
type Log = dyn Debug;

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum Phase {
    Idle,
    Commit,
    Reply {
        hash: crate::crypto::Hash,
        owner_hash: bool,
    },
    Reveal {
        hash: crate::crypto::Hash,
        owner_hash: bool,
        reply: Vec<u8>,
    },
    Randomized(Rc<RefCell<rand_xorshift::XorShiftRng>>),
}

struct XorShiftRngFuture(Rc<RefCell<Phase>>);

impl Future for XorShiftRngFuture {
    type Output = ContextRandom;

    fn poll(self: Pin<&mut Self>, _: &mut task::Context) -> Poll<Self::Output> {
        if let Ok(phase) = self.0.try_borrow() {
            if let Phase::Randomized(random) = &*phase {
                Poll::Ready(ContextRandom(random.clone()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

struct ContextRandom(Rc<RefCell<rand_xorshift::XorShiftRng>>);

impl rand::RngCore for ContextRandom {
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

fn phantom_waker() -> Waker {
    unsafe {
        Waker::from_raw(RawWaker::new(
            ptr::null(),
            &RawWakerVTable::new(
                |_| panic!("Waker::clone"),
                |_| panic!("Waker::wake"),
                |_| panic!("Waker::wake_by_ref"),
                |_| {},
            ),
        ))
    }
}
