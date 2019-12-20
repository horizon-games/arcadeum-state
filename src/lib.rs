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

#[cfg(feature = "std")]
use std::{
    collections::BTreeMap,
    convert::TryInto,
    fmt::{Debug, Error, Formatter},
    mem::size_of,
    ops::{Deref, DerefMut, Range},
};

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use {
    alloc::{collections::BTreeMap, format, prelude::v1::*, vec},
    core::{
        convert::TryInto,
        fmt::{Debug, Error, Formatter},
        mem::size_of,
        ops::{Deref, DerefMut, Range},
    },
};

/// Client store
pub mod store;

/// Cryptographic utilities
pub mod crypto;

/// Utilities
pub mod utils;

#[cfg(feature = "debug")]
#[doc(hidden)]
pub mod debug;

mod error;

/// Authenticated state
pub struct Proof<S: State> {
    root: RootProof<S>,
    actions: Vec<ProofAction<S::Action>>,
    proofs: [Option<PlayerProof<S>>; 3],
    hash: crypto::Hash,
    state: ProofState<S>,
}

impl<S: State> Proof<S> {
    /// Constructs a bare proof from a root proof.
    pub fn new(root: RootProof<S>) -> Self {
        let actions = root.actions.clone();

        let proofs = [
            Some(PlayerProof {
                state: root.state.clone(),
                range: 0..root.actions.len(),
                signature: root.signature,
            }),
            None,
            None,
        ];

        let state = root.compute_state();

        let mut proof = Self {
            root,
            actions,
            proofs,
            hash: Default::default(),
            state,
        };

        proof.hash = proof.compute_hash();

        proof
    }

    /// Updates the proof's state from a binary representation.
    ///
    /// `data` must have been constructed using [Proof::serialize] on a proof with the same root.
    pub fn deserialize(&mut self, data: &[u8]) -> Result<(), String> {
        self.deserialize_and_init(data, |_| ())
    }

    /// Generates a binary representation that can be used to reconstruct the proof.
    ///
    /// See [Proof::deserialize].
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        let state = self
            .proofs
            .iter()
            .filter_map(Option::as_ref)
            .find(|proof| proof.range.start == 0)
            .unwrap()
            .state
            .serialize()
            .unwrap();

        utils::write_u32_usize(&mut data, state.len()).unwrap();
        data.extend(state);

        utils::write_u32_usize(&mut data, self.actions.len()).unwrap();

        for action in &self.actions {
            let action = action.serialize();
            utils::write_u32_usize(&mut data, action.len()).unwrap();
            data.extend(action);
        }

        for proof in &self.proofs {
            if let Some(proof) = proof {
                utils::write_u8_bool(&mut data, true);

                utils::write_u32_usize(&mut data, proof.range.start).unwrap();
                utils::write_u32_usize(&mut data, proof.range.end).unwrap();

                data.extend(proof.signature.iter());
            } else {
                utils::write_u8_bool(&mut data, false);
            }
        }

