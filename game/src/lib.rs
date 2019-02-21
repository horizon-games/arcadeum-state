#![cfg_attr(not(feature = "std"), no_std)]
#![feature(try_from)]

#[cfg(feature = "bindings")]
use serde::Serialize;
#[cfg(feature = "bindings")]
use wasm_bindgen::prelude::*;

#[cfg(not(feature = "std"))]
use core::convert::TryFrom;
#[cfg(feature = "std")]
use std::convert::TryFrom;

#[cfg_attr(feature = "bindings", wasm_bindgen)]
#[cfg_attr(feature = "bindings", derive(Serialize))]
#[derive(Clone, Copy)]
pub struct State {
    nonce: i32,
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

    #[cfg(feature = "bindings")]
    pub fn decode(&self) -> Result<JsValue, Error> {
        JsValue::from_serde(self).or(Err(ErrorCode::BadEncoding.into()))
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

#[cfg(feature = "bindings")]
impl Serialize for Player {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (*self as i32).serialize(serializer)
    }
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
    BadEncoding = 0,
    WrongLength = 1,
    NotPlayer = 2,
    WrongTurn = 3,
    BadRow = 4,
    BadColumn = 5,
    AlreadyPlayed = 6,
}

pub fn error_string(error: Error) -> &'static str {
    #[cfg(not(feature = "bindings"))]
    let error = Some(error);
    #[cfg(feature = "bindings")]
    let error = error.as_f64();

    if let Some(code) = error {
        match code as i32 {
            0 => "ErrorCode::BadEncoding",
            1 => "ErrorCode::WrongLength",
            2 => "ErrorCode::NotPlayer",
            3 => "ErrorCode::WrongTurn",
            4 => "ErrorCode::BadRow",
            5 => "ErrorCode::BadColumn",
            6 => "ErrorCode::AlreadyPlayed",
            _ => "not an ErrorCode",
        }
    } else {
        "not an ErrorCode"
    }
}
