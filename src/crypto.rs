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
use {
    serde::Deserialize,
    std::{convert::TryInto, mem::size_of},
};

#[cfg(not(feature = "std"))]
use {
    alloc::{format, prelude::v1::*, vec},
    core::{convert::TryInto, mem::size_of},
};

/// Public key address
pub type Address = [u8; 20];

/// Message signature
pub type Signature = [u8; 65];

/// Message digest
pub type Hash = [u8; 32];

/// Signs a message with a secp256k1 ECDSA secret key.
///
/// # Examples
///
/// ```
/// let secret = secp256k1::SecretKey::random(&mut libsecp256k1_rand::thread_rng());
/// let message = b"quod erat demonstrandum";
/// let signature = arcadeum::crypto::sign(message, &secret).unwrap();
///
/// assert_eq!(
///     arcadeum::crypto::recover(message, &signature).unwrap(),
///     arcadeum::crypto::address(&secp256k1::PublicKey::from_secret_key(&secret))
/// );
/// ```
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

/// Recovers the address of the key that signed a message.
///
/// # Examples
///
/// ```
/// let message = b"quod erat demonstrandum";
///
/// let signature = b"\
///     \x02\x83\xdb\x3b\xa1\x91\xf3\x2f\xbd\x9a\xdb\x53\xe1\x62\x00\x79\
///     \x94\x45\x4b\xf0\x65\x52\xb0\xa0\xdd\x48\x90\xc3\xb5\x96\xdc\x4b\
///     \x44\xd6\x97\x15\x99\xbf\x24\xaf\xbe\x33\x79\x83\xae\x3d\x31\xc1\
///     \xf7\xfd\xa2\xf6\x49\xd8\x8b\x0d\x5c\xd2\xfd\xec\x18\xfa\xb7\xc8\
///     \x1b";
///
/// assert_eq!(
///     arcadeum::crypto::recover(message, signature).as_ref(),
///     Ok(b"\xdf\x55\x60\xB8\x13\x8C\xfa\x93\x86\x4B\xBD\xDe\x4D\xe4\xfF\xBD\x6C\x54\x69\xBF"),
/// );
/// ```
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

/// Computes the address of a secp256k1 ECDSA public key.
pub fn address(public: &secp256k1::PublicKey) -> Address {
    tiny_keccak::keccak256(&public.serialize()[1..])[size_of::<Hash>() - size_of::<Address>()..]
        .try_into()
        .unwrap()
}

/// Computes the EIP 55 representation of an address.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\x5a\xAe\xb6\x05\x3F\x3E\x94\xC9\xb9\xA0\x9f\x33\x66\x94\x35\xE7\xEf\x1B\xeA\xed"
///     ),
///     "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\xfB\x69\x16\x09\x5c\xa1\xdf\x60\xbB\x79\xCe\x92\xcE\x3E\xa7\x4c\x37\xc5\xd3\x59"
///     ),
///     "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\xdb\xF0\x3B\x40\x7c\x01\xE7\xcD\x3C\xBe\xa9\x95\x09\xd9\x3f\x8D\xDD\xC8\xC6\xFB"
///     ),
///     "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\xD1\x22\x0A\x0c\xf4\x7c\x7B\x9B\xe7\xA2\xE6\xBA\x89\xF4\x29\x76\x2e\x7b\x9a\xDb"
///     ),
///     "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\x52\x90\x84\x00\x09\x85\x27\x88\x6E\x0F\x70\x30\x06\x98\x57\xD2\xE4\x16\x9E\xE7"
///     ),
///     "0x52908400098527886E0F7030069857D2E4169EE7"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\x86\x17\xE3\x40\xB3\xD0\x1F\xA5\xF1\x1F\x30\x6F\x40\x90\xFD\x50\xE2\x38\x07\x0D"
///     ),
///     "0x8617E340B3D01FA5F11F306F4090FD50E238070D"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\xde\x70\x9f\x21\x02\x30\x62\x20\x92\x10\x60\x31\x47\x15\x62\x90\x80\xe2\xfb\x77"
///     ),
///     "0xde709f2102306220921060314715629080e2fb77"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\x27\xb1\xfd\xb0\x47\x52\xbb\xc5\x36\x00\x7a\x92\x0d\x24\xac\xb0\x45\x56\x1c\x26"
///     ),
///     "0x27b1fdb04752bbc536007a920d24acb045561c26"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\x5a\xAe\xb6\x05\x3F\x3E\x94\xC9\xb9\xA0\x9f\x33\x66\x94\x35\xE7\xEf\x1B\xeA\xed"
///     ),
///     "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\xfB\x69\x16\x09\x5c\xa1\xdf\x60\xbB\x79\xCe\x92\xcE\x3E\xa7\x4c\x37\xc5\xd3\x59"
///     ),
///     "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\xdb\xF0\x3B\x40\x7c\x01\xE7\xcD\x3C\xBe\xa9\x95\x09\xd9\x3f\x8D\xDD\xC8\xC6\xFB"
///     ),
///     "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB"
/// );
///
/// assert_eq!(
///     arcadeum::crypto::eip55(
///         b"\xD1\x22\x0A\x0c\xf4\x7c\x7B\x9B\xe7\xA2\xE6\xBA\x89\xF4\x29\x76\x2e\x7b\x9a\xDb"
///     ),
///     "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb"
/// );
/// ```
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

