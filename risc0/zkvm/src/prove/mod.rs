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
//!
//! # Usage
//! The primary use of this module is to provably run a zkVM guest by use of a
//! [Prover]. See the [Prover] documentation for more detailed usage
//! information.
//!
//! ```ignore
//! // In real code, the ELF & Image ID would be generated by `risc0-build` scripts for the guest code
//! use methods::{EXAMPLE_ELF, EXAMPLE_ID};
//! use risc0_zkvm::Prover;
//!
//! let mut prover = Prover::new(&EXAMPLE_ELF, EXAMPLE_ID)?;
//! prover.add_input_u32_slice(&to_vec(&input)?);
//! let receipt = prover.run()?;
//! ```

mod exec;
pub mod io;
pub(crate) mod loader;
mod plonk;
#[cfg(feature = "profiler")]
pub mod profiler;

use std::{
    cell::RefCell,
    cmp::min,
    collections::HashMap,
    fmt::Debug,
    io::{stderr, stdout, BufRead, Write},
    mem::take,
    rc::Rc,
    str::from_utf8,
};

use anyhow::{bail, Result};
use io::{PosixIo, SliceIo, Syscall};
use risc0_circuit_rv32im::{REGISTER_GROUP_ACCUM, REGISTER_GROUP_CODE, REGISTER_GROUP_DATA};
use risc0_core::field::baby_bear::{BabyBear, BabyBearElem, BabyBearExtElem};
use risc0_zkp::{
    adapter::TapsProvider,
    core::sha::Digest,
    hal::{EvalCheck, Hal},
    prove::adapter::ProveAdapter,
};
use risc0_zkvm_platform::{
    fileno,
    memory::MEM_SIZE,
    syscall::{
        nr::{SYS_READ, SYS_READ_AVAIL, SYS_WRITE},
        reg_abi::{REG_A3, REG_A4},
        SyscallName,
    },
    WORD_SIZE,
};

use crate::binfmt::elf::Program;
use crate::{
    prove::exec::MemoryState,
    receipt::{insecure_skip_seal, Receipt},
    CIRCUIT,
};

/// Options available to modify the prover's behavior.
pub struct ProverOpts<'a> {
    pub(crate) skip_seal: bool,

    pub(crate) skip_verify: bool,

    pub(crate) syscall_handlers: HashMap<String, Box<dyn Syscall + 'a>>,

    pub(crate) io: Option<PosixIo<'a>>,
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

    /// Add a handler for a syscall which inputs and outputs a slice
    /// of plain old data..  The guest can call these by invoking
    /// `risc0_zkvm::guest::env::send_recv_slice`
    pub fn with_slice_io(self, syscall: SyscallName, handler: impl SliceIo + 'a) -> Self {
        self.with_syscall(syscall, handler.to_syscall())
    }

    /// Add a handler for a syscall which inputs and outputs a slice
    /// of plain old data.  The guest can call these callbacks by
    /// invoking `risc0_zkvm::guest::env::send_recv_slice`.
    pub fn with_sendrecv_callback(
        self,
        syscall: SyscallName,
        f: impl Fn(&[u8]) -> Vec<u8> + 'a,
    ) -> Self {
        self.with_slice_io(syscall, io::slice_io_from_fn(f))
    }

    /// Add a handler for a raw syscall implementation.  The guest can
    /// invoke these using the risc0_zkvm_platform::syscall!  macro.
    pub fn with_syscall(mut self, syscall: SyscallName, handler: impl Syscall + 'a) -> Self {
        self.syscall_handlers
            .insert(syscall.as_str().to_string(), Box::new(handler));
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

    /// Add a posix-style file descriptor for reading
    pub fn with_read_fd_ref(mut self, fd: u32, reader: Rc<RefCell<impl BufRead + 'a>>) -> Self {
        let io = self.io.unwrap_or_default();
        self.io = Some(io.with_read_fd(fd, reader));
        self
    }

    /// Add a posix-style file descriptor for reading
    pub fn with_read_fd(self, fd: u32, reader: impl BufRead + 'a) -> Self {
        self.with_read_fd_ref(fd, Rc::new(RefCell::new(reader)))
    }

    /// Add a posix-style file descriptor for writing
    pub fn with_write_fd_ref(mut self, fd: u32, writer: Rc<RefCell<impl Write + 'a>>) -> Self {
        let io = self.io.unwrap_or_default();
        self.io = Some(io.with_write_fd(fd, writer));
        self
    }

    /// Add a posix-style file descriptor for writing
    pub fn with_write_fd(self, fd: u32, writer: impl Write + 'a) -> Self {
        self.with_write_fd_ref(fd, Rc::new(RefCell::new(writer)))
    }
}