        data
    }

    /// Gets the digest of the proof.
    pub fn hash(&self) -> &crypto::Hash {
        &self.hash
    }

    /// Gets the state of the proof.
    pub fn state(&self) -> &ProofState<S> {
        &self.state
    }

    /// Verifies and applies a cryptographically constructed diff to the proof.
    ///
    /// `diff` must have been constructed using [Proof::diff] on a proof with the same digest.
    pub fn apply(&mut self, diff: &Diff<S::Action>) -> Result<(), error::Error> {
        forbid!(diff.proof != self.hash);

        let player = if diff.author == self.root.author {
            None
        } else {
            let player = self.state.player(&diff.author);
            forbid!(player.is_none());
            player
        };

        let proof = self
            .proofs
            .iter()
            .filter_map(Option::as_ref)
            .max_by_key(|proof| proof.range.end)
            .unwrap();

        let mut state = proof.state.clone();
        let mut start = proof.range.start;

        let mut latest = self.compute_state();

        for (i, action) in diff.actions.iter().enumerate() {
            slash!(action.player != player);

            latest.apply(action).map_err(error::Error::Hard)?;

            if latest.serialize().is_some() {
                state = latest.clone();
                start = self.actions.len() + i + 1;
            }
        }

        match player {
            None => {
                let mut actions =
                    Vec::with_capacity(self.actions.len() + diff.actions.len() - start);

                if start < self.actions.len() {
                    actions.extend(self.actions[start..].iter().cloned());
                    actions.extend(diff.actions.iter().cloned());
                } else {
                    actions.extend(diff.actions[start - self.actions.len()..].iter().cloned());
                }

                let mut message = state.serialize().unwrap();
                message.extend(actions.iter().flat_map(ProofAction::serialize));

                slash!(
                    crypto::recover(&message, &diff.proof_signature).map_err(error::Error::Hard)?
                        != self.root.author
                );

                self.proofs = [
                    Some(PlayerProof {
                        state,
                        range: 0..actions.len(),
                        signature: diff.proof_signature,
                    }),
                    None,
                    None,
                ];

                self.actions = actions;
            }
            Some(player) => {
                let consensus = self.proofs[1..]
                    .iter()
                    .enumerate()
                    .all(|(i, proof)| i == usize::from(player) || proof.is_some());

                let offset = if consensus {
                    self.proofs[1..]
                        .iter()
                        .enumerate()
                        .filter_map(|(i, proof)| {
                            if i == usize::from(player) {
                                Some(start)
                            } else {
                                proof.as_ref().map(|proof| proof.range.start)
                            }
                        })
                        .min()
                        .unwrap()
                } else {
                    self.proofs
                        .iter()
                        .enumerate()
                        .filter_map(|(i, proof)| {
                            if i == 1 + usize::from(player) {
                                Some(start)
                            } else {
                                proof.as_ref().map(|proof| proof.range.start)
                            }
                        })
                        .min()
                        .unwrap()
                };

                let mut actions =
                    Vec::with_capacity(self.actions.len() + diff.actions.len() - offset);

                actions.extend(self.actions[offset..].iter().cloned());
                actions.extend(diff.actions.iter().cloned());

                let mut message = state.serialize().unwrap();

                message.extend(
                    actions[start - offset..]
                        .iter()
                        .flat_map(ProofAction::serialize),
                );

                slash!(
                    latest.player(
                        &crypto::recover(&message, &diff.proof_signature)
                            .map_err(error::Error::Hard)?
                    ) != Some(player)
                );

                self.proofs[1 + usize::from(player)] = Some(PlayerProof {
                    state,
                    range: start..self.actions.len() + diff.actions.len(),
                    signature: diff.proof_signature,
                });

                if consensus
                    && self.proofs[0].as_ref().is_some()
                    && self.proofs[0].as_ref().unwrap().range.end <= offset
                {
                    self.proofs[0] = None;
                }

                for proof in &mut self.proofs {
                    if let Some(proof) = proof {
                        proof.range.start -= offset;
                        proof.range.end -= offset;
                    }
                }

                self.actions = actions;
            }
        }

        self.hash = self.compute_hash();
        self.state = latest;

        Ok(())
    }

    /// Generates a diff that can be applied to a proof with the same digest.
    ///
    /// See [Proof::apply].
    pub fn diff(
        &self,
        actions: Vec<ProofAction<S::Action>>,
        sign: &mut impl FnMut(&[u8]) -> Result<crypto::Signature, String>,
    ) -> Result<Diff<S::Action>, String> {
        let proof = self
            .proofs
            .iter()
            .filter_map(Option::as_ref)
            .max_by_key(|proof| proof.range.end)
            .unwrap();

        let mut state = proof.state.clone();
        let mut start = proof.range.start;

        let mut latest = self.compute_state();

        for (i, action) in actions.iter().enumerate() {
            forbid!(action.player != actions.first().unwrap().player);

            latest.apply(action)?;

            if latest.serialize().is_some() {
                state = latest.clone();
                start = self.actions.len() + i + 1;
            }
        }

        let mut message = state.serialize().unwrap();

        if start < self.actions.len() {
            message.extend(
                self.actions[start..]
                    .iter()
                    .flat_map(ProofAction::serialize),
            );

            message.extend(actions.iter().flat_map(ProofAction::serialize));
        } else {
            message.extend(
                actions[start - self.actions.len()..]
                    .iter()
                    .flat_map(ProofAction::serialize),
            );
        }

        let signature = sign(&message)?;
        let author = crypto::recover(&message, &signature)?;

        if author != self.root.author {
            let player = latest.player(&author);

            forbid!(player.is_none());

            if let Some(action) = actions.first() {
                forbid!(player != action.player);
            }
        }

        Ok(Diff::new(self.hash, actions, signature, sign)?)
    }

    fn deserialize_and_init(
        &mut self,
        mut data: &[u8],
        init: impl FnOnce(&mut S),
    ) -> Result<(), String> {
        forbid!(
            data.len()
                < size_of::<u32>()
                    + size_of::<u32>()
                    + 3
                    + size_of::<u32>()
                    + size_of::<u32>()
                    + size_of::<crypto::Signature>()
        );

        let hash = tiny_keccak::keccak256(data);

        let mut state = {
            let size = utils::read_u32_usize(&mut data)?;

            forbid!(data.len() < size);
            let state = ProofState::<S>::deserialize_and_init(&data[..size], init)?;
            data = &data[size..];

            state
        };

        forbid!(state.id != self.root.state.id);
        forbid!(state.players != self.root.state.players);

        let actions = {
            let length = utils::read_u32_usize(&mut data)?;

            let mut actions = Vec::with_capacity(length);

            for _ in 0..length {
                let size = utils::read_u32_usize(&mut data)?;

                forbid!(data.len() < size);
                actions.push(ProofAction::deserialize(&data[..size])?);
                data = &data[size..];
            }

            actions
        };

        let (ranges, signatures) = {
            let mut ranges = Vec::with_capacity(3);
            let mut signatures = Vec::with_capacity(ranges.capacity());

            let mut minimal = false;

            for _ in 0..ranges.capacity() {
                if utils::read_u8_bool(&mut data)? {
                    ranges.push(Some({
                        let range =
                            utils::read_u32_usize(&mut data)?..utils::read_u32_usize(&mut data)?;

                        forbid!(range.end > actions.len());
                        forbid!(range.start > range.end);

                        if range.start == 0 {
                            minimal = true;
                        }

                        range
                    }));

                    signatures.push(Some({
                        let mut signature = [0; size_of::<crypto::Signature>()];

                        forbid!(data.len() < size_of::<crypto::Signature>());
                        signature.copy_from_slice(&data[..size_of::<crypto::Signature>()]);
                        data = &data[size_of::<crypto::Signature>()..];

                        signature
                    }));
                } else {
                    ranges.push(None);

                    signatures.push(None);
                }
            }

            forbid!(!minimal);

            (ranges, signatures)
        };

        forbid!(ranges[0].is_none() && ranges[1..].iter().any(Option::is_none));
        forbid!(!data.is_empty());

        let proofs = {
            let mut proofs = [None, None, None];

            for i in 0..=actions.len() {
                let serializable = ranges
                    .iter()
                    .filter_map(Option::as_ref)
                    .any(|range| range.start == i);

                let unserializable = ranges
                    .iter()
                    .filter_map(Option::as_ref)
                    .any(|range| range.start < i && i <= range.end);

                if serializable || unserializable {
                    let data = state.serialize();

                    forbid!(serializable && data.is_none());
                    forbid!(unserializable && data.is_some());

                    if serializable {
                        for (j, range) in ranges.iter().enumerate() {
                            if let Some(range) = range {
                                if range.start == i {
                                    proofs[j] = Some(PlayerProof {
                                        state: state.clone(),
                                        range: range.clone(),
                                        signature: signatures[j].unwrap(),
                                    });
                                }
                            }
                        }
                    }
                }

                if i < actions.len() {
                    let action = &actions[i];

                    let range = ranges[0].as_ref();

                    match action.player {
                        None => {
                            forbid!(range.is_none());
                            forbid!(range.unwrap().end <= i);
                        }
                        Some(player) => {
                            if range.is_none() || range.unwrap().end <= i {
                                forbid!(1 + usize::from(player) >= ranges.len());

                                let range = ranges[1 + usize::from(player)].as_ref();

                                forbid!(range.is_none());
                                forbid!(range.unwrap().end <= i);
                            }
                        }
                    }

                    state.apply(action)?;
                }
            }

            for (i, proof) in proofs.iter().enumerate() {
                if let Some(proof) = proof {
                    let mut message = proof.state.serialize().unwrap();

                    message.extend(
                        actions[proof.range.clone()]
                            .iter()
                            .flat_map(ProofAction::serialize),
                    );

                    let author = crypto::recover(&message, &proof.signature)?;

                    match i {
                        0 => forbid!(author != self.root.author),
                        i => forbid!(state.player(&author).map(usize::from) != Some(i - 1)),
                    }
                }
            }

            proofs
        };

        self.actions = actions;
        self.proofs = proofs;
        self.hash = hash;
        self.state = state;

        Ok(())
    }

    fn compute_hash(&self) -> crypto::Hash {
        tiny_keccak::keccak256(&self.serialize())
    }

    fn compute_state(&self) -> ProofState<S> {
        let proof = self
            .proofs
            .iter()
            .filter_map(Option::as_ref)
            .max_by_key(|proof| proof.range.end)
            .unwrap();

        let mut state = proof.state.clone();

        for action in &self.actions[proof.range.start..] {
            state.apply(action).unwrap();
        }

        state
    }
}

