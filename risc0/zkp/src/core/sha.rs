// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Simple SHA-256 wrappers.

use alloc::vec::Vec;
use core::{
    fmt::{Debug, Display, Formatter},
    mem,
    ops::Deref,
};

use bytemuck::{Pod, Zeroable};
use hex::{FromHex, FromHexError};
use risc0_zeroio::{Deserialize as ZeroioDeserialize, Serialize as ZeroioSerialize};
use serde::{Deserialize, Serialize};

/// The number of words in the representation of a [Digest].
// We represent a SHA-256 digest as 8 32-bit words instead of the
// traditional 32 8-bit bytes.
pub const DIGEST_WORDS: usize = 8;

/// The size of a word in bytes within a [Digest] (32-bits = 4 bytes).
pub const DIGEST_WORD_SIZE: usize = mem::size_of::<u32>();

/// Size of the [Digest] representation in bytes.
/// Note that digests are stored in memory as words instead of bytes.
pub const DIGEST_BYTES: usize = DIGEST_WORDS * DIGEST_WORD_SIZE;

/// The size of a SHA-256 block in bytes.
pub const BLOCK_SIZE: usize = DIGEST_BYTES * 2;

/// Standard SHA-256 initialization vector.
pub static SHA256_INIT: Digest = Digest([
    0x6a09e667_u32.to_be(),
    0xbb67ae85_u32.to_be(),
    0x3c6ef372_u32.to_be(),
    0xa54ff53a_u32.to_be(),
    0x510e527f_u32.to_be(),
    0x9b05688c_u32.to_be(),
    0x1f83d9ab_u32.to_be(),
    0x5be0cd19_u32.to_be(),
]);

/// The result of the SHA-256 hash algorithm.
///
/// Note: Bytes in the [Digest] type are stored in big-endian order regardless
/// of the host architecture. When interpretted as words, the numerical result
/// will depend on the architecture.
// TODO(victor) Removing the Copy trait also means this types cannot be bytemuck::Pod. Is this what
// we want?
// TODO(victor) Decide whether or not to make the inner struct pub. It would make things somewhat
// simpler, but it is also somewhat clean that in the current factoring it is kind of agnostic to
// whether it is bytes or words internally.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Pod,
    Zeroable,
    Serialize,
    Deserialize,
    ZeroioSerialize,
    ZeroioDeserialize,
)]
#[repr(transparent)]
pub struct Digest([u32; DIGEST_WORDS]);

impl Digest {
    /// Returns a reference to the Digest as a slice of words.
    pub fn as_words(&self) -> &[u32] {
        &self.0
    }

    /// Returns a reference to the DIgest as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0)
    }

    /// Returns a mutable slice of words.
    pub fn as_mut_words(&mut self) -> &mut [u32] {
        &mut self.0
    }

    /// Returns a mutable slice of bytes.
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(&mut self.0)
    }
}

impl From<&Digest> for Digest {
    fn from(digest: &Digest) -> Self {
        digest.clone()
    }
}

impl From<&[u8]> for Digest {
    fn from(bytes: &[u8]) -> Self {
        Digest::from_bytes(bytes)
    }
}

impl Default for Digest {
    fn default() -> Digest {
        Digest([0; DIGEST_WORDS])
    }
}

/// Create a new [Digest] from an array of words.
impl From<[u32; DIGEST_WORDS]> for Digest {
    fn from(data: [u32; DIGEST_WORDS]) -> Self {
        Self(data)
    }
}

/// Create a new [Digest] from an array of bytes.
impl From<[u8; DIGEST_BYTES]> for Digest {
    fn from(data: [u8; DIGEST_BYTES]) -> Self {
        Self(bytemuck::cast(data))
    }
}

impl FromHex for Digest {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(<[u8; DIGEST_BYTES]>::from_hex(hex)?.into())
    }
}

impl TryFrom<&[u8]> for Digest {
    type Error = core::array::TryFromSliceError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        Ok(<[u8; DIGEST_BYTES]>::try_from(data)?.into())
    }
}

impl TryFrom<&[u32]> for Digest {
    type Error = core::array::TryFromSliceError;

    fn try_from(data: &[u32]) -> Result<Self, Self::Error> {
        Ok(<[u32; DIGEST_WORDS]>::try_from(data)?.into())
    }
}

impl TryFrom<Vec<u8>> for Digest {
    type Error = Vec<u8>;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(<[u8; DIGEST_BYTES]>::try_from(data)?.into())
    }
}

impl TryFrom<Vec<u32>> for Digest {
    type Error = Vec<u32>;

