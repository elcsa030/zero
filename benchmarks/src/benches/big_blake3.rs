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

use std::time::{Duration, Instant};

use rand::{rngs::StdRng, RngCore, SeedableRng};
use risc0_zkvm::{
    sha::{Digest, DIGEST_WORDS},
    ExecutorEnv, ExecutorImpl, MemoryImage, Receipt, Session,
};

use crate::{get_cycles, get_image, Benchmark};

pub struct Job {
    pub guest_input: Vec<u8>,
    pub image: MemoryImage,
    pub session: Option<Session>,
}

pub fn new_jobs() -> Vec<<Job as Benchmark>::Spec> {
    let mut rand = StdRng::seed_from_u64(1337);
    let mut jobs = Vec::new();
    for job_size in [1024, 2048, 4096, 8192] {
        let mut guest_input = vec![0; job_size];
        for i in 0..guest_input.len() {
            guest_input[i] = rand.next_u32() as u8;
        }

        jobs.push(guest_input);
    }
    jobs
}

const METHOD_ID: [u32; DIGEST_WORDS] = risc0_benchmark_methods::BIG_BLAKE3_ID;
const METHOD_PATH: &'static str = risc0_benchmark_methods::BIG_BLAKE3_PATH;

impl Benchmark for Job {
    const NAME: &'static str = "big_blake2b";
    type Spec = Vec<u8>;
    type ComputeOut = Digest;
    type ProofType = Receipt;

    fn job_size(spec: &Self::Spec) -> u32 {
        spec.len() as u32
    }

    fn output_size_bytes(_output: &Self::ComputeOut, proof: &Self::ProofType) -> u32 {
        proof.journal.bytes.len() as u32
    }

    fn proof_size_bytes(proof: &Self::ProofType) -> u32 {
        (proof
            .inner
            .composite()
            .unwrap()
            .segments
            .iter()
            .fold(0, |acc, segment| acc + segment.get_seal_bytes().len())) as u32
    }

    fn new(guest_input: Self::Spec) -> Self {
        let image = get_image(METHOD_PATH);
        let session = None;

        Job {
            guest_input,
            image,
            session,
        }
    }

    fn spec(&self) -> &Self::Spec {
        &self.guest_input
    }

    fn exec_compute(&mut self) -> (u32, u32, Duration) {
        let env = ExecutorEnv::builder()
            .write(&self.guest_input)
            .unwrap()
            .build()
            .unwrap();
        let mut exec = ExecutorImpl::new(env, self.image.clone()).unwrap();
        let start = Instant::now();
        let session = exec.run().unwrap();
        let elapsed = start.elapsed();
        let segments = session.resolve().unwrap();
        let (exec_cycles, prove_cycles) = get_cycles(segments);
        self.session = Some(session);
        (prove_cycles as u32, exec_cycles as u32, elapsed)
    }

    fn guest_compute(&mut self) -> (Self::ComputeOut, Self::ProofType) {
        let receipt = self.session.as_ref().unwrap().prove().expect("receipt");
        let guest_output: Digest = receipt.journal.decode().unwrap();
        (guest_output, receipt)
    }

    fn verify_proof(&self, _output: &Self::ComputeOut, proof: &Self::ProofType) -> bool {
        let result = proof.verify(METHOD_ID);

        match result {
            Ok(_) => true,
            Err(err) => {
                println!("{}", err);
                false
            }
        }
    }
}