impl<S: State> Clone for Proof<S> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            actions: self.actions.clone(),
            proofs: self.proofs.clone(),
            hash: self.hash,
            state: self.compute_state(),
        }
    }
}

/// Authenticated initial state
pub struct RootProof<S: State> {
    state: ProofState<S>,
    actions: Vec<ProofAction<S::Action>>,
    signature: crypto::Signature,
    hash: crypto::Hash,
    author: crypto::Address,
    latest: ProofState<S>,
}

impl<S: State> Clone for RootProof<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            actions: self.actions.clone(),
            signature: self.signature,
            hash: self.hash,
            author: self.author,
            latest: self.compute_state(),
        }
    }
}

impl<S: State> RootProof<S> {
    /// Constructs a root proof from `state` and `actions`.
    ///
    /// `state` must be serializable.
    pub fn new(
        mut state: ProofState<S>,
        actions: Vec<ProofAction<S::Action>>,
        sign: &mut impl FnMut(&[u8]) -> Result<crypto::Signature, String>,
    ) -> Result<Self, String> {
        let mut start = 0;

        let mut latest = state.clone();

        for (i, action) in actions.iter().enumerate() {
            latest.apply(action)?;

            if latest.serialize().is_some() {
                state = latest.clone();
                start = i + 1;
            }
        }

        let actions = actions[start..].to_vec();

        let mut message = state.serialize().unwrap();
        message.extend(actions.iter().flat_map(ProofAction::serialize));

        let signature = sign(&message)?;

        let mut proof = Self {
            state,
            actions,
            signature,
            hash: Default::default(),
            author: crypto::recover(&message, &signature)?,
            latest,
        };

        proof.hash = tiny_keccak::keccak256(&proof.serialize());

        Ok(proof)
    }

