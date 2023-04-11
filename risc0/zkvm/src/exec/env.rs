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

//! TODO

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Cursor, Read, Write},
};

use bytemuck::Pod;
use risc0_zkvm_platform::{
    fileno,
    syscall::{
        nr::{SYS_GETENV, SYS_READ, SYS_READ_AVAIL, SYS_WRITE},
        SyscallName,
    },
};

use super::io::{slice_io_from_fn, syscalls, PosixIo, SliceIo, Syscall, SyscallTable};

const DEFAULT_SEGMENT_LIMIT_PO2: usize = 20; // 1M cycles

const DEFAULT_SESSION_LIMIT: usize = 64 * 1024 * 1024; // 64M cycles

/// TODO
#[derive(Clone)]
pub struct ExecutorEnvBuilder<'a> {
    inner: ExecutorEnv<'a>,
}

/// TODO
#[derive(Clone)]
pub struct ExecutorEnv<'a> {
    pub(crate) env_vars: HashMap<String, String>,
    pub(crate) segment_limit_po2: usize,
    pub(crate) session_limit: usize,
    pub(crate) syscalls: SyscallTable<'a>,
    pub(crate) io: PosixIo<'a>,
    input: Vec<u8>,
}

impl<'a> ExecutorEnv<'a> {
    /// TODO
    pub fn default() -> ExecutorEnvBuilder<'a> {
        ExecutorEnvBuilder::default()
    }
}

impl<'a> Default for ExecutorEnvBuilder<'a> {
    fn default() -> Self {
        Self {
            inner: ExecutorEnv {
                env_vars: Default::default(),
                segment_limit_po2: DEFAULT_SEGMENT_LIMIT_PO2,
                session_limit: DEFAULT_SESSION_LIMIT,
                syscalls: Default::default(),
                io: Default::default(),
                input: Default::default(),
            },
        }
    }
}

impl<'a> ExecutorEnvBuilder<'a> {
    /// TODO
    pub fn build(&mut self) -> ExecutorEnv<'a> {
        let mut result = self.clone();
        let getenv = syscalls::Getenv(self.inner.env_vars.clone());
        if !self.inner.input.is_empty() {
            let reader = Cursor::new(self.inner.input.clone());
            result.inner.io.with_read_fd(fileno::STDIN, reader);
        }
        let io = result.inner.io.clone();
        result
            .syscall(SYS_GETENV, getenv)
            .syscall(SYS_READ, io.clone())
            .syscall(SYS_READ_AVAIL, io.clone())
            .syscall(SYS_WRITE, io);
        result.inner.clone()
    }

    /// TODO
    pub fn segment_limit_po2(&mut self, limit: usize) -> &mut Self {
        self.inner.segment_limit_po2 = limit;
        self
    }

    /// TODO
    pub fn session_limit(&mut self, limit: usize) -> &mut Self {
        self.inner.session_limit = limit;
        self
    }

    /// TODO
    pub fn env_vars(&mut self, vars: HashMap<String, String>) -> &mut Self {
        self.inner.env_vars = vars;
        self
    }

    /// Add an environment variable to the guest environment.
    pub fn env_var(&mut self, name: &str, val: &str) -> &mut Self {
        self.inner
            .env_vars
            .insert(name.to_string(), val.to_string());
        self
    }

    /// TODO
    pub fn add_input<T: Pod>(&mut self, slice: &[T]) -> &mut Self {
        self.inner
            .input
            .extend_from_slice(bytemuck::cast_slice(slice));
        self
    }

    /// Add a handler for a raw syscall implementation.
    pub fn syscall(&mut self, syscall: SyscallName, handler: impl Syscall + 'a) -> &mut Self {
        self.inner.syscalls.with_syscall(syscall, handler);
        self
    }

    /// Add a posix-style standard input.
    pub fn stdin(&mut self, reader: impl Read + 'a) -> &mut Self {
        self.read_fd(fileno::STDIN, BufReader::new(reader))
    }

    /// Add a posix-style standard output.
    pub fn stdout(&mut self, writer: impl Write + 'a) -> &mut Self {
        self.write_fd(fileno::STDOUT, writer)
    }

    /// Add a posix-style file descriptor for reading.
    pub fn read_fd(&mut self, fd: u32, reader: impl BufRead + 'a) -> &mut Self {
        self.inner.io.with_read_fd(fd, reader);
        self
    }

    /// Add a posix-style file descriptor for writing.
    pub fn write_fd(&mut self, fd: u32, writer: impl Write + 'a) -> &mut Self {
        self.inner.io.with_write_fd(fd, writer);
        self
    }

    /// Add a handler for a syscall which inputs and outputs a slice
    /// of plain old data. The guest can call these by invoking
    /// `risc0_zkvm::guest::env::send_recv_slice`
    pub fn slice_io(&mut self, syscall: SyscallName, handler: impl SliceIo + 'a) -> &mut Self {
        self.syscall(syscall, handler.to_syscall());
        self
    }

    /// Add a handler for a syscall which inputs and outputs a slice
    /// of plain old data. The guest can call these callbacks by
    /// invoking `risc0_zkvm::guest::env::send_recv_slice`.
    pub fn io_callback(
        &mut self,
        syscall: SyscallName,
        f: impl Fn(&[u8]) -> Vec<u8> + 'a,
    ) -> &mut Self {
        self.slice_io(syscall, slice_io_from_fn(f));
        self
    }
}