impl<'a> Default for ProverOpts<'a> {
    fn default() -> ProverOpts<'a> {
        ProverOpts {
            io: None,
            skip_seal: false,
            skip_verify: false,
            syscall_handlers: HashMap::new(),
            trace_callback: None,
        }
        .with_write_fd(fileno::STDOUT, stdout())
        .with_write_fd(fileno::STDERR, stderr())
    }
}

/// Manages communication with and execution of a zkVM [Program]
///
/// # Usage
/// A [Prover] is constructed from the ELF code and an Image ID generated from
/// the guest code to be proven (see
/// [risc0_build](https://docs.rs/risc0-build/latest/risc0_build/) for more
/// information about how these are generated). Use [Prover::new] if you want
/// the default [ProverOpts], or [Prover::new_with_opts] to use custom options.
/// ```ignore
/// // In real code, the ELF & Image ID would be generated by risc0 build scripts from guest code
/// use methods::{EXAMPLE_ELF, EXAMPLE_ID};
/// use risc0_zkvm::Prover;
///
/// let mut prover = Prover::new(&EXAMPLE_ELF, EXAMPLE_ID)?;
/// ```
/// Provers should essentially always be mutable so that their [Prover::run]
/// method may be called.
///
/// Input data can be passed to the Prover with [Prover::add_input_u32_slice]
/// (or [Prover::add_input_u8_slice]). After all inputs have been added, call
/// [Prover::run] to execute the guest code and produce a [Receipt] proving
/// execution.
/// ```ignore
/// prover.add_input_u32_slice(&risc0_zkvm::serde::to_vec(&input)?);
/// let receipt = prover.run()?;
/// ```
/// After running the prover, publicly proven results can be accessed from the
/// [Receipt].
/// ```ignore
/// let receipt = prover.run()?;
/// let proven_result: ResultType = risc0_zkvm::serde::from_slice(&receipt.journal)?;
/// ```
pub struct Prover<'a> {
    elf: Program,
    inner: ProverImpl<'a>,
    image_id: Digest,
    /// How many cycles executing the guest took.
    ///
    /// Initialized to 0 by [Prover::new], then computed when [Prover::run] is
    /// called. Note that this is privately shared with the host; it is not
    /// present in the [Receipt].
    pub cycles: usize,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "cuda")] {
        use risc0_circuit_rv32im::{CircuitImpl, cpu::CpuEvalCheck, cuda::CudaEvalCheck};
        use risc0_zkp::hal::cuda::CudaHal;
        use risc0_zkp::hal::cpu::BabyBearPoseidonCpuHal;

        /// Returns the default SHA-256 HAL for the RISC Zero circuit
        pub fn default_hal() -> (Rc<CudaHal>, CudaEvalCheck) {
            let hal = Rc::new(CudaHal::new());
            let eval = CudaEvalCheck::new(hal.clone());
            (hal, eval)
        }

        /// Falls back to the CPU for Poseidon for now
        pub fn default_poseidon_hal() -> (Rc<BabyBearPoseidonCpuHal>, CpuEvalCheck<'static, CircuitImpl>) {
            let hal = Rc::new(BabyBearPoseidonCpuHal::new());
            let eval = CpuEvalCheck::new(&CIRCUIT);
            (hal, eval)
        }
    } else if #[cfg(feature = "metal")] {
        use risc0_circuit_rv32im::metal::{MetalEvalCheck, MetalEvalCheckSha256};
        use risc0_zkp::hal::metal::{MetalHalSha256, MetalHalPoseidon, MetalHashPoseidon};

        /// Returns the default SHA-256 HAL for the RISC Zero circuit
        pub fn default_hal() -> (Rc<MetalHalSha256>, MetalEvalCheckSha256) {
            let hal = Rc::new(MetalHalSha256::new());
            let eval = MetalEvalCheckSha256::new(hal.clone());
            (hal, eval)
        }

        /// Returns the default Poseidon HAL for the RISC Zero circuit
        pub fn default_poseidon_hal() -> (Rc<MetalHalPoseidon>, MetalEvalCheck<MetalHashPoseidon>) {
            let hal = Rc::new(MetalHalPoseidon::new());
            let eval = MetalEvalCheck::<MetalHashPoseidon>::new(hal.clone());
            (hal, eval)
        }
    } else {
        use risc0_circuit_rv32im::{CircuitImpl, cpu::CpuEvalCheck};
        use risc0_zkp::hal::cpu::{BabyBearSha256CpuHal, BabyBearPoseidonCpuHal};

        /// Returns the default SHA-256 HAL for the RISC Zero circuit
        ///
        /// RISC Zero uses a
        /// [HAL](https://docs.rs/risc0-zkp/latest/risc0_zkp/hal/index.html)
        /// (Hardware Abstraction Layer) to interface with the zkVM circuit.
        /// This function returns the default HAL for the selected `risc0-zkvm`
        /// features. It also returns the associated
        /// [EvalCheck](https://docs.rs/risc0-zkp/latest/risc0_zkp/hal/trait.EvalCheck.html)
        /// used for computing the cryptographic check polynomial.
        ///
        /// Note that this function will return different types when
        /// `risc0-zkvm` is built with features that select different the target
        /// hardware. The version documented here is used when no special
        /// hardware features are selected.
        pub fn default_hal() -> (Rc<BabyBearSha256CpuHal>, CpuEvalCheck<'static, CircuitImpl>) {
            let hal = Rc::new(BabyBearSha256CpuHal::new());
            let eval = CpuEvalCheck::new(&CIRCUIT);
            (hal, eval)
        }

                /// Returns the default Poseidon HAL for the RISC Zero circuit
        ///
        /// The same as [default_hal] except it gives the default HAL for
        /// securing the circuit using Poseidon (instead of SHA-256).
        pub fn default_poseidon_hal() -> (Rc<BabyBearPoseidonCpuHal>, CpuEvalCheck<'static, CircuitImpl>) {
            let hal = Rc::new(BabyBearPoseidonCpuHal::new());
            let eval = CpuEvalCheck::new(&CIRCUIT);
            (hal, eval)
        }

    }
}