    /// Constructs a root proof from its binary representation.
    ///
    /// `data` must have been constructed using [RootProof::serialize].
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        Self::deserialize_and_init(data, |_| ())
    }

    /// Generates a binary representation that can be used to reconstruct the root proof.
    ///
    /// See [RootProof::deserialize].
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        let state = self.state.serialize().unwrap();
        utils::write_u32_usize(&mut data, state.len()).unwrap();
        data.extend(state);

        utils::write_u32_usize(&mut data, self.actions.len()).unwrap();

        for action in &self.actions {
            let action = action.serialize();
            utils::write_u32_usize(&mut data, action.len()).unwrap();
            data.extend(action);
        }

        data.extend(self.signature.iter());

        data
    }

    /// Gets the digest of the root proof.
    pub fn hash(&self) -> &crypto::Hash {
        &self.hash
    }

    /// Gets the state of the root proof.
    pub fn state(&self) -> &ProofState<S> {
        &self.latest
    }

    fn deserialize_and_init(mut data: &[u8], init: impl FnOnce(&mut S)) -> Result<Self, String> {
        forbid!(data.len() < size_of::<u32>() + size_of::<u32>() + size_of::<crypto::Signature>());

        let hash = tiny_keccak::keccak256(data);

        let size = utils::read_u32_usize(&mut data)?;

        forbid!(data.len() < size);
        let state = ProofState::<S>::deserialize_and_init(&data[..size], init)?;
        data = &data[size..];

        let length = utils::read_u32_usize(&mut data)?;

        let mut actions = Vec::with_capacity(length);
        let mut latest = state.clone();

        for _ in 0..length {
            let size = utils::read_u32_usize(&mut data)?;

            forbid!(data.len() < size);
            let action = ProofAction::deserialize(&data[..size])?;
            data = &data[size..];

            latest.apply(&action)?;

            forbid!(latest.serialize().is_some());

            actions.push(action);
        }

        forbid!(data.len() != size_of::<crypto::Signature>());
        let mut signature = [0; size_of::<crypto::Signature>()];
        signature.copy_from_slice(data);

        let mut message = state.serialize().unwrap();
        message.extend(actions.iter().flat_map(ProofAction::serialize));

        Ok(Self {
            state,
            actions,
            signature,
            hash,
            author: crypto::recover(&message, &signature)?,
            latest,
        })
    }

    fn compute_state(&self) -> ProofState<S> {
        let mut state = self.state.clone();

        for action in &self.actions {
            state.apply(action).unwrap();
        }

        state
    }
}

