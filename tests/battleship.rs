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
#![feature(alloc_prelude)]

use arcadeum::{
    crypto,
    store::{Context, State, Tester},
    Player,
};

use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};

extern crate alloc;

use {
    alloc::{format, prelude::v1::*, vec},
    core::{convert::TryInto, future::Future, mem::size_of, pin::Pin},
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

#[cfg(feature = "std")]
arcadeum::bind!(Battleship);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Battleship {
    nonce: u8,
    score: [u8; 2],
    roots: [crypto::Hash; 2],
}

impl State for Battleship {
    type ID = [u8; 16];
    type Nonce = u8;
    type Action = u8;
    type Event = bool;
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
        mut context: Context<Self::Secret, Self::Event>,
    ) -> Pin<Box<dyn Future<Output = (Self, Context<Self::Secret, Self::Event>)>>> {
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

                context.log(*proof.element());

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
                random.try_fill_bytes(&mut elements).unwrap();
                elements.iter().map(|element| element % 2 != 0).collect()
            },
            16,
            &mut random,
        )
        .unwrap(),
        crypto::MerkleTree::with_salt(
            {
                let mut elements = [0; 100];
                random.try_fill_bytes(&mut elements).unwrap();
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

    let mut tester = Tester::new(
        state,
        secrets,
        Vec::new(),
        |player, _, _| println!("[{:?}: ready]", player),
        |player, target, event| println!("[{:?} (target {:?}): log] {:?}", player, target, event),
    )
    .unwrap();

    // In your tests, you can assert that specific information was revealed during application of an action.
    // This is returned as ProofAction structs, so you can use debug or serialized representations to make assertions.

    let revealed = format!(
        "{:#?}",
        tester
            .apply(Some(0), &(random.next_u32() % 100).try_into().unwrap())
            .unwrap()
    );

    let expected =
"[
    ProofAction {
        player: None,
        action: Play(
            Reveal(
                0xa667656c656d656e74f56473616c749018a6188418c418e8188818b318b81888182918cf189718ec186718fc1885184965696e64657816666c656e6774681864666861736865738698201518fc1866189618e218b0184e18ca03189d10181b18c718fb18c218d118a6187f184b183118531885183d185518b6184e13189f18bf18bc188e18b5982018181843188718f518de18f218b0184518df18c618b2189e189918b0186018fa18c518cf18b518e618ec18af187a18671835189d189a0e188818f4189518a8982018fa181b187718ce18bb18c5189518480b185818df1819189e1889185b1418a8151856181d187318c718b9183718241837184f181e18e31899181d187698201862182b1834184d1867181a18f418a3181f185e18a018f50118b418ec18f5182c18ad18bf181b0018ab18e6181f1875188f18fd18e018b7189a184618dc98201418401867189918ac18bb18a50d050218b918e7183918b718be189518d2186018ef181a18f718b318f2189e1883188018c518ff1858181f1859185498200c186b09188e18eb1875187418d218901897182118351878187f18fd189e186c188d00188118c3185a18f818b30718e4186a18fb185918311849185864726f6f749820189818c9185118fb18bc07182e0918fc181c18c018d91836185d181c18e218fd18ee189418701879189518e418e818fc187818f3188d1716041869,
            ),
        ),
    },
]";

    assert_eq!(revealed, expected);

    for _ in 0..20 {
        tester
            .apply(Some(1), &(random.next_u32() % 100).try_into().unwrap())
            .unwrap();

        tester
            .apply(Some(0), &(random.next_u32() % 100).try_into().unwrap())
            .unwrap();
    }
}
