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

//! Utilities

#[cfg(feature = "std")]
use std::{
    convert::TryInto,
    fmt::{Error, Formatter},
    mem::size_of,
};

#[cfg(not(feature = "std"))]
use {
    alloc::{format, prelude::v1::*},
    core::{
        convert::TryInto,
        fmt::{Error, Formatter},
        mem::size_of,
    },
};

/// Encodes a byte string to its hexadecimal representation.
///
/// Its hexadecimal representation begins with the characters `"0x"` followed by decimal digits and lowercase `'a'` to `'f'`.
/// The length is always even, and each byte is always encoded with the most significant nibble preceding the least significant one.
///
/// See [unhex].
///
/// # Examples
///
/// ```
/// assert_eq!(
///     &arcadeum::utils::hex(b"quod erat demonstrandum"),
///     "0x71756f6420657261742064656d6f6e737472616e64756d",
/// );
/// ```
pub fn hex(data: &[u8]) -> String {
    let mut hex = String::with_capacity("0x".len() + 2 * data.len());

    hex += "0x";
    hex.extend(data.iter().map(|byte| format!("{:02x}", byte)));

    hex
}

/// Decodes the hexadecimal representation of a byte string.
///
/// `hex` may begin with an optional `"0x"` or `"0X"` prefix.
/// `hex` must have even length.
/// Aside from any optional prefix, `hex` must consist only of decimal digits, and characters `'a'` to `'f'`, lowercase or uppercase.
/// Each byte must be encoded with the most significant nibble preceding the least significant one.
///
/// See [hex].
///
/// # Examples
///
/// ```
/// assert_eq!(
///     arcadeum::utils::unhex("0x71756f6420657261742064656d6f6e737472616e64756d"),
///     Ok(b"quod erat demonstrandum".to_vec()),
/// );
/// ```
pub fn unhex(mut hex: &str) -> Result<Vec<u8>, String> {
    crate::forbid!(hex.len() % 2 != 0);

    if hex.starts_with("0x") || hex.starts_with("0X") {
        hex = &hex["0x".len()..];
    }

    let value = |byte| match byte {
        b'0' => Ok(0),
        b'1' => Ok(1),
        b'2' => Ok(2),
        b'3' => Ok(3),
        b'4' => Ok(4),
        b'5' => Ok(5),
        b'6' => Ok(6),
        b'7' => Ok(7),
        b'8' => Ok(8),
        b'9' => Ok(9),
        b'A' | b'a' => Ok(10),
        b'B' | b'b' => Ok(11),
        b'C' | b'c' => Ok(12),
        b'D' | b'd' => Ok(13),
        b'E' | b'e' => Ok(14),
        b'F' | b'f' => Ok(15),
        byte => Err(format!("byte = {}", byte)),
    };

    let mut data = Vec::with_capacity(hex.len() / 2);

    for chunk in hex.as_bytes().chunks_exact(2) {
        data.push(16 * value(chunk[0])? + value(chunk[1])?);
    }

    Ok(data)
}

#[cfg(feature = "std")]
#[doc(hidden)]
pub fn from_js<T: for<'a> serde::Deserialize<'a>>(
    value: wasm_bindgen::JsValue,
) -> Result<T, String> {
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

#[cfg(feature = "std")]
#[doc(hidden)]
pub fn to_js<T: serde::Serialize + ?Sized>(value: &T) -> Result<wasm_bindgen::JsValue, String> {
    serde_wasm_bindgen::to_value(&value).map_err(|error| error.to_string())
}

pub(crate) fn fmt_hex(data: &impl AsRef<[u8]>, f: &mut Formatter<'_>) -> Result<(), Error> {
    write!(f, "{}", hex(data.as_ref()))
}

pub(crate) fn read_u32_usize(data: &mut &[u8]) -> Result<usize, String> {
    crate::forbid!(data.len() < size_of::<u32>());

    let value = u32::from_le_bytes(
        data[..size_of::<u32>()]
            .try_into()
            .map_err(|error| format!("{}", error))?,
    )
    .try_into()
    .map_err(|error| format!("{}", error))?;

    *data = &data[size_of::<u32>()..];

    Ok(value)
}

pub(crate) fn write_u32_usize(data: &mut Vec<u8>, value: usize) -> Result<(), String> {
    let value: u32 = value.try_into().map_err(|error| format!("{}", error))?;

    data.extend(&value.to_le_bytes());

    Ok(())
}

pub(crate) fn read_u8(data: &mut &[u8]) -> Result<u8, String> {
    crate::forbid!(data.is_empty());

    let byte = data[0];
    *data = &data[1..];

    Ok(byte)
}

pub(crate) fn write_u8(data: &mut Vec<u8>, value: u8) {
    data.push(value);
}

pub(crate) fn read_u8_bool(data: &mut &[u8]) -> Result<bool, String> {
    crate::forbid!(data.is_empty());

    let byte = data[0];
    crate::forbid!(byte != 0 && byte != 1);
    *data = &data[1..];

    Ok(byte != 0)
}

pub(crate) fn write_u8_bool(data: &mut Vec<u8>, value: bool) {
    write_u8(data, value.into());
}