#[derive(Clone)]
struct PlayerProof<S: State> {
    state: ProofState<S>,
    range: Range<usize>,
    signature: crypto::Signature,
}

/// Authenticated state transition
#[derive(Clone)]
pub struct Diff<A: Action> {
    proof: crypto::Hash,
    actions: Vec<ProofAction<A>>,
    proof_signature: crypto::Signature,
    signature: crypto::Signature,
    author: crypto::Address,
}

impl<A: Action> Diff<A> {
    /// Constructs a diff from its binary representation.
    ///
    /// `data` must have been constructed using [Diff::serialize].
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        forbid!(
            data.len()
                < size_of::<crypto::Hash>()
                    + size_of::<u32>()
                    + size_of::<crypto::Signature>()
                    + size_of::<crypto::Signature>()
        );

        let author = crypto::recover(
            &data[..data.len() - size_of::<crypto::Signature>()],
            &data[data.len() - size_of::<crypto::Signature>()..],
        )?;

        let proof = data[..size_of::<crypto::Hash>()]
            .try_into()
            .map_err(|error| format!("{}", error))?;

        let mut data = &data[size_of::<crypto::Hash>()..];

        let length = utils::read_u32_usize(&mut data)?;

        let mut actions = Vec::with_capacity(length);

        for _ in 0..length {
            let size = utils::read_u32_usize(&mut data)?;

            forbid!(data.len() < size);
            actions.push(ProofAction::deserialize(&data[..size])?);
            data = &data[size..];
        }

        forbid!(data.len() != size_of::<crypto::Signature>() + size_of::<crypto::Signature>());

        let mut proof_signature = [0; size_of::<crypto::Signature>()];
        proof_signature.copy_from_slice(&data[..size_of::<crypto::Signature>()]);

        let mut signature = [0; size_of::<crypto::Signature>()];
        signature.copy_from_slice(&data[size_of::<crypto::Signature>()..]);

        Ok(Self {
            proof,
            actions,
            proof_signature,
            signature,
            author,
        })
    }

    /// Generates a binary representation that can be used to reconstruct the diff.
    ///
    /// See [Diff::deserialize].
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend(&self.proof);

        utils::write_u32_usize(&mut data, self.actions.len()).unwrap();

        for action in &self.actions {
            let action = action.serialize();
            utils::write_u32_usize(&mut data, action.len()).unwrap();
            data.extend(action);
        }

        data.extend(self.proof_signature.iter());
        data.extend(self.signature.iter());

        data
    }

    fn new(
        proof: crypto::Hash,
        actions: Vec<ProofAction<A>>,
        proof_signature: crypto::Signature,
        sign: &mut impl FnMut(&[u8]) -> Result<crypto::Signature, String>,
    ) -> Result<Self, String> {
        let mut diff = Self {
            proof,
            actions,
            proof_signature,
            signature: [0; size_of::<crypto::Signature>()],
            author: Default::default(),
        };

        let message = diff.serialize();
        let message = &message[..message.len() - size_of::<crypto::Signature>()];

        diff.signature = sign(message)?;
        diff.author = crypto::recover(message, &diff.signature)?;

        Ok(diff)
    }
}