impl<'a> Prover<'a> {
    /// Construct a new prover using the default options
    ///
    /// This will return an `Err` if `elf` is not a valid ELF file
    pub fn new<D>(elf: &[u8], image_id: D) -> Result<Self>
    where
        Digest: From<D>,
    {
        Self::new_with_opts(elf, image_id, ProverOpts::default())
    }

    /// Construct a new prover using custom [ProverOpts]
    ///
    /// This will return an `Err` if `elf` is not a valid ELF file
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

    /// Provide input data to the guest. This data can be read by the guest
    /// via [crate::guest::env::read].
    ///
    /// It is possible to provide multiple inputs to the guest so long as the
    /// guest reads them in the same order they are added by the [Prover].
    /// However, to reduce maintenance burden and the chance of mistakes, we
    /// recommend instead using a single `struct` to hold all the inputs and
    /// calling [Prover::add_input_u8_slice] just once (on the serialized
    /// representation of that input).
    pub fn add_input_u8_slice(&mut self, slice: &[u8]) {
        self.inner.input.extend_from_slice(slice);
    }

    /// Provide input data to the guest. This data can be read by the guest
    /// via [crate::guest::env::read].
    ///
    /// It is possible to provide multiple inputs to the guest so long as the
    /// guest reads them in the same order they are added by the [Prover].
    /// However, to reduce maintenance burden and the chance of mistakes, we
    /// recommend instead using a single `struct` to hold all the inputs and
    /// calling [Prover::add_input_u32_slice] just once (on the serialized
    /// representation of that input).
    pub fn add_input_u32_slice(&mut self, slice: &[u32]) {
        self.inner
            .input
            .extend_from_slice(bytemuck::cast_slice(slice));
    }

