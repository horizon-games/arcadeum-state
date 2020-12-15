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
    let mut random = rand::rngs::StdRng::from_seed([1; 32]);

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
                0xa667656c656d656e74f46473616c7490186518a2188c18c318fd041882182818d3051895186f18c118440c188d65696e646578185c666c656e677468186466686173686573879820121899187418f5189e182b18d51876188318d2187018281836182318dd1847182e186106185b188418d118a70a18d218be185c1872184818c418f118b99820186a187218840c1890185618f6182e18320818b318c1185a18fe1858186b0618ce18c10c18d718c4189f18e118d518f618aa1877185c182118c4184c9820184b184718bc18c4187d186e183e18d2187a08184d18f018d018b71895189e181f18da1854184818c618d018db18f217181918b716184e18c518ce18689820185818f6185818b7182a18ad18bc187c18ee1862189d18b518b0186018ea18ea18f5182b189b18c418fd18ad18ab0818f8187818ee18a9183c1885188718ea982018ee182518f0184a189318b618a5184118e018f7182118ce18f618b618a518550718ce18aa18d318c018c418a71891188b183708187f0d185218dd187598200a18840518c3183718f9181d18c21849185e18ce18c6186818b61865181d18fc18b8188218b818db18a118c718f2189518b218e018441867187b1842186c982018bb18f118a6183a183b187f185b185b188c0a18ad186b182218331886189a186a18201891181f18d018a118ac18db18a518a20218a818a8181806187e64726f6f7498201838186b18bb18b218ee18401825185a189818421879187a0c18e11883187418631827182f18de188a1861188d0c18ed18b418ca189818c9183218fb18ac,
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
