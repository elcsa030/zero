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

//! Manages the output and cryptographic data for a proven computation.

use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};
use core::fmt::Debug;

use anyhow::Result;
use risc0_binfmt::SystemState;
use risc0_circuit_rv32im::layout;
use risc0_core::field::baby_bear::BabyBear;
use risc0_zkp::{
    core::{
        digest::Digest,
        hash::{
            blake2b::Blake2bCpuHashSuite, poseidon::PoseidonHashSuite, sha::Sha256HashSuite,
            HashSuite,
        },
    },
    layout::Buffer,
    verify::VerificationError,
};
use risc0_zkvm_platform::WORD_SIZE;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::control_id::{BLAKE2B_CONTROL_ID, POSEIDON_CONTROL_ID, SHA256_CONTROL_ID};
// Make succinct receipt available through this `receipt` module.
pub use super::recursion::SuccinctReceipt;
use crate::{
    host::groth16::{Groth16Proof, Groth16Seal},
    receipt_metadata::{Assumptions, MaybePruned, Output},
    serde::{from_slice, Error},
    sha::{Digestible, Sha256},
    ExitCode, ReceiptMetadata,
};

/// A receipt attesting to the execution of a Session.
///
/// A Receipt is a zero-knowledge proof of computation. It attests that the
/// [Receipt::journal] was produced by executing a [crate::Session] based on a
/// specified memory image. This image is _not_ included in the receipt; the
/// verifier must provide an [ImageID](https://dev.risczero.com/terminology),
/// a cryptographic hash corresponding to the expected image.
///
/// A prover can provide a Receipt to an untrusting party to convince them that
/// the results contained within the Receipt (in the [Receipt::journal]) came
/// from running specific code. Conversely, a verifier can inspect a receipt to
/// confirm that its results must have been generated from the expected code,
/// even when this code was run by an untrusted source.
/// # Example
///
/// To create a [Receipt] attesting to the faithful execution of your code, run
/// one of the `prove` functions from a [crate::Prover].
///
/// ```rust
/// # #[cfg(feature = "prove")]
/// use risc0_zkvm::{default_prover, ExecutorEnv};
/// use risc0_zkvm_methods::FIB_ELF;
///
/// # #[cfg(not(feature = "cuda"))]
/// # #[cfg(feature = "prove")]
/// # {
/// let env = ExecutorEnv::builder().write_slice(&[20]).build().unwrap();
/// let receipt = default_prover().prove_elf(env, FIB_ELF).unwrap();
/// # }
/// ```
///
/// To confirm that a [Receipt] was honestly generated, use
/// [Receipt::verify] and supply the ImageID of the code that should
/// have been executed as a parameter. (See
/// [risc0_build](https://docs.rs/risc0-build/latest/risc0_build/) for more
/// information about how ImageIDs are generated.)
/// ```rust
/// use risc0_zkvm::Receipt;
/// # #[cfg(feature = "prove")]
/// # use risc0_zkvm::{default_prover, ExecutorEnv};
/// # use risc0_zkvm_methods::{FIB_ELF, FIB_ID};
///
/// # #[cfg(not(feature = "cuda"))]
/// # #[cfg(feature = "prove")]
/// # {
/// # let env = ExecutorEnv::builder().write_slice(&[20]).build().unwrap();
/// # let receipt = default_prover().prove_elf(env, FIB_ELF).unwrap();
/// receipt.verify(FIB_ID).unwrap();
/// # }
/// ```
///
/// The public outputs of the [Receipt] are contained in the
/// [Receipt::journal]. We provide serialization tools in the zkVM
/// [serde](crate::serde) module, which can be used to read data from the
/// journal as the same type it was written to the journal. If you prefer, you
/// can also directly access the [Receipt::journal] as a `Vec<u8>`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Receipt {
    /// The polymorphic [InnerReceipt].
    pub inner: InnerReceipt,

    /// The public commitment written by the guest.
    ///
    /// This data is cryptographically authenticated in
    /// [Receipt::verify].
    pub journal: Journal,
}

impl Receipt {
    /// Construct a new Receipt
    pub fn new(inner: InnerReceipt, journal: Vec<u8>) -> Self {
        Self {
            inner,
            journal: Journal::new(journal),
        }
    }

