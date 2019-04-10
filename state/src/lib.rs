#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
#![cfg_attr(not(feature = "std"), feature(alloc_prelude))]

#[cfg(not(feature = "std"))]
pub extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(not(feature = "std"))]
use alloc::prelude::v1::*;
#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use std::collections::VecDeque;
#[cfg(feature = "std")]
use std::fmt;

#[cfg(feature = "bindings")]
extern crate serde;
#[cfg(feature = "bindings")]
extern crate wasm_bindgen;
#[cfg(feature = "bindings")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "std")]
pub extern crate rand;
pub extern crate rand_core;
extern crate rand_xorshift;
use rand_core::SeedableRng;

extern crate tiny_keccak;

#[cfg(all(feature = "std", not(feature = "bindings")))]
#[macro_export]
macro_rules! create_game {
    ($shared:ident, $local:ident) => {
        use arcadeum_state::rand::RngCore;

        pub struct Game(arcadeum_state::Store<$shared, $local>);

        impl Game {
            pub fn owner() -> Vec<u8> {
                <$shared as arcadeum_state::SharedState>::owner()
            }

            pub fn new(
                match_seed: &[u8],
                public_seed_1: &[u8],
                public_seed_2: &[u8],
                player: Option<arcadeum_state::Player>,
                secret_seed: Option<&[u8]>,
                listener: Option<Box<dyn FnMut()>>,
                sender: Option<Box<dyn FnMut(&[u8])>>,
            ) -> Self {
                Self(arcadeum_state::Store::new(
                    player,
                    <$shared as arcadeum_state::SharedState>::new(
                        match_seed,
                        public_seed_1,
                        public_seed_2,
                    ),
                    secret_seed.map(<$local as arcadeum_state::LocalState>::new),
                    listener,
                    sender,
                    Some(Box::new(arcadeum_state::rand::thread_rng())),
                ))
            }

            pub fn player(&self) -> Option<arcadeum_state::Player> {
                self.0.player
            }

            pub fn shared_state(&self) -> &$shared {
                &self.0.shared_state
            }

            pub fn local_state(&self) -> &Option<$local> {
                &self.0.local_state
            }

            pub fn mut_shared_state(&mut self) -> &mut $shared {
                &mut self.0.shared_state
            }

            pub fn mut_local_state(&mut self) -> &mut Option<$local> {
                &mut self.0.local_state
            }

            pub fn winner(&self) -> Option<arcadeum_state::Player> {
                self.0.winner()
            }

            pub fn next_player(&self) -> Option<arcadeum_state::Player> {
                self.0.next_player()
            }

            pub fn mutate(
                &mut self,
                player: arcadeum_state::Player,
                action: &[u8],
            ) -> Result<(), Error> {
                self.0.mutate(player, action)
            }
        }

        pub use arcadeum_state::Player;

        pub type Error = &'static str;
    };
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! create_game {
    ($shared:ident, $local:ident) => {
        use arcadeum_state::alloc::prelude::v1::Box;
        use arcadeum_state::alloc::prelude::v1::Vec;
        use arcadeum_state::rand_core::RngCore;

        pub struct Game(arcadeum_state::Store<$shared, $local>);

        impl Game {
            pub fn owner() -> Vec<u8> {
                <$shared as arcadeum_state::SharedState>::owner()
            }

            pub fn new(
                match_seed: &[u8],
                public_seed_1: &[u8],
                public_seed_2: &[u8],
                player: Option<arcadeum_state::Player>,
                secret_seed: Option<&[u8]>,
                listener: Option<Box<dyn FnMut()>>,
                sender: Option<Box<dyn FnMut(&[u8])>>,
            ) -> Self {
                Self(arcadeum_state::Store::new(
                    player,
                    <$shared as arcadeum_state::SharedState>::new(
                        match_seed,
                        public_seed_1,
                        public_seed_2,
                    ),
                    secret_seed.map(<$local as arcadeum_state::LocalState>::new),
                    listener,
                    sender,
                    Some(Box::new(Seeder)),
                ))
            }

            pub fn player(&self) -> Option<arcadeum_state::Player> {
                self.0.player
            }

            pub fn shared_state(&self) -> &$shared {
                &self.0.shared_state
            }

            pub fn local_state(&self) -> &Option<$local> {
                &self.0.local_state
            }

            pub fn mut_shared_state(&mut self) -> &mut $shared {
                &mut self.0.shared_state
            }

            pub fn mut_local_state(&mut self) -> &mut Option<$local> {
                &mut self.0.local_state
            }

            pub fn winner(&self) -> Option<arcadeum_state::Player> {
                self.0.winner()
            }

            pub fn next_player(&self) -> Option<arcadeum_state::Player> {
                self.0.next_player()
            }

            pub fn mutate(
                &mut self,
                player: arcadeum_state::Player,
                action: &[u8],
            ) -> Result<(), Error> {
                self.0.mutate(player, action)
            }
        }

        pub use arcadeum_state::Player;

        pub type Error = &'static str;

        struct Seeder;

        impl RngCore for Seeder {
            fn next_u32(&mut self) -> u32 {
                arcadeum_state::rand_core::impls::next_u32_via_fill(self)
            }

            fn next_u64(&mut self) -> u64 {
                arcadeum_state::rand_core::impls::next_u64_via_fill(self)
            }

            fn fill_bytes(&mut self, dest: &mut [u8]) {
                self.try_fill_bytes(dest).unwrap()
            }

            fn try_fill_bytes(
                &mut self,
                _dest: &mut [u8],
            ) -> Result<(), arcadeum_state::rand_core::Error> {
                Err(arcadeum_state::rand_core::Error::new(
                    arcadeum_state::rand_core::ErrorKind::Unavailable,
                    "no seeder",
                ))
            }
        }
    };
}

#[cfg(feature = "bindings")]
#[macro_export]
macro_rules! create_game {
    ($shared:ident, $local:ident) => {
        extern crate js_sys;
        extern crate serde;
        extern crate wasm_bindgen;
        use arcadeum_state::rand_core::RngCore;
        use serde::{Deserialize, Serialize};
        use wasm_bindgen::prelude::*;

        #[wasm_bindgen]
        pub struct Game(arcadeum_state::Store<$shared, $local>);

        #[wasm_bindgen]
        impl Game {
            pub fn owner() -> Vec<u8> {
                <$shared as arcadeum_state::SharedState>::owner()
            }

            #[wasm_bindgen(constructor)]
            pub fn new(
                match_seed: &[u8],
                public_seed_1: &[u8],
                public_seed_2: &[u8],
                player: Option<arcadeum_state::Player>,
                secret_seed: Option<Vec<u8>>,
                logger: Option<js_sys::Function>,
                listener: Option<js_sys::Function>,
                sender: Option<js_sys::Function>,
                seeder: Option<js_sys::Function>,
            ) -> Self {
                Self(arcadeum_state::Store::new(
                    player,
                    <$shared as arcadeum_state::SharedState>::new(
                        match_seed,
                        public_seed_1,
                        public_seed_2,
                    ),
                    secret_seed.map(|secret_seed| {
                        <$local as arcadeum_state::LocalState>::new(&secret_seed)
                    }),
                    logger.map(|logger| {
                        Box::new(move |message: &JsValue| {
                            logger.call1(&JsValue::UNDEFINED, message).unwrap();
                        }) as Box<dyn FnMut(&JsValue)>
                    }),
                    listener.map(|listener| {
                        Box::new(move || {
                            listener.call0(&JsValue::UNDEFINED).unwrap();
                        }) as Box<dyn FnMut()>
                    }),
                    sender.map(|sender| {
                        Box::new(move |action: &[u8]| {
                            sender
                                .call1(&JsValue::UNDEFINED, &JsValue::from_serde(action).unwrap())
                                .unwrap();
                        }) as Box<dyn FnMut(&[u8])>
                    }),
                    seeder.map(|seeder| Box::new(Seeder(seeder)) as Box<dyn RngCore>),
                ))
            }

            pub fn player(&self) -> Option<arcadeum_state::Player> {
                self.0.player
            }

            #[wasm_bindgen(js_name = sharedState)]
            pub fn shared_state(&self) -> Result<JsValue, Error> {
                JsValue::from_serde(&self.0.shared_state)
                    .map_err(|error| JsValue::from_str(&format!("{}", error)))
            }

            #[wasm_bindgen(js_name = localState)]
            pub fn local_state(&self) -> Result<JsValue, Error> {
                JsValue::from_serde(&self.0.local_state)
                    .map_err(|error| JsValue::from_str(&format!("{}", error)))
            }

            pub fn winner(&self) -> Option<arcadeum_state::Player> {
                self.0.winner()
            }

            #[wasm_bindgen(js_name = nextPlayer)]
            pub fn next_player(&self) -> Option<arcadeum_state::Player> {
                self.0.next_player()
            }

            pub fn mutate(
                &mut self,
                player: arcadeum_state::Player,
                action: &[u8],
            ) -> Result<(), Error> {
                self.0.mutate(player, action).map_err(JsValue::from_str)
            }
        }

        pub use arcadeum_state::Player;

        pub type Error = JsValue;

        struct Seeder(js_sys::Function);

        impl RngCore for Seeder {
            fn next_u32(&mut self) -> u32 {
                arcadeum_state::rand_core::impls::next_u32_via_fill(self)
            }

            fn next_u64(&mut self) -> u64 {
                arcadeum_state::rand_core::impls::next_u64_via_fill(self)
            }

            fn fill_bytes(&mut self, dest: &mut [u8]) {
                self.try_fill_bytes(dest).unwrap()
            }

            fn try_fill_bytes(
                &mut self,
                dest: &mut [u8],
            ) -> Result<(), arcadeum_state::rand_core::Error> {
                let result = self
                    .0
                    .call1(&JsValue::UNDEFINED, &JsValue::from(dest.len() as u32))
                    .or(Err(arcadeum_state::rand_core::Error::new(
                        arcadeum_state::rand_core::ErrorKind::Unexpected,
                        "self.0.call1(&context, &length).is_err()",
                    )))?;

                let result: Vec<u8> =
                    result
                        .into_serde()
                        .or(Err(arcadeum_state::rand_core::Error::new(
                            arcadeum_state::rand_core::ErrorKind::Unexpected,
                            "result.into_serde().is_err()",
                        )))?;

                if result.len() != dest.len() {
                    return Err(arcadeum_state::rand_core::Error::new(
                        arcadeum_state::rand_core::ErrorKind::Unexpected,
                        "result.len() != dest.len()",
                    ));
                }

                dest.copy_from_slice(&result);

                Ok(())
            }
        }
    };
}

pub struct Store<Shared, Local>
where
    Shared: State<Shared, Local>,
{
    pub player: Option<Player>,

    pub shared_state: Shared,
    pub local_state: Option<Local>,

    #[cfg(feature = "bindings")]
    pub logger: Option<Box<dyn FnMut(&JsValue)>>,

    listener: Option<Box<dyn FnMut()>>,
    sender: Option<Box<dyn FnMut(&[u8])>>,
    requests: VecDeque<Request<Shared, Local>>,

    seeder: Option<Box<dyn rand_core::RngCore>>,
    commit: Option<[u8; 32]>,
    reply: Option<Seed>,
}

#[cfg_attr(feature = "bindings", wasm_bindgen)]
#[cfg_attr(
    feature = "bindings",
    derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)
)]
#[cfg_attr(not(feature = "bindings"), derive(Debug, Copy, Clone, PartialEq))]
pub enum Player {
    One = 0,
    Two = 1,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Player::One => write!(f, "Player::One"),
            Player::Two => write!(f, "Player::Two"),
        }
    }
}

