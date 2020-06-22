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
    crypto,
    store::{Context, State, Tester},
    Player,
};

use rand::{RngCore, SeedableRng};
use serde::Serialize;

#[cfg(feature = "std")]
use std::{convert::TryInto, future::Future, mem::size_of, pin::Pin};

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use {
    alloc::{collections::VecDeque, format, prelude::v1::*, rc::Rc, vec},
    core::{cell::RefCell, convert::TryInto, future::Future, mem::size_of, pin::Pin},
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
arcadeum::bind!(Battleship);

#[derive(Serialize, Clone, Debug, Default)]
struct Battleship {
    nonce: u8,
    score: [u8; 2],
    roots: [crypto::Hash; 2],
}

impl State for Battleship {
    type ID = [u8; 16];
    type Nonce = u8;
    type Action = u8;
    type Secret = crypto::MerkleTree<bool>;

    fn version() -> &'static [u8] {
        "Battleship".as_bytes()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() != 1 + 2 + 2 * size_of::<crypto::Hash>() {
            return Err("data.len() != 1 + 2 + 2 * size_of::<crypto::Hash>()".to_string());
        }

        Ok(Self {
            nonce: data[0],
            score: [data[1], data[2]],
            roots: [
                data[3..][..size_of::<crypto::Hash>()].try_into().unwrap(),
                data[3 + size_of::<crypto::Hash>()..].try_into().unwrap(),
            ],
        })
    }

    fn is_serializable(&self) -> bool {
        true
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        let mut data = vec![self.nonce, self.score[0], self.score[1]];
        data.extend(self.roots.concat());

        Some(data)
    }

    fn verify(&self, player: Option<crate::Player>, action: &Self::Action) -> Result<(), String> {
        if player != Some(self.nonce % 2) {
            return Err("player != Some(self.nonce % 2)".to_string());
        }

        if *action > 100 {
            return Err("*action > 100".to_string());
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
                let proof = context
                    .reveal_unique(
                        1 - player.unwrap(),
                        move |secret| secret.proof(usize::from(action)).unwrap(),
                        {
                            let roots = self.roots;

                            move |proof: &crypto::MerkleProof<bool>| {
                                proof.index() == usize::from(action)
                                    && proof.length() == 100
                                    && *proof.root() == roots[1 - usize::from(player.unwrap())]
                            }
                        },
                    )
                    .await;

                context.log(proof.element());

                if *proof.element() {
                    self.score[usize::from(player.unwrap())] += 1;
                }

                self.nonce += 1;

                (self, context)
            }
        })
    }
}

#[test]
fn test_battleship() {
    let mut random = rand::rngs::StdRng::from_seed([0; 32]);

    let secrets = [
        crypto::MerkleTree::with_salt(
            {
                let mut elements = [0; 100];
                random.fill_bytes(&mut elements);
                elements.iter().map(|element| element % 2 != 0).collect()
            },
            16,
            &mut random,
        )
        .unwrap(),
        crypto::MerkleTree::with_salt(
            {
                let mut elements = [0; 100];
                random.fill_bytes(&mut elements);
                elements.iter().map(|element| element % 2 != 0).collect()
            },
            16,
            &mut random,
        )
        .unwrap(),
    ];

    let state = Battleship {
        nonce: Default::default(),
        score: Default::default(),
        roots: [
            secrets[0].root()[..].try_into().unwrap(),
            secrets[1].root()[..].try_into().unwrap(),
        ],
    };

    let mut tester = Tester::new(state, secrets, Vec::new()).unwrap();

    let revealed_data = format!(
        "{:#?}",
        tester
            .apply(Some(0), &(random.next_u32() % 100).try_into().unwrap())
            .unwrap()
    );
    let expected_revealed_data =
"[
    ProofAction {
        player: None,
        action: PlayerAction::Play(StoreAction::Reveal(0xa667656c656d656e74f56473616c749018a6188418c418e8188818b318b81888182918cf189718ec186718fc1885184965696e64657816666c656e6774681864666861736865738698201518fc1866189618e218b0184e18ca03189d10181b18c718fb18c218d118a6187f184b183118531885183d185518b6184e13189f18bf18bc188e18b5982018181843188718f518de18f218b0184518df18c618b2189e189918b0186018fa18c518cf18b518e618ec18af187a18671835189d189a0e188818f4189518a8982018fa181b187718ce18bb18c5189518480b185818df1819189e1889185b1418a8151856181d187318c718b9183718241837184f181e18e31899181d187698201862182b1834184d1867181a18f418a3181f185e18a018f50118b418ec18f5182c18ad18bf181b0018ab18e6181f1875188f18fd18e018b7189a184618dc98201418401867189918ac18bb18a50d050218b918e7183918b718be189518d2186018ef181a18f718b318f2189e1883188018c518ff1858181f1859185498200c186b09188e18eb1875187418d218901897182118351878187f18fd189e186c188d00188118c3185a18f818b30718e4186a18fb185918311849185864726f6f749820189818c9185118fb18bc07182e0918fc181c18c018d91836185d181c18e218fd18ee189418701879189518e418e818fc187818f3188d1716041869)),
    },
]";
    assert_eq!(revealed_data, expected_revealed_data);

    for _ in 0..20 {
        tester
            .apply(Some(1), &(random.next_u32() % 100).try_into().unwrap())
            .unwrap();

        tester
            .apply(Some(0), &(random.next_u32() % 100).try_into().unwrap())
            .unwrap();
    }
}