    /// Verify that this receipt proves a successful execution of the zkVM from
    /// the given `image_id`.
    ///
    /// Uses the zero-knowledge proof system to verify the seal, and decodes the
    /// proven [ReceiptMetadata]. This method additionally ensures that the
    /// guest exited with a successful status code (e.g. `Halted(0)` or
    /// `Paused(0)`), the image ID is as expected, and the journal
    /// has not been tampered with.
    pub fn verify(&self, image_id: impl Into<Digest>) -> Result<(), VerificationError> {
        self.verify_with_context(&VerifierContext::default(), image_id)
    }

    /// Verify that this receipt proves a successful execution of the zkVM from
    /// the given `image_id`.
    ///
    /// Uses the zero-knowledge proof system to verify the seal, and decodes the
    /// proven [ReceiptMetadata]. This method additionally ensures that the
    /// guest exited with a successful status code (e.g. `Halted(0)` or
    /// `Paused(0)`), the image ID is as expected, and the journal
    /// has not been tampered with.
    pub fn verify_with_context(
        &self,
        ctx: &VerifierContext,
        image_id: impl Into<Digest>,
    ) -> Result<(), VerificationError> {
        self.inner.verify_integrity_with_context(ctx)?;

        // NOTE: Post-state digest and input digest are unconstrained by this method.
        let metadata = self.inner.get_metadata()?;

        if metadata.pre.digest() != image_id.into() {
            return Err(VerificationError::ImageVerificationError);
        }

        // Check the exit code. This verification method requires execution to be
        // successful.
        let (ExitCode::Halted(0) | ExitCode::Paused(0)) = metadata.exit_code else {
            return Err(VerificationError::UnexpectedExitCode);
        };

        // Finally check the output hash in the decoded metadata against the expected
        // output.
        let expected_output = Output {
            journal: MaybePruned::Pruned(self.journal.digest()),
            // It is expected that there are no (unresolved) assumptions.
            assumptions: Assumptions(vec![]).into(),
        };

        if metadata.output.digest() != expected_output.digest() {
            let empty_output = metadata.output.is_none() && self.journal.bytes.is_empty();
            if !empty_output {
                tracing::debug!(
                    "journal: 0x{}, expected output digest: 0x{}, decoded output digest: 0x{}",
                    hex::encode(&self.journal.bytes),
                    hex::encode(expected_output.digest()),
                    hex::encode(metadata.output.digest()),
                );
                return Err(VerificationError::JournalDigestMismatch);
            }
            tracing::debug!("accepting zero digest for output of receipt with empty journal");
        }

        Ok(())
    }

    /// Verify the integrity of this receipt, ensuring the metadata and jounral
    /// are attested to by the seal.
    ///
    /// This does not verify the success of the guest execution. In
    /// particular, the guest could have exited with an error (e.g.
    /// `ExitCode::Halted(1)`) or faulted state. It also does not check the
    /// image ID, or otherwise constrain what guest was executed. After calling
    /// this method, the caller should check the [ReceiptMetadata] fields
    /// relevant to their application. If you need to verify a successful
    /// guest execution and access the journal, the `verify` function is
    /// recommended.
    pub fn verify_integrity_with_context(
        &self,
        ctx: &VerifierContext,
    ) -> Result<(), VerificationError> {
        self.inner.verify_integrity_with_context(ctx)?;

        // Check that self.journal is attested to by the inner receipt.
        let metadata = self.inner.get_metadata()?;

        let expected_output = metadata.exit_code.expects_output().then(|| Output {
            journal: MaybePruned::Pruned(self.journal.digest()),
            // TODO(#982): It would be reasonable for this method to allow integrity verification
            // for receipts that have a non-empty assumptions list, but it is not supported here
            // because we don't have a enough information to open the assumptions list unless we
            // require it be empty.
            assumptions: Assumptions(vec![]).into(),
        });

        if metadata.output.digest() != expected_output.digest() {
            let empty_output = metadata.output.is_none() && self.journal.bytes.is_empty();
            if !empty_output {
                tracing::debug!(
                    "journal: 0x{}, expected output digest: 0x{}, decoded output digest: 0x{}",
                    hex::encode(&self.journal.bytes),
                    hex::encode(expected_output.digest()),
                    hex::encode(metadata.output.digest()),
                );
                return Err(VerificationError::JournalDigestMismatch);
            }
            tracing::debug!("accepting zero digest for output of receipt with empty journal");
        }

        Ok(())
    }