    fn try_from(data: Vec<u32>) -> Result<Self, Self::Error> {
        Ok(<[u32; DIGEST_WORDS]>::try_from(data)?.into())
    }
}

impl Into<[u8; DIGEST_BYTES]> for Digest {
    fn into(self) -> [u8; DIGEST_BYTES] {
        bytemuck::cast(self.0)
    }
}

impl Into<[u32; DIGEST_WORDS]> for Digest {
    fn into(self) -> [u32; DIGEST_WORDS] {
        self.0
    }
}

impl AsRef<[u8; DIGEST_BYTES]> for Digest {
    fn as_ref(&self) -> &[u8; DIGEST_BYTES] {
        bytemuck::cast_ref(&self.0)
    }
}

impl AsMut<[u8; DIGEST_BYTES]> for Digest {
    fn as_mut(&mut self) -> &mut [u8; DIGEST_BYTES] {
        bytemuck::cast_mut(&mut self.0)
    }
}

impl AsRef<[u32; DIGEST_WORDS]> for Digest {
    fn as_ref(&self) -> &[u32; DIGEST_WORDS] {
        &self.0
    }
}

impl AsMut<[u32; DIGEST_WORDS]> for Digest {
    fn as_mut(&mut self) -> &mut [u32; DIGEST_WORDS] {
        &mut self.0
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for Digest {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_bytes()
    }
}

impl AsRef<[u32]> for Digest {
    fn as_ref(&self) -> &[u32] {
        self.as_words()
    }
}

impl AsMut<[u32]> for Digest {
    fn as_mut(&mut self) -> &mut [u32] {
        self.as_mut_words()
    }
}

// TODO(victor) Do these formatting definitions result in what a user my expect?
impl Display for Digest {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.write_str(&hex::encode(&self))
    }
}

impl Debug for Digest {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.write_str(&hex::encode(&self))
    }
}

/// An implementation of the SHA-256 hashing algorithm of [FIPS 180-4].
///
/// [FIPS 180-4] https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf
pub trait Sha: Clone + Debug {
    /// A pointer to the created digest.
    ///
    /// This may either be a `Box<Digest>` or some other pointer in case the
    /// implementation wants to manage its own memory.
    type DigestPtr: Deref<Target = Digest> + Debug;

    /// Generate a SHA from a slice of bytes, padding to block size
    /// and adding the SHA trailer.
    fn hash_bytes(&self, bytes: &[u8]) -> Self::DigestPtr;

    /// Generate a SHA from a slice of words, padding to block size
    /// and adding the SHA trailer.
    fn hash_words(&self, words: &[u32]) -> Self::DigestPtr {
        self.hash_bytes(bytemuck::cast_slice(words) as &[u8])
    }

    /// Generate a SHA from a slice of words without adding padding or
    /// length.
    // TODO(victor): IIUC, this function is unused and probably should be removed.
    fn hash_raw_words(&self, words: &[u32]) -> Self::DigestPtr;

    /// Generate a SHA from a pair of [Digests](Digest).
    // TODO(victor) This is an efficient way to produce H(a || b), which I am
    // guesing is designed for use in Merkle trees. Is this the best method to
    // be exposing here though? It does not use domain speration or added padding
    // and length.
    fn hash_pair(&self, a: &Digest, b: &Digest) -> Self::DigestPtr {
        self.compress(&SHA256_INIT, a, b)
    }

    /// Execute the SHA-256 compression function.
    /// The block is specified as two half-blocks.
    /// Not all implementations provide this.
    ///
    /// DANGER:
    fn compress(
        &self,
        state: &Digest,
        block_half1: &Digest,
        block_half2: &Digest,
    ) -> Self::DigestPtr;

    /// Generate a SHA from a slice of anything that can be
    /// represented as plain old data. Pads up to the Sha block
    /// boundry, but does not add the standard SHA trailer.
    // TODO(victor): Understand why this function exists.
    fn hash_raw_pod_slice<T: bytemuck::Pod>(&self, slice: &[T]) -> Self::DigestPtr;

    /// Generate a new digest by mixing two digests together via XOR,
    /// and storing into the first digest.
    // TODO(victor): I'm guessing this is for use by a randomness pool. I may
    // extract this function to there.
    fn mix(&self, pool: &mut Self::DigestPtr, val: &Digest);
}

#[cfg(test)]
mod tests {
    use hex::FromHex;

    use super::Digest;

