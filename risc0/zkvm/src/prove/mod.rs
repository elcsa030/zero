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

//! Run the zkVM guest and prove its results

mod exec;
pub mod io;
pub(crate) mod loader;
mod plonk;
#[cfg(feature = "profiler")]
pub mod profiler;

use std::{cmp::min, collections::HashMap, fmt::Debug, io::Write, rc::Rc};

use anyhow::{bail, Result};
use io::{RawIoHandler, SliceIoHandler};
use risc0_circuit_rv32im::{REGISTER_GROUP_ACCUM, REGISTER_GROUP_CODE, REGISTER_GROUP_DATA};
use risc0_core::field::baby_bear::{BabyBear, BabyBearElem, BabyBearExtElem};
use risc0_zkp::{
    adapter::{PolyExt, TapsProvider},
    core::sha::Digest,
    hal::{EvalCheck, Hal},
    prove::adapter::ProveAdapter,
};
use risc0_zkvm_platform::{
    io::{
        SENDRECV_CHANNEL_COMPUTE_POLY, SENDRECV_CHANNEL_INITIAL_INPUT, SENDRECV_CHANNEL_JOURNAL,
        SENDRECV_CHANNEL_STDERR, SENDRECV_CHANNEL_STDOUT,
    },
    memory::MEM_SIZE,
    WORD_SIZE,
};

use crate::binfmt::elf::Program;
use crate::{
    receipt::{insecure_skip_seal, Receipt},
    CIRCUIT,
};

/// Options available to modify the prover's behavior.
pub struct ProverOpts<'a> {
    pub(crate) skip_seal: bool,

    pub(crate) skip_verify: bool,

    pub(crate) io_handlers: HashMap<u32, Box<dyn RawIoHandler + 'a>>,

    pub(crate) trace_callback: Option<Box<dyn FnMut(TraceEvent) -> Result<()> + 'a>>,
}

impl<'a> ProverOpts<'a> {
    /// If true, skip generating the seal in receipt.  This should
    /// only be used for testing.  In this case, performace will be
    /// much better but we will not be able to cryptographically
    /// verify the execution.
    pub fn with_skip_seal(self, skip_seal: bool) -> Self {
        Self { skip_seal, ..self }
    }

    /// If true, don't verify the seal after creating it.  This
    /// is useful if you wish to use a non-standard verifier for
    /// example.
    pub fn with_skip_verify(self, skip_verify: bool) -> Self {
        Self {
            skip_verify,
            ..self
        }
    }

    /// Add a handler for sendrecv ports which is just a callback.
    /// The guest can call these callbacks by invoking
    /// `risc0_zkvm::guest::env::send_recv_slice`.
    pub fn with_sendrecv_callback(
        self,
        channel_id: u32,
        f: impl Fn(&[u8]) -> Vec<u8> + 'a,
    ) -> Self {
        self.with_raw_io_handler(channel_id, io::handler_from_fn(f))
    }

    /// Add a handler for sendrecv ports, indexed by channel numbers.
    /// The guest can call these callbacks by invoking
    /// `risc0_zkvm::guest::env::send_recv_slice`
    pub fn with_slice_io_handler(self, channel_id: u32, handler: impl SliceIoHandler + 'a) -> Self {
        self.with_raw_io_handler(channel_id, io::handler_from_slice_handler(handler))
    }

    /// Add a handler for sendrecv channels that handles its own
    /// allocation and sizing.  The guets can call these callbacks by
    /// invoking `risc0_zkvm_guest::env::send_recv_raw'.
    pub fn with_raw_io_handler(mut self, channel_id: u32, handler: impl RawIoHandler + 'a) -> Self {
        self.io_handlers.insert(channel_id, Box::new(handler));
        self
    }

    /// Add a callback handler for raw trace messages.
    pub fn with_trace_callback(
        mut self,
        callback: impl FnMut(TraceEvent) -> Result<()> + 'a,
    ) -> Self {
        assert!(!self.trace_callback.is_some(), "Duplicate trace callback");
        self.trace_callback = Some(Box::new(callback));
        self
    }
}

