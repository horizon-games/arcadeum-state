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

use std::mem::size_of;

pub type Address = [u8; 20];
pub type Signature = [u8; 65];
pub type Hash = [u8; 32];

pub fn sign(message: &[u8], secret: &secp256k1::SecretKey) -> Result<Signature, String> {
    let message = [
        format!("\x19Ethereum Signed Message:\n{}", message.len()).as_bytes(),
        message,
    ]
    .concat();

    let message = secp256k1::Message::parse(&tiny_keccak::keccak256(&message));

    let (mut signature, recovery) = crate::error::check(secp256k1::sign(&message, secret))?;
    signature.normalize_s();

    let mut data = [0; size_of::<Signature>()];
    data[..size_of::<Signature>() - 1].copy_from_slice(&signature.serialize());
    data[size_of::<Signature>() - 1] = 27 + recovery.serialize();
    Ok(data)
}

pub fn recover(message: &[u8], signature: &[u8]) -> Result<Address, String> {
    crate::forbid!(signature.len() != size_of::<Signature>());

    let message = [
        format!("\x19Ethereum Signed Message:\n{}", message.len()).as_bytes(),
        message,
    ]
    .concat();

    let message = secp256k1::Message::parse(&tiny_keccak::keccak256(&message));

    let recovery = crate::error::check(secp256k1::RecoveryId::parse(
        match signature[size_of::<Signature>() - 1] {
            0 | 27 => 0,
            1 | 28 => 1,
            2 | 29 => 2,
            3 | 30 => 3,
            recovery => return Err(format!("recovery == {}", recovery)),
        },
    ))?;

    let signature = crate::error::check(secp256k1::Signature::parse_slice(
        &signature[..size_of::<Signature>() - 1],
    ))?;

    let public = crate::error::check(secp256k1::recover(&message, &signature, &recovery))?;

    Ok(address(&public))
}

pub fn address(public: &secp256k1::PublicKey) -> Address {
    let mut address = [0; size_of::<Address>()];

    address.copy_from_slice(
        &tiny_keccak::keccak256(&public.serialize()[1..])
            [size_of::<Hash>() - size_of::<Address>()..],
    );

    address
}

pub fn eip55(address: &Address) -> String {
    let mut address = crate::utils::hex(address).into_bytes();
    let hash = tiny_keccak::keccak256(&address["0x".len()..]);

    for i in 0..size_of::<Address>() {
        if hash[i] & 0x80 != 0 {
            address["0x".len() + 2 * i].make_ascii_uppercase();
        }

        if hash[i] & 0x08 != 0 {
            address["0x".len() + 2 * i + 1].make_ascii_uppercase();
        }
    }

    String::from_utf8(address).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        let secret = secp256k1::SecretKey::random(&mut libsecp256k1_rand::thread_rng());
        let message = b"quod erat demonstrandum";
        let signature = sign(message, &secret).unwrap();

        assert_eq!(
            recover(message, &signature).unwrap(),
            address(&secp256k1::PublicKey::from_secret_key(&secret))
        );
    }

    #[test]
    fn test_recover() {
        let message = b"quod erat demonstrandum";

        let signature = b"\
            \x02\x83\xdb\x3b\xa1\x91\xf3\x2f\xbd\x9a\xdb\x53\xe1\x62\x00\x79\
            \x94\x45\x4b\xf0\x65\x52\xb0\xa0\xdd\x48\x90\xc3\xb5\x96\xdc\x4b\
            \x44\xd6\x97\x15\x99\xbf\x24\xaf\xbe\x33\x79\x83\xae\x3d\x31\xc1\
            \xf7\xfd\xa2\xf6\x49\xd8\x8b\x0d\x5c\xd2\xfd\xec\x18\xfa\xb7\xc8\
            \x1b";

        assert_eq!(
            &recover(message, signature).unwrap(),
            b"\xdf\x55\x60\xB8\x13\x8C\xfa\x93\x86\x4B\xBD\xDe\x4D\xe4\xfF\xBD\x6C\x54\x69\xBF",
        );
    }

    #[test]
    fn test_eip55() {
        assert_eq!(
            eip55(
                b"\x5a\xAe\xb6\x05\x3F\x3E\x94\xC9\xb9\xA0\x9f\x33\x66\x94\x35\xE7\xEf\x1B\xeA\xed"
            ),
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        );

        assert_eq!(
            eip55(
                b"\xfB\x69\x16\x09\x5c\xa1\xdf\x60\xbB\x79\xCe\x92\xcE\x3E\xa7\x4c\x37\xc5\xd3\x59"
            ),
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359"
        );

        assert_eq!(
            eip55(
                b"\xdb\xF0\x3B\x40\x7c\x01\xE7\xcD\x3C\xBe\xa9\x95\x09\xd9\x3f\x8D\xDD\xC8\xC6\xFB"
            ),
            "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB"
        );

        assert_eq!(
            eip55(
                b"\xD1\x22\x0A\x0c\xf4\x7c\x7B\x9B\xe7\xA2\xE6\xBA\x89\xF4\x29\x76\x2e\x7b\x9a\xDb"
            ),
            "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb"
        );

        assert_eq!(
            eip55(
                b"\x52\x90\x84\x00\x09\x85\x27\x88\x6E\x0F\x70\x30\x06\x98\x57\xD2\xE4\x16\x9E\xE7"
            ),
            "0x52908400098527886E0F7030069857D2E4169EE7"
        );

        assert_eq!(
            eip55(
                b"\x86\x17\xE3\x40\xB3\xD0\x1F\xA5\xF1\x1F\x30\x6F\x40\x90\xFD\x50\xE2\x38\x07\x0D"
            ),
            "0x8617E340B3D01FA5F11F306F4090FD50E238070D"
        );

        assert_eq!(
            eip55(
                b"\xde\x70\x9f\x21\x02\x30\x62\x20\x92\x10\x60\x31\x47\x15\x62\x90\x80\xe2\xfb\x77"
            ),
            "0xde709f2102306220921060314715629080e2fb77"
        );

        assert_eq!(
            eip55(
                b"\x27\xb1\xfd\xb0\x47\x52\xbb\xc5\x36\x00\x7a\x92\x0d\x24\xac\xb0\x45\x56\x1c\x26"
            ),
            "0x27b1fdb04752bbc536007a920d24acb045561c26"
        );

        assert_eq!(
            eip55(
                b"\x5a\xAe\xb6\x05\x3F\x3E\x94\xC9\xb9\xA0\x9f\x33\x66\x94\x35\xE7\xEf\x1B\xeA\xed"
            ),
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        );

        assert_eq!(
            eip55(
                b"\xfB\x69\x16\x09\x5c\xa1\xdf\x60\xbB\x79\xCe\x92\xcE\x3E\xa7\x4c\x37\xc5\xd3\x59"
            ),
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359"
        );

        assert_eq!(
            eip55(
                b"\xdb\xF0\x3B\x40\x7c\x01\xE7\xcD\x3C\xBe\xa9\x95\x09\xd9\x3f\x8D\xDD\xC8\xC6\xFB"
            ),
            "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB"
        );

        assert_eq!(
            eip55(
                b"\xD1\x22\x0A\x0c\xf4\x7c\x7B\x9B\xe7\xA2\xE6\xBA\x89\xF4\x29\x76\x2e\x7b\x9a\xDb"
            ),
            "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb"
        );
    }
}
