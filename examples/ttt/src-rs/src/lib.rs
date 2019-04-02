#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate arcadeum_state;

create_game!(SharedState, LocalState);

#[cfg_attr(not(feature = "bindings"), derive(Debug, Default))]
#[cfg_attr(feature = "bindings", derive(Deserialize, Serialize, Debug, Default))]
pub struct SharedState {
    board: [[Option<arcadeum_state::Player>; 3]; 3],
    count: u32,
}

pub type LocalState = ();

impl arcadeum_state::State<SharedState, LocalState> for SharedState {
    fn owner() -> Vec<u8> {
        b"\x37\x35\x13\xE3\x6c\x78\x04\x4A\x08\xA3\x5D\x23\x7C\x94\xEc\x49\xF3\x62\xe3\x72".to_vec()
    }

    fn winner(&self) -> Option<arcadeum_state::Player> {
        if self.board[0][0].is_some()
            && self.board[0][0] == self.board[0][1]
            && self.board[0][1] == self.board[0][2]
        {
            self.board[0][0]
        } else if self.board[1][0].is_some()
            && self.board[1][0] == self.board[1][1]
            && self.board[1][1] == self.board[1][2]
        {
            self.board[1][0]
        } else if self.board[2][0].is_some()
            && self.board[2][0] == self.board[2][1]
            && self.board[2][1] == self.board[2][2]
        {
            self.board[2][0]
        } else if self.board[0][0].is_some()
            && self.board[0][0] == self.board[1][0]
            && self.board[1][0] == self.board[2][0]
        {
            self.board[0][0]
        } else if self.board[0][1].is_some()
            && self.board[0][1] == self.board[1][1]
            && self.board[1][1] == self.board[2][1]
        {
            self.board[0][1]
        } else if self.board[0][2].is_some()
            && self.board[0][2] == self.board[1][2]
            && self.board[1][2] == self.board[2][2]
        {
            self.board[0][2]
        } else if self.board[0][0].is_some()
            && self.board[0][0] == self.board[1][1]
            && self.board[1][1] == self.board[2][2]
        {
            self.board[0][0]
        } else if self.board[0][2].is_some()
            && self.board[0][2] == self.board[1][1]
            && self.board[1][1] == self.board[2][0]
        {
            self.board[0][2]
        } else {
            None
        }
    }

    fn next_player(&self) -> Option<arcadeum_state::Player> {
        match self.count % 2 {
            0 => Some(arcadeum_state::Player::One),
            1 => Some(arcadeum_state::Player::Two),
            _ => None,
        }
    }

    fn verify(
        store: &arcadeum_state::Store<SharedState, LocalState>,
        _player: arcadeum_state::Player,
        action: &[u8],
    ) -> Result<(), arcadeum_state::Error> {
        if action.len() != 2 {
            return Err("action.len() != 2");
        }

        let (row, column) = (action[0] as usize, action[1] as usize);

        if row >= 3 {
            return Err("row >= 3");
        }

        if column >= 3 {
            return Err("column >= 3");
        }

        if store.shared_state.board[row][column].is_some() {
            return Err("store.shared_state.board[row][column].is_some()");
        }

        Ok(())
    }

    fn mutate(
        store: &mut arcadeum_state::Store<SharedState, LocalState>,
        player: arcadeum_state::Player,
        action: &[u8],
    ) {
        let (row, column) = (action[0] as usize, action[1] as usize);

        store.shared_state.board[row][column] = Some(player);
        store.shared_state.count += 1;

        let square = |row: usize, column: usize| match store.shared_state.board[row][column] {
            Some(arcadeum_state::Player::One) => 'x',
            Some(arcadeum_state::Player::Two) => 'o',
            _ => ' ',
        };

        let message = JsValue::from_str(&format!(
            "   |   |   \n {} | {} | {} \n   |   |   \n---+---+---\n   |   |   \n {} | {} | {} \n   |   |   \n---+---+---\n   |   |   \n {} | {} | {} \n   |   |   ",
            square(0, 0),
            square(0, 1),
            square(0, 2),
            square(1, 0),
            square(1, 1),
            square(1, 2),
            square(2, 0),
            square(2, 1),
            square(2, 2)
        ));

        log!(store, &message);
    }
}