/// Balanced Merkle tree
///
/// The leaves of the tree are positioned according to their indices in truncated binary encoding.
/// This results in every Merkle proof having a number of hashes within one of any other proof, even when the number of elements isn't a power of two.
///
/// # Examples
///
/// ```
/// let tree = arcadeum::crypto::MerkleTree::with_salt(
///     vec![
///         vec![0; 0],
///         vec![1; 1],
///         vec![2; 2],
///         vec![3; 3],
///         vec![4; 4],
///         vec![5; 5],
///         vec![6; 6],
///         vec![7; 7],
///         vec![8; 8],
///         vec![9; 9],
///     ],
///     16,
///     &mut rand::thread_rng(),
/// )
/// .unwrap();
///
/// assert_eq!(
///     arcadeum::crypto::MerkleTree::deserialize(&tree.serialize()).unwrap(),
///     tree
/// );
///
/// for i in 0..tree.len() {
///     let proof = tree.proof(i).unwrap();
///
///     assert_eq!(proof.root(), tree.root());
///
///     assert_eq!(
///         arcadeum::crypto::MerkleProof::deserialize(&proof.serialize()).unwrap(),
///         proof
///     );
/// }
/// ```
#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MerkleTree<T: MerkleLeaf> {
    elements: Vec<T>,
    salts: Option<Vec<Vec<u8>>>,
    root: Hash,
}

impl<T: MerkleLeaf> MerkleTree<T> {
    /// Constructs an unsalted Merkle tree from a vector.
    ///
    /// See [MerkleTree::with_salt].
    pub fn new(elements: Vec<T>) -> Self {
        let mut tree = Self {
            elements,
            salts: None,
            root: Default::default(),
        };

        tree.root = tree.compute_root(&Default::default());

        tree
    }

    /// Constructs a salted Merkle tree from a vector and a source of entropy.
    ///
    /// See [MerkleTree::new].
    pub fn with_salt(
        elements: Vec<T>,
        salt_size: usize,
        random: &mut dyn rand::RngCore,
    ) -> Result<Self, String> {
        if salt_size == 0 {
            return Ok(Self::new(elements));
        }

        let salts = {
            let mut salt = vec![0; elements.len() * salt_size];

            random
                .try_fill_bytes(&mut salt)
                .map_err(|error| error.to_string())?;

            salt.chunks(salt_size).map(<[u8]>::to_vec).collect()
        };

        let mut tree = Self {
            elements,
            salts: Some(salts),
            root: Default::default(),
        };

        tree.root = tree.compute_root(&Default::default());

        Ok(tree)
    }

    /// Constructs a Merkle tree from its binary representation.
    ///
    /// `data` must have been constructed using [MerkleTree::serialize].
    pub fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        crate::forbid!(data.len() < size_of::<u32>());

        let length = crate::utils::read_u32_usize(&mut data)?;

        let elements = {
            let mut elements = Vec::with_capacity(length);

            for _ in 0..length {
                let size = crate::utils::read_u32_usize(&mut data)?;

                crate::forbid!(data.len() < size);
                elements.push(T::deserialize(&data[..size])?);
                data = &data[size..];
            }

            elements
        };

        crate::forbid!(data.len() % length != 0);