impl<A: Action + Debug> Debug for Diff<A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if f.alternate() {
            writeln!(
                f,
                "\
Diff {{
    proof: {:#?},
    actions: {:#?},
    proof_signature: {},
    signature: {},
    author: {},
}}\
                ",
                self.proof,
                self.actions,
                utils::hex(&self.proof_signature),
                utils::hex(&self.signature),
                crypto::eip55(&self.author),
            )
        } else {
            writeln!(
                f,
                "\
Diff {{
    proof: {:?},
    actions: {:?},
    proof_signature: {},
    signature: {},
    author: {},
}}\
                ",
                self.proof,
                self.actions,
                utils::hex(&self.proof_signature),
                utils::hex(&self.signature),
                crypto::eip55(&self.author),
            )
        }
    }
}

/// Consensus state
#[derive(Clone)]
pub struct ProofState<S: State> {
    id: S::ID,
    nonce: S::Nonce,
    players: [crypto::Address; 2],
    signatures: BTreeMap<crypto::Address, crypto::Signature>,
    state: S,
}

impl<S: State> ProofState<S> {
    /// Constructs a consensus state.
    ///
    /// `state` must be serializable.
    pub fn new(id: S::ID, players: [crypto::Address; 2], state: S) -> Result<Self, String> {
        forbid!(state.serialize().is_none());

        Ok(Self {
            id,
            nonce: Default::default(),
            players,
            signatures: BTreeMap::new(),
            state,
        })
    }

    /// Gets the identifier of the state.
    pub fn id(&self) -> &S::ID {
        &self.id
    }

    /// Gets the player associated with the given `address`, if any, otherwise [None].
    pub fn player(&self, address: &crypto::Address) -> Option<Player> {
        if let Some(player) = self.players.iter().position(|player| player == address) {
            return player.try_into().ok();
        }

        if let Some(signature) = self.signatures.get(address) {
            if let Ok(address) = &crypto::recover(S::certificate(address).as_bytes(), signature) {
                if let Some(player) = self.players.iter().position(|player| player == address) {
                    return player.try_into().ok();
                }
            }
        }

        None
    }

    /// Gets the domain-specific state.
    pub fn state(&self) -> &S {
        &self.state
    }

    fn deserialize_and_init(mut data: &[u8], init: impl FnOnce(&mut S)) -> Result<Self, String> {
        let id = S::ID::deserialize(&mut data)?;
        let nonce = S::Nonce::deserialize(&mut data)?;

        forbid!(data.len() < 2 * size_of::<crypto::Address>() + size_of::<u32>());

        let players: [crypto::Address; 2] = [
            data[..size_of::<crypto::Address>()]
                .try_into()
                .map_err(|error| format!("{}", error))?,
            data[size_of::<crypto::Address>()..2 * size_of::<crypto::Address>()]
                .try_into()
                .map_err(|error| format!("{}", error))?,
        ];

        data = &data[2 * size_of::<crypto::Address>()..];

        let length = utils::read_u32_usize(&mut data)?;

        forbid!(
            data.len() < length * (size_of::<crypto::Address>() + size_of::<crypto::Signature>())
        );

        let mut signatures = BTreeMap::new();
        let mut previous = None;

        for _ in 0..length {
            let address = data[..size_of::<crypto::Address>()]
                .try_into()
                .map_err(|error| format!("{}", error))?;

            data = &data[size_of::<crypto::Address>()..];

            if let Some(previous) = previous {
                forbid!(address <= previous);
            }

            previous = Some(address);

            let mut signature = [0; size_of::<crypto::Signature>()];
            signature.copy_from_slice(&data[..size_of::<crypto::Signature>()]);
            data = &data[size_of::<crypto::Signature>()..];

            signatures.insert(address, signature);
        }

        Ok(Self {
            id,
            nonce,
            players,
            signatures,
            state: {
                let mut state = S::deserialize(data)?;

                init(&mut state);

                state
            },
        })
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        let state = if let Some(state) = self.state.serialize() {
            state
        } else {
            return None;
        };

        let id = self.id.serialize();
        let nonce = self.nonce.serialize();

        let mut data = Vec::with_capacity(
            id.len()
                + nonce.len()
                + self.players.len() * size_of::<crypto::Address>()
                + size_of::<u32>()
                + self.signatures.len()
                    * (size_of::<crypto::Address>() + size_of::<crypto::Signature>())
                + state.len(),
        );

        data.extend(id);
        data.extend(nonce);

        for player in &self.players {
            data.extend(player);
        }

        if utils::write_u32_usize(&mut data, self.signatures.len()).is_err() {
            return None;
        }

        for (address, signature) in &self.signatures {
            data.extend(address);
            data.extend(signature.iter());
        }

        data.extend(state);

        Some(data)
    }