pub trait SharedState {
    fn owner() -> Vec<u8>;

    fn new(match_seed: &[u8], public_seed_1: &[u8], public_seed_2: &[u8]) -> Self;
}

pub trait LocalState {
    fn new(secret_seed: &[u8]) -> Self;
}

pub trait State<Shared, Local>
where
    Shared: State<Shared, Local>,
{
    fn winner(&self) -> Option<Player>;
    fn next_player(&self) -> Option<Player>;

    fn verify(store: &Store<Shared, Local>, player: Player, action: &[u8]) -> Result<(), Error>;
    fn mutate(store: &mut Store<Shared, Local>, player: Player, action: &[u8]);
}

pub type Error = &'static str;

#[cfg(not(feature = "bindings"))]
#[macro_export]
macro_rules! log {
    ($store:ident, $message:expr) => {};
}

#[cfg(feature = "bindings")]
#[macro_export]
macro_rules! log {
    ($store:ident, $message:expr) => {
        let message = $message;
        if let Some(logger) = &mut $store.logger {
            logger(message);
        }
    };
}

pub struct Request<Shared, Local>
where
    Shared: State<Shared, Local>,
{
    pub player: Player,

    pub reveal: Option<Box<dyn FnMut(&mut Store<Shared, Local>) -> Vec<u8>>>,
    pub verify: Box<dyn Fn(&Store<Shared, Local>, Player, &[u8]) -> Result<(), Error>>,
    pub mutate: Box<dyn FnOnce(&mut Store<Shared, Local>, Player, &[u8])>,
}