    /// Extract the [ReceiptMetadata] from this receipt.
    pub fn get_metadata(&self) -> Result<ReceiptMetadata, VerificationError> {
        self.inner.get_metadata()
    }
}

/// A journal is a record of all public commitments for a given proof session.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Journal {
    /// The raw bytes of the journal.
    pub bytes: Vec<u8>,
}

impl Journal {
    /// Construct a new [Journal].
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Decode the journal bytes by using the risc0 deserializer.
    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, Error> {
        from_slice(&self.bytes)
    }
}

impl risc0_binfmt::Digestible for Journal {
    fn digest<S: Sha256>(&self) -> Digest {
        *S::hash_bytes(&self.bytes)
    }
}

impl AsRef<[u8]> for Journal {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// An inner receipt can take the form of a [CompositeReceipt] or a
/// [SuccinctReceipt].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum InnerReceipt {
    /// A non-succinct [CompositeReceipt].
    Composite(CompositeReceipt),

    /// The [SuccinctReceipt].
    Succinct(SuccinctReceipt),

    /// The [Groth16Receipt].
    Groth16(Groth16Receipt),

    /// A fake receipt for testing and development.
    ///
    /// This receipt is not valid and will fail verification unless the
    /// environment variable `RISC0_DEV_MODE` is set to `true`, in which case a
    /// pass-through 'verification' will be performed, but it *does not*
    /// represent any meaningful attestation of receipt's integrity.
    ///
    /// This type solely exists to improve development experience, for further
    /// information about development-only mode see our [dev-mode
    /// documentation](https://dev.risczero.com/zkvm/dev-mode).
    Fake {
        /// [ReceiptMetadata] for this fake receipt.
        metadata: ReceiptMetadata,
    },
}

impl InnerReceipt {
    /// Verify the integrity of this receipt, ensuring the metadata is attested
    /// to by the seal.
    pub fn verify_integrity_with_context(
        &self,
        ctx: &VerifierContext,
    ) -> Result<(), VerificationError> {
        match self {
            InnerReceipt::Composite(x) => x.verify_integrity_with_context(ctx),
            InnerReceipt::Groth16(x) => x.verify_integrity(),
            InnerReceipt::Succinct(x) => x.verify_integrity_with_context(ctx),
            InnerReceipt::Fake { .. } => {
                #[cfg(feature = "std")]
                if crate::is_dev_mode() {
                    return Ok(());
                }
                Err(VerificationError::InvalidProof)
            }
        }
    }

    /// Returns the [InnerReceipt::Composite] arm.
    pub fn composite(&self) -> Result<&CompositeReceipt, VerificationError> {
        if let InnerReceipt::Composite(x) = self {
            Ok(&x)
        } else {
            Err(VerificationError::ReceiptFormatError)
        }
    }

    /// Returns the [InnerReceipt::Groth16] arm.
    pub fn groth16(&self) -> Result<&Groth16Receipt, VerificationError> {
        if let InnerReceipt::Groth16(x) = self {
            Ok(&x)
        } else {
            Err(VerificationError::ReceiptFormatError)
        }
    }

    /// Returns the [InnerReceipt::Succinct] arm.
    pub fn succinct(&self) -> Result<&SuccinctReceipt, VerificationError> {
        if let InnerReceipt::Succinct(x) = self {
            Ok(x)
        } else {
            Err(VerificationError::ReceiptFormatError)
        }
    }

    /// Extract the [ReceiptMetadata] from this receipt.
    pub fn get_metadata(&self) -> Result<ReceiptMetadata, VerificationError> {
        match self {
            InnerReceipt::Composite(ref receipt) => receipt.get_metadata(),
            InnerReceipt::Groth16(ref groth16_receipt) => Ok(groth16_receipt.meta.clone()),
            InnerReceipt::Succinct(ref succinct_recipt) => Ok(succinct_recipt.meta.clone()),
            InnerReceipt::Fake { metadata } => Ok(metadata.clone()),
        }
    }
}

/// A receipt composed of a Groth16 over the BN_254 curve
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Groth16Receipt {
    /// A Groth16 proof of a zkVM execution with the associated metadata.
    pub seal: Vec<u8>,

    /// [ReceiptMetadata] containing information about the execution that this
    /// receipt proves.
    pub meta: ReceiptMetadata,
}

