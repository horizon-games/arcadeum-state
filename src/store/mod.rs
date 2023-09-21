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

//! Client store

use {
    alloc::{
        boxed::Box,
        fmt::Debug,
        format,
        rc::Rc,
        string::{String, ToString},
        vec,
        vec::Vec,
    },
    core::{
        cell::{Ref, RefCell},
        column,
        convert::TryInto,
        file,
        future::Future,
        line,
        mem::size_of,
        ops::{Deref, DerefMut},
        pin::Pin,
        ptr, task,
        task::{Poll, RawWaker, RawWakerVTable, Waker},
    },
};

mod tester;

#[cfg(feature = "std")]
pub mod bindings;

#[derive(PartialEq)]
pub enum SecretKnowledge {
    Both,
    None,
    Some(crate::Player),
}

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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        player: Option<crate::Player>,
        root: &[u8],
        [secret1, secret2]: [Option<(S::Secret, [u8; 16])>; 2],
        p2p: bool,
        ready: impl FnMut(&S, [Option<&S::Secret>; 2]) + 'static,
        sign: impl FnMut(&[u8]) -> Result<crate::crypto::Signature, String> + 'static,
        send: impl FnMut(&StoreDiff<S>) + 'static,
        log: impl FnMut(Option<crate::Player>, S::Event) + 'static,
        random: impl rand::RngCore + 'static,
        no_version_check: bool,
    ) -> Result<Self, String> {
        Ok(Self {
            player,
            proof: crate::Proof::new(crate::RootProof::<StoreState<S>>::deserialize_and_init(
                root,
                |state| {
                    if let Some(_StoreState::Ready { secrets, .. }) = &mut state.0 {
                        *secrets = [
                            secret1
                                .map(|(secret, seed)| (secret, rand::SeedableRng::from_seed(seed))),
                            secret2
                                .map(|(secret, seed)| (secret, rand::SeedableRng::from_seed(seed))),
                        ];
                    } else {
                        unreachable!("{}:{}:{}", file!(), line!(), column!());
                    }

                    state.set_logger(Rc::new(RefCell::new(Logger::new(log))));
                },
                no_version_check,
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
    #[allow(clippy::too_many_arguments)]
    pub fn deserialize(
        mut data: &[u8],
        p2p: bool,
        ready: impl FnMut(&S, [Option<&S::Secret>; 2]) + 'static,
        sign: impl FnMut(&[u8]) -> Result<crate::crypto::Signature, String> + 'static,
        send: impl FnMut(&StoreDiff<S>) + 'static,
        log: impl FnMut(Option<crate::Player>, S::Event) + 'static,
        random: impl rand::RngCore + 'static,
        no_version_check: bool,
    ) -> Result<Self, String> {
        crate::forbid!(data.len() < 1 + size_of::<u32>() + size_of::<u32>() + 1);

        let player = match crate::utils::read_u8(&mut data)? {
            0 => None,
            3 => None,
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

        let root = crate::RootProof::<StoreState<S>>::deserialize_and_init(
            &data[..size],
            |state| {
                if let Some(_StoreState::Ready {
                    secrets: state_secrets,
                    ..
                }) = &mut state.0
                {
                    *state_secrets = secrets;
                } else {
                    unreachable!("{}:{}:{}", file!(), line!(), column!());
                }

                state.set_logger(log.clone());
            },
            no_version_check,
        )?;

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

        proof.deserialize_and_init(
            &data[..size],
            |state| {
                if let Some(_StoreState::Ready {
                    secrets: state_secrets,
                    ..
                }) = &mut state.0
                {
                    *state_secrets = secrets;
                } else {
                    unreachable!("{}:{}:{}", file!(), line!(), column!());
                }

                state.set_logger(log);
            },
            no_version_check,
        )?;

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
    pub fn serialize(&self, with_knowledge: SecretKnowledge) -> Vec<u8> {
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
                None => match with_knowledge {
                    SecretKnowledge::None => 0,
                    SecretKnowledge::Both => 3,
                    SecretKnowledge::Some(player) => 1 + player,
                },
                Some(player) => 1 + player,
            },
        );

        if let Some(_StoreState::Ready { secrets, .. }) = &self.proof.root.state.state.0 {
            for (i, secret) in secrets.iter().enumerate() {
                if with_knowledge == SecretKnowledge::Both
                    || with_knowledge == SecretKnowledge::Some(i.try_into().unwrap())
                {
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
            unreachable!("{}:{}:{}", file!(), line!(), column!());
        }

        crate::utils::write_u32_usize(&mut data, root.len()).unwrap();
        data.extend(root);

        if let Some(_StoreState::Ready { secrets, .. }) = &self
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
            .0
        {
            for (i, secret) in secrets.iter().enumerate() {
                if with_knowledge == SecretKnowledge::Both
                    || with_knowledge == SecretKnowledge::Some(i.try_into().unwrap())
                {
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
            unreachable!("{}:{}:{}", file!(), line!(), column!());
        }

        crate::utils::write_u32_usize(&mut data, proof.len()).unwrap();
        data.extend(proof);

        if with_knowledge == SecretKnowledge::Both || with_knowledge == SecretKnowledge::Some(0) {
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

    /// Gets the author of the store's root proof.
    pub fn owner(&self) -> &crate::crypto::Address {
        self.proof.root.author()
    }

    /// Gets the hash of the store's proof.
    pub fn hash(&self) -> &crate::crypto::Hash {
        &self.proof.hash
    }

    /// Gets the state of the store's proof.
    pub fn state(&self) -> &crate::ProofState<StoreState<S>> {
        &self.proof.state
    }

    /// Gets the player who must act if in a pending state.
    pub fn pending_player(&self) -> Result<Option<crate::Player>, String> {
        if let _StoreState::Pending { phase, .. } = self
            .proof
            .state
            .state
            .0
            .as_ref()
            .ok_or("self.proof.state.state.0.is_none()")?
        {
            match *phase.try_borrow().map_err(|error| error.to_string())? {
                Phase::RandomCommit => Ok(Some(0)),
                Phase::RandomReply { .. } => Ok(Some(1)),
                Phase::RandomReveal {
                    owner_hash: false, ..
                } => Ok(Some(0)),
                Phase::RandomReveal {
                    owner_hash: true, ..
                } => Ok(None),
                Phase::Reveal {
                    request: RevealRequest { player, .. },
                    ..
                } => Ok(Some(player)),
                _ => unreachable!("{}:{}:{}", file!(), line!(), column!()),
            }
        } else {
            Err("self.proof.state.state.0 != _StoreState::Pending { .. }".to_string())
        }
    }

    /// Dispatches an action that will continue a stalled commit-reveal sequence.
    ///
    /// Only call this if the pending player isn't live.
    /// Only the owner can call this.
    pub fn dispatch_timeout(&mut self) -> Result<(), String> {
        crate::forbid!(self.player.is_some());

        let action = match self
            .proof
            .state
            .state
            .0
            .as_ref()
            .ok_or("self.proof.state.state.0.is_none()")?
        {
            _StoreState::Pending { phase, .. } => match &*phase.try_borrow().unwrap() {
                Phase::RandomCommit => {
                    let seed = {
                        let mut seed =
                            <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                        self.random
                            .try_fill_bytes(&mut seed)
                            .map_err(|error| error.to_string())?;

                        seed
                    };

                    self.seed = Some(seed.to_vec());

                    Some(crate::ProofAction {
                        player: None,
                        action: crate::PlayerAction::Play(StoreAction(_StoreAction::RandomCommit(
                            crate::crypto::keccak256(&seed),
                        ))),
                    })
                }
                Phase::RandomReply { .. } => {
                    let seed = {
                        let mut seed =
                            <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                        self.random
                            .try_fill_bytes(&mut seed)
                            .map_err(|error| error.to_string())?;

                        seed
                    };

                    Some(crate::ProofAction {
                        player: None,
                        action: crate::PlayerAction::Play(StoreAction(_StoreAction::RandomReply(
                            seed.to_vec(),
                        ))),
                    })
                }
                Phase::RandomReveal {
                    owner_hash: false, ..
                } => {
                    let seed = {
                        let mut seed =
                            <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                        self.random
                            .try_fill_bytes(&mut seed)
                            .map_err(|error| error.to_string())?;

                        seed
                    };

                    Some(crate::ProofAction {
                        player: None,
                        action: crate::PlayerAction::Play(StoreAction(_StoreAction::RandomReveal(
                            seed.to_vec(),
                        ))),
                    })
                }
                _ => None,
            },
            _StoreState::Ready { state, secrets, .. } => {
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
        let actions = self.flush_actions(true)?;

        if !actions.is_empty() {
            let diff = self.diff(
                actions
                    .into_iter()
                    .map(|action| crate::ProofAction {
                        player: self.player,
                        action: crate::PlayerAction::Play(action),
                    })
                    .collect(),
            )?;

            (self.send)(&diff);

            self.apply(&diff)?;
        } else if let _StoreState::Ready { state, secrets, .. } = self
            .proof
            .state
            .state
            .0
            .as_ref()
            .ok_or("self.proof.state.state.0.is_none()")?
        {
            self.seed = None;

            (self.ready)(
                state,
                [
                    secrets[0].as_ref().map(|(secret, _)| secret),
                    secrets[1].as_ref().map(|(secret, _)| secret),
                ],
            );
        }

        Ok(())
    }

    #[doc(hidden)]
    /// Gets any actions the client is required to send.
    ///
    /// If `check_sender` is `false`, also includes actions that the client is capable of sending
    /// due to knowing secret information, but shouldn't due to not being the expected sender.
    pub fn flush_actions(&mut self, check_sender: bool) -> Result<Vec<StoreAction<S>>, String> {
        self.proof
            .state
            .state
            .logger()
            .try_borrow_mut()
            .map_err(|error| error.to_string())?
            .enabled = false;

        let mut state = self.proof.compute_state().state;
        let mut actions = Vec::new();

        loop {
            let action = match state.0.as_ref().ok_or("state.0.is_none()")? {
                _StoreState::Pending { phase, secrets, .. } => {
                    match (&*phase.try_borrow().unwrap(), self.player) {
                        (Phase::RandomCommit, Some(0)) => {
                            let seed = {
                                let mut seed = <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                                self.random
                                    .try_fill_bytes(&mut seed)
                                    .map_err(|error| error.to_string())?;

                                seed
                            };

                            self.seed = Some(seed.to_vec());

                            Some(_StoreAction::RandomCommit(crate::crypto::keccak256(&seed)))
                        }
                        (Phase::RandomReply { .. }, Some(1)) => {
                            let seed = {
                                let mut seed = <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                                self.random
                                    .try_fill_bytes(&mut seed)
                                    .map_err(|error| error.to_string())?;

                                seed
                            };

                            Some(_StoreAction::RandomReply(seed.to_vec()))
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
                                crate::forbid!(&crate::crypto::keccak256(seed) != hash);

                                Some(_StoreAction::RandomReveal(seed.to_vec()))
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
                            if !check_sender
                                || self.player.is_none() && !self.p2p
                                || self.player == Some(*player) && self.p2p
                            {
                                if let Some(secret) = &secrets[usize::from(*player)] {
                                    let secret = reveal(
                                        &secret.try_borrow().map_err(|error| error.to_string())?.0,
                                    );

                                    crate::forbid!(!verify(&secret));

                                    Some(_StoreAction::Reveal(secret))
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
                _StoreState::Ready { .. } => None,
            };

            match action {
                Some(action) => {
                    let action = StoreAction(action);

                    crate::State::apply(&mut state, self.player, &action)?;

                    actions.push(action);
                }
                None => break,
            }
        }

        Ok(actions)
    }

    /// Verifies and applies a cryptographically constructed diff to the store, then calls .flush().
    ///
    /// `diff` must have been constructed using [Store::diff] on a store with the same state.
    pub fn apply(&mut self, diff: &StoreDiff<S>) -> Result<(), String> {
        self.raw_apply(diff)?;

        self.flush()
    }
    /// Verifies and applies a cryptographically constructed diff to the store.
    ///
    /// `diff` must have been constructed using [Store::diff] on a store with the same state.
    pub fn raw_apply(&mut self, diff: &StoreDiff<S>) -> Result<(), String> {
        self.proof
            .state
            .state
            .logger()
            .try_borrow_mut()
            .map_err(|error| error.to_string())?
            .enabled = true;

        self.proof.apply(diff)?;
        Ok(())
    }

    /// Generates a diff that can be applied to a store with the same state.
    ///
    /// See [Store::apply].
    pub fn diff(
        &mut self,
        actions: Vec<crate::ProofAction<StoreState<S>>>,
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

type StoreDiff<S> = crate::Diff<StoreState<S>>;

/// Client store state
#[derive(Clone)]
pub struct StoreState<S: State>(Option<_StoreState<S>>);

impl<S: State> StoreState<S> {
    /// Constructs a new store state.
    pub fn new(
        state: S,
        secrets: [Option<(S::Secret, rand_xorshift::XorShiftRng)>; 2],
        log: impl FnMut(Option<crate::Player>, S::Event) + 'static,
    ) -> Self {
        Self(Some(_StoreState::Ready {
            state,
            secrets,
            action_count: Default::default(),
            reveal_count: Default::default(),
            event_count: Default::default(),
            logger: Rc::new(RefCell::new(Logger::new(log))),
        }))
    }

    /// Constructs a state from its binary representation and a log function.
    ///
    /// `data` must have been constructed using [crate::State::serialize].
    pub fn deserialize(
        data: &[u8],
        log: impl FnMut(Option<crate::Player>, S::Event) + 'static,
    ) -> Result<Self, String> {
        let mut state: Self = crate::State::deserialize(data)?;

        state.set_logger(Rc::new(RefCell::new(Logger::new(log))));

        Ok(state)
    }

    /// Gets the state of the store state.
    pub fn state(&self) -> Option<&S> {
        if let _StoreState::Ready { state, .. } = self.0.as_ref()? {
            Some(state)
        } else {
            None
        }
    }

    /// Gets a player's secret state, if available.
    pub fn secret<'a>(
        &'a self,
        player: crate::Player,
    ) -> Option<Box<dyn Deref<Target = S::Secret> + 'a>> {
        match self.0.as_ref()? {
            _StoreState::Ready { secrets, .. } => secrets[usize::from(player)]
                .as_ref()
                .map(|(secret, _)| Box::new(secret) as Box<dyn Deref<Target = S::Secret>>),

            _StoreState::Pending { secrets, .. } => {
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

    /// Gets the number of actions applied since construction.
    pub fn action_count(&self) -> usize {
        match self.0.as_ref().unwrap() {
            _StoreState::Ready { action_count, .. } | _StoreState::Pending { action_count, .. } => {
                *action_count
            }
        }
    }

    /// Gets the number of secrets revealed since construction.
    pub fn reveal_count(&self) -> usize {
        match self.0.as_ref().unwrap() {
            _StoreState::Ready { reveal_count, .. } | _StoreState::Pending { reveal_count, .. } => {
                *reveal_count
            }
        }
    }

    /// Generates an event log resulting from applying an action to this state.
    pub fn simulate(
        &self,
        player: Option<crate::Player>,
        action: &S::Action,
        using_secrets: [bool; 2],
    ) -> Result<Log<S>, String>
    where
        S::Event: serde::Serialize + 'static,
    {
        if let _StoreState::Ready {
            state,
            secrets,
            action_count,
            reveal_count,
            ..
        } = self.0.as_ref().ok_or("self.0.is_none()")?
        {
            let events = Rc::new(RefCell::new(Vec::new()));

            Ok({
                let mut state = Self(Some(_StoreState::Ready {
                    state: state.clone(),
                    secrets: crate::utils::keep_by_array(secrets.clone(), using_secrets),
                    action_count: *action_count,
                    reveal_count: *reveal_count,
                    event_count: Default::default(),
                    logger: Rc::new(RefCell::new(Logger::new({
                        let events = events.clone();

                        move |target, event| {
                            if target.is_none() || target == player {
                                events.try_borrow_mut().unwrap().push(event);
                            }
                        }
                    }))),
                }));

                crate::State::apply(&mut state, player, &StoreAction::new(action.clone()))?;

                let mut complete = true;

                while let _StoreState::Pending { secrets, phase, .. } =
                    state.0.as_ref().ok_or("state.0.is_none()")?
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

                    crate::State::apply(
                        &mut state,
                        Some(player),
                        &StoreAction(_StoreAction::Reveal(secret)),
                    )?;
                }

                if complete {
                    Log::Complete
                } else {
                    Log::Incomplete
                }
            }(
                Rc::try_unwrap(events).ok().unwrap().into_inner()
            ))
        } else {
            Err("self.0 != _StoreState::Ready { .. }".to_string())
        }
    }

    #[doc(hidden)]
    /// Applies an action by a given player to the state using the provided random number generator
    /// instead of doing a commit-reveal for randomness.
    pub fn apply_with_random(
        &mut self,
        player: Option<crate::Player>,
        action: S::Action,
        random: &mut impl rand::RngCore,
    ) -> Result<(), String> {
        crate::State::apply(self, player, &StoreAction::new(action))?;

        while let _StoreState::Pending { secrets, phase, .. } =
            self.0.as_ref().ok_or("self.0.is_none()")?
        {
            let borrowed_phase = phase.try_borrow().map_err(|error| error.to_string())?;

            match &*borrowed_phase {
                Phase::RandomCommit => {
                    drop(borrowed_phase);

                    phase.replace(Phase::Idle {
                        random: Some(Rc::new(RefCell::new({
                            rand::SeedableRng::from_seed({
                                let mut seed = <rand_xorshift::XorShiftRng as rand::SeedableRng>::Seed::default();

                                random.try_fill_bytes(&mut seed).map_err(|error| error.to_string())?;

                                seed
                            })
                        }))),
                        secret: None,
                    });

                    if let Some(state) = self.0.take() {
                        drop(self.0.replace(
                            if let _StoreState::Pending {
                                mut state,
                                secrets: [secret1, secret2],
                                action_count,
                                reveal_count,
                                phase,
                                logger,
                            } = state
                            {
                                if let Poll::Ready((state, context)) = state
                                    .as_mut()
                                    .poll(&mut task::Context::from_waker(&phantom_waker()))
                                {
                                    drop(context.secrets);

                                    _StoreState::Ready {
                                        state,
                                        secrets: [
                                            secret1.map(|secret| {
                                                Rc::try_unwrap(secret).ok().unwrap().into_inner()
                                            }),
                                            secret2.map(|secret| {
                                                Rc::try_unwrap(secret).ok().unwrap().into_inner()
                                            }),
                                        ],
                                        action_count,
                                        reveal_count: reveal_count + 3,
                                        event_count: context.event_count,
                                        logger,
                                    }
                                } else {
                                    _StoreState::Pending {
                                        state,
                                        secrets: [secret1, secret2],
                                        action_count,
                                        reveal_count: reveal_count + 3,
                                        phase,
                                        logger,
                                    }
                                }
                            } else {
                                state
                            },
                        ));
                    } else {
                        unreachable!("{}:{}:{}", file!(), line!(), column!());
                    }
                }
                Phase::Reveal {
                    request:
                        RevealRequest {
                            player,
                            reveal,
                            verify,
                        },
                    ..
                } => {
                    let player = *player;

                    if let Some(secret) = &secrets[usize::from(player)] {
                        let secret =
                            reveal(&secret.try_borrow().map_err(|error| error.to_string())?.0);

                        crate::forbid!(!verify(&secret));

                        drop(borrowed_phase);

                        crate::State::apply(
                            self,
                            Some(player),
                            &StoreAction(_StoreAction::Reveal(secret)),
                        )?;
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn logger(&self) -> &Rc<RefCell<Logger<S::Event>>> {
        match self.0.as_ref().unwrap() {
            _StoreState::Ready { logger, .. } | _StoreState::Pending { logger, .. } => logger,
        }
    }

    fn set_logger(&mut self, logger: Rc<RefCell<Logger<S::Event>>>) {
        match self.0.as_mut().unwrap() {
            _StoreState::Ready {
                logger: state_logger,
                ..
            }
            | _StoreState::Pending {
                logger: state_logger,
                ..
            } => *state_logger = logger,
        }
    }
}

impl<S: State> crate::State for StoreState<S> {
    type ID = S::ID;
    type Nonce = S::Nonce;
    type Action = StoreAction<S>;

    fn version() -> &'static [u8] {
        S::version()
    }

    fn challenge(address: &crate::crypto::Address) -> String {
        S::challenge(address)
    }

    fn approval(player: &crate::crypto::Address, subkey: &crate::crypto::Address) -> String {
        S::approval(player, subkey)
    }

    fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        crate::forbid!(data.len() < 3 * size_of::<u32>());

        Ok(Self(Some(_StoreState::Ready {
            state: {
                let state = S::deserialize(&data[..data.len() - 3 * size_of::<u32>()])?;
                data = &data[data.len() - 3 * size_of::<u32>()..];
                state
            },
            secrets: Default::default(),
            action_count: crate::utils::read_u32_usize(&mut data)?,
            reveal_count: crate::utils::read_u32_usize(&mut data)?,
            event_count: crate::utils::read_u32_usize(&mut data)?,
            logger: Rc::new(RefCell::new(Logger::new(|_, _| ()))),
        })))
    }

    fn is_serializable(&self) -> bool {
        match &self.0 {
            Some(_StoreState::Ready {
                state,
                action_count,
                reveal_count,
                event_count,
                ..
            }) => {
                TryInto::<u32>::try_into(*action_count).is_ok()
                    && TryInto::<u32>::try_into(*reveal_count).is_ok()
                    && TryInto::<u32>::try_into(*event_count).is_ok()
                    && state.is_serializable()
            }
            _ => false,
        }
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        match self.0.as_ref()? {
            _StoreState::Ready {
                state,
                action_count,
                reveal_count,
                event_count,
                ..
            } => {
                let mut data = State::serialize(state)?;

                crate::utils::write_u32_usize(&mut data, *action_count).ok()?;
                crate::utils::write_u32_usize(&mut data, *reveal_count).ok()?;
                crate::utils::write_u32_usize(&mut data, *event_count).ok()?;

                Some(data)
            }
            _ => None,
        }
    }

    fn apply(
        &mut self,
        player: Option<crate::Player>,
        action: &Self::Action,
    ) -> Result<(), String> {
        match &action.0 {
            _StoreAction::Play(action) => {
                if let _StoreState::Ready { state, .. } =
                    self.0.as_ref().ok_or("self.0.is_none()")?
                {
                    state.verify(player, action)?;

                    if let Some(state) = self.0.take() {
                        drop(self.0.replace(
                            if let _StoreState::Ready {
                                state,
                                secrets: [secret1, secret2],
                                action_count,
                                reveal_count,
                                event_count,
                                logger,
                            } = state
                            {
                                let secrets = [
                                    secret1.map(|secret| Rc::new(RefCell::new(secret))),
                                    secret2.map(|secret| Rc::new(RefCell::new(secret))),
                                ];

                                let phase = Rc::new(RefCell::new(Phase::Idle {
                                    random: None,
                                    secret: None,
                                }));

                                _StoreState::Pending {
                                    state: state.apply(
                                        player,
                                        action,
                                        Context {
                                            phase: phase.clone(),
                                            secrets: secrets.clone(),
                                            event_count,
                                            logger: (true, logger.clone()),
                                        },
                                    ),
                                    secrets,
                                    action_count: action_count + 1,
                                    reveal_count,
                                    phase,
                                    logger,
                                }
                            } else {
                                unreachable!("{}:{}:{}", file!(), line!(), column!());
                            },
                        ));
                    } else {
                        unreachable!("{}:{}:{}", file!(), line!(), column!());
                    }
                } else {
                    return Err("self.0 != _StoreState::Ready { .. }".to_string());
                }
            }
            _StoreAction::RandomCommit(hash) => {
                if let _StoreState::Pending {
                    phase,
                    reveal_count,
                    ..
                } = self.0.as_mut().ok_or("self.0.is_none()")?
                {
                    let borrowed_phase = phase.try_borrow().map_err(|error| error.to_string())?;

                    if let Phase::RandomCommit = *borrowed_phase {
                        drop(borrowed_phase);

                        crate::forbid!(player != None && player != Some(0));

                        *reveal_count += 1;

                        phase.replace(Phase::RandomReply {
                            hash: *hash,
                            owner_hash: player.is_none(),
                        });
                    } else {
                        return Err("borrowed_phase != Phase::RandomCommit".to_string());
                    }
                } else {
                    return Err("self.0 != _StoreState::Pending { .. }".to_string());
                }
            }
            _StoreAction::RandomReply(seed) => {
                if let _StoreState::Pending {
                    phase,
                    reveal_count,
                    ..
                } = self.0.as_mut().ok_or("self.0.is_none()")?
                {
                    let borrowed_phase = phase.try_borrow().map_err(|error| error.to_string())?;

                    if let Phase::RandomReply { hash, owner_hash } = *borrowed_phase {
                        drop(borrowed_phase);

                        crate::forbid!(player != None && player != Some(1));

                        *reveal_count += 1;

                        phase.replace(Phase::RandomReveal {
                            hash,
                            owner_hash,
                            reply: seed.to_vec(),
                        });
                    } else {
                        return Err("borrowed_phase != Phase::RandomReply { .. }".to_string());
                    }
                } else {
                    return Err("self.0 != _StoreState::Pending { .. }".to_string());
                }
            }
            _StoreAction::RandomReveal(seed) => {
                if let _StoreState::Pending {
                    phase,
                    reveal_count,
                    ..
                } = self.0.as_mut().ok_or("self.0.is_none()")?
                {
                    let borrowed_phase = phase.try_borrow().map_err(|error| error.to_string())?;

                    if let Phase::RandomReveal {
                        hash,
                        owner_hash,
                        reply,
                    } = &*borrowed_phase
                    {
                        if *owner_hash {
                            crate::forbid!(player != None);
                        } else {
                            crate::forbid!(player != None && player != Some(0));
                        }

                        if player.is_some() || *owner_hash {
                            crate::forbid!(crate::crypto::keccak256(seed) != *hash);
                        }

                        let seed = reply
                            .iter()
                            .zip(seed)
                            .map(|(x, y)| x ^ y)
                            .collect::<Vec<_>>()
                            .as_slice()
                            .try_into()
                            .map_err(|error| format!("{}", error))?;

                        drop(borrowed_phase);

                        *reveal_count += 1;

                        phase.replace(Phase::Idle {
                            random: Some(Rc::new(RefCell::new(rand::SeedableRng::from_seed(seed)))),
                            secret: None,
                        });
                    } else {
                        return Err("borrowed_phase != Phase::RandomReveal { .. }".to_string());
                    }
                } else {
                    return Err("self.0 != _StoreState::Pending { .. }".to_string());
                }
            }
            _StoreAction::Reveal(secret) => {
                if let _StoreState::Pending {
                    phase,
                    reveal_count,
                    ..
                } = self.0.as_mut().ok_or("self.0.is_none()")?
                {
                    let borrowed_phase = phase.try_borrow().map_err(|error| error.to_string())?;

                    if let Phase::Reveal {
                        random,
                        request:
                            RevealRequest {
                                player: revealer,
                                verify,
                                ..
                            },
                    } = &*borrowed_phase
                    {
                        crate::forbid!(player != None && player != Some(*revealer));
                        crate::forbid!(!verify(secret));

                        let random = random.clone();

                        drop(borrowed_phase);

                        *reveal_count += 1;

                        phase.replace(Phase::Idle {
                            random,
                            secret: Some(secret.clone()),
                        });
                    } else {
                        return Err("borrowed_phase != Phase::Reveal { .. }".to_string());
                    }
                } else {
                    return Err("self.0 != _StoreState::Pending { .. }".to_string());
                }
            }
        }

        if let Some(state) = self.0.take() {
            drop(self.0.replace(
                if let _StoreState::Pending {
                    mut state,
                    secrets: [secret1, secret2],
                    action_count,
                    reveal_count,
                    phase,
                    logger,
                } = state
                {
                    if let Poll::Ready((state, context)) = state
                        .as_mut()
                        .poll(&mut task::Context::from_waker(&phantom_waker()))
                    {
                        drop(context.secrets);

                        _StoreState::Ready {
                            state,
                            secrets: [
                                secret1.map(|secret| {
                                    Rc::try_unwrap(secret).ok().unwrap().into_inner()
                                }),
                                secret2.map(|secret| {
                                    Rc::try_unwrap(secret).ok().unwrap().into_inner()
                                }),
                            ],
                            action_count,
                            reveal_count,
                            event_count: context.event_count,
                            logger,
                        }
                    } else {
                        _StoreState::Pending {
                            state,
                            secrets: [secret1, secret2],
                            action_count,
                            reveal_count,
                            phase,
                            logger,
                        }
                    }
                } else {
                    state
                },
            ));
        } else {
            unreachable!("{}:{}:{}", file!(), line!(), column!());
        }

        Ok(())
    }
}

enum _StoreState<S: State> {
    Ready {
        state: S,
        secrets: [Option<(S::Secret, rand_xorshift::XorShiftRng)>; 2],
        action_count: usize,
        reveal_count: usize,
        event_count: usize,
        logger: Rc<RefCell<Logger<S::Event>>>,
    },
    Pending {
        state: Pin<Box<dyn Future<Output = (S, Context<S::Secret, S::Event>)>>>,
        secrets: [Option<Rc<RefCell<(S::Secret, rand_xorshift::XorShiftRng)>>>; 2],
        action_count: usize,
        reveal_count: usize,
        phase: Rc<RefCell<Phase<S::Secret>>>,
        logger: Rc<RefCell<Logger<S::Event>>>,
    },
}

impl<S: State> Clone for _StoreState<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Ready {
                state,
                secrets,
                action_count,
                reveal_count,
                event_count,
                logger,
            } => Self::Ready {
                state: state.clone(),
                secrets: secrets.clone(),
                action_count: *action_count,
                reveal_count: *reveal_count,
                event_count: *event_count,
                logger: logger.clone(),
            },
            _ => panic!("{}", "_StoreState::Pending {{ .. }}.clone()"),
        }
    }
}

/// Simulation event log
#[derive(serde::Serialize)]
#[serde(tag = "status", content = "events")]
pub enum Log<S: State> {
    /// A log for a complete transition.
    #[serde(rename = "complete")]
    #[serde(bound = "Vec<S::Event>: serde::Serialize")]
    Complete(Vec<S::Event>),

    /// A log for an incomplete transition.
    #[serde(rename = "incomplete")]
    #[serde(bound = "Vec<S::Event>: serde::Serialize")]
    Incomplete(Vec<S::Event>),
}

/// Client store state transition
#[derive(derivative::Derivative, Clone)]
#[derivative(Debug = "transparent")]
#[derivative(Debug(bound = "S::Action: Debug"))]
pub struct StoreAction<S: State>(_StoreAction<S>);

impl<S: State> StoreAction<S> {
    /// Constructs a new store state transition.
    pub fn new(action: S::Action) -> Self {
        Self(_StoreAction::Play(action))
    }
}

impl<S: State> crate::Action for StoreAction<S> {
    fn deserialize(data: &[u8]) -> Result<Self, String> {
        Ok(Self(_StoreAction::deserialize(data)?))
    }

    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }
}

#[derive(derivative::Derivative, Clone)]
#[derivative(Debug)]
enum _StoreAction<S: State> {
    Play(S::Action),
    RandomCommit(#[derivative(Debug(format_with = "crate::utils::fmt_hex"))] crate::crypto::Hash),
    RandomReply(#[derivative(Debug(format_with = "crate::utils::fmt_hex"))] Vec<u8>),
    RandomReveal(#[derivative(Debug(format_with = "crate::utils::fmt_hex"))] Vec<u8>),
    Reveal(#[derivative(Debug(format_with = "crate::utils::fmt_hex"))] Vec<u8>),
}

impl<S: State> crate::Action for _StoreAction<S> {
    fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        match crate::utils::read_u8(&mut data)? {
            0 => Ok(Self::Play(S::Action::deserialize(data)?)),
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
            Self::Play(action) => {
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

/// Domain-specific store state trait
pub trait State: Clone {
    /// Identifier type
    type ID: crate::ID;

    /// Nonce type
    type Nonce: crate::Nonce;

    /// Action type
    type Action: crate::Action;

    /// Event type
    type Event;

    /// Secret type
    type Secret: Secret;

    /// Gets the ABI version of this implementation.
    ///
    /// See [super::tag] and [super::version::version] for potentially helpful utilities.
    fn version() -> &'static [u8];

    /// Gets the challenge that must be signed in order to certify the subkey with the given address.
    fn challenge(address: &crate::crypto::Address) -> String {
        format!(
            "Sign to play! This won't cost anything.\n\n{}\n",
            crate::crypto::Addressable::eip55(address)
        )
    }

    /// Gets the approval that must be signed by the owner in order to approve a subkey for a player.
    fn approval(player: &crate::crypto::Address, subkey: &crate::crypto::Address) -> String {
        format!(
            "Approve {} for {}.",
            crate::crypto::Addressable::eip55(subkey),
            crate::crypto::Addressable::eip55(player),
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
        context: Context<Self::Secret, Self::Event>,
    ) -> Pin<Box<dyn Future<Output = (Self, Context<Self::Secret, Self::Event>)>>>;
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
pub struct Context<S: Secret, E> {
    phase: Rc<RefCell<Phase<S>>>,
    secrets: [Option<Rc<RefCell<(S, rand_xorshift::XorShiftRng)>>>; 2],
    event_count: usize,
    logger: (bool, Rc<RefCell<Logger<E>>>),
}

impl<S: Secret, E> Context<S, E> {
    /// Mutates a player's secret information.
    pub fn mutate_secret(
        &mut self,
        player: crate::Player,
        mutate: impl Fn(MutateSecretInfo<S, E>),
    ) {
        self.event_count += 1;

        if let Some(secret) = &self.secrets[usize::from(player)] {
            let (secret, random) = &mut *secret.try_borrow_mut().unwrap();

            if self.logger.0 {
                if let Ok(mut logger) = self.logger.1.try_borrow_mut() {
                    if logger.enabled && self.event_count > logger.event_count {
                        logger.event_count = self.event_count;

                        mutate(MutateSecretInfo {
                            secret,
                            random,
                            log: &mut |event| (logger.log)(Some(player), event),
                        });
                    } else {
                        mutate(MutateSecretInfo {
                            secret,
                            random,
                            log: &mut |_| (),
                        });
                    }
                } else {
                    mutate(MutateSecretInfo {
                        secret,
                        random,
                        log: &mut |_| (),
                    });
                }
            } else {
                mutate(MutateSecretInfo {
                    secret,
                    random,
                    log: &mut |_| (),
                });
            };
        }
    }

    /// Mutates a player's secret information, or if we don't have that secret, emit an arbitrary event
    pub fn mutate_secret_or_log(
        &mut self,
        player: crate::Player,
        mutate: impl Fn(MutateSecretInfo<S, E>),
        event: E,
    ) {
        if self.secrets[usize::from(player)].is_some() {
            self.mutate_secret(player, mutate);
        } else {
            self.event_count += 1;
            if self.logger.0 {
                if let Ok(mut logger) = self.logger.1.try_borrow_mut() {
                    logger.log(self.event_count, None, event);
                }
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
        reveal: impl Fn(&S) -> T + 'static,
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
        reveal: impl Fn(&S) -> T + 'static,
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
    pub fn random(&mut self) -> impl Future<Output = impl rand::RngCore> {
        let phase = self.phase.try_borrow().unwrap();

        if let Phase::Idle { random: None, .. } = *phase {
            drop(phase);

            self.phase.replace(Phase::RandomCommit);
        }

        SharedXorShiftRngFuture(self.phase.clone())
    }

    /// Logs an event if logging is enabled.
    ///
    /// See [Context::enable_logs].
    pub fn log(&mut self, event: E) {
        if self.logger.0 {
            if let Ok(mut logger) = self.logger.1.try_borrow_mut() {
                self.event_count += 1;

                logger.log(self.event_count, None, event);
            }
        }
    }

    /// Enables or disables logging.
    ///
    /// See [Context::log].
    pub fn enable_logs(&mut self, enabled: bool) {
        self.logger.0 = enabled;
    }
}

/// [Context::mutate_secret] callback data
#[non_exhaustive]
pub struct MutateSecretInfo<'a, S: Secret, E> {
    /// The secret.
    pub secret: &'a mut S,

    /// A source of entropy.
    pub random: &'a mut dyn rand::RngCore,

    /// An event logger.
    pub log: &'a mut dyn FnMut(E),
}

impl<S: Secret, E> Deref for MutateSecretInfo<'_, S, E> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.secret
    }
}

impl<S: Secret, E> DerefMut for MutateSecretInfo<'_, S, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.secret
    }
}

impl<S: Secret, E> MutateSecretInfo<'_, S, E> {
    /// Logs an event.
    pub fn log(&mut self, event: E) {
        (self.log)(event)
    }
}

pub use tester::Tester;

#[derive(Debug)]
enum Phase<S: Secret> {
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

#[derive(derivative::Derivative)]
#[derivative(Debug)]
struct RevealRequest<S: Secret> {
    player: crate::Player,
    #[derivative(Debug = "ignore")]
    reveal: Box<dyn Fn(&S) -> Vec<u8>>,
    #[derivative(Debug = "ignore")]
    verify: Box<dyn Fn(&[u8]) -> bool>,
}

struct Logger<E> {
    log: Box<dyn FnMut(Option<crate::Player>, E)>,
    event_count: usize,
    enabled: bool,
}

impl<E> Logger<E> {
    fn new(log: impl FnMut(Option<crate::Player>, E) + 'static) -> Self {
        Self {
            log: Box::new(log),
            event_count: Default::default(),
            enabled: true,
        }
    }

    fn log(&mut self, event_count: usize, target: Option<crate::Player>, event: E) {
        if self.enabled && event_count > self.event_count {
            self.event_count = event_count;

            (self.log)(target, event);
        }
    }
}

struct SharedXorShiftRngFuture<S: Secret>(Rc<RefCell<Phase<S>>>);

impl<S: Secret> Future for SharedXorShiftRngFuture<S> {
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

struct RevealFuture<S: Secret>(Rc<RefCell<Phase<S>>>);

impl<S: Secret> Future for RevealFuture<S> {
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
