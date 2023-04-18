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
//!
//! The primary component of this module is the [SessionReceipt]. A
//! [SessionReceipt] contains the result of a zkVM guest execution and
//! cryptographic proof of how it was generated. The prover can provide a
//! [SessionReceipt] to an untrusting party to convince them that the results
//! contained within the [SessionReceipt] came from running specific code.
//! Conversely, a verifier can inspect a [SessionReceipt] to confirm that its
//! results must have been generated from the expected code, even when this code
//! was run by an untrused source.
//!
//! # Usage
//! To create a receipt, use [crate::Session::prove]:
//! ```
//! use risc0_zkvm::{prove::default_hal, ControlId, Executor, ExecutorEnv, Session, SessionReceipt};
//! use risc0_zkvm_methods::FIB_ELF;
//!
//! let env = ExecutorEnv::builder().add_input(&[20]).build();
//! let mut exec = Executor::from_elf(env, FIB_ELF).unwrap();
//! let session = exec.run().unwrap();
//! let (hal, eval) = default_hal();
//! let receipt = session.prove(hal.as_ref(), &eval).unwrap();
//! ```
//!
//! To confirm that a [SessionReceipt] was honestly generated, use
//! [SessionReceipt::verify] and supply the ImageID of the code that should have
//! been executed as a parameter. (See
//! [risc0_build](https://docs.rs/risc0-build/latest/risc0_build/) for more
//! information about how ImageIDs are generated.)
//! ```
//! use risc0_zkvm::SessionReceipt;
//!
//! # use risc0_zkvm::{prove::default_hal, ControlId, Executor, ExecutorEnv, Session};
//! # use risc0_zkvm_methods::{FIB_ELF, FIB_ID};
//!
//! # let env = ExecutorEnv::builder().add_input(&[20]).build();
//! # let mut exec = Executor::from_elf(env, FIB_ELF).unwrap();
//! # let session = exec.run().unwrap();
//! # let (hal, eval) = default_hal();
//! # let receipt = session.prove(hal.as_ref(), &eval).unwrap();
//! receipt.verify(FIB_ID).unwrap();
//! ```
//!
//! The public outputs of the [SessionReceipt] are contained in the
//! [SessionReceipt::journal]. We provide serialization tools in the zkVM
//! [serde](crate::serde) module, which can be used to read data from the
//! journal as the same type it was written to the journal. If you prefer, you
//! can also directly access the [SessionReceipt::journal] as a `Vec<u8>`.

use alloc::vec::Vec;

use anyhow::Result;
use hex::FromHex;
use risc0_circuit_rv32im::layout;
use risc0_core::field::baby_bear::BabyBearElem;
use risc0_zkp::{core::digest::Digest, layout::Buffer, verify::VerificationError, MIN_CYCLES_PO2};
use serde::{Deserialize, Serialize};

use crate::{
    sha::rust_crypto::{Digest as _, Sha256},
    ControlId, CIRCUIT,
};

/// TODO
#[derive(Debug)]
pub struct SystemState {
    /// TODO
    pub pc: u32,

    /// TODO
    pub image_id: Digest,
}

/// TODO
#[derive(Debug)]
pub struct Global {
    /// TODO
    pub pre: SystemState,

    /// TODO
    pub post: SystemState,

    /// TODO
    pub check_dirty: u32,

    /// TODO
    pub output: Digest,
}

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SessionReceipt {
    /// TODO
    pub segments: Vec<SegmentReceipt>,

    /// TODO
    pub journal: Vec<u8>,
}

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SegmentReceipt {
    /// TODO
    pub seal: Vec<u32>,
}

impl SessionReceipt {
    /// TODO
    #[cfg(not(target_os = "zkvm"))]
    pub fn verify(&self, image_id: impl Into<Digest>) -> Result<(), VerificationError> {
        use risc0_zkp::core::hash::sha::Sha256HashSuite;
        let hal =
            risc0_zkp::verify::CpuVerifyHal::<_, Sha256HashSuite<_, crate::sha::Impl>, _>::new(
                &crate::CIRCUIT,
            );
        self.verify_with_hal(&hal, image_id)
    }

