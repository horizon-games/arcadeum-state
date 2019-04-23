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

/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use support::{decl_event, decl_module, decl_storage, StorageMap};

use byteorder::ByteOrder;
use itoa::Integer;
use rstd::prelude::Vec;

// The module's configuration trait.
pub trait Trait: system::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Results {
        Wins: map Vec<u8> => u32;
        Draws: map Vec<u8> => u32;
        Losses: map Vec<u8> => u32;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn prove(_origin, proof: Vec<u8>) -> support::dispatch::Result {
            let mut buffer = proof.as_slice();
            let message = Message::new(&mut buffer)?;

            if message.author != game::Game::owner().as_slice() {
                return Err("message.author != game::Game::owner().as_slice()");
            }

            if message.parent != [0; 32] {
                return Err("message.parent != [0; 32]");
            }

            if message.message.len() < 16 + 2 * 20 + 4 {
                return Err("message.message.len() < 16 + 2 * 20 + 4");
            }

            let match_seed_length = byteorder::LE::read_u32(&message.message[16 + 2 * 20..]) as usize;
            if message.message.len() < 16 + 2 * 20 + 4 + match_seed_length + 4 {
                return Err("message.message.len() < 16 + 2 * 20 + 4 + match_seed_length + 4");
            }

            let public_seed_1_length = byteorder::LE::read_u32(&message.message[16 + 2 * 20 + 4 + match_seed_length..]) as usize;
            if message.message.len() < 16 + 2 * 20 + 4 + match_seed_length + 4 + public_seed_1_length + 4 {
                return Err("message.message.len() < 16 + 2 * 20 + 4 + match_seed_length + 4 + public_seed_1_length + 4");
            }

            let public_seed_2_length = byteorder::LE::read_u32(&message.message[16 + 2 * 20 + 4 + match_seed_length + 4 + public_seed_1_length..]) as usize;
            if message.message.len() != 16 + 2 * 20 + 4 + match_seed_length + 4 + public_seed_1_length + 4 + public_seed_2_length {
                return Err("message.message.len() != 16 + 2 * 20 + 4 + match_seed_length + 4 + public_seed_1_length + 4 + public_seed_2_length");
            }

            let mut account_1 = [0; 20];
            account_1.copy_from_slice(&message.message[16..16 + 20]);
            let mut account_2 = [0; 20];
            account_2.copy_from_slice(&message.message[16 + 20..16 + 2 * 20]);
            let match_seed = &message.message[16 + 2 * 20 + 4..16 + 2 * 20 + 4 + match_seed_length];
            let public_seed_1 = &message.message[16 + 2 * 20 + 4 + match_seed_length + 4..16 + 2 * 20 + 4 + match_seed_length + 4 + public_seed_1_length];
            let public_seed_2 = &message.message[16 + 2 * 20 + 4 + match_seed_length + 4 + public_seed_1_length + 4..];

            let parent = &message.hash;

            let message = Message::new(&mut buffer)?;

            if message.author != account_1 {
                return Err("&message.author != account_1");
            }

            if message.parent != parent {
                return Err("message.parent != parent");
            }

            if message.message.len() != 20 {
                return Err("message.message.len() != 20");
            }

            let subkey_1 = message.message;

            let parent = &message.hash;

            let message = Message::new(&mut buffer)?;

            if message.author != account_2 {
                return Err("&message.author != account_2");
            }

            if message.parent != parent {
                return Err("message.parent != parent");
            }

            if message.message.len() != 20 {
                return Err("message.message.len() != 20");
            }

            let subkey_2 = message.message;

            let mut store = game::Game::new(match_seed, public_seed_1, public_seed_2, None, None, None, None);

            let mut parent = [0; 32];
            parent.copy_from_slice(&message.hash);

            while buffer.len() != 0 {
                let message = Message::new(&mut buffer)?;

                let (next_player, subkey) = match store.next_player() {
                    Some(game::Player::One) => (game::Player::One, subkey_1),
                    Some(game::Player::Two) => (game::Player::Two, subkey_2),
                    _ => return Err("store.next_player().is_none()"),
                };

                if message.author != subkey {
                    return Err("message.author != subkey");
                }

                if message.parent != parent {
                    return Err("message.parent != parent");
                }

                store.mutate(next_player, message.message)?;

                parent.copy_from_slice(&message.hash);
            }

            let account_1 = account_1.to_vec();
            let account_2 = account_2.to_vec();

            match store.winner() {
                None => {
                    if store.next_player().is_none() {
                        <Draws<T>>::insert(&account_1, if <Draws<T>>::exists(&account_1) {
                            <Draws<T>>::get(&account_1) + 1
                        } else {
                            1
                        });

                        <Draws<T>>::insert(&account_2, if <Draws<T>>::exists(&account_2) {
                            <Draws<T>>::get(&account_2) + 1
                        } else {
                            1
                        });
                    } else {
                        return Err("match not finished");
                    }
                },
                Some(game::Player::One) => {
                    <Wins<T>>::insert(&account_1, if <Wins<T>>::exists(&account_1) {
                        <Wins<T>>::get(&account_1) + 1
                    } else {
                        1
                    });

                    <Losses<T>>::insert(&account_2, if <Losses<T>>::exists(&account_2) {
                        <Losses<T>>::get(&account_2) + 1
                    } else {
                        1
                    });
                },
                Some(game::Player::Two) => {
                    <Losses<T>>::insert(&account_1, if <Losses<T>>::exists(&account_1) {
                        <Losses<T>>::get(&account_1) + 1
                    } else {
                        1
                    });

                    <Wins<T>>::insert(&account_2, if <Wins<T>>::exists(&account_2) {
                        <Wins<T>>::get(&account_2) + 1
                    } else {
                        1
                    });
                },
            }

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit funtion, from our runtime funtions
        SomethingStored(u32, AccountId),
    }
);

