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
//! The primary component of this module is the [Receipt]. A [Receipt] contains
//! the result of a zkVM guest execution and cryptographic proof of how it was
//! generated. The prover can provide a [Receipt] to an untrusting party to
//! convince them that the results contained within the [Receipt] came from
//! running specific code. Conversely, a verifier can inspect a [Receipt] to
//! confirm that its results must have been generated from the expected code,
//! even when this code was run by an untrused source.
//!
//! # Usage
//! To create a receipt, use [crate::prove::Prover::run] or
//! [crate::prove::Prover::run_with_hal]:
//! ```ignore
//! // In real code, the ELF & Image ID would be generated by risc0 build scripts from guest code
//! use methods::{EXAMPLE_ELF, EXAMPLE_ID};
//! use risc0_zkvm::Prover;
//!
//! let mut prover = Prover::new(&EXAMPLE_ELF, EXAMPLE_ID)?;
//! let receipt = prover.run()?;
//! ```
//! To confirm that a [Receipt] was honestly generated, use [Receipt::verify]
//! and supply the ImageID of the code that should have been executed as a
//! parameter. (See
//! [risc0_build](https://docs.rs/risc0-build/latest/risc0_build/) for more
//! information about how ImageIDs are generated.)
//! ```
//! use risc0_zkvm::Receipt;
//! # use risc0_zkvm::serde::Result;
//!
//! # fn main() -> Result<()> {
//! # // Need to awkwardly set up a fake Receipt since we can't use the guest in docs
//! # let journal_words = risc0_zkvm::serde::to_vec(&String::from("test"))
//! #        .unwrap()
//! #        .iter()
//! #        .map(|&x| x.to_le_bytes())
//! #        .collect::<Vec<[u8; 4]>>();
//! # let mut journal: Vec<u8> = vec!();
//! # for word in journal_words.iter() {
//! #    for byte in word.iter() {
//! #        journal.push(*byte);
//! #    }
//! # }
//! # let receipt = Receipt {
//! #    seal: vec!(),
//! #    journal: journal,
//! # };
//! # use crate::risc0_zkvm::sha::Sha256;
//! # const IMAGE_ID: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
//! // Here `receipt` is a Receipt whose journal contains the String "test"
//! receipt.verify(&IMAGE_ID);
//! let committed_value: String = risc0_zkvm::serde::from_slice(&receipt.journal)?;
//! assert_eq!("test", committed_value);
//! # Ok(())
//! # }
//! ```
//! The public outputs of the [Receipt] are contained in the [Receipt::journal].
//! We provide serialization tools in the zkVM [serde](crate::serde) module,
//! which can be used to read data from the journal as the same type it was
//! written to the journal. If you prefer, you can also directly
//! access the [Receipt::journal] as a `Vec<u8>`.

use alloc::vec::Vec;

use anyhow::{anyhow, Result};
use hex::FromHex;
use risc0_circuit_rv32im::layout;
#[cfg(not(target_os = "zkvm"))]
use risc0_core::field::baby_bear::BabyBear;
use risc0_core::field::baby_bear::BabyBearElem;
#[cfg(not(target_os = "zkvm"))]
use risc0_zkp::core::hash::{sha::Sha256HashSuite, HashSuite};
use risc0_zkp::{core::digest::Digest, layout::Buffer, verify::VerificationError, MIN_CYCLES_PO2};
use serde::{Deserialize, Serialize};

use crate::{
    sha::rust_crypto::{Digest as _, Sha256},
    ControlId, CIRCUIT,
};

/// Reports whether the zkVM is in the insecure seal skipping mode.
///
/// Returns `true` when in the insecure seal skipping mode. Returns `false` when
/// in normal secure mode.
///
/// When `insecure_skip_seal` is `false`, [crate::prove::Prover::run] will
/// generate a seal when run that proves faithful execution, and
/// [Receipt::verify] will check the seal and return an `Err` if the seal is
/// missing or invalid.
///
/// When `insecure_skip_seal` is `true`, [crate::prove::Prover::run] will not
/// generate a seal and the Receipt does not contain a proof of execution, and
/// [Receipt::verify] will not check the seal and will return an `Ok` even if
/// the seal is missing or invalid.
///
/// In particular, if [Receipt::verify] is run with `insecure_skip_seal` being
/// `false`, it will always return an `Err` for any [Receipt] generated while
/// `insecure_skip_seal` was `true`.
#[cfg(all(feature = "std", not(target_os = "zkvm")))]
pub fn insecure_skip_seal() -> bool {
    cfg!(feature = "insecure_skip_seal")
        && std::env::var("RISC0_INSECURE_SKIP_SEAL").unwrap_or_default() == "1"
}