    #[test]
    fn test_from_hex() {
        assert_eq!(
            Digest::from_hex("00000077000000AA0000001200000034000000560000007a000000a900000009")
                .unwrap(),
            Digest::from([
                0x77_u32.to_be(),
                0xaa_u32.to_be(),
                0x12_u32.to_be(),
                0x34_u32.to_be(),
                0x56_u32.to_be(),
                0x7a_u32.to_be(),
                0xa9_u32.to_be(),
                0x09_u32.to_be(),
            ])
        );
    }

    #[test]
    fn test_roundtrip() {
        const HEX: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(hex::encode(Digest::from_hex(HEX).unwrap()), HEX);
    }
}

#[allow(missing_docs)]
pub mod testutil {
    // TODO(victor) Fix these tests.
    use alloc::vec::Vec;
    use core::ops::Deref;

    use hex::FromHex;

    use super::{Digest, Sha};
    use crate::field::baby_bear::{BabyBearElem, BabyBearExtElem};

    // Runs conformance test on a SHA implementation to make sure it properly
    // behaves.
    pub fn test_sha_impl<S: Sha>(sha: &S) {
        test_hash_pair(sha);
        test_hash_raw_words(sha);
        test_hash_raw_pod_slice(sha);
        test_sha_basics(sha);
        test_elems(sha);
        test_extelems(sha);

        crate::core::sha_rng::testutil::test_sha_rng_impl(sha);
    }

