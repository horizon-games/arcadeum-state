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
arcadeum::bind!(Blank);

#[derive(Serialize, Clone, Debug, Default)]
struct Blank;

impl State for Blank {
    type ID = [u8; 16];
    type Nonce = u8;
    type Action = ();
    type Event = ();
    type Secret = ();

    fn version() -> &'static [u8] {
        "Blank".as_bytes()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() > 0 {
            return Err("data.len() > 0".to_string());
        }

        Ok(Self)
    }

    fn is_serializable(&self) -> bool {
        true
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        Some(vec![])
    }

    fn verify(&self, _player: Option<crate::Player>, _action: &Self::Action) -> Result<(), String> {
        Ok(())
    }

    fn apply(
        self,
        _player: Option<crate::Player>,
        _action: &Self::Action,
        context: Context<Self>,
    ) -> Pin<Box<dyn Future<Output = (Self, Context<Self>)>>> {
        Box::pin(async move { (self, context) })
    }
}

#[test]
fn test_blank_game() {
    let mut tester = Tester::new(
        Blank::default(),
        Default::default(),
        Vec::new(),
        |player, _, _| println!("[{:?}: ready]", player),
        |player, event| println!("[{:?}: log] {:?}", player, event),
    )
    .unwrap();

    tester.apply(None, &()).unwrap();
    tester.apply(Some(0), &()).unwrap();
    tester.apply(Some(1), &()).unwrap();
}
