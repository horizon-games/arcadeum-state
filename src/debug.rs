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

/// `console.log()`
#[macro_export]
macro_rules! console_log {
    ($(,)?) => {
        $crate::debug::log_0();
    };
    (
        $arg_1:expr
        $(,)?
    ) => {
        if let Ok(arg_1) = &$crate::utils::to_js(&$arg_1) {
            $crate::debug::log_1(arg_1);
        } else {
            $crate::debug::log_1(&"serde_wasm_bindgen::to_value(_) == Err(_)".into());
        }
    };
    (
        $arg_1:expr,
        $arg_2:expr
        $(,)?
    ) => {
        if let (Ok(arg_1), Ok(arg_2)) =
            &($crate::utils::to_js(&$arg_1), $crate::utils::to_js(&$arg_2))
        {
            $crate::debug::log_2(arg_1, arg_2);
        } else {
            $crate::debug::log_1(&"serde_wasm_bindgen::to_value(_) == Err(_)".into());
        }
    };
    (
        $arg_1:expr,
        $arg_2:expr,
        $arg_3:expr
        $(,)?
    ) => {
        if let (Ok(arg_1), Ok(arg_2), Ok(arg_3)) = &(
            $crate::utils::to_js(&$arg_1),
            $crate::utils::to_js(&$arg_2),
            $crate::utils::to_js(&$arg_3),
        ) {
            $crate::debug::log_3(arg_1, arg_2, arg_3);
        } else {
            $crate::debug::log_1(&"serde_wasm_bindgen::to_value(_) == Err(_)".into());
        }
    };
    (
        $arg_1:expr,
        $arg_2:expr,
        $arg_3:expr,
        $arg_4:expr
        $(,)?
    ) => {
        if let (Ok(arg_1), Ok(arg_2), Ok(arg_3), Ok(arg_4)) = &(
            $crate::utils::to_js(&$arg_1),
            $crate::utils::to_js(&$arg_2),
            $crate::utils::to_js(&$arg_3),
            $crate::utils::to_js(&$arg_4),
        ) {
            $crate::debug::log_4(arg_1, arg_2, arg_3, arg_4);
        } else {
            $crate::debug::log_1(&"serde_wasm_bindgen::to_value(_) == Err(_)".into());
        }
    };
    (
        $arg_1:expr,
        $arg_2:expr,
        $arg_3:expr,
        $arg_4:expr,
        $arg_5:expr
        $(,)?
    ) => {
        if let (Ok(arg_1), Ok(arg_2), Ok(arg_3), Ok(arg_4), Ok(arg_5)) = &(
            $crate::utils::to_js(&$arg_1),
            $crate::utils::to_js(&$arg_2),
            $crate::utils::to_js(&$arg_3),
            $crate::utils::to_js(&$arg_4),
            $crate::utils::to_js(&$arg_5),
        ) {
            $crate::debug::log_5(arg_1, arg_2, arg_3, arg_4, arg_5);
        } else {
            $crate::debug::log_1(&"serde_wasm_bindgen::to_value(_) == Err(_)".into());
        }
    };
    (
        $arg_1:expr,
        $arg_2:expr,
        $arg_3:expr,
        $arg_4:expr,
        $arg_5:expr,
        $arg_6:expr
        $(,)?
    ) => {
        if let (Ok(arg_1), Ok(arg_2), Ok(arg_3), Ok(arg_4), Ok(arg_5), Ok(arg_6)) = &(
            $crate::utils::to_js(&$arg_1),
            $crate::utils::to_js(&$arg_2),
            $crate::utils::to_js(&$arg_3),
            $crate::utils::to_js(&$arg_4),
            $crate::utils::to_js(&$arg_5),
            $crate::utils::to_js(&$arg_6),
        ) {
            $crate::debug::log_6(arg_1, arg_2, arg_3, arg_4, arg_5, arg_6);
        } else {
            $crate::debug::log_1(&"serde_wasm_bindgen::to_value(_) == Err(_)".into());
        }
    };
    (
        $arg_1:expr,
        $arg_2:expr,
        $arg_3:expr,
        $arg_4:expr,
        $arg_5:expr,
        $arg_6:expr,
        $arg_7:expr
        $(,)?
    ) => {
        if let (Ok(arg_1), Ok(arg_2), Ok(arg_3), Ok(arg_4), Ok(arg_5), Ok(arg_6), Ok(arg_7)) = &(
            $crate::utils::to_js(&$arg_1),
            $crate::utils::to_js(&$arg_2),
            $crate::utils::to_js(&$arg_3),
            $crate::utils::to_js(&$arg_4),
            $crate::utils::to_js(&$arg_5),
            $crate::utils::to_js(&$arg_6),
            $crate::utils::to_js(&$arg_7),
        ) {
            $crate::debug::log_7(arg_1, arg_2, arg_3, arg_4, arg_5, arg_6, arg_7);
        } else {
            $crate::debug::log_1(&"serde_wasm_bindgen::to_value(_) == Err(_)".into());
        }
    };
}

pub use web_sys::console::{log_0, log_1, log_2, log_3, log_4, log_5, log_6, log_7};
