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
