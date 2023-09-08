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

use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use risc0_benchmark::Benchmark;
use risc0_zkvm::{
    serde::to_vec, sha::DIGEST_WORDS, Executor, ExecutorEnv, ExitCode, MemoryImage, Program,
    Receipt, Session, MEM_SIZE, PAGE_SIZE,
};
use std::time::{Duration, Instant};

pub struct Job<'a> {
    pub spec: u32,
    pub env: ExecutorEnv<'a>,
    pub image: MemoryImage,
    pub session: Session,
}

pub fn new_jobs() -> Vec<<Job<'static> as Benchmark>::Spec> {
    vec![1, 10, 100]
}

const METHOD_ID: [u32; DIGEST_WORDS] = risc0_benchmark_methods::ITER_BLAKE2B_ID;
const METHOD_PATH: &'static str = risc0_benchmark_methods::ITER_BLAKE2B_PATH;

impl Benchmark for Job<'_> {
    const NAME: &'static str = "iter_blake2b";
    type Spec = u32;
    type ComputeOut = risc0_zkvm::sha::Digest;
    type ProofType = Receipt;

    fn job_size(spec: &Self::Spec) -> u32 {
        *spec
    }

    fn output_size_bytes(_output: &Self::ComputeOut, proof: &Self::ProofType) -> u32 {
        (proof.journal.len()) as u32
    }

    fn proof_size_bytes(proof: &Self::ProofType) -> u32 {
        (proof
            .inner
            .flat()
            .unwrap()
            .iter()
            .fold(0, |acc, segment| acc + segment.get_seal_bytes().len())) as u32
    }

    fn new(spec: Self::Spec) -> Self {
        let image = std::fs::read(METHOD_PATH).expect("image");

        let mut guest_input = Vec::from([0u8; 36]);
        guest_input[0] = spec as u8;
        guest_input[1] = (spec >> 8) as u8;
        guest_input[2] = (spec >> 16) as u8;
        guest_input[3] = (spec >> 24) as u8;

        let env = ExecutorEnv::builder()
            .add_input(&to_vec(&guest_input).unwrap())
            .build()
            .unwrap();

        let program = Program::load_elf(&image, MEM_SIZE as u32).unwrap();
        let image = MemoryImage::new(&program, PAGE_SIZE as u32).unwrap();
        let session = Session::new(vec![], vec![], ExitCode::Halted(0));

        Job {
            spec,
            env,
            image,
            session,
        }
    }

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }

    fn host_compute(&mut self) -> Option<Self::ComputeOut> {
        let mut data = [0u8; 32];

        for _i in 0..self.spec {
            let mut hasher = Blake2bVar::new(32).expect("Initializing Blake2bVar failed");
            hasher.update(&data);
            hasher
                .finalize_variable(&mut data)
                .expect("Finalizing Blake2bVar failed");
        }

        Some(risc0_zkvm::sha::Digest::try_from(data).unwrap())
    }

    fn exec_compute(&mut self) -> (u32, Duration) {
        let mut exec = Executor::new(self.env.clone(), self.image.clone()).unwrap();
        let start = Instant::now();
        self.session = exec.run().unwrap();
        let elapsed = start.elapsed();
        let mut cycles = 0usize;
        let segments = self.session.resolve().unwrap();
        for segment in segments {
            cycles += segment.insn_cycles
        }
        (cycles as u32, elapsed)
    }

    fn guest_compute(&mut self) -> (Self::ComputeOut, Self::ProofType) {
        let receipt = self.session.prove().expect("receipt");

        let result = risc0_zkvm::sha::Digest::try_from(receipt.journal.clone())
            .unwrap()
            .try_into()
            .unwrap();
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
