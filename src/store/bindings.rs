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

//! WebAssembly-specific utilities

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