        let salts = match data.len() {
            0 => None,
            _ => Some(
                data.chunks(data.len() / length)
                    .map(|chunk| chunk.try_into().unwrap())
                    .collect(),
            ),
        };

        let mut tree = Self {
            elements,
            salts,
            root: Default::default(),
        };

        tree.root = tree.compute_root(&Default::default());

        Ok(tree)
    }

    /// Generates a binary representation that can be used to reconstruct the Merkle tree.
    ///
    /// See [MerkleTree::deserialize].
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        crate::utils::write_u32_usize(&mut data, self.len()).unwrap();

        for element in &self.elements {
            let element = element.serialize();
            crate::utils::write_u32_usize(&mut data, element.len()).unwrap();
            data.extend(element);
        }

        if let Some(salts) = &self.salts {
            data.extend(salts.concat());
        }

        data
    }

    /// Gets the elements in the Merkle tree.
    pub fn elements(&self) -> &Vec<T> {
        &self.elements
    }

    /// `true` if the Merkle tree is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Gets the number of elements in the Merkle tree.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Gets the root hash of the Merkle tree.
    pub fn root(&self) -> &Hash {
        &self.root
    }

    /// Generates a Merkle proof for the element at the given index.
    pub fn proof(&self, index: usize) -> Result<MerkleProof<T>, String> {
        crate::forbid!(index >= self.len());

        let hashes = {
            let mut hashes = Vec::new();

            let mut path = MerklePath::new(index, self.len());

            while path.mask != 0 {
                path.path ^= 1;

                hashes.push(self.compute_root(&path));

                path = path.parent().unwrap();
            }

            hashes
        };

        let mut proof = MerkleProof {
            element: self.elements[index].clone(),
            salt: self.salts.as_ref().map(|salts| salts[index].clone()),
            index,
            length: self.len(),
            hashes,
            root: Default::default(),
        };

        proof.root = proof.compute_root()?;

        Ok(proof)
    }

    fn compute_root(&self, path: &MerklePath) -> Hash {
        tiny_keccak::keccak256(&match path.index(self.len()) {
            Some(index) => match &self.salts {
                Some(salts) => {
                    [self.elements[index].serialize().as_slice(), &salts[index]].concat()
                }
                None => self.elements[index].serialize(),
            },
            None => [
                self.compute_root(&path.left()),
                self.compute_root(&path.right()),
            ]
            .concat(),
        })
    }
}

/// Merkle proof
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MerkleProof<T: MerkleLeaf> {
    element: T,
    salt: Option<Vec<u8>>,
    index: usize,
    length: usize,
    hashes: Vec<Hash>,
    root: Hash,
}

impl<T: MerkleLeaf> MerkleProof<T> {
    /// Constructs a Merkle proof from its binary representation.
    ///
    /// `data` must have been constructed using [MerkleProof::serialize].
    pub fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        crate::forbid!(
            data.len() < size_of::<u32>() + size_of::<u32>() + size_of::<u32>() + size_of::<u32>()
        );

        let size = crate::utils::read_u32_usize(&mut data)?;

        crate::forbid!(data.len() < size);
        let element = T::deserialize(&data[..size])?;
        data = &data[size..];

        let size = crate::utils::read_u32_usize(&mut data)?;

        crate::forbid!(data.len() < size);

        let salt = match size {
            0 => None,
            size => Some(data[..size].to_vec()),
        };

        data = &data[size..];

        let index = crate::utils::read_u32_usize(&mut data)?;
        let length = crate::utils::read_u32_usize(&mut data)?;

        crate::forbid!(data.len() % size_of::<Hash>() != 0);

        let hashes = data
            .chunks_exact(size_of::<Hash>())
            .map(|chunk| chunk.try_into().unwrap())
            .collect();

        let mut proof = Self {
            element,
            salt,
            index,
            length,
            hashes,
            root: Default::default(),
        };

        proof.root = proof.compute_root()?;

