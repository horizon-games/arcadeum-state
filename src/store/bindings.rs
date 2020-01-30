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

/// Random number generator using an external JavaScript function for entropy
pub struct JsRng(pub js_sys::Function);

impl rand::RngCore for JsRng {
    fn next_u32(&mut self) -> u32 {
        rand_core::impls::next_u32_via_fill(self)
    }

    fn next_u64(&mut self) -> u64 {
        rand_core::impls::next_u64_via_fill(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.try_fill_bytes(dest).unwrap();
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        let length: u32 = dest.len().try_into().map_err(rand::Error::new)?;

        let random: Vec<u8> = crate::utils::from_js(
            self.0
                .call1(&wasm_bindgen::JsValue::UNDEFINED, &length.into())
                .map_err(|error| rand::Error::new(JsRngError(format!("{:?}", error))))?,
        )
        .map_err(rand::Error::new)?;

        if random.len() != dest.len() {
            return Err(rand::Error::new(JsRngError(
                "random.len() != dest.len()".to_string(),
            )));
        }

        dest.copy_from_slice(&random);

        Ok(())
    }
}

#[derive(Debug)]
struct JsRngError(String);

impl std::error::Error for JsRngError {}

impl std::fmt::Display for JsRngError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        std::fmt::Display::fmt(&self.0, f)
    }
}
