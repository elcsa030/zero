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

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use risc0_zkp::core::sha::default_implementation;
use risc0_zkp::verify::adapter::VerifyAdapter;

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

impl Receipt {
    #[cfg(feature = "verify")]
    pub fn verify<'a, M>(&self, method_id: &'a M) -> Result<()>
    where
        M: ?Sized,
        MethodId: From<&'a M>,
    {
        use risc0_zkp::{
            core::{log2_ceil, sha::Digest},
            verify::verify,
            MIN_CYCLES,
        };
        let method_id: MethodId = method_id.into();
        let check_code = |po2, merkle_root: &Digest| {
            let which = po2 as usize - log2_ceil(MIN_CYCLES);
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

        let sha = default_implementation();
        let circuit: &risc0_zkvm_circuit::CircuitImpl = &crate::CIRCUIT;
        let mut adapter = VerifyAdapter::new(circuit);
        verify(sha, &mut adapter, &self.seal, check_code)
            .map_err(|err| anyhow!("Verification failed: {:?}", err))
    }

    // Compatible API with FFI-based prover.
    pub fn get_journal_vec(&self) -> Result<Vec<u32>> {
        Ok(self.journal.clone())
    }

    // Compatible API with FFI-based prover.
    pub fn get_journal(&self) -> Result<&[u8]> {
        Ok(bytemuck::cast_slice(self.journal.as_slice()))
    }

    // Compatible API with FFI-based prover.
    // FIXME: Change API to avoid copy.
    pub fn get_seal(&self) -> Result<&[u32]> {
        Ok(self.seal.as_slice())
    }
}