type Seed = <rand_xorshift::XorShiftRng as rand_core::SeedableRng>::Seed;

impl<Shared, Local> Store<Shared, Local>
where
    Shared: State<Shared, Local>,
{
    #[cfg(not(feature = "bindings"))]
    pub fn new(
        player: Option<Player>,
        shared_state: Shared,
        local_state: Option<Local>,
        listener: Option<Box<dyn FnMut()>>,
        sender: Option<Box<dyn FnMut(&[u8])>>,
        seeder: Option<Box<dyn rand_core::RngCore>>,
    ) -> Self {
        Self {
            player,
            shared_state,
            local_state,
            listener,
            sender,
            requests: VecDeque::new(),
            seeder,
            commit: None,
            reply: None,
        }
    }

    #[cfg(feature = "bindings")]
    pub fn new(
        player: Option<Player>,
        shared_state: Shared,
        local_state: Option<Local>,
        logger: Option<Box<dyn FnMut(&JsValue)>>,
        listener: Option<Box<dyn FnMut()>>,
        sender: Option<Box<dyn FnMut(&[u8])>>,
        seeder: Option<Box<dyn rand_core::RngCore>>,
    ) -> Self {
        Self {
            player,
            shared_state,
            local_state,
            logger,
            listener,
            sender,
            requests: VecDeque::new(),
            seeder,
            commit: None,
            reply: None,
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
                        (request.mutate)(self, player, action);

                        self.process_requests();
                    } else {
                        self.requests.push_front(request);
                    }

                    result
                }
                None => {
                    let result = Shared::verify(self, player, action);

                    if result.is_ok() {
                        Shared::mutate(self, player, action);

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

    pub fn request(&mut self, request: Request<Shared, Local>) {
        self.requests.push_back(request);
    }

    pub fn random(
        &mut self,
        continuation: impl FnOnce(&mut Self, rand_xorshift::XorShiftRng) + 'static,
    ) {
        let seed = if self.player.is_some() {
            self.seeder.as_mut().map(|seeder| {
                let mut seed = [0; 16];
                seeder.fill_bytes(&mut seed);
                seed
            })
        } else {
            None
        };

        self.request(Request {
            player: Player::One,

            reveal: if let (Some(Player::One), Some(seed)) = (&self.player, seed) {
                Some(Box::new(move |_| tiny_keccak::keccak256(&seed).to_vec()))
            } else {
                None
            },

            verify: Box::new(|_, _, hash| match hash.len() {
                32 => Ok(()),
                _ => Err("hash.len() != 32"),
            }),

            mutate: Box::new(|store, _, hash| {
                let mut commit = [0; 32];
                commit.copy_from_slice(hash);
                store.commit = Some(commit);
            }),
        });

        self.request(Request {
            player: Player::Two,

            reveal: if let (Some(Player::Two), Some(seed)) = (&self.player, seed) {
                Some(Box::new(move |_| seed.to_vec()))
            } else {
                None
            },

            verify: Box::new(|_, _, seed| match seed.len() {
                16 => Ok(()),
                _ => Err("seed.len() != 16"),
            }),

            mutate: Box::new(|store, _, seed| {
                let mut reply = [0; 16];
                reply.copy_from_slice(seed);
                store.reply = Some(reply);
            }),
        });

        self.request(Request {
            player: Player::One,

            reveal: if let (Some(Player::One), Some(seed)) = (&self.player, seed) {
                Some(Box::new(move |_| seed.to_vec()))
            } else {
                None
            },

            verify: Box::new(|store, _, seed| match seed.len() {
                16 => {
                    if &store.commit.unwrap() == &tiny_keccak::keccak256(seed) {
                        Ok(())
                    } else {
                        Err("&store.commit.unwrap() != &tiny_keccak::keccak256(seed)")
                    }
                }
                _ => Err("seed.len() != 16"),
            }),

            mutate: Box::new(|store, _, seed| {
                let xor: Vec<u8> = seed
                    .iter()
                    .zip(&store.reply.unwrap())
                    .map(|(x, y)| x ^ y)
                    .collect();

                let mut seed = [0; 16];
                seed.copy_from_slice(&xor);

                store.commit = None;
                store.reply = None;

                continuation(store, rand_xorshift::XorShiftRng::from_seed(seed));
            }),
        });
    }

    fn process_requests(&mut self) {
        while let Some(request) = self.requests.pop_front() {
            let player = request.player;

            if Some(player) == self.player {
                let action = request.reveal.unwrap()(self);
                (request.verify)(self, player, &action).unwrap();
                (request.mutate)(self, player, &action);

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
