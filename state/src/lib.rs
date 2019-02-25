#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
#![cfg_attr(not(feature = "std"), feature(alloc_prelude))]

#[cfg(not(feature = "std"))]
pub extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(not(feature = "std"))]
use alloc::prelude::v1::*;

#[cfg(feature = "std")]
use std::collections::VecDeque;

#[cfg(feature = "bindings")]
extern crate serde;
#[cfg(feature = "bindings")]
extern crate wasm_bindgen;
#[cfg(feature = "bindings")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use wasm_bindgen::prelude::*;

pub struct Store<'a, SharedState, LocalState>
where
    SharedState: State<SharedState, LocalState>,
{
    pub player: Option<Player>,

    pub shared_state: SharedState,
    pub local_state: LocalState,

    listener: Option<Box<dyn FnMut()>>,
    sender: Option<Box<dyn FnMut(&[u8])>>,
    requests: VecDeque<Request<'a, SharedState, LocalState>>,
}

#[cfg_attr(feature = "bindings", wasm_bindgen)]
#[cfg_attr(
    feature = "bindings",
    derive(Deserialize, Serialize, Copy, Clone, PartialEq)
)]
#[cfg_attr(not(feature = "bindings"), derive(Copy, Clone, PartialEq))]
pub enum Player {
    One = 0,
    Two = 1,
}

pub trait State<SharedState, LocalState>
where
    SharedState: State<SharedState, LocalState>,
{
    fn owner() -> Vec<u8>;

    fn winner(&self) -> Option<Player>;
    fn next_player(&self) -> Option<Player>;

    fn verify(
        store: &Store<SharedState, LocalState>,
        player: Player,
        action: &[u8],
    ) -> Result<(), Error>;

    fn mutate(store: &mut Store<SharedState, LocalState>, player: Player, action: &[u8]);
}

pub type Error = &'static str;

pub struct Request<'a, SharedState, LocalState>
where
    SharedState: State<SharedState, LocalState>,
{
    pub player: Player,

    pub reveal: Option<Box<dyn Fn(&mut Store<SharedState, LocalState>) -> Vec<u8>>>,
    pub verify: Box<dyn Fn(&Store<SharedState, LocalState>, Player, &[u8]) -> Result<(), Error>>,
    pub mutate: Box<dyn Mutator<SharedState, LocalState> + 'a>,
}

pub trait Mutator<SharedState, LocalState>
where
    SharedState: State<SharedState, LocalState>,
{
    fn call(
        self: Box<Self>,
        store: &mut Store<SharedState, LocalState>,
        player: Player,
        action: &[u8],
    );
}

impl<'a, SharedState, LocalState> Store<'a, SharedState, LocalState>
where
    SharedState: State<SharedState, LocalState>,
{
    pub fn new(
        player: Option<Player>,
        shared_state: SharedState,
        local_state: LocalState,
        listener: Option<Box<dyn FnMut()>>,
        sender: Option<Box<dyn FnMut(&[u8])>>,
    ) -> Self {
        Self {
            player,
            shared_state,
            local_state,
            listener,
            sender,
            requests: VecDeque::new(),
        }
    }

    pub fn winner(&self) -> Option<Player> {
        self.shared_state.winner()
    }

    pub fn next_player(&self) -> Option<Player> {
        if self.winner().is_some() {
            None
        } else {
            match self.requests.front() {
                Some(request) => Some(request.player),
                None => self.shared_state.next_player(),
            }
        }
    }

    pub fn mutate(&mut self, player: Player, action: &[u8]) -> Result<(), Error> {
        if Some(player) != self.next_player() {
            Err("Some(player) != self.next_player()")
        } else {
            match self.requests.pop_front() {
                Some(request) => {
                    let result = (request.verify)(self, player, action);

                    if result.is_ok() {
                        request.mutate.call(self, player, action);

                        self.process_requests();
                    } else {
                        self.requests.push_front(request);
                    }

                    result
                }
                None => {
                    let result = SharedState::verify(self, player, action);

                    if result.is_ok() {
                        SharedState::mutate(self, player, action);

                        if Some(player) == self.player {
                            if let Some(sender) = &mut self.sender {
                                sender(action);
                            }
                        }

                        self.process_requests();
                    }

                    result
                }
            }
        }
    }

    pub fn request(&mut self, request: Request<'a, SharedState, LocalState>) {
        self.requests.push_back(request);
    }

    fn process_requests(&mut self) {
        while let Some(request) = self.requests.pop_front() {
            let player = request.player;

            if Some(player) == self.player {
                let action = request.reveal.unwrap()(self);
                (request.verify)(self, player, &action).unwrap();
                request.mutate.call(self, player, &action);

                if let Some(sender) = &mut self.sender {
                    sender(&action);
                }
            } else {
                self.requests.push_front(request);
                break;
            }
        }

        if self.requests.is_empty() {
            if let Some(listener) = &mut self.listener {
                listener()
            }
        }
    }
}

impl<F, SharedState, LocalState> Mutator<SharedState, LocalState> for F
where
    F: FnOnce(&mut Store<SharedState, LocalState>, Player, &[u8]),
    SharedState: State<SharedState, LocalState>,
{
    fn call(
        self: Box<Self>,
        store: &mut Store<SharedState, LocalState>,
        player: Player,
        action: &[u8],
    ) {
        self(store, player, action)
    }
}
