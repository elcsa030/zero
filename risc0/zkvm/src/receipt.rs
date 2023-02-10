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

//! Manages the output and cryptographic data for a proven computation

use alloc::vec::Vec;

use anyhow::{anyhow, Result};
#[cfg(not(target_os = "zkvm"))]
use risc0_core::field::baby_bear::BabyBear;
use risc0_core::field::baby_bear::BabyBearElem;
use risc0_zeroio::{Deserialize as ZeroioDeserialize, Serialize as ZeroioSerialize};
#[cfg(not(target_os = "zkvm"))]
use risc0_zkp::core::config::HashSuiteSha256;
use risc0_zkp::{
    core::sha::{Digest, Sha256},
    verify::VerificationError,
    MIN_CYCLES_PO2,
};
use risc0_zkvm_platform::{
    syscall::{DIGEST_BYTES, DIGEST_WORDS},
    WORD_SIZE,
};
use serde::{Deserialize, Serialize};

use crate::{sha, ControlId, CIRCUIT};

#[cfg(all(feature = "std", not(target_os = "zkvm")))]
pub fn insecure_skip_seal() -> bool {
    cfg!(feature = "insecure_skip_seal")
        && std::env::var("RISC0_INSECURE_SKIP_SEAL").unwrap_or_default() == "1"
}

#[cfg(target_os = "zkvm")]
pub fn insecure_skip_seal() -> bool {
    cfg!(feature = "insecure_skip_seal")
}

/// The receipt serves as a zero-knowledge proof of computation.
#[derive(Deserialize, Serialize, ZeroioSerialize, ZeroioDeserialize, Clone, Debug)]
pub struct Receipt {
    /// The journal contains the public outputs of the computation.
    pub journal: Vec<u32>,
    /// The seal is an opaque cryptographic blob that attests to the integrity
    /// of the computation. It consists of merkle commitments and query data for
    /// an AIR-FRI STARK that includes a PLONK-based permutation argument.
    pub seal: Vec<u32>,
}

pub fn verify_with_hal<'a, H, D>(hal: &H, image_id: D, seal: &[u32], journal: &[u32]) -> Result<()>
where
    H: risc0_zkp::verify::VerifyHal<Elem = BabyBearElem>,
    &'a Digest: From<D>,
{
    let image_id: &Digest = image_id.into();
    let check_globals = |io: &[BabyBearElem]| -> Result<(), VerificationError> {
        // verify the image_id
        // Convert to u32 first
        let io: Vec<u32> = io.iter().map(|x| u32::from(*x)).collect();
        #[cfg(not(target_os = "zkvm"))]
        for (i, word) in io.iter().enumerate() {
            log::debug!("io: 0x{i:02x} -> 0x{word:08x}");
        }
        let slice = &io[WORD_SIZE..WORD_SIZE + DIGEST_BYTES];
        let bytes: Vec<u8> = slice.iter().map(|x| *x as u8).collect();
        let actual = Digest::try_from(bytes);
        if !actual.map(|digest| image_id == &digest).unwrap_or(false) {
            return Err(VerificationError::ImageVerificationError);
        }

        // Each global output is generated by an output ecall. The handler
        // for the output ecall splits the 32-bit value supplied to the
        // ecall into two 16-bit chunks. Since each index of the journal
        // contains all 32 bits of the output ecall value, we must shift
        // and combine the two 16-bit values before comparing them to
        // the journal.

        let slice = &io[WORD_SIZE + DIGEST_BYTES..];
        let outputs: Vec<u32> = slice.chunks_exact(2).map(|x| x[0] | x[1] << 16).collect();

        let digest = sha::Impl::hash_words(journal);
        if digest.as_words() != &outputs[..DIGEST_WORDS] {
            return Err(VerificationError::JournalSealRootMismatch);
        }

        Ok(())
    };

    #[cfg(any(feature = "std", target_os = "zkvm"))]
    if insecure_skip_seal() {
        return Ok(());
    }

    let control_id = ControlId::new();
    let check_code = |po2: u32, merkle_root: &Digest| -> Result<(), VerificationError> {
        let po2 = po2 as usize;
        let which = po2 - MIN_CYCLES_PO2;
        if which >= control_id.table.len() {
            return Err(VerificationError::ControlVerificationError);
        }
        if control_id.table[which] != *merkle_root {
            return Err(VerificationError::ControlVerificationError);
        }
        Ok(())
    };

    risc0_zkp::verify::verify(hal, &CIRCUIT, seal, check_code, check_globals)
        .map_err(|err| anyhow!("Verification failed: {}", err))
}

impl Receipt {
    pub fn new(journal: &[u32], seal: &[u32]) -> Self {
        Self {
            journal: Vec::from(journal),
            seal: Vec::from(seal),
        }
    }

    #[cfg(not(target_os = "zkvm"))]
    /// Verifies a receipt using CPU.
    pub fn verify<'a, D>(&self, image_id: D) -> Result<()>
    where
        &'a Digest: From<D>,
    {
        let hal = risc0_zkp::verify::CpuVerifyHal::<
            BabyBear,
            HashSuiteSha256<BabyBear, sha::Impl>,
            _,
        >::new(&crate::CIRCUIT);

        verify_with_hal(&hal, image_id, &self.seal, &self.journal)
    }

    /// Verifies a receipt using the hardware acceleration layer.
    pub fn verify_with_hal<'a, H, D>(&self, hal: &H, image_id: D) -> Result<()>
    where
        H: risc0_zkp::verify::VerifyHal<Elem = BabyBearElem>,
        &'a Digest: From<D>,
    {
        verify_with_hal(hal, image_id, &self.seal, &self.journal)
    }

    /// Extracts the journal from the receipt, as a series of bytes.
    pub fn get_journal_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self.journal.as_slice())
    }

    /// Extracts the seal from the receipt, as a series of bytes.
    pub fn get_seal_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self.seal.as_slice())
    }
}
