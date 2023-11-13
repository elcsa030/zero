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

use risc0_zkvm::{
    default_prover,
    sha::{Digest, DIGEST_WORDS},
    ExecutorEnv, ExecutorImpl, MemoryImage, ProverOpts, Receipt, Session, VerifierContext,
};
use sha3::{Digest as _, Keccak256};

use crate::{get_cycles, get_image, Benchmark, BenchmarkAverage};

pub struct Job {
    pub spec: u32,
    pub image: MemoryImage,
    pub session: Option<Session>,
}

pub fn new_jobs() -> Vec<<Job as Benchmark>::Spec> {
    vec![1, 10, 100]
}

const METHOD_ID: [u32; DIGEST_WORDS] = risc0_benchmark_methods::ITER_KECCAK_ID;
const METHOD_PATH: &'static str = risc0_benchmark_methods::ITER_KECCAK_PATH;

impl Benchmark for Job {
    const NAME: &'static str = "iter_keccak";
    type Spec = u32;
    type ComputeOut = Digest;
    type ProofType = Receipt;

    fn job_size(spec: &Self::Spec) -> u32 {
        *spec
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

    fn new(spec: Self::Spec) -> Self {
        let image = get_image(METHOD_PATH);
        let session = None;

        Job {
            spec,
            image,
            session,
        }
    }

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }

    fn host_compute(&mut self) -> Option<Self::ComputeOut> {
        let mut data = Vec::from([0u8; 32]);

        for _i in 0..self.spec {
            let mut hasher = Keccak256::new();
            hasher.update(&data);
            data = hasher.finalize().to_vec();
        }

        Some(Digest::try_from(data.as_slice()).unwrap())
    }

    fn exec_compute(&mut self) -> (u32, u32, Duration) {
        let guest_input = (self.spec, vec![0u8; 32]);
        let env = ExecutorEnv::builder()
            .write(&guest_input)
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
        let result = receipt.journal.decode().unwrap();
        (result, receipt)
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

impl BenchmarkAverage for Job {
    const NAME: &'static str = "keccak";
    type Spec = u32;

    fn job_size(spec: &Self::Spec) -> u32 {
        *spec
    }

    fn new(spec: Self::Spec) -> Self {
        let image = get_image(METHOD_PATH);
        let session = None;

        Job {
            spec,
            image,
            session,
        }
    }

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }

    fn guest_compute(&mut self) -> Duration {
        let guest_input = (self.spec, vec![0u8; 32]);
        let env = ExecutorEnv::builder()
            .write(&guest_input)
            .unwrap()
            .build()
            .unwrap();

        let prover = default_prover();
        let start = Instant::now();
        prover
            .prove(
                env,
                &VerifierContext::default(),
                &ProverOpts::default(),
                self.image.clone(),
            )
            .expect("receipt");
        let elapsed = start.elapsed();
        elapsed
    }
}
