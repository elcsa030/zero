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

//! A SHA-256 based CRNG used in Fiat-Shamir.

use core::marker::PhantomData;

use rand_core::{impls, Error, RngCore};

use super::sha::{Digest, Sha256, DIGEST_WORDS};

/// A random number generator driven by a [Sha256].
#[derive(Clone, Debug)]
pub struct ShaRng<S: Sha256> {
    // Pool 0 receives new entropy and is where values are drawn from.
    pool0: S::DigestPtr,
    // Pool 1 provides secret state in the step function. It is never observable.
    pool1: S::DigestPtr,
    pool_used: usize,
    phantom_sha: PhantomData<S>,
}

// TODO(victor) Examine the usages of this type to understand it better. Is this
// something someone from the public might use, or should it be crate-scoped?
// It might be possible to improve on this with inspiration from rand_chacha.
// https://docs.rs/rand_chacha/0.3.1/rand_chacha/struct.ChaCha12Rng.html#impl-RngCore
// Might also read the NIST-800 recomendations to see if there are any
// improvements or issues we need to watch out for.
// Should new _require_ a seed such that we ensure no unseeded usage occurs?
// https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-90Ar1.pdf
//
// NOTE: This is used in the IOP as a transcript hashing primitve. Are there any
// lessons about transcript hashing that we need to think about?
impl<S: Sha256> ShaRng<S> {
    /// Create a new [ShaRng] from a given [Sha256].
    pub fn new() -> ShaRng<S> {
        Self {
            pool0: S::hash_bytes(b"Hello"),
            pool1: S::hash_bytes(b"World"),
            pool_used: 0,
            phantom_sha: PhantomData,
        }
    }

    /// Mix the pool with a specified [Digest].
    pub fn mix(&mut self, val: &Digest) {
        S::mix(&mut self.pool0, val);
        self.step();
    }

    fn step(&mut self) {
        self.pool0 = S::hash_pair(&self.pool0, &self.pool1);
        self.pool1 = S::hash_pair(&self.pool0, &self.pool1);
        self.pool_used = 0;
    }
}

impl<S: Sha256> RngCore for ShaRng<S> {
    fn next_u32(&mut self) -> u32 {
        if self.pool_used == DIGEST_WORDS {
            self.step();
        }
        let out = self.pool0.as_words()[self.pool_used];
        // Mark this word as used.
        self.pool_used += 1;
        out
    }

    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}

#[allow(missing_docs)]
pub mod testutil {
    use rand_core::RngCore;

    use super::ShaRng;
    use crate::core::sha::Sha256;

    // Runs conformance test on a SHA implementation to make sure it
    // properly behaves for generating pseudo-random numbers.
    pub fn test_sha_rng_impl<S: Sha256>() {
        let mut x = ShaRng::<S>::new();
        for _ in 0..10 {
            x.next_u32();
        }
        assert_eq!(x.next_u32(), 785921476);
        x.mix(&*S::hash_bytes(b"foo"));
        assert_eq!(x.next_u32(), 4167871101);
    }
}