impl<'a> Default for ProverOpts<'a> {
    fn default() -> ProverOpts<'a> {
        ProverOpts {
            skip_seal: false,
            skip_verify: false,
            io_handlers: HashMap::new(),
            trace_callback: None,
        }
    }
}

/// Manages communication with and execution of a zkVM [Program]
pub struct Prover<'a> {
    elf: Program,
    inner: ProverImpl<'a>,
    image_id: Digest,
    pub cycles: usize,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "cuda")] {
        use risc0_circuit_rv32im::cuda::CudaEvalCheck;
        use risc0_zkp::hal::cuda::CudaHal;

        pub fn default_hal() -> (Rc<CudaHal>, CudaEvalCheck) {
            let hal = Rc::new(CudaHal::new());
            let eval = CudaEvalCheck::new(hal.clone());
            (hal, eval)
        }
    } else if #[cfg(feature = "metal")] {
        use risc0_circuit_rv32im::metal::MetalEvalCheckSha256;
        use risc0_zkp::hal::metal::MetalHalSha256;

        pub fn default_hal() -> (Rc<MetalHalSha256>, MetalEvalCheckSha256) {
            let hal = Rc::new(MetalHalSha256::new());
            let eval = MetalEvalCheckSha256::new(hal.clone());
            (hal, eval)
        }
    } else {
        use risc0_circuit_rv32im::{CircuitImpl, cpu::CpuEvalCheck};
        use risc0_zkp::hal::cpu::BabyBearSha256CpuHal;

        pub fn default_hal() -> (Rc<BabyBearSha256CpuHal>, CpuEvalCheck<'static, CircuitImpl>) {
            let hal = Rc::new(BabyBearSha256CpuHal::new());
            let eval = CpuEvalCheck::new(&CIRCUIT);
            (hal, eval)
        }
    }
}

impl<'a> Prover<'a> {
    pub fn new<D>(elf: &[u8], image_id: D) -> Result<Self>
    where
        Digest: From<D>,
    {
        Self::new_with_opts(elf, image_id, ProverOpts::default())
    }