/// Reports whether the zkVM is in the insecure seal skipping mode
///
/// See the non-zkvm version of this documentation for details.
#[cfg(target_os = "zkvm")]
pub fn insecure_skip_seal() -> bool {
    cfg!(feature = "insecure_skip_seal")
}

/// The Receipt is a zero-knowledge proof of computation.
///
/// A Receipt is an attestation that specific code, identified by an `ImageID`,
/// was executed and generated the results included in the Receipt's
/// [Receipt::journal].
///
/// This Receipt is zero-knowledge in the sense that no information that is not
/// deliberately published is included in the receipt. Everything in the
/// [Receipt::journal] was explicitly written to the journal by the zkVM guest,
/// and the [Receipt::seal] is cryptographically opaque.
///
/// See [the module-level documentation](crate::receipt) for usage examples.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Receipt {
    /// The journal contains the public outputs of the computation.
    ///
    /// Specifically, the journal contains the data explicitly written to it by
    /// the guest with [crate::guest::env::commit].
    pub journal: Vec<u8>,

    /// The seal is an opaque cryptographic blob that attests to the integrity
    /// of the computation. It consists of merkle commitments and query data for
    /// an AIR-FRI STARK that includes a PLONK-based permutation argument.
    pub seal: Vec<u32>,
}

/// Verifies the `seal` and `journal` form a valid [Receipt]
///
/// This is the underlying function used by [Receipt::verify],
/// [Receipt::verify_with_hash], and [Receipt::verify_with_hal]. It is exposed
/// for use cases where `seal` and `journal` are not already part of a [Receipt]
/// object.
///
/// This verifies that this receipt was constructed by running code whose
/// ImageID is `image_id`. Returns `Ok(())` if this is true. If the code used to
/// generate this Receipt has a different ImageID, or if it was generated by an
/// insecure or malicious prover, this will return an `Err`.
///
/// This function allows the user to specify the Hardware Abstraction Layer
/// (HAL) to be used for verification with the `hal` parameter.
pub fn verify_with_hal<'a, H, D>(hal: &H, image_id: D, seal: &[u32], journal: &[u8]) -> Result<()>
where
    H: risc0_zkp::verify::VerifyHal<Elem = BabyBearElem>,
    H::HashFn: ControlId,
    &'a Digest: From<D>,
{
    let image_id: &Digest = image_id.into();
    let check_globals = |io: &[BabyBearElem]| -> Result<(), VerificationError> {
        // Decode the global outputs
        let global = Global::decode(layout::OutBuffer(io))?;
        #[cfg(not(target_os = "zkvm"))]
        log::debug!("io: {global:#?}");

        // verify the image_id
        if global.pre.image_id != *image_id {
            return Err(VerificationError::ImageVerificationError);
        }

        // verify the output matches the digest of the journal
        let digest = Sha256::digest(journal);
        let digest_words: &[u32] = bytemuck::cast_slice(digest.as_slice());
        let output_words = global.output.as_words();
        let is_journal_valid = || {
            (journal.is_empty() && output_words.iter().all(|x| *x == 0))
                || digest_words == output_words
        };
        if !is_journal_valid() {
            #[cfg(not(target_os = "zkvm"))]
            log::debug!(
                "journal: \"{}\", digest: 0x{}, output: 0x{}",
                hex::encode(journal),
                hex::encode(bytemuck::cast_slice(digest_words)),
                hex::encode(bytemuck::cast_slice(output_words))
            );
            return Err(VerificationError::JournalDigestMismatch);
        }

        Ok(())
    };

    #[cfg(any(feature = "std", target_os = "zkvm"))]
    if insecure_skip_seal() {
        return Ok(());
    }

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

    risc0_zkp::verify::verify(hal, &CIRCUIT, seal, check_code, check_globals)
        .map_err(|err| anyhow!("Verification failed: {}", err))
}

impl Receipt {
    /// Constructs a Receipt from a journal and a seal
    ///
    /// A Receipt is more commonly constructed as the output of
    /// [crate::prove::Prover::run], but since it has no data beyond its
    /// [journal](Receipt::journal) and [seal](Receipt::seal) it can be
    /// directly constructed from them.
    pub fn new(journal: &[u8], seal: &[u32]) -> Self {
        Self {
            journal: Vec::from(journal),
            seal: Vec::from(seal),
        }
    }

