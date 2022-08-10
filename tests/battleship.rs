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
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    },
    arcadeum::{
        crypto,
        store::{Context, State, Tester},
        Player,
    },
    core::{convert::TryInto, future::Future, mem::size_of, pin::Pin},
    rand::{RngCore, SeedableRng},
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
    let mut random = rand_xorshift::XorShiftRng::from_seed([1; 16]);

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
        false,
    )
    .unwrap();

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
                0xa667656c656d656e74f46473616c74901853185718fe18b518ab184818e8182b18b4188b1894185318db18531839183665696e6465781823666c656e6774681864666861736865738798201824031518411895186e184c18e31819184617185d18a6187418a518f918b718b51874182d18e8183b07189d18d318780f18a81857183f18e318629820182b18c618471893187f18e618b4189118f8186a181b184018a9183a18d218df1836184118ce18b218c318841855183a18de18db184218c4185d1845184f18e89820187d18c418e618ab18e51827183118bc18650418e518471857184d18af186e185e18d5183118a418a31898187a181a18690318b61618b5186018ff18a998201875188e18af189918be183118d518b41867188f188b18ef18d607185418f118f618d118bb01183b18b218c818d518820818eb18aa18ce181c184518a6982018e1186618d918f4182018e018651848189618661833184418b918b51518c5183118ee18a31884186018a3186e07185818aa18ae186c18ea18d118fc15982018ef183b08185b0118b1188118af18b618bd0518cc189a18cc18dd0e1822189d1892184c184b181918461892184d186d1828188e18e5188f18a2181f9820187418e518b018f71899181a184318a218e1188218a0182b18581852189b181a18dc18c818b20c188606184818a418da1870189b18501885181e1892189664726f6f74982018f3189d18a718d5187b1827186d186a0018a614189b189f050a0b18f018b418ae185b18ae187418c018991887189a1892186818ef1823183418a8,
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
