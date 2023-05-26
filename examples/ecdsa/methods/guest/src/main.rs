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

use k256::{
    ecdsa::{signature::Verifier, Signature, VerifyingKey},
    EncodedPoint,
};
use risc0_zkvm::guest::env;

// Example of using the risc0_zkvm::sha module to hash data.
fn main() {
    // Decode the verifying key, message, and signature from the inputs.
    let (encoded_verifying_key, message, signature): (EncodedPoint, Vec<u8>, Signature) =
        env::read();
    let verifying_key = VerifyingKey::from_encoded_point(&encoded_verifying_key).unwrap();

    let start = env::get_cycle_count();

    // Verify the signature, panicking if verification fails.
    assert!(verifying_key.verify(&message, &signature).is_ok());

    let end = env::get_cycle_count();
    println!("ECDSA signature verification took {} cycles", end - start);

    // Commit to the journal the verifying key and messge that was signed.
    env::commit(&(encoded_verifying_key, message));
}
