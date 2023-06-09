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

use clap::{Arg, Command};
use risc0_zkvm::{
    serde::{from_slice, to_vec},
    sha::Digest,
    Executor, ExecutorEnv, SessionReceipt,
};
use sha_methods::{HASH_ELF, HASH_ID, HASH_RUST_CRYPTO_ELF};

/// Hash the given bytes, returning the digest and a [SessionReceipt] that can
/// be used to verify that the that the hash was computed correctly (i.e. that
/// the Prover knows a preimage for the given SHA-256 hash)
///
/// Select which method to use with `use_rust_crypto`.
/// HASH_ELF uses the risc0_zkvm::sha interface for hashing.
/// HASH_RUST_CRYPTO_ELF uses RustCrypto's [sha2] crate, patched to use the RISC
/// Zero accelerator. See `src/methods/guest/Cargo.toml` for the patch
/// definition, which can be used to enable SHA-256 accelerrator support
/// everywhere the [sha2] crate is used.
fn provably_hash(input: &str, use_rust_crypto: bool) -> (Digest, Box<dyn SessionReceipt>) {
    let env = ExecutorEnv::builder()
        .add_input(&to_vec(input).unwrap())
        .build();

    let elf = if use_rust_crypto {
        HASH_RUST_CRYPTO_ELF
    } else {
        HASH_ELF
    };

    let mut exec = Executor::from_elf(env, elf).unwrap();
    let session = exec.run().unwrap();
    let receipt = session.prove().unwrap();

    let digest = from_slice::<Vec<u8>, _>(&receipt.get_journal())
        .unwrap()
        .try_into()
        .unwrap();
    (digest, receipt)
}

fn main() {
    // Parse command line
    let matches = Command::new("hash")
        .arg(Arg::new("message").default_value(""))
        .get_matches();
    let message = matches.get_one::<String>("message").unwrap();

    // Prove hash the message.
    let (digest, receipt) = provably_hash(message, false);

    // Verify the receipt, ensuring the prover knows a valid SHA-256 preimage.
    receipt
        .verify(HASH_ID.into())
        .expect("receipt verification failed");

    println!("I provably know data whose SHA-256 hash is {}", digest);
}

#[cfg(test)]
mod tests {
    use sha_methods::{HASH_ID, HASH_RUST_CRYPTO_ID};

    #[test]
    fn hash_abc() {
        let (digest, receipt) = super::provably_hash("abc", false);
        receipt.verify(HASH_ID).unwrap();
        assert_eq!(
            hex::encode(digest.as_bytes()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "We expect to match the reference SHA-256 hash of the standard test value 'abc'"
        );
    }

    #[test]
    fn hash_abc_rust_crypto() {
        let (digest, receipt) = super::provably_hash("abc", true);
        receipt.verify(HASH_RUST_CRYPTO_ID).unwrap();
        assert_eq!(
            hex::encode(digest.as_bytes()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "We expect to match the reference SHA-256 hash of the standard test value 'abc'"
        );
    }
}