    fn apply(&mut self, action: &ProofAction<S::Action>) -> Result<(), String> {
        let player = action.player;

        forbid!(player.is_some() && usize::from(player.unwrap()) >= self.players.len());

        match &action.action {
            PlayerAction::Play(action) => self.state.apply(player, &action)?,

            PlayerAction::Certify { address, signature } => {
                forbid!(player.is_none());

                forbid!(self.signatures.contains_key(address));

                forbid!(
                    crypto::recover(S::certificate(address).as_bytes(), signature)?
                        != self.players[usize::from(player.unwrap())]
                );

                self.signatures.insert(*address, *signature);
            }
        }

        self.nonce = self.nonce.next();

        Ok(())
    }
}

/// Attributable state transition
#[derive(Clone)]
pub struct ProofAction<A: Action> {
    /// The player performing the action, or [None] if performed by the root author.
    pub player: Option<Player>,

    /// The action.
    pub action: PlayerAction<A>,
}

impl<A: Action> ProofAction<A> {
    fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        let player = match utils::read_u8(&mut data)? {
            0 => None,
            byte => Some(byte - 1),
        };

        let action = match utils::read_u8(&mut data)? {
            0 => PlayerAction::Play(A::deserialize(data)?),
            1 => {
                forbid!(
                    data.len() != size_of::<crypto::Address>() + size_of::<crypto::Signature>()
                );

                let address = data[..size_of::<crypto::Address>()]
                    .try_into()
                    .map_err(|error| format!("{}", error))?;

                let mut signature = [0; size_of::<crypto::Signature>()];
                signature.copy_from_slice(&data[size_of::<crypto::Address>()..]);

                PlayerAction::Certify { address, signature }
            }
            byte => return Err(format!("byte == {}", byte)),
        };

        Ok(Self { player, action })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = vec![match self.player {
            None => 0,
            Some(player) => 1 + player,
        }];

        match &self.action {
            PlayerAction::Play(action) => {
                let action = action.serialize();

                data.reserve_exact(1 + action.len());

                utils::write_u8(&mut data, 0);
                data.extend(action);
            }

            PlayerAction::Certify { address, signature } => {
                data.reserve_exact(
                    1 + size_of::<crypto::Address>() + size_of::<crypto::Signature>(),
                );

                utils::write_u8(&mut data, 1);
                data.extend(address);
                data.extend(signature.iter());
            }
        }

        data
    }
}

impl<A: Action + Debug> Debug for ProofAction<A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if f.alternate() {
            writeln!(
                f,
                "\
ProofAction {{
    player: {:#?},
    action: {:#?},
}}\
                ",
                self.player, self.action,
            )
        } else {
            write!(
                f,
                "ProofAction {{ player: {:?}, action: {:?} }}",
                self.player, self.action,
            )
        }
    }
}

/// State transition
#[derive(Clone)]
pub enum PlayerAction<A: Action> {
    /// A domain-specific state transition.
    Play(A),

    /// A subkey certification.
    Certify {
        /// The subkey address.
        address: crypto::Address,

        /// The signature of the subkey address certificate.
        ///
        /// See [State::certificate].
        signature: crypto::Signature,
    },
}

