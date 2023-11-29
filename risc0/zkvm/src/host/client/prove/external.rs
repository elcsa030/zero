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

use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};
use risc0_binfmt::{MemoryImage, Program};
use risc0_zkvm_platform::{memory::GUEST_MAX_MEM, PAGE_SIZE};

use super::{Executor, Prover, ProverOpts};
use crate::{
    host::api::AssetRequest, sha::Digestible, ApiClient, Binary, ExecutorEnv, Receipt, SessionInfo,
    VerifierContext,
};

/// An implementation of a [Prover] that runs proof workloads via an external
/// `r0vm` process.
pub struct ExternalProver {
    name: String,
    r0vm_path: PathBuf,
}

impl ExternalProver {
    /// Construct an [ExternalProver].
    pub fn new<P: AsRef<Path>>(name: &str, r0vm_path: P) -> Self {
        Self {
            name: name.to_string(),
            r0vm_path: r0vm_path.as_ref().to_path_buf(),
        }
    }
}

impl Prover for ExternalProver {
    fn prove(
        &self,
        env: ExecutorEnv<'_>,
        ctx: &VerifierContext,
        opts: &ProverOpts,
        image: MemoryImage,
    ) -> Result<Receipt> {
        tracing::debug!("Launching {}", &self.r0vm_path.to_string_lossy());

        let image_id = image.compute_id()?;
        let client = ApiClient::new_sub_process(&self.r0vm_path)?;
        let receipt = client.prove(&env, opts.clone(), image.into())?;
        if opts.prove_guest_errors {
            receipt.verify_integrity_with_context(ctx)?;
            ensure!(
                receipt.get_metadata()?.pre.digest() == image_id,
                "received unexpected image ID: expected {}, found {}",
                hex::encode(&image_id),
                hex::encode(&receipt.get_metadata()?.pre.digest())
            );
        } else {
            receipt.verify_with_context(ctx, image_id)?;
        }

        Ok(receipt)
    }

    fn prove_elf_with_ctx(
        &self,
        env: ExecutorEnv<'_>,
        ctx: &VerifierContext,
        elf: &[u8],
        opts: &ProverOpts,
    ) -> Result<Receipt> {
        tracing::debug!("Launching {}", &self.r0vm_path.to_string_lossy());

        let program = Program::load_elf(elf, GUEST_MAX_MEM as u32)?;
        let image = MemoryImage::new(&program, PAGE_SIZE as u32)?;
        let image_id = image.compute_id()?;
        let client = ApiClient::new_sub_process(&self.r0vm_path)?;
        let binary = Binary::new_elf_inline(elf.to_vec().into());
        let receipt = client.prove(&env, opts.clone(), binary)?;
        if opts.prove_guest_errors {
            receipt.verify_integrity_with_context(ctx)?;
            ensure!(
                receipt.get_metadata()?.pre.digest() == image_id,
                "received unexpected image ID: expected {}, found {}",
                hex::encode(&image_id),
                hex::encode(&receipt.get_metadata()?.pre.digest())
            );
        } else {
            receipt.verify_with_context(ctx, image_id)?;
        }

        Ok(receipt)
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl Executor for ExternalProver {
    fn execute(&self, env: ExecutorEnv<'_>, image: MemoryImage) -> Result<SessionInfo> {
        let client = ApiClient::new_sub_process(&self.r0vm_path)?;
        let segments_out = AssetRequest::Inline;
        client.execute(&env, image.into(), segments_out, |_, _| Ok(()))
    }

    fn execute_elf(&self, env: ExecutorEnv<'_>, elf: &[u8]) -> Result<SessionInfo> {
        let binary = Binary::new_elf_inline(elf.to_vec().into());
        let client = ApiClient::new_sub_process(&self.r0vm_path)?;
        let segments_out = AssetRequest::Inline;
        client.execute(&env, binary, segments_out, |_, _| Ok(()))
    }
}