    #[cfg(not(target_os = "zkvm"))]
    /// Verifies a SHA-256 receipt using CPU
    ///
    /// Verifies that this receipt was constructed by running code whose ImageID
    /// is `image_id`. Returns `Ok(())` if this is true. If the code used to
    /// generate this Receipt has a different ImageID, or if it was generated by
    /// an insecure or malicious prover, this will return an `Err`.
    ///
    /// This runs the verification on the CPU and is for receipts using SHA-256
    /// as their hash function.
    pub fn verify<'a, D>(&self, image_id: D) -> Result<()>
    where
        &'a Digest: From<D>,
    {
        self.verify_with_hash::<Sha256HashSuite<BabyBear, crate::sha::Impl>, _>(image_id)
    }

    /// Verifies a receipt with a user-specified hash function using the CPU.
    ///
    /// Verifies that this receipt was constructed by running code whose ImageID
    /// is `image_id`. Returns `Ok(())` if this is true. If the code used to
    /// generate this Receipt has a different ImageID, or if it was generated by
    /// an insecure or malicious prover, this will return an `Err`.
    ///
    /// This runs the verification on the CPU and is for receipts using the hash
    /// function specified by `HS`.
    #[cfg(not(target_os = "zkvm"))]
    pub fn verify_with_hash<'a, HS, D>(&self, image_id: D) -> Result<()>
    where
        HS: HashSuite<BabyBear>,
        HS::HashFn: ControlId,
        &'a Digest: From<D>,
    {
        let hal = risc0_zkp::verify::CpuVerifyHal::<BabyBear, HS, _>::new(&crate::CIRCUIT);

        self.verify_with_hal(&hal, image_id)
    }

    /// Verifies a receipt with a user-specified hash function and HAL
    ///
    /// Verifies that this receipt was constructed by running code whose ImageID
    /// is `image_id`. Returns `Ok(())` if this is true. If the code used to
    /// generate this Receipt has a different ImageID, or if it was generated by
    /// an insecure or malicious prover, this will return an `Err`.
    ///
    /// This function allows the user to specify the Hardware Abstraction Layer
    /// (HAL) to be used for verification with the `hal` parameter. Note that
    /// the selection of a HAL also implies the selection of the hash function
    /// associated with that HAL.
    pub fn verify_with_hal<'a, H, D>(&self, hal: &H, image_id: D) -> Result<()>
    where
        H: risc0_zkp::verify::VerifyHal<Elem = BabyBearElem>,
        H::HashFn: ControlId,
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

#[derive(Debug)]
struct SystemState {
    _pc: u32,
    image_id: Digest,
}

#[derive(Debug)]
struct Global {
    pre: SystemState,
    _post: SystemState,
    _check_dirty: u32,
    output: Digest,
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
        let _pc = io
            .tree(sys_state.pc)
            .get_u32()
            .or(Err(VerificationError::ReceiptFormatError))?;
        let image_id = Digest::try_from(bytes).or(Err(VerificationError::ReceiptFormatError))?;
        Ok(Self { _pc, image_id })
    }
}

impl Global {
    fn decode(io: layout::OutBuffer) -> Result<Self, VerificationError> {
        let body = layout::LAYOUT.mux.body;
        let pre = SystemState::decode(io, body.pre)?;
        let _post = SystemState::decode(io, body.post)?;
        let bytes: Vec<u8> = io
            .tree(body.output)
            .get_bytes()
            .or(Err(VerificationError::ReceiptFormatError))?;
        let output = Digest::try_from(bytes).or(Err(VerificationError::ReceiptFormatError))?;
        let _check_dirty = io.get_u64(body.check_dirty) as u32;

        Ok(Self {
            pre,
            _post,
            _check_dirty,
            output,
        })
    }
}

/// TODO
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SessionReceipt {
    /// TODO
    pub segments: Vec<SegmentReceipt>,

    /// TODO
    pub journal: Vec<u8>,
}

/// TODO
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SegmentReceipt {
    /// TODO
    pub seal: Vec<u32>,
}

/// TODO
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ReceiptMetadata {}

impl SessionReceipt {
    /// TODO
    pub fn verify<D>(&self, _image_id: D) -> Result<()>
    where
        Digest: From<D>,
    {
        todo!()
    }
}

impl SegmentReceipt {
    /// TODO
    pub fn get_metadata(&self) -> ReceiptMetadata {
        todo!()
    }

    /// TODO
    pub fn verify(&self) -> Result<()> {
        todo!()
    }

    /// Extracts the seal from the receipt, as a series of bytes.
    pub fn get_seal_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self.seal.as_slice())
    }
}

impl ReceiptMetadata {}
