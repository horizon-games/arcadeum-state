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
use std::{convert::TryInto, mem::size_of};

#[cfg(not(feature = "std"))]
use {
    alloc::{format, prelude::v1::*},
    core::{convert::TryInto, mem::size_of},
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

pub(crate) fn read_u32_usize(data: &mut &[u8]) -> Result<usize, String> {
    crate::forbid!(data.len() < size_of::<u32>());

    let value = crate::error::check(
        u32::from_le_bytes(crate::error::check(data[..size_of::<u32>()].try_into())?).try_into(),
    )?;

    *data = &data[size_of::<u32>()..];

    Ok(value)
}

pub(crate) fn write_u32_usize(data: &mut Vec<u8>, value: usize) -> Result<(), String> {
    let value: u32 = crate::error::check(value.try_into())?;

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
