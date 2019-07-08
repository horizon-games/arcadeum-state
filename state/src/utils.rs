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

use std::convert::TryInto;
use std::mem::size_of;

pub fn hex(data: &[u8]) -> String {
    let mut hex = String::with_capacity("0x".len() + 2 * data.len());

    hex += "0x";
    hex.extend(data.iter().map(|byte| format!("{:02x}", byte)));

    hex
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
