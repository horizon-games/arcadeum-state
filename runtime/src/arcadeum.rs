use byteorder::ByteOrder;
use itoa::Integer;
use rstd::prelude::*;
use srml_support::StorageMap;

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as Records {
        Wins: map Vec<u8> => u32;
        Draws: map Vec<u8> => u32;
        Losses: map Vec<u8> => u32;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn prove(_origin, proof: Vec<u8>) -> srml_support::dispatch::Result {
            let mut buffer = proof.as_slice();
            let message = Message::new(&mut buffer)?;
            let matcher = b"\x37\x35\x13\xE3\x6c\x78\x04\x4A\x08\xA3\x5D\x23\x7C\x94\xEc\x49\xF3\x62\xe3\x72";

            if &message.author != matcher {
                return Err("&message.author != matcher");
            }

            if message.parent != [0; 32] {
                return Err("message.parent != [0; 32]");
            }

            if message.message.len() != 16 + 2 * 20 {
                return Err("message.message.len() != 16 + 2 * 20");
            }

            let _match_id = &message.message[..16];
            let mut account_1 = [0; 20];
            account_1.copy_from_slice(&message.message[16..16 + 20]);
            let mut account_2 = [0; 20];
            account_2.copy_from_slice(&message.message[16 + 20..]);

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

            let mut state = game::State::new();

            let mut parent = [0; 32];
            parent.copy_from_slice(&message.hash);

            while buffer.len() != 0 {
                let message = Message::new(&mut buffer)?;

                if message.author != match state.next_player() {
                    game::Player::One => subkey_1,
                    game::Player::Two => subkey_2,
                    _ => return Err("message.author != state.next_player"),
                } {
                    return Err("message.author != state.next_player");
                }

                if message.parent != parent {
                    return Err("message.parent != parent");
                }

                if message.message.len() < 32 {
                    return Err("message.message.len() < 32");
                }

                let commit = &message.message[..32];
                let action = &message.message[32..];

                let inner_parent = &message.hash;

                let message = Message::new(&mut buffer)?;

                if message.author != match state.next_player() {
                    game::Player::One => subkey_2,
                    game::Player::Two => subkey_1,
                    _ => return Err("message.author != state.next_player.opponent"),
                } {
                    return Err("message.author != state.next_player.opponent");
                }

                if message.parent != inner_parent {
                    return Err("message.parent != inner_parent");
                }

                if message.message.len() != 16 {
                    return Err("message.message.len() != 16");
                }

                let random = message.message;

                let inner_parent = &message.hash;

                let message = Message::new(&mut buffer)?;

                if message.author != match state.next_player() {
                    game::Player::One => subkey_1,
                    game::Player::Two => subkey_2,
                    _ => return Err("message.author != state.next_player"),
                } {
                    return Err("message.author != state.next_player");
                }

                if message.parent != inner_parent {
                    return Err("message.parent != inner_parent");
                }

                if message.message.len() != 16 {
                    return Err("message.message.len() != 16");
                }

                if runtime_io::keccak_256(message.message) != commit {
                    return Err("runtime_io::keccak_256(message.message) != commit");
                }

                let random: Vec<_> = random.iter().zip(message.message).map(|(x, y)| x ^ y).collect();

                state = state.next(state.next_player(), action, &random).map_err(game::error_string)?;

                parent.copy_from_slice(&message.hash);
            }

            let account_1 = account_1.to_vec();
            let account_2 = account_2.to_vec();

            match state.winner() {
                game::Player::None => {
                    match state.next_player() {
                        game::Player::None => {
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
                        },
                        _ => {
                            return Err("match not finished");
                        },
                    }
                },
                game::Player::One => {
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
                game::Player::Two => {
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
