#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate arcadeum_state;

create_game!(SharedState, LocalState);

#[cfg_attr(not(feature = "bindings"), derive(Debug, Default))]
#[cfg_attr(feature = "bindings", derive(Deserialize, Serialize, Debug, Default))]
pub struct SharedState {
    score: (u32, u32),
    count: u32,
}

#[cfg_attr(not(feature = "bindings"), derive(Debug, Default))]
#[cfg_attr(feature = "bindings", derive(Deserialize, Serialize, Debug, Default))]
pub struct LocalState;

impl arcadeum_state::SharedState<LocalState> for SharedState {
    fn owner() -> Vec<u8> {
        b"\x37\x35\x13\xE3\x6c\x78\x04\x4A\x08\xA3\x5D\x23\x7C\x94\xEc\x49\xF3\x62\xe3\x72".to_vec()
    }
}

impl arcadeum_state::LocalState for LocalState {
    fn new(_secret_seed: &[u8]) -> Self {
        Default::default()
    }
}

impl arcadeum_state::State<SharedState, LocalState> for SharedState {
    fn winner(&self) -> Option<arcadeum_state::Player> {
        if self.score.0 >= 10 || self.score.1 >= 10 {
            if self.score.0 > self.score.1 + 1 {
                return Some(arcadeum_state::Player::One);
            } else if self.score.1 > self.score.0 + 1 {
                return Some(arcadeum_state::Player::Two);
            }
        }

        None
    }

    fn next_player(&self) -> Option<arcadeum_state::Player> {
        match self.count % 2 {
            0 => Some(arcadeum_state::Player::One),
            1 => Some(arcadeum_state::Player::Two),
            _ => None,
        }
    }

    fn verify(
        _store: &arcadeum_state::Store<Self, LocalState>,
        _player: arcadeum_state::Player,
        action: &[u8],
    ) -> Result<(), arcadeum_state::Error> {
        match action.len() {
            1 => Ok(()),
            _ => Err("action.len() != 1"),
        }
    }

    fn mutate(
        store: &mut arcadeum_state::Store<Self, LocalState>,
        player: arcadeum_state::Player,
        action: &[u8],
    ) {
        let guess = action[0];

        store.random(move |store, mut random| {
            let result = random.next_u32();
            let correct = (result % 2) as u8 == guess % 2;

            if correct {
                match player {
                    arcadeum_state::Player::One => store.shared_state.score.0 += 1,
                    arcadeum_state::Player::Two => store.shared_state.score.1 += 1,
                }
            }

            store.shared_state.count += 1;

            log!(
                store,
                &JsValue::from_str(&format!(
                    "{:?}: {}: guess: {}, result: {}, correct: {}",
                    store.shared_state, player, guess, result, correct
                ))
            );
        });
    }
}
