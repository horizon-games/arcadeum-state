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

//! Error utilities

#[cfg(feature = "std")]
use std::fmt::Debug;

#[cfg(not(feature = "std"))]
use {alloc::prelude::v1::*, core::fmt::Debug};

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