impl<A: Action + Debug> Debug for PlayerAction<A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if f.alternate() {
            match self {
                Self::Play(action) => write!(f, "PlayerAction::Play({:#?})", action),
                Self::Certify { address, signature } => writeln!(
                    f,
                    "\
PlayerAction::Certify {{
    address: {},
    signature: {},
}}\
                    ",
                    crypto::eip55(address),
                    utils::hex(signature),
                ),
            }
        } else {
            match self {
                Self::Play(action) => write!(f, "PlayerAction::Play({:?})", action),
                Self::Certify { address, signature } => write!(
                    f,
                    "PlayerAction::Certify {{ address: {}, signature: {} }}",
                    crypto::eip55(address),
                    utils::hex(signature),
                ),
            }
        }
    }
}

/// Player identifier
pub type Player = u8;

/// Domain-specific state trait
pub trait State: Clone {
    /// Identifier type
    type ID: ID;

    /// Nonce type
    type Nonce: Nonce;

    /// Action type
    type Action: Action;

    /// Formats the message that must be signed in order to certify the subkey for a given address.
    fn certificate(address: &crypto::Address) -> String {
        format!(
            "Sign to play! This won't cost anything.\n\n{}\n",
            crypto::eip55(address)
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

    /// Applies an action by a given player to the state.
    fn apply(&mut self, player: Option<Player>, action: &Self::Action) -> Result<(), String>;
}

impl<S: State> State for Box<S> {
    type ID = S::ID;
    type Nonce = S::Nonce;
    type Action = S::Action;

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        S::deserialize(data).map(Self::new)
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        self.deref().serialize()
    }

    fn apply(&mut self, player: Option<Player>, action: &Self::Action) -> Result<(), String> {
        self.deref_mut().apply(player, action)
    }
}

/// Domain-specific identifier trait
pub trait ID: Clone + Eq {
    /// Consumes an identifier from binary data.
    ///
    /// The identifier must have been constructed using [ID::serialize].
    fn deserialize(data: &mut &[u8]) -> Result<Self, String>;

    /// Generates a binary representation that can be used to reconstruct the identifier.
    ///
    /// See [ID::deserialize].
    fn serialize(&self) -> Vec<u8>;
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + Clone + Eq> ID for T {
    fn deserialize(data: &mut &[u8]) -> Result<Self, String> {
        let mut deserializer = serde_cbor::Deserializer::from_slice(data);

        let id = serde::Deserialize::deserialize(&mut deserializer)
            .map_err(|error| error.to_string())?;

        *data = &data[deserializer.byte_offset()..];

        Ok(id)
    }

    fn serialize(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).unwrap()
    }
}

/// Domain-specific nonce trait
pub trait Nonce: Clone + Default {
    /// Consumes a nonce from binary data.
    ///
    /// The nonce must have been constructed using [Nonce::serialize].
    fn deserialize(data: &mut &[u8]) -> Result<Self, String>;

    /// Generates a binary representation that can be used to reconstruct the nonce.
    ///
    /// See [Nonce::deserialize].
    fn serialize(&self) -> Vec<u8>;

    /// Gets the next nonce in sequence.
    fn next(&self) -> Self;
}

macro_rules! impl_Nonce {
    ($($type:ty),*) => {
        $(
            impl Nonce for $type {
                fn deserialize(data: &mut &[u8]) -> Result<Self, String> {
                    forbid!(data.len() < size_of::<Self>());

                    let nonce = Self::from_le_bytes(
                        error::check(data[..size_of::<Self>()]
                            .try_into()
                            )?,
                    );

                    *data = &data[size_of::<Self>()..];

                    Ok(nonce)
                }

                fn serialize(&self) -> Vec<u8> {
                    self.to_le_bytes().to_vec()
                }

                fn next(&self) -> Self {
                    self + 1
                }
            }
        )*
    };
}

impl_Nonce![i8, i16, i32, i64];
impl_Nonce![u8, u16, u32, u64];

/// Domain-specific state transition trait
pub trait Action: Clone {
    /// Constructs an action from its binary representation.
    ///
    /// `data` must have been constructed using [Action::serialize].
    fn deserialize(data: &[u8]) -> Result<Self, String>;

    /// Generates a binary representation that can be used to reconstruct the action.
    ///
    /// See [Action::deserialize].
    fn serialize(&self) -> Vec<u8>;
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + Clone> Action for T {
    fn deserialize(data: &[u8]) -> Result<Self, String> {
        serde_cbor::from_slice(data).map_err(|error| error.to_string())
    }

    fn serialize(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).unwrap()
    }
}
