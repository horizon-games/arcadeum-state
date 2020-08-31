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

    let expected =
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
]";

    assert_eq!(revealed, expected);

    tester.apply(Some(1), &true).unwrap();
    tester.apply(Some(0), &true).unwrap();
    tester.apply(Some(1), &true).unwrap();
    tester.apply(Some(0), &true).unwrap();
    tester.apply(Some(1), &true).unwrap();
    tester.apply(Some(0), &true).unwrap();
}
