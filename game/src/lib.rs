#![cfg_attr(not(feature = "std"), no_std)]
#![feature(try_from)]

#[cfg(feature = "bindings")]
use wasm_bindgen::prelude::*;

#[cfg(not(feature = "std"))]
use core::convert::TryFrom;
#[cfg(feature = "std")]
use std::convert::TryFrom;

#[cfg_attr(feature = "bindings", wasm_bindgen)]
#[derive(Clone, Copy)]
pub struct State {
    pub nonce: i32,
    board: [[Option<Player>; 3]; 3],
}

#[cfg_attr(feature = "bindings", wasm_bindgen)]
impl State {
    #[cfg_attr(feature = "bindings", wasm_bindgen(constructor))]
    pub fn new() -> Self {
        Self {
            nonce: 0,
            board: [[None, None, None], [None, None, None], [None, None, None]],
        }
    }

    pub fn winner(&self) -> Option<Player> {
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

    #[cfg_attr(feature = "bindings", wasm_bindgen(js_name = nextPlayer))]
    pub fn next_player(&self) -> Option<Player> {
        if self.winner().is_some() {
            return None;
        }

        match self.nonce {
            0 | 2 | 4 | 6 | 8 => Some(Player::One),
            1 | 3 | 5 | 7 => Some(Player::Two),
            _ => None,
        }
    }

    pub fn next(&self, player: Player, action: &[u8], _random: &[u8]) -> Result<State, Error> {
        if action.len() != 2 {
            return Err(ErrorCode::WrongLength.into());
        }

        if Some(player) != self.next_player() {
            return Err(ErrorCode::WrongTurn.into());
        }

        let (row, column) = (action[0] as usize, action[1] as usize);

        if row >= 3 {
            return Err(ErrorCode::BadRow.into());
        }

        if column >= 3 {
            return Err(ErrorCode::BadColumn.into());
        }

        if self.board[row][column].is_some() {
            return Err(ErrorCode::AlreadyPlayed.into());
        }

        let mut next = *self;
        next.nonce += 1;
        next.board[row][column] = Some(player);
        Ok(next)
    }
}

#[cfg_attr(feature = "bindings", wasm_bindgen)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Player {
    One = 1,
    Two = 2,
}

impl TryFrom<i32> for Player {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Player::One),
            2 => Ok(Player::Two),
            _ => Err(ErrorCode::NotPlayer.into()),
        }
    }
}

#[cfg(not(feature = "bindings"))]
type Error = i32;
#[cfg(feature = "bindings")]
type Error = JsValue;

impl From<ErrorCode> for Error {
    fn from(code: ErrorCode) -> Self {
        (code as i32).into()
    }
}

enum ErrorCode {
    WrongLength = 0,
    NotPlayer = 1,
    WrongTurn = 2,
    BadRow = 3,
    BadColumn = 4,
    AlreadyPlayed = 5,
}

pub fn error_string(error: Error) -> &'static str {
    #[cfg(not(feature = "bindings"))]
    let error = Some(error);
    #[cfg(feature = "bindings")]
    let error = error.as_f64();

    if let Some(code) = error {
        match code as i32 {
            0 => "ErrorCode::WrongLength",
            1 => "ErrorCode::NotPlayer",
            2 => "ErrorCode::WrongTurn",
            3 => "ErrorCode::BadRow",
            4 => "ErrorCode::BadColumn",
            5 => "ErrorCode::AlreadyPlayed",
            _ => "not an ErrorCode",
        }
    } else {
        "not an ErrorCode"
    }
}