    /// TODO
    pub fn verify_with_hal<H>(
        &self,
        hal: &H,
        image_id: impl Into<Digest>,
    ) -> Result<(), VerificationError>
    where
        H: risc0_zkp::verify::VerifyHal<Elem = BabyBearElem>,
        H::HashFn: ControlId,
    {
        let (final_receipt, receipts) = self
            .segments
            .as_slice()
            .split_last()
            .ok_or(VerificationError::ReceiptFormatError)?;
        let mut prev_image_id = image_id.into();
        for receipt in receipts {
            receipt.verify_with_hal(hal)?;
            let metadata = receipt.get_metadata()?;
            if prev_image_id != metadata.pre.image_id {
                return Err(VerificationError::ImageVerificationError);
            }
            // assert_eq!(metadata.exit_code, ExitCode::SystemSplit);
            prev_image_id = metadata.post.image_id;
        }
        final_receipt.verify_with_hal(hal)?;
        let metadata = final_receipt.get_metadata()?;
        // log::debug!("metadata: {metadata:#?}");
        if prev_image_id != metadata.pre.image_id {
            return Err(VerificationError::ImageVerificationError);
        }

        let digest = Sha256::digest(&self.journal);
        let digest_words: &[u32] = bytemuck::cast_slice(digest.as_slice());
        let output_words = metadata.output.as_words();
        let is_journal_valid = || {
            (self.journal.is_empty() && output_words.iter().all(|x| *x == 0))
                || digest_words == output_words
        };
        if !is_journal_valid() {
            #[cfg(not(target_os = "zkvm"))]
            log::debug!(
                "journal: \"{}\", digest: 0x{}, output: 0x{}, {:?}",
                hex::encode(&self.journal),
                hex::encode(bytemuck::cast_slice(digest_words)),
                hex::encode(bytemuck::cast_slice(output_words)),
                self.journal
            );
            return Err(VerificationError::JournalDigestMismatch);
        }

        // assert_ne!(metadata.exit_code, ExitCode::SystemSplit);
        // Ok(metadata.exit_code)
        Ok(())
    }
}

impl SegmentReceipt {
    /// TODO
    pub fn get_metadata(&self) -> Result<Global, VerificationError> {
        let elems = bytemuck::cast_slice(&self.seal);
        Global::decode(layout::OutBuffer(elems))
    }

    /// TODO
    #[cfg(not(target_os = "zkvm"))]
    pub fn verify(&self) -> Result<(), VerificationError> {
        use risc0_zkp::core::hash::sha::Sha256HashSuite;
        let hal =
            risc0_zkp::verify::CpuVerifyHal::<_, Sha256HashSuite<_, crate::sha::Impl>, _>::new(
                &crate::CIRCUIT,
            );
        self.verify_with_hal(&hal)
    }

    /// TODO
    pub fn verify_with_hal<H>(&self, hal: &H) -> Result<(), VerificationError>
    where
        H: risc0_zkp::verify::VerifyHal<Elem = BabyBearElem>,
        H::HashFn: ControlId,
    {
        let control_id = &H::HashFn::CONTROL_ID;
        let check_code = |po2: u32, merkle_root: &Digest| -> Result<(), VerificationError> {
            let po2 = po2 as usize;
            let which = po2 - MIN_CYCLES_PO2;
            if which < control_id.len() {
                let entry: Digest = Digest::from_hex(control_id[which]).unwrap();
                if entry == *merkle_root {
                    return Ok(());
                }
            }
            Err(VerificationError::ControlVerificationError)
        };
        risc0_zkp::verify::verify(hal, &CIRCUIT, &self.seal, check_code)
    }

    /// Extracts the seal from the receipt, as a series of bytes.
    pub fn get_seal_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self.seal.as_slice())
    }
}

impl SystemState {
    fn decode(
        io: layout::OutBuffer,
        sys_state: &layout::SystemState,
    ) -> Result<Self, VerificationError> {
        let bytes: Vec<u8> = io
            .tree(sys_state.image_id)
            .get_bytes()
            .or(Err(VerificationError::ReceiptFormatError))?;
        let pc = io
            .tree(sys_state.pc)
            .get_u32()
            .or(Err(VerificationError::ReceiptFormatError))?;
        let image_id = Digest::try_from(bytes).or(Err(VerificationError::ReceiptFormatError))?;
        Ok(Self { pc, image_id })
    }
}

impl Global {
    fn decode(io: layout::OutBuffer) -> Result<Self, VerificationError> {
        let body = layout::LAYOUT.mux.body;
        let pre = SystemState::decode(io, body.pre)?;
        let post = SystemState::decode(io, body.post)?;
        let bytes: Vec<u8> = io
            .tree(body.output)
            .get_bytes()
            .or(Err(VerificationError::ReceiptFormatError))?;
        let output = Digest::try_from(bytes).or(Err(VerificationError::ReceiptFormatError))?;
        let check_dirty = io.get_u64(body.check_dirty) as u32;

        Ok(Self {
            pre,
            post,
            check_dirty,
            output,
        })
    }
}
