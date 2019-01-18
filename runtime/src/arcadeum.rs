use byteorder::ByteOrder;
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