    pub fn new_with_opts<D>(elf: &[u8], image_id: D, opts: ProverOpts<'a>) -> Result<Self>
    where
        Digest: From<D>,
    {
        Ok(Prover {
            elf: Program::load_elf(&elf, MEM_SIZE as u32)?,
            inner: ProverImpl::new(opts),
            image_id: image_id.into(),
            cycles: 0,
        })
    }

    pub fn add_input_u8_slice(&mut self, slice: &[u8]) {
        self.inner.input.extend_from_slice(slice);
    }

    pub fn add_input_u32_slice(&mut self, slice: &[u32]) {
        self.inner
            .input
            .extend_from_slice(bytemuck::cast_slice(slice));
    }

    pub fn get_output_u8_slice(&self) -> &[u8] {
        &self.inner.output
    }

    pub fn get_output_u32_vec(&self) -> Result<Vec<u32>> {
        if self.inner.output.len() % WORD_SIZE != 0 {
            bail!("Private output must be word-aligned");
        }
        Ok(self
            .inner
            .output
            .chunks_exact(WORD_SIZE)
            .map(|x| u32::from_ne_bytes(x.try_into().unwrap()))
            .collect())
    }

    #[tracing::instrument(skip_all)]
    pub fn run(&mut self) -> Result<Receipt> {
        let (hal, eval) = default_hal();
        cfg_if::cfg_if! {
            if #[cfg(feature = "dual")] {
                let cpu_hal = risc0_zkp::hal::cpu::BabyBearSha256CpuHal::new();
                let cpu_eval = risc0_circuit_rv32im::cpu::CpuEvalCheck::new(&CIRCUIT);
                let hal = risc0_zkp::hal::dual::DualHal::new(hal.as_ref(), &cpu_hal);
                let eval = risc0_zkp::hal::dual::DualEvalCheck::new(eval, &cpu_eval);
                self.run_with_hal(&hal, &eval)
            } else {
                self.run_with_hal(hal.as_ref(), &eval)
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn run_with_hal<H, E>(&mut self, hal: &H, eval: &E) -> Result<Receipt>
    where
        H: Hal<Field = BabyBear, Elem = BabyBearElem, ExtElem = BabyBearExtElem>,
        E: EvalCheck<H>,
    {
        let skip_seal = self.inner.opts.skip_seal || insecure_skip_seal();

        let mut executor = exec::RV32Executor::new(&CIRCUIT, &self.elf, &mut self.inner);
        self.cycles = executor.run()?;

        let mut adapter = ProveAdapter::new(&mut executor.executor);
        let mut prover = risc0_zkp::prove::Prover::new(hal, CIRCUIT.get_taps());

        adapter.execute(prover.iop());

        let seal = if skip_seal {
            Vec::new()
        } else {
            prover.set_po2(adapter.po2() as usize);

            prover.commit_group(
                REGISTER_GROUP_CODE,
                hal.copy_from_elem("code", &adapter.get_code().as_slice()),
            );
            prover.commit_group(
                REGISTER_GROUP_DATA,
                hal.copy_from_elem("data", &adapter.get_data().as_slice()),
            );
            adapter.accumulate(prover.iop());
            prover.commit_group(
                REGISTER_GROUP_ACCUM,
                hal.copy_from_elem("accum", &adapter.get_accum().as_slice()),
            );

            let mix = hal.copy_from_elem("mix", &adapter.get_mix().as_slice());
            let out = hal.copy_from_elem("out", &adapter.get_io().as_slice());

            prover.finalize(&[&mix, &out], eval)
        };

        // Attach the full version of the output journal & construct receipt object
        let receipt = Receipt {
            journal: self.inner.journal.clone(),
            seal,
        };

        if !skip_seal && !self.inner.opts.skip_verify {
            // Verify receipt to make sure it works
            receipt.verify(&self.image_id)?;
        }

        Ok(receipt)
    }
}

struct ProverImpl<'a> {
    pub input: Vec<u8>,
    pub output: Vec<u8>,
    pub journal: Vec<u8>,
    pub opts: ProverOpts<'a>,
    pub compute_poly_data: Vec<Vec<u32>>,
}

impl<'a> ProverImpl<'a> {
    fn new(opts: ProverOpts<'a>) -> Self {
        Self {
            input: Vec::new(),
            output: Vec::new(),
            journal: Vec::new(),
            opts,
            compute_poly_data: Vec::new(),
        }
    }
}

impl<'a> exec::HostHandler for ProverImpl<'a> {
    fn on_txrx(
        &mut self,
        channel: u32,
        from_guest_buf: &[u8],
        from_host_buf: &mut [u32],
    ) -> Result<(u32, u32)> {
        if let Some(cb) = self.opts.io_handlers.get(&channel) {
            return Ok(cb.handle_raw_io(from_guest_buf, from_host_buf));
        }
        match channel {
            SENDRECV_CHANNEL_INITIAL_INPUT => {
                log::debug!("SENDRECV_CHANNEL_INITIAL_INPUT: {}", from_guest_buf.len());
                let copy_bytes = min(from_host_buf.len() * WORD_SIZE, self.input.len());
                bytemuck::cast_slice_mut(from_host_buf)[..copy_bytes]
                    .clone_from_slice(&self.input[..copy_bytes]);
                Ok((self.input.len() as u32, 0))
            }
            SENDRECV_CHANNEL_STDOUT => {
                log::debug!("SENDRECV_CHANNEL_STDOUT: {}", from_guest_buf.len());
                self.output.extend(from_guest_buf);
                Ok((0, 0))
            }
            SENDRECV_CHANNEL_STDERR => {
                log::debug!("SENDRECV_CHANNEL_STDERR: {}", from_guest_buf.len());
                std::io::stderr().lock().write_all(from_guest_buf).unwrap();
                Ok((0, 0))
            }
            // TODO: Convert this to FFPU at some point so we can get secure verifies in the guest
            SENDRECV_CHANNEL_COMPUTE_POLY => {
                log::debug!("SENDRECV_CHANNEL_COMPUTE_POLY: {}", from_guest_buf.len());
                assert!(from_guest_buf.len() % WORD_SIZE == 0);
                let nwords = from_guest_buf.len() / WORD_SIZE;
                let mut data: Vec<u32> = Vec::new();
                data.resize(nwords, 0);
                bytemuck::cast_slice_mut(&mut data).clone_from_slice(from_guest_buf);
                self.compute_poly_data.push(data);

                if !from_host_buf.is_empty() {
                    assert_eq!(self.compute_poly_data.len(), 4);
                    let eval_u = bytemuck::cast_slice(self.compute_poly_data[0].as_slice());
                    let poly_mix = bytemuck::cast_slice(self.compute_poly_data[1].as_slice());
                    let out = bytemuck::cast_slice(&self.compute_poly_data[2].as_slice());
                    let mix = bytemuck::cast_slice(&self.compute_poly_data[3].as_slice());

                    let result = CIRCUIT.poly_ext(&poly_mix[0], &eval_u, &[&out, &mix]);
                    from_host_buf.clone_from_slice(bytemuck::cast_slice(&[result.tot]));

                    self.compute_poly_data.clear()
                }
                Ok((0, 0))
            }
            SENDRECV_CHANNEL_JOURNAL => {
                log::debug!("SENDRECV_CHANNEL_JOURNAL: {}", from_guest_buf.len());
                self.journal.extend_from_slice(from_guest_buf);
                Ok((0, 0))
            }
            _ => bail!("Unknown channel: {channel}"),
        }
    }

    fn is_trace_enabled(&self) -> bool {
        self.opts.trace_callback.is_some()
    }

    fn on_trace(&mut self, event: TraceEvent) -> Result<()> {
        if let Some(ref mut cb) = self.opts.trace_callback {
            cb(event)
        } else {
            Ok(())
        }
    }

    fn on_panic(&mut self, msg: &str) -> Result<()> {
        bail!("{}", msg)
    }
}

fn split_word8(value: u32) -> (BabyBearElem, BabyBearElem, BabyBearElem, BabyBearElem) {
    (
        BabyBearElem::new(value & 0xff),
        BabyBearElem::new(value >> 8 & 0xff),
        BabyBearElem::new(value >> 16 & 0xff),
        BabyBearElem::new(value >> 24 & 0xff),
    )
}

fn split_word16(value: u32) -> (BabyBearElem, BabyBearElem) {
    (
        BabyBearElem::new(value & 0xffff),
        BabyBearElem::new(value >> 16),
    )
}

fn merge_word8((x0, x1, x2, x3): (BabyBearElem, BabyBearElem, BabyBearElem, BabyBearElem)) -> u32 {
    let x0: u32 = x0.into();
    let x1: u32 = x1.into();
    let x2: u32 = x2.into();
    let x3: u32 = x3.into();
    x0 | x1 << 8 | x2 << 16 | x3 << 24
}

/// An event traced from the running VM.
#[non_exhaustive]
#[derive(PartialEq)]
pub enum TraceEvent {
    /// An instruction has started at the given program counter
    InstructionStart {
        /// Cycle number since startup
        cycle: u32,
        /// Program counter of the instruction being executed
        pc: u32,
    },

    /// A register has been set
    RegisterSet {
        /// Register ID (0-16)
        reg: usize,
        /// New value in the register
        value: u32,
    },

    /// A memory location has been written
    MemorySet {
        /// Address of word that's been written
        addr: u32,
        /// Value of word that's been written
        value: u32,
    },
}

impl Debug for TraceEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InstructionStart { cycle, pc } => {
                write!(f, "InstructionStart({cycle}, 0x{pc:08X})")
            }
            Self::RegisterSet { reg, value } => write!(f, "RegisterSet({reg}, 0x{value:08X})"),
            Self::MemorySet { addr, value } => write!(f, "MemorySet(0x{addr:08X}, 0x{value:08X})"),
        }
    }
}