    /// Run the guest code. If the guest exits successfully, this returns a
    /// [Receipt] that proves execution. If the execution of the guest fails for
    /// any reason, this instead returns an `Err`.
    ///
    /// This function uses the default HAL (Hardware Abstraction Layer) to
    /// run the guest. If you want to use a different HAL, you can do so either
    /// by changing the default using risc0_zkvm feature flags, or by using
    /// [Prover::run_with_hal].
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

    /// Run the guest code. Like [Prover::run], but with parameters for
    /// selecting a HAL, allowing the use of HALs other than [default_hal].
    /// People creating or using a third-party HAL can use this function to run
    /// the Prover with that HAL.
    #[tracing::instrument(skip_all)]
    pub fn run_with_hal<H, E>(&mut self, hal: &H, eval: &E) -> Result<Receipt>
    where
        H: Hal<Field = BabyBear, Elem = BabyBearElem, ExtElem = BabyBearExtElem>,
        E: EvalCheck<H>,
    {
        if let Some(io) = take(&mut self.inner.opts.io) {
            let io = Rc::new(io);
            let opts = take(&mut self.inner.opts);
            self.inner.opts = opts
                .with_syscall(SYS_READ, io.clone())
                .with_syscall(SYS_READ_AVAIL, io.clone())
                .with_syscall(SYS_WRITE, io);
        }
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
            journal: self.inner.journal.borrow().clone(),
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
    pub journal: Rc<RefCell<Vec<u8>>>,
    pub opts: ProverOpts<'a>,
}

impl<'a> ProverImpl<'a> {
    fn new(opts: ProverOpts<'a>) -> Self {
        let journal = Rc::new(RefCell::new(Vec::new()));
        let opts = opts.with_write_fd_ref(fileno::JOURNAL, journal.clone());
        Self {
            input: Vec::new(),
            journal,
            opts,
        }
    }
}

impl<'a> exec::HostHandler for ProverImpl<'a> {
    fn on_txrx(
        &mut self,
        mem: &MemoryState,
        syscall: &str,
        cycle: usize,
        to_guest: &mut [u32],
    ) -> Result<(u32, u32)> {
        log::debug!("syscall {syscall}, {} words to guest", to_guest.len());
        if let Some(cb) = self.opts.syscall_handlers.get(syscall) {
            return Ok(cb.syscall(syscall, mem, to_guest));
        }
        // TODO: Use the standard syscall handler framework for this instead of matching
        // on name.
        let buf_ptr = mem.load_register(REG_A3);
        let buf_len = mem.load_register(REG_A4);
        let from_guest = mem.load_region(buf_ptr, buf_len);
        match syscall
            .strip_prefix("risc0_zkvm_platform::syscall::nr::")
            .unwrap_or(syscall)
        {
            "SYS_PANIC" => {
                let msg = from_utf8(&from_guest)?;
                bail!("{}", msg)
            }
            "SYS_LOG" => {
                let msg = from_utf8(&from_guest)?;
                println!("R0VM[{cycle}] {}", msg);
                Ok((0, 0))
            }
            "SYS_CYCLE_COUNT" => Ok((cycle as u32, 0)),
            "SYS_INITIAL_INPUT" => {
                log::debug!("SYS_INITIAL_INPUT: {}", from_guest.len());
                let copy_bytes = min(to_guest.len() * WORD_SIZE, self.input.len());
                bytemuck::cast_slice_mut(to_guest)[..copy_bytes]
                    .clone_from_slice(&self.input[..copy_bytes]);
                Ok((self.input.len() as u32, 0))
            }
            "SYS_RANDOM" => {
                log::debug!("SYS_RANDOM: {}", to_guest.len());
                let mut rand_buf = vec![0u8; to_guest.len() * WORD_SIZE];
                getrandom::getrandom(rand_buf.as_mut_slice())?;
                bytemuck::cast_slice_mut(to_guest).clone_from_slice(rand_buf.as_slice());
                Ok((0, 0))
            }
            _ => bail!("Unknown syscall: {syscall}"),
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