impl Groth16Receipt {
    /// Verify the integrity of this receipt, ensuring the metadata is attested
    /// to by the seal.
    pub fn verify_integrity(&self) -> Result<(), VerificationError> {
        Groth16Proof::from_seal(
            &Groth16Seal::from_vec(&self.seal).map_err(|_| VerificationError::InvalidProof)?,
            self.meta.digest().into(),
        )
        .map_err(|_| VerificationError::InvalidProof)?;

        // Everything passed
        Ok(())
    }
}

/// A receipt composed of one or more [SegmentReceipt] structs proving a single
/// execution with continuations, and zero or more [Receipt] stucts proving any
/// assumptions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct CompositeReceipt {
    /// Segment receipts forming the proof of a execution with continuations.
    pub segments: Vec<SegmentReceipt>,

    /// An ordered list of assumptions, either proven or unresolved, made within
    /// the continuation represented by the segment receipts. If any
    /// assumptions are unresolved, this receipt is only _conditionally_
    /// valid.
    // TODO(#982): Allow for unresolved assumptions in this list.
    pub assumptions: Vec<InnerReceipt>,

    /// Digest of journal included in the final output of the continuation. Will
    /// be `None` if the continuation has no output (e.g. it ended in
    /// `Fault`).
    // NOTE: This field is needed in order to open the assumptions digest from the output digest.
    pub journal_digest: Option<Digest>,
}

impl CompositeReceipt {
    /// Verify the integrity of this receipt, ensuring the metadata is attested
    /// to by the seal.
    pub fn verify_integrity_with_context(
        &self,
        ctx: &VerifierContext,
    ) -> Result<(), VerificationError> {
        // Verify the continuation, by verifying every segment receipt in order.
        let (final_receipt, receipts) = self
            .segments
            .as_slice()
            .split_last()
            .ok_or(VerificationError::ReceiptFormatError)?;

        // Verify each segment and its chaining to the next.
        let mut prev_image_id = None;
        for receipt in receipts {
            receipt.verify_integrity_with_context(ctx)?;
            let metadata = receipt.get_metadata()?;
            tracing::debug!("metadata: {metadata:#?}");
            if let Some(id) = prev_image_id {
                if id != metadata.pre.digest() {
                    return Err(VerificationError::ImageVerificationError);
                }
            }
            if metadata.exit_code != ExitCode::SystemSplit {
                return Err(VerificationError::UnexpectedExitCode);
            }
            if !metadata.output.is_none() {
                return Err(VerificationError::ReceiptFormatError);
            }
            prev_image_id = Some(metadata.post.digest());
        }

        // Verify the last receipt in the continuation.
        final_receipt.verify_integrity_with_context(ctx)?;
        let final_receipt_metadata = final_receipt.get_metadata()?;
        tracing::debug!("final: {final_receipt_metadata:#?}");
        if let Some(id) = prev_image_id {
            if id != final_receipt_metadata.pre.digest() {
                return Err(VerificationError::ImageVerificationError);
            }
        }

        // Verify all corroborating receipts attached to this composite receipt.
        for receipt in self.assumptions.iter() {
            tracing::debug!(
                "verifying assumption: {:?}",
                receipt.get_metadata()?.digest()
            );
            receipt.verify_integrity_with_context(ctx)?;
        }

        // Verify decoded output digest is consistent with the journal_digest and
        // assumptions.
        self.verify_output_consistency(&final_receipt_metadata)?;

        Ok(())
    }

    /// Returns the [ReceiptMetadata] for this [CompositeReceipt].
    pub fn get_metadata(&self) -> Result<ReceiptMetadata, VerificationError> {
        let first_metadata = self
            .segments
            .first()
            .ok_or(VerificationError::ReceiptFormatError)?
            .get_metadata()?;
        let last_metadata = self
            .segments
            .last()
            .ok_or(VerificationError::ReceiptFormatError)?
            .get_metadata()?;

        // After verifying the internally consistency of this receipt, we can use
        // self.assumptions and self.journal_digest in place of
        // last_metadata.output, which is equal.
        self.verify_output_consistency(&last_metadata)?;
        let output: Option<Output> = last_metadata
            .output
            .is_some()
            .then(|| {
                Ok(Output {
                    journal: MaybePruned::Pruned(
                        self.journal_digest
                            .ok_or(VerificationError::ReceiptFormatError)?,
                    ),
                    // TODO(#982): Adjust this if unresolved assumptions are allowed on
                    // CompositeReceipt.
                    // NOTE: Proven assumptions are not included in the CompositeReceipt metadata.
                    assumptions: Assumptions(vec![]).into(),
                })
            })
            .transpose()?;

        Ok(ReceiptMetadata {
            pre: first_metadata.pre,
            post: last_metadata.post,
            exit_code: last_metadata.exit_code,
            input: first_metadata.input,
            output: output.into(),
        })
    }