    fn test_sha_basics<S: Sha>(sha: &S) {
        // Standard test vectors
        assert_eq!(
            hex::encode(sha.hash_bytes("abc".as_bytes()).deref()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex::encode(sha.hash_bytes("".as_bytes()).deref()),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex::encode(
                &sha.hash_bytes(
                    "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".as_bytes()
                )
                .deref()
            ),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(hex::encode(&sha.hash_bytes(
            "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu" .as_bytes()).deref()),
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1");
        // Test also the 'hexDigest' bit.
        // Python says:
        // >>> hashlib.sha256("Byzantium").hexdigest()
        // 'f75c763b4a52709ac294fc7bd7cf14dd45718c3d50b36f4732b05b8c6017492a'
        assert_eq!(
            hex::encode(&sha.hash_bytes(&"Byzantium".as_bytes()).deref()),
            "f75c763b4a52709ac294fc7bd7cf14dd45718c3d50b36f4732b05b8c6017492a"
        );
    }

    fn hash_elems<S: Sha>(sha: &S, len: usize) -> Digest {
        let items: Vec<BabyBearElem> = (0..len as u32)
            .into_iter()
            .map(|x| BabyBearElem::new(x))
            .collect();
        *sha.hash_raw_pod_slice(items.as_slice())
    }

    fn hash_extelems<S: Sha>(sha: &S, len: usize) -> Digest {
        let items: Vec<BabyBearExtElem> = (0..len as u32)
            .into_iter()
            .map(|x| {
                BabyBearExtElem::new(
                    BabyBearElem::new(x * 4),
                    BabyBearElem::new(x * 4 + 1),
                    BabyBearElem::new(x * 4 + 2),
                    BabyBearElem::new(x * 4 + 3),
                )
            })
            .collect();
        *sha.hash_raw_pod_slice(items.as_slice())
    }

    fn test_elems<S: Sha>(sha: &S) {
        const LENS: &[usize] = &[0, 1, 7, 8, 9];
        // It doesn't matter what elems hash to, as long as they're consistent.
        const EXPECTED_STRS: &[&str] = &[
            "6a09e667bb67ae853c6ef372a54ff53a510e527f9b05688c1f83d9ab5be0cd19",
            "da5698be17b9b46962335799779fbeca8ce5d491c0d26243bafef9ea1837a9d8",
            "643f71dab15c4f6a6e8820dee5f59cc07818b9c4473b47bba9516cc3be992f1c",
            "3dae53575097f63d0a461048813cc9ab870f0ddbcf9e4aea8dcddecc0aea736d",
            "903fe671a0971f6dea6e8a1180dcd1ce87b56d0b42ee3861212e86428a983a5b",
        ];

        let expected: Vec<Digest> = EXPECTED_STRS
            .iter()
            .map(|x| Digest::from_hex(x).unwrap())
            .collect();
        let actual: Vec<Digest> = LENS.iter().map(|x| hash_elems(sha, *x)).collect();
        assert_eq!(expected, actual);
    }

    fn test_extelems<S: Sha>(sha: &S) {
        const LENS: &[usize] = &[0, 1, 7, 8, 9];
        // It doesn't matter what extelems hash to, as long as they're consistent.
        const EXPECTED_STRS: &[&str] = &[
            "6a09e667bb67ae853c6ef372a54ff53a510e527f9b05688c1f83d9ab5be0cd19",
            "6343c9ca9260f2d6cf190c2d2bbff0bf928789e4d2c1a24654137a5d48f254bc",
            "07d3bfa65009530790a51cca21b83dd492c60ade96ee1d2c5b25c4c5cfe257b0",
            "60a53ad42dfe03c7c0d1d46790a832356d09b52c6812eada27622476d6180392",
            "5af62d0303208f4573656ac707d7447f0303fd76a134a775f329104d03c37985",
        ];

        let expected: Vec<Digest> = EXPECTED_STRS
            .iter()
            .map(|x| Digest::from_hex(x).unwrap())
            .collect();
        let actual: Vec<Digest> = LENS.iter().map(|x| hash_extelems(sha, *x)).collect();
        assert_eq!(expected, actual);
    }

    fn test_hash_raw_pod_slice<S: Sha>(sha: &S) {
        {
            let items: &[u32] = &[1];
            assert_eq!(
                *sha.hash_raw_pod_slice(items),
                Digest::from_hex(
                    "e3050856aac389661ae490656ad0ea57df6aff0ff6eef306f8cc2eed4f240249"
                )
                .unwrap()
            );
        }
        {
            let items: &[u32] = &[1, 2];
            assert_eq!(
                *sha.hash_raw_pod_slice(items),
                Digest::from_hex(
                    "4138ebae12299733cc677d1150c2a0139454662fc76ec95da75d2bf9efddc57a"
                )
                .unwrap()
            );
        }
        {
            let items: &[u32] = &[0xffffffff];
            assert_eq!(
                *sha.hash_raw_pod_slice(items),
                Digest::from_hex(
                    "a3dba037d56175209dfd4191f727e91c5feb67e65a6ab5ed4daf0893c89598c8"
                )
                .unwrap()
            );
        }
    }

    fn test_hash_pair<S: Sha>(sha: &S) {
        assert_eq!(
            *sha.hash_pair(
                &Digest::from_hex(
                    "67e6096a85ae67bb72f36e3c3af54fa57f520e518c68059babd9831f19cde05b"
                )
                .unwrap(),
                &Digest::from_hex(
                    "ad5c37ed90bb53c604e9ce787f6feeac7674bff229c92dc97ce2ba1115c0eb41"
                )
                .unwrap()
            ),
            Digest::from_hex("3aa2c47c47cd9e5c5259fd1c3428c30b9608201f5e163061deea8d2d7c65f2c3")
                .unwrap()
        );
        assert_eq!(
            *sha.hash_pair(
                &Digest::from_hex(
                    "0000000000000000000000000000000000000000000000000000000000000000"
                )
                .unwrap(),
                &Digest::from_hex(
                    "0000000000000000000000000000000000000000000000000000000000000000"
                )
                .unwrap()
            ),
            Digest::from_hex("da5698be17b9b46962335799779fbeca8ce5d491c0d26243bafef9ea1837a9d8")
                .unwrap()
        );
    }

    fn test_hash_raw_words<S: Sha>(sha: &S) {
        assert_eq!(
            *sha.hash_raw_words(&[
                1u32.to_be(),
                2u32.to_be(),
                3u32.to_be(),
                4u32.to_be(),
                5u32.to_be(),
                6u32.to_be(),
                7u32.to_be(),
                8u32.to_be(),
                7u32.to_be(),
                6u32.to_be(),
                5u32.to_be(),
                4u32.to_be(),
                3u32.to_be(),
                2u32.to_be(),
                1u32.to_be(),
                0u32.to_be(),
            ]),
            Digest::from_hex("b6f1e1b52e435545aa21cc9d3ce54e9af9da118042163abf2a739aebd413ac8d")
                .unwrap()
        );

        assert_eq!(
            *sha.hash_raw_words(&[1, 2, 3, 4, 5, 6, 7, 8, 7, 6, 5, 4, 3, 2, 1, 0,]),
            Digest::from_hex("0410500505eb63608def984ecc0b7820cba1012570e3d288c483f35021c971a6")
                .unwrap()
        );

        assert_eq!(
            *sha.hash_raw_words(&[
                1, 2, 3, 4, 5, 6, 7, 8, 7, 6, 5, 4, 3, 2, 1, 0, //
                1, 2, 3, 4, 5, 6, 7, 8, 7, 6, 5, 4, 3, 2, 1, 0,
            ]),
            Digest::from_hex("0343d500097e63123d3c7f418f465bfd2253652f351c90c75a05cb33946e71f1")
                .unwrap()
        );
    }
}
