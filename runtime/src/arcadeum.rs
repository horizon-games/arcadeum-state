use byteorder::ByteOrder;
use itoa::Integer;
use rstd::prelude::*;
use srml_support::StorageMap;

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as Records {
        Wins: map T::AccountId => u32;
        Draws: map T::AccountId => u32;
        Losses: map T::AccountId => u32;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn prove(origin, proof: Vec<u8>) -> srml_support::dispatch::Result {
            let sender = system::ensure_signed(origin)?;

            let mut state = game::State::new();
            let mut buffer = proof.as_slice();

            while buffer.len() >= 4 {
                let length = byteorder::LE::read_u32(buffer) as usize;

                if 4 + length > buffer.len() {
                    return Err("4 + length > buffer.len()");
                }

                state = state.next(&buffer[4..4 + length]).or(Err("invalid transition"))?;
                buffer = &buffer[4 + length..];
            }

            if buffer.len() != 0 {
                return Err("proof.len() != 0");
            }

            match state.winner() {
                game::Player::None => {
                    if state.next_player() == game::Player::None {
                        <Draws<T>>::insert(sender.clone(), if <Draws<T>>::exists(sender.clone()) {
                            <Draws<T>>::get(sender.clone()) + 1
                        } else {
                            1
                        });
                    } else {
                        return Err("incomplete proof");
                    }
                },
                game::Player::One => {
                    <Wins<T>>::insert(sender.clone(), if <Wins<T>>::exists(sender.clone()) {
                        <Wins<T>>::get(sender.clone()) + 1
                    } else {
                        1
                    });
                },
                game::Player::Two => {
                    <Losses<T>>::insert(sender.clone(), if <Losses<T>>::exists(sender.clone()) {
                        <Losses<T>>::get(sender.clone()) + 1
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