        Ok(proof)
    }

    /// Generates a binary representation that can be used to reconstruct the Merkle proof.
    ///
    /// See [MerkleProof::deserialize].
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        let element = self.element.serialize();
        crate::utils::write_u32_usize(&mut data, element.len()).unwrap();
        data.extend(element);

        if let Some(salt) = &self.salt {
            crate::utils::write_u32_usize(&mut data, salt.len()).unwrap();
            data.extend(salt);
        } else {
            crate::utils::write_u32_usize(&mut data, 0).unwrap();
        }

        crate::utils::write_u32_usize(&mut data, self.index).unwrap();
        crate::utils::write_u32_usize(&mut data, self.length).unwrap();

        for hash in &self.hashes {
            data.extend(hash);
        }

        data
    }

    /// Gets the element of the Merkle proof.
    pub fn element(&self) -> &T {
        &self.element
    }

    /// Gets the index of the element in the Merkle tree.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Gets the length of the Merkle tree.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Gets the root hash of the Merkle proof.
    pub fn root(&self) -> &Hash {
        &self.root
    }

    fn compute_root(&self) -> Result<Hash, String> {
        let mut root = tiny_keccak::keccak256(&match &self.salt {
            Some(salt) => [self.element.serialize().as_slice(), salt].concat(),
            None => self.element.serialize(),
        });

        let mut path = MerklePath::new(self.index, self.length);

        for hash in &self.hashes {
            crate::forbid!(path.mask == 0);

            root = tiny_keccak::keccak256(
                &match path.path % 2 {
                    0 => [&root[..], &hash[..]],
                    1 => [&hash[..], &root[..]],
                    _ => unreachable!(),
                }
                .concat(),
            );

            path = path.parent().unwrap();
        }

        crate::forbid!(path.mask != 0);

        Ok(root)
    }
}

/// Merkle tree element trait
pub trait MerkleLeaf: Clone {
    /// Constructs an element from its binary representation.
    ///
    /// `data` must have been constructed using [MerkleLeaf::serialize].
    fn deserialize(data: &[u8]) -> Result<Self, String>;

    /// Generates a binary representation that can be used to reconstruct the element.
    ///
    /// See [MerkleLeaf::deserialize].
    fn serialize(&self) -> Vec<u8>;
}

impl MerkleLeaf for Vec<u8> {
    fn deserialize(data: &[u8]) -> Result<Self, String> {
        Ok(data.to_vec())
    }

    fn serialize(&self) -> Vec<u8> {
        self.clone()
    }
}

macro_rules! impl_MerkleLeaf {
    ($($type:ty),*) => {
        $(
            impl MerkleLeaf for $type {
                fn deserialize(data: &[u8]) -> Result<Self, String> {
                    crate::forbid!(data.len() != size_of::<Self>());

                    Ok(Self::from_le_bytes(crate::error::check(data.try_into())?))
                }

                fn serialize(&self) -> Vec<u8> {
                    self.to_le_bytes().to_vec()
                }
            }
        )*
    };
}

impl_MerkleLeaf![i8, i16, i32, i64];
impl_MerkleLeaf![u8, u16, u32, u64];

impl MerkleLeaf for bool {
    fn deserialize(data: &[u8]) -> Result<Self, String> {
        crate::forbid!(data.len() != size_of::<Self>());
        crate::forbid!(data[0] != 0 && data[0] != 1);

        Ok(data[0] != 0)
    }

    fn serialize(&self) -> Vec<u8> {
        vec![(*self).into()]
    }
}

#[derive(Default)]
struct MerklePath {
    path: usize,
    mask: usize,
}

impl MerklePath {
    fn new(index: usize, length: usize) -> Self {
        let full = length.next_power_of_two();

        if index < full - length {
            Self {
                path: index,
                mask: (full - 2) / 2,
            }
        } else {
            Self {
                path: (full - length) + index,
                mask: full - 1,
            }
        }
    }

    fn index(&self, length: usize) -> Option<usize> {
        let full = length.next_power_of_two();

        if self.mask == (full - 2) / 2 {
            if self.path < full - length {
                Some(self.path)
            } else {
                None
            }
        } else if self.mask == full - 1 {
            Some(self.path - (full - length))
        } else {
            None
        }
    }

    fn parent(&self) -> Option<Self> {
        match self.mask {
            0 => None,
            mask => Some(Self {
                path: self.path / 2,
                mask: mask / 2,
            }),
        }
    }

    fn left(&self) -> Self {
        Self {
            path: 2 * self.path,
            mask: 2 * self.mask + 1,
        }
    }

    fn right(&self) -> Self {
        Self {
            path: 2 * self.path + 1,
            mask: 2 * self.mask + 1,
        }
    }
}
