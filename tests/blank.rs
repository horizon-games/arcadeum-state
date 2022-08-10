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

extern crate alloc;

use {
    alloc::{
        boxed::Box,
        string::{String, ToString},
        vec,
        vec::Vec,
    },
    arcadeum::{
        store::{Context, State, Tester},
        Player,
    },
    core::{future::Future, pin::Pin},
    serde::{Deserialize, Serialize},
};

#[cfg(not(feature = "std"))]
macro_rules! println {
    () => {
        ()
    };
    ($($arg:tt),*) => {
        {
            $(drop($arg);)*
        }
    };
}

#[cfg(feature = "std")]
arcadeum::bind!(Blank);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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
        context: Context<Self::Secret, Self::Event>,
    ) -> Pin<Box<dyn Future<Output = (Self, Context<Self::Secret, Self::Event>)>>> {
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
        |player, target, event| println!("[{:?} (target {:?}): log] {:?}", player, target, event),
        false,
    )
    .unwrap();

    tester.apply(None, &()).unwrap();
    tester.apply(Some(0), &()).unwrap();
    tester.apply(Some(1), &()).unwrap();
}