    /// Check that the output fields in the given receipt metadata are
    /// consistent with the exit code, and with the journal_digest and
    /// assumptions encoded on self.
    fn verify_output_consistency(
        &self,
        metadata: &ReceiptMetadata,
    ) -> Result<(), VerificationError> {
        tracing::debug!("checking output: exit_code = {:?}", metadata.exit_code);
        if metadata.exit_code.expects_output() && metadata.output.is_some() {
            let self_output = Output {
                journal: MaybePruned::Pruned(
                    self.journal_digest
                        .ok_or(VerificationError::ReceiptFormatError)?,
                ),
                assumptions: self.assumptions_metadata()?.into(),
            };

            // If these digests do not match, this receipt is internally inconsistent.
            if self_output.digest() != metadata.output.digest() {
                let empty_output = metadata.output.is_none()
                    && self
                        .journal_digest
                        .ok_or(VerificationError::ReceiptFormatError)?
                        == Vec::<u8>::new().digest();
                if !empty_output {
                    tracing::debug!(
                        "output digest does not match: expected {:?}; decoded {:?}",
                        &self_output,
                        &metadata.output
                    );
                    return Err(VerificationError::ReceiptFormatError);
                }
            }
        } else {
            // Ensure all output fields are empty. If not, this receipt is internally
            // inconsistent.
            if metadata.output.is_some() {
                tracing::debug!(
                    "unexpected non-empty metadata output: {:?}",
                    &metadata.output
                );
                return Err(VerificationError::ReceiptFormatError);
            }
            if !self.assumptions.is_empty() {
                tracing::debug!(
                    "unexpected non-empty composite receipt assumptions: {:?}",
                    &self.assumptions
                );
                return Err(VerificationError::ReceiptFormatError);
            }
            if self.journal_digest.is_some() {
                tracing::debug!(
                    "unexpected non-empty composite receipt journal_digest: {:?}",
                    &self.journal_digest
                );
                return Err(VerificationError::ReceiptFormatError);
            }
        }
        Ok(())
    }