struct Message<'a> {
    message: &'a [u8],
    author: [u8; 20],
    parent: &'a [u8],
    hash: [u8; 32],
}

impl<'a> Message<'a> {
    fn new(buffer: &mut &'a [u8]) -> Result<Self, &'static str> {
        if buffer.len() < 65 + 32 + 4 {
            return Err("buffer.len() < 65 + 32 + 4");
        }

        let length = byteorder::LE::read_u32(&buffer[65 + 32..]) as usize;

        if buffer.len() < 65 + 32 + 4 + length {
            return Err("buffer.len() < 65 + 32 + 4 + length");
        }

        let mut signature = [0; 65];
        signature.copy_from_slice(&buffer[..65]);

        let signer = runtime_io::secp256k1_ecdsa_recover(
            &signature,
            &digest(&buffer[65..65 + 32 + 4 + length]),
        )
        .map_err(|error| match error {
            runtime_io::EcdsaVerifyError::BadRS => "runtime_io::EcdsaVerifyError::BadRS",
            runtime_io::EcdsaVerifyError::BadV => "runtime_io::EcdsaVerifyError::BadV",
            runtime_io::EcdsaVerifyError::BadSignature => {
                "runtime_io::EcdsaVerifyError::BadSignature"
            }
        })?;

        let mut author = [0; 20];
        author.copy_from_slice(&runtime_io::keccak_256(&signer)[32 - 20..]);

        let message = Message {
            message: &buffer[65 + 32 + 4..65 + 32 + 4 + length],
            author,
            parent: &buffer[65..65 + 32],
            hash: runtime_io::keccak_256(&buffer[..65 + 32 + 4 + length]),
        };

        *buffer = &buffer[65 + 32 + 4 + length..];

        Ok(message)
    }
}

fn digest(message: &[u8]) -> [u8; 32] {
    let mut buffer = itoa::Buffer::new();
    let length = message.len().write(&mut buffer);

    let mut string = Vec::with_capacity(26 + length.len() + message.len());
    string.extend(b"\x19Ethereum Signed Message:\n");
    string.extend(length.bytes());
    string.extend(message);

    runtime_io::keccak_256(&string)
}
