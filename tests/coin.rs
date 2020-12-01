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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc_prelude))]

use arcadeum::{
    store::{Context, State, Tester},
    Player,
};

use rand::RngCore;
use serde::Serialize;

#[cfg(feature = "std")]
use std::{future::Future, pin::Pin};

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
    type Event = u32;
    type Secret = ();

    fn version() -> &'static [u8] {
        "Coin".as_bytes()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() != 1 + 2 {
            return Err("data.len() != 1 + 2".to_string());
        }

        Ok(Self {
            nonce: data[0],
            score: [data[1], data[2]],
        })
    }

    fn is_serializable(&self) -> bool {
        true
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
                let random = context.random().await.next_u32();

                context.log(random);

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
    let mut tester = Tester::new(
        Coin::default(),
        Default::default(),
        Vec::new(),
        |player, _, _| println!("[{:?}: ready]", player),
        |player, event| println!("[{:?}: log] {:?}", player, event),
    )
    .unwrap();

    // In your tests, you can assert that specific information was revealed during application of an action.
    // This is returned as ProofAction structs, so you can use debug or serialized representations to make assertions.

    let revealed = format!("{:#?}", tester.apply(Some(0), &true).unwrap());

    let expected = if cfg!(not(feature = "no-crypto")) {
        "[
    ProofAction {
        player: Some(
            0,
        ),
        action: PlayerAction::Play(StoreAction::RandomCommit(0x5a925a41304569c6f4a7b42288395db96fc0c0a0f7ddc874e8142218eee97ee9)),
    },
    ProofAction {
        player: Some(
            1,
        ),
        action: PlayerAction::Play(StoreAction::RandomReply(0x0624278306003680d4b0951350801092)),
    },
    ProofAction {
        player: Some(
            0,
        ),
        action: PlayerAction::Play(StoreAction::RandomReveal(0x039213c113101b407ac84a8928400849)),
    },
]"
    } else {
        "[
    ProofAction {
        player: Some(
            0,
        ),
        action: PlayerAction::Play(StoreAction::RandomCommit(0x325ff735cad243cfae119dfa01cb7d8368b2c5409a0bcb8c60e01e249da55e3a)),
    },
    ProofAction {
        player: Some(
            1,
        ),
        action: PlayerAction::Play(StoreAction::RandomReply(0xc2801080c2801092c62630124236a002)),
    },
    ProofAction {
        player: Some(
            0,
        ),
        action: PlayerAction::Play(StoreAction::RandomReveal(0x614008406140084963131809211b5001)),
    },
]"
    };

    assert_eq!(revealed, expected);

    tester.apply(Some(1), &true).unwrap();
    tester.apply(Some(0), &true).unwrap();
    tester.apply(Some(1), &true).unwrap();
    tester.apply(Some(0), &true).unwrap();
    tester.apply(Some(1), &true).unwrap();
    tester.apply(Some(0), &true).unwrap();
}
