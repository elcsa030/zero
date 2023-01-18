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

//! Simple wrappers for a CPU-based SHA-256 implementation.

use alloc::{boxed::Box, vec::Vec};
use core::slice;

use sha2::{
    compress256,
    digest::generic_array::{typenum::U64, GenericArray},
    Digest as ShaDigest, Sha256,
};

use super::sha::{Digest, Sha, DIGEST_WORDS, SHA256_INIT};

/// A CPU-based [Sha] implementation.
#[derive(Clone)]
pub struct Impl {}

fn set_word(buf: &mut [u8], idx: usize, word: u32) {
    buf[(4 * idx)..(4 * idx + 4)].copy_from_slice(&word.to_ne_bytes());
}

impl Impl {
    /// Compute the hash of a slice of plain-old-data using the
    /// specified offset and stride.  'size' specifies the number of
    /// elements to hash.
    pub fn hash_pod_stride<T: bytemuck::Pod>(
        &self,
        pods: &[T],
        offset: usize,
        size: usize,
        stride: usize,
    ) -> Box<Digest> {
        let mut state = *SHA256_INIT.get();
        for word in state.iter_mut() {
            *word = word.to_be();
        }

        let mut block: GenericArray<u8, U64> = GenericArray::default();

        let mut bytes = pods
            .iter()
            .skip(offset)
            .step_by(stride)
            .take(size)
            .flat_map(|pod| bytemuck::cast_slice(slice::from_ref(pod)) as &[u8])
            .cloned()
            .fuse();

        let mut off = 0;
        while let Some(b1) = bytes.next() {
            let b2 = bytes.next().unwrap_or(0);
            let b3 = bytes.next().unwrap_or(0);
            let b4 = bytes.next().unwrap_or(0);
            set_word(
                block.as_mut_slice(),
                off,
                u32::from_ne_bytes([b1, b2, b3, b4]),
            );
            off += 1;
            if off == 16 {
                compress256(&mut state, slice::from_ref(&block));
                off = 0;
            }
        }
        if off != 0 {
            block[off * 4..].fill(0);
            compress256(&mut state, slice::from_ref(&block));
        }

        for word in state.iter_mut() {
            *word = word.to_be();
        }
        Box::new(Digest::new(state))
    }
}

impl Sha for Impl {
    type DigestPtr = Box<Digest>;

    fn hash_bytes(&self, bytes: &[u8]) -> Self::DigestPtr {
        let digest = Sha256::digest(bytes);
        let words: Vec<u32> = digest
            .as_slice()
            .chunks(4)
            .map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect();
        Box::new(Digest::new(words.try_into().unwrap()))
    }

    fn hash_words(&self, words: &[u32]) -> Self::DigestPtr {
        self.hash_bytes(bytemuck::cast_slice(words) as &[u8])
    }

    fn hash_raw_words(&self, words: &[u32]) -> Self::DigestPtr {
        assert!(
            words.len() % 16 == 0,
            "{} should be a multiple of 16, the number of words per SHA block",
            words.len()
        );
        let mut state = *SHA256_INIT.get();
        for word in state.iter_mut() {
            *word = word.to_be();
        }
        for block in words.chunks(16) {
            let block_u8: &[u8] = bytemuck::cast_slice(block);
            compress256(
                &mut state,
                slice::from_ref(GenericArray::from_slice(block_u8)),
            )
        }
        for word in state.iter_mut() {
            *word = word.to_be();
        }
        Box::new(Digest::new(state))
    }

    fn hash_raw_pod_slice<T: bytemuck::Pod>(&self, pod: &[T]) -> Self::DigestPtr {
        let u8s: &[u8] = bytemuck::cast_slice(pod);
        let mut state = *SHA256_INIT.get();
        for word in state.iter_mut() {
            *word = word.to_be();
        }
        let mut blocks = u8s.chunks_exact(64);
        for block in blocks.by_ref() {
            compress256(&mut state, slice::from_ref(GenericArray::from_slice(block)));
        }
        let remainder = blocks.remainder();
        if remainder.len() > 0 {
            let mut last_block: GenericArray<u8, U64> = GenericArray::default();
            bytemuck::cast_slice_mut(last_block.as_mut_slice())[..remainder.len()]
                .clone_from_slice(remainder);
            compress256(&mut state, slice::from_ref(&last_block));
        }
        for word in state.iter_mut() {
            *word = word.to_be();
        }
        Box::new(Digest::new(state))
    }

    // Digest two digest into one
    fn compress(
        &self,
        orig_state: &Digest,
        block_half1: &Digest,
        block_half2: &Digest,
    ) -> Self::DigestPtr {
        let mut state: [u32; DIGEST_WORDS] = *orig_state.get();
        for word in state.iter_mut() {
            *word = word.to_be();
        }
        let mut block: GenericArray<u8, U64> = GenericArray::default();
        for i in 0..8 {
            set_word(block.as_mut_slice(), i, block_half1.as_slice()[i]);
            set_word(block.as_mut_slice(), 8 + i, block_half2.as_slice()[i]);
        }
        compress256(&mut state, slice::from_ref(&block));
        for word in state.iter_mut() {
            *word = word.to_be();
        }
        Box::new(Digest::new(state))
    }

    // Generate a new digest by mixing two digests together via XOR,
    // and stores it back in the pool.
    fn mix(&self, pool: &mut Self::DigestPtr, val: &Digest) {
        // CPU based sha can do this in place without generating another digest pointer.
        for (pool_word, val_word) in pool.get_mut().iter_mut().zip(val.get()) {
            *pool_word ^= *val_word;
        }
    }
}

impl core::fmt::Debug for Impl {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "CPU SHA256 implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::Impl;

    #[test]
    fn test_impl() {
        crate::core::sha::testutil::test_sha_impl(&Impl {})
    }
}
