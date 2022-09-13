// Copyright 2022 Risc0, Inc.
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

use alloc::vec::Vec;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::method_id::MethodId;

#[derive(Deserialize, Serialize, Clone)]
pub struct Receipt {
    pub journal: Vec<u32>,
    pub seal: Vec<u32>,
}

// FIXME: Remove this temporary conversion once our API is the same between
// FFI and rust-based provers.
impl From<&MethodId> for MethodId {
    fn from(method_id: &MethodId) -> Self {
        method_id.clone()
    }
}

#[cfg(feature = "verify")]
pub fn verify_with_hal<'a, M, H>(hal: &H, method_id: &'a M, seal: &[u32]) -> Result<()>
where
    H: risc0_zkp::verify::VerifyHal,
    M: ?Sized,
    MethodId: From<&'a M>,
{
    use anyhow::anyhow;
    use risc0_zkp::{
        core::{log2_ceil, sha::Digest},
        verify::verify,
        MIN_CYCLES,
    };

    use crate::CIRCUIT;

    let method_id: MethodId = method_id.into();
    let check_code = |po2, merkle_root: &Digest| {
        let which = po2 as usize - log2_ceil(MIN_CYCLES);
        #[cfg(not(target_arch = "riscv32"))]
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("merkle_root: {merkle_root}");
            log::debug!("MethodId");
            for (i, entry) in method_id.table.iter().enumerate() {
                let marker = if i == which { "*" } else { "" };
                log::debug!("  {i}: {entry}{marker}");
            }
        }
        method_id.table[which] == *merkle_root
    };

    verify(hal, &CIRCUIT, seal, check_code).map_err(|err| anyhow!("Verification failed: {:?}", err))
}

impl Receipt {
    /// Verifies the proof receipt generated when the guest program is run.
    ///
    /// # Arguments
    ///
    /// * `MethodID` - The unique method ID of the guest binary.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// receipt.verify(MY_PROGRAM_ID).unwrap();
    /// ```
    #[cfg(all(feature = "verify", feature = "host"))]
    pub fn verify<'a, M>(&self, method_id: &'a M) -> Result<()>
    where
        M: ?Sized,
        MethodId: From<&'a M>,
    {
        use risc0_zkp::{core::sha::default_implementation, verify::CpuVerifyHal};

        use crate::CIRCUIT;

        let sha = default_implementation();
        let hal = CpuVerifyHal::new(sha, &CIRCUIT);
        self.verify_with_hal(&hal, method_id)
    }

    /// This function is called by [verify](Receipt::verify), which provides the
    /// CPU HAL generated using the program circuit.
    ///
    /// # Arguments
    ///
    /// * `hal` - the HAL used to represent the guest program circuit.
    /// * `MethodID` - The unique method ID of the guest binary.
    #[cfg(feature = "verify")]
    pub fn verify_with_hal<'a, M, H>(&self, hal: &H, method_id: &'a M) -> Result<()>
    where
        H: risc0_zkp::verify::VerifyHal,
        M: ?Sized,
        MethodId: From<&'a M>,
    {
        verify_with_hal(hal, method_id, &self.seal)
    }

    // Compatible API with FFI-based prover.
    /// Retrieves the receipt journal as a vector of `u32` values.
    /// This journal contains all values publicly committed to the journal by
    /// the guest.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// let receipt: Receipt = prover.run();
    /// let result: Vec<u32> = self.receipt.get_journal_vec()?;
    /// ```
    pub fn get_journal_vec(&self) -> Result<Vec<u32>> {
        Ok(self.journal.clone())
    }

    // Compatible API with FFI-based prover.
    /// Retrieves the receipt journal as a byte array.
    pub fn get_journal(&self) -> Result<&[u8]> {
        Ok(bytemuck::cast_slice(self.journal.as_slice()))
    }

    // Compatible API with FFI-based prover.
    // FIXME: Change API to avoid copy.
    /// Retrieves the receipt seal as a vector of `u32` values.
    pub fn get_seal(&self) -> Result<&[u32]> {
        Ok(self.seal.as_slice())
    }
}
