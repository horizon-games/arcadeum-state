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

#[cfg(feature = "std")]
use std::fmt::Debug;

#[cfg(not(feature = "std"))]
use {
    alloc::{format, prelude::v1::*},
    core::fmt::Debug,
};

#[derive(Debug)]
pub enum Error {
    Soft(String),
    Hard(String),
}

impl From<Error> for String {
    fn from(error: Error) -> Self {
        match error {
            Error::Soft(string) | Error::Hard(string) => string,
        }
    }
}

impl From<String> for Error {
    fn from(string: String) -> Self {
        Self::Soft(string)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! forbid {
    ($condition:expr) => {
        if $condition {
            return Err(format!(
                "{}:{}:{}: {}",
                module_path!(),
                line!(),
                column!(),
                stringify!($condition)
            )
            .into());
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! slash {
    ($condition:expr) => {
        if $condition {
            return Err($crate::error::Error::Hard(format!(
                "{}:{}:{}: {}",
                module_path!(),
                line!(),
                column!(),
                stringify!($condition)
            )));
        }
    };
}