    fn assumptions_metadata(&self) -> Result<Assumptions, VerificationError> {
        Ok(Assumptions(
            self.assumptions
                .iter()
                .map(|a| Ok(a.get_metadata()?.into()))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

/// A receipt attesting to the execution of a Segment.
///
/// A SegmentReceipt attests that a [crate::Segment] was executed in a manner
/// consistent with the [ReceiptMetadata] included in the receipt.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SegmentReceipt {
    /// The cryptographic data attesting to the validity of the code execution.
    ///
    /// This data is used by the ZKP Verifier (as called by
    /// [SegmentReceipt::verify_integrity_with_context]) to cryptographically
    /// prove that this Segment was faithfully executed. It is largely
    /// opaque cryptographic data, but contains a non-opaque metadata
    /// component, which can be conveniently accessed with
    /// [SegmentReceipt::get_metadata].
    pub seal: Vec<u32>,

    /// Segment index within the [Receipt]
    pub index: u32,

    /// Name of the hash function used to create this receipt.
    pub hashfn: String,
}

impl SegmentReceipt {
    /// Verify the integrity of this receipt, ensuring the metadata is attested
    /// to by the seal.
    pub fn verify_integrity_with_context(
        &self,
        ctx: &VerifierContext,
    ) -> Result<(), VerificationError> {
        use hex::FromHex;
        let check_code = |_, control_id: &Digest| -> Result<(), VerificationError> {
            POSEIDON_CONTROL_ID
                .into_iter()
                .chain(SHA256_CONTROL_ID)
                .chain(BLAKE2B_CONTROL_ID)
                .find(|x| Digest::from_hex(x).unwrap() == *control_id)
                .map(|_| ())
                .ok_or(VerificationError::ControlVerificationError)
        };
        let suite = ctx
            .suites
            .get(&self.hashfn)
            .ok_or(VerificationError::InvalidHashSuite)?;
        risc0_zkp::verify::verify(&super::CIRCUIT, suite, &self.seal, check_code)
    }

    /// Returns the [ReceiptMetadata] for this receipt.
    pub fn get_metadata(&self) -> Result<ReceiptMetadata, VerificationError> {
        let elems = bytemuck::cast_slice(&self.seal);
        decode_receipt_metadata_from_io(layout::OutBuffer(elems))
    }

    /// Return the seal for this receipt, as a vector of bytes.
    pub fn get_seal_bytes(&self) -> Vec<u8> {
        self.seal.iter().flat_map(|x| x.to_le_bytes()).collect()
    }
}

/// An assumption attached with a guest execution as a result of calling
/// `env::verify` or `env::verify_integrity`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Assumption {
    /// A [Receipt] for a proven assumption.
    Proven(Receipt),

    /// [ReceiptMetadata] digest for an assumption that is not directly proven
    /// to be true.
    ///
    /// Proving an execution with an unresolved assumption will result in a
    /// conditional receipt. In order for the verifier to accept a
    /// conditional receipt, they must be given a [Receipt] proving the
    /// assumption, or explicitly accept the assumption without proof.
    Unresolved(MaybePruned<ReceiptMetadata>),
}

impl Assumption {
    /// Returns the [ReceiptMetadata] for this [Assumption].
    pub fn get_metadata(&self) -> Result<MaybePruned<ReceiptMetadata>, VerificationError> {
        match self {
            Self::Proven(receipt) => Ok(receipt.get_metadata()?.into()),
            Self::Unresolved(metadata) => Ok(metadata.clone()),
        }
    }

    #[cfg(feature = "prove")]
    pub(crate) fn as_receipt(&self) -> Result<&Receipt> {
        match self {
            Self::Proven(receipt) => Ok(receipt),
            Self::Unresolved(_) => Err(anyhow::anyhow!(
                "no receipt available for unresolved assumption"
            )),
        }
    }
}

impl From<Receipt> for Assumption {
    /// Create a proven assumption from a [Receipt].
    fn from(receipt: Receipt) -> Self {
        Self::Proven(receipt)
    }
}

impl From<MaybePruned<ReceiptMetadata>> for Assumption {
    /// Create an unresolved assumption from a [MaybePruned] [ReceiptMetadata].
    fn from(metadata: MaybePruned<ReceiptMetadata>) -> Self {
        Self::Unresolved(metadata)
    }
}

impl From<ReceiptMetadata> for Assumption {
    /// Create an unresolved assumption from a [ReceiptMetadata].
    fn from(metadata: ReceiptMetadata) -> Self {
        Self::Unresolved(metadata.into())
    }
}

/// Context available to the verification process.
pub struct VerifierContext {
    /// A registry of hash functions to be used by the verification process.
    pub suites: BTreeMap<String, HashSuite<BabyBear>>,
}

fn decode_system_state_from_io(
    io: layout::OutBuffer,
    sys_state: &layout::SystemState,
) -> Result<SystemState, VerificationError> {
    let bytes: Vec<u8> = io
        .tree(sys_state.image_id)
        .get_bytes()
        .or(Err(VerificationError::ReceiptFormatError))?;
    let pc = io
        .tree(sys_state.pc)
        .get_u32()
        .or(Err(VerificationError::ReceiptFormatError))?;
    let merkle_root = Digest::try_from(bytes).or(Err(VerificationError::ReceiptFormatError))?;
    Ok(SystemState { pc, merkle_root })
}

fn decode_receipt_metadata_from_io(
    io: layout::OutBuffer,
) -> Result<ReceiptMetadata, VerificationError> {
    let body = layout::LAYOUT.mux.body;
    let pre = decode_system_state_from_io(io, body.global.pre)?;
    let mut post = decode_system_state_from_io(io, body.global.post)?;
    // In order to avoid extra logic in the rv32im circuit to perform arithmetic on
    // the PC with carry, the PC is always recorded as the current PC + 4. Thus
    // we need to adjust the decoded PC for the post SystemState.
    post.pc = post
        .pc
        .checked_sub(WORD_SIZE as u32)
        .ok_or(VerificationError::ReceiptFormatError)?;

    let input_bytes: Vec<u8> = io
        .tree(body.global.input)
        .get_bytes()
        .or(Err(VerificationError::ReceiptFormatError))?;
    let input = Digest::try_from(input_bytes).or(Err(VerificationError::ReceiptFormatError))?;

    let output_bytes: Vec<u8> = io
        .tree(body.global.output)
        .get_bytes()
        .or(Err(VerificationError::ReceiptFormatError))?;
    let output = Digest::try_from(output_bytes).or(Err(VerificationError::ReceiptFormatError))?;

    let sys_exit = io.get_u64(body.global.sys_exit_code) as u32;
    let user_exit = io.get_u64(body.global.user_exit_code) as u32;
    let exit_code =
        ExitCode::from_pair(sys_exit, user_exit).or(Err(VerificationError::ReceiptFormatError))?;

    Ok(ReceiptMetadata {
        pre: pre.into(),
        post: post.into(),
        exit_code,
        input,
        output: MaybePruned::Pruned(output),
    })
}

impl Default for VerifierContext {
    fn default() -> Self {
        Self {
            suites: BTreeMap::from([
                ("blake2b".into(), Blake2bCpuHashSuite::new_suite()),
                ("poseidon".into(), PoseidonHashSuite::new_suite()),
                ("sha-256".into(), Sha256HashSuite::new_suite()),
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use hex::FromHex;

    use super::*;
    use crate::{receipt_metadata::MaybePruned, ExitCode::Halted};

    const IMAGE_ID: [u32; 8] = [
        3877313773, 4166950669, 1851257837, 1474316178, 3714943358, 2342301681, 2883381307,
        234838297,
    ];

    const RISC0_GROTH16_SEAL: &str = r#"
    {
        "a":[[26,63,155,211,133,192,185,234,51,172,152,49,113,248,13,45,155,140,75,98,171,225,72,44,133,246,88,199,37,103,28,56],
            [22,46,102,79,175,35,89,153,55,78,200,143,26,196,209,62,247,200,136,247,101,65,101,157,59,33,20,91,191,43,246,84]],
        "b":[[[20,0,192,47,211,1,254,55,118,229,52,232,89,161,51,100,224,242,246,5,106,190,188,113,187,230,100,7,255,70,192,153],
            [9,206,194,232,36,249,103,125,57,1,189,209,245,133,230,79,219,98,176,253,221,160,20,78,189,142,46,52,171,1,162,203]],
            [[32,70,252,38,183,118,240,156,230,16,28,10,122,111,184,65,239,158,193,102,94,156,5,56,24,236,174,103,160,172,89,109],
            [4,151,70,37,225,160,65,7,230,206,172,247,41,192,58,149,244,201,31,171,7,137,232,107,29,143,227,49,96,29,198,35]]],
        "c":[[20,112,216,216,113,186,224,139,117,16,31,102,238,29,76,150,58,216,254,75,147,172,89,147,216,138,41,130,172,106,148,8],
            [23,250,144,90,188,98,158,62,214,76,108,236,158,31,125,183,20,200,101,254,212,15,32,174,120,230,219,15,71,206,189,55]]
    }
    "#;

    #[ignore]
    #[test]
    fn test_groth16_receipt() {
        let seal: Groth16Seal = serde_json::from_str(RISC0_GROTH16_SEAL).unwrap();
        let journal: Vec<u8> = vec![135, 1, 0, 0, 0, 0, 0, 0];
        let merkle_root =
            Digest::from_hex("5bcc4b8e50095f5a5e28f324170ef29d25ee52d966ad996159644c63f3b11eba")
                .unwrap();
        let meta: ReceiptMetadata = ReceiptMetadata {
            pre: MaybePruned::Value(SystemState {
                pc: 2103560,
                merkle_root,
            }),
            post: MaybePruned::Value(SystemState {
                pc: 2111560,
                merkle_root,
            }),
            exit_code: Halted(0),
            input: Digest::default(),
            output: MaybePruned::Value(Some(Output {
                journal: MaybePruned::Value(journal.clone()),
                assumptions: MaybePruned::Value(Assumptions(vec![])),
            })),
        };
        let receipt = Receipt::new(
            InnerReceipt::Groth16(Groth16Receipt {
                seal: seal.to_vec(),
                meta,
            }),
            journal,
        );
        receipt.verify(IMAGE_ID).unwrap();
    }
}
