// Copyright 2022 Risc0, Inc.
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

#![doc = include_str!("README.md")]

use anyhow::Result;
use cxx::let_cxx_string;

#[cxx::bridge(namespace = "risc0::circuit")]
mod ffi {
    unsafe extern "C++" {
        include!("risc0/zkvm/circuit/make_circuit.h");

        fn make_circuit(path: &CxxString) -> Result<()>;
    }
}

/// Produces a machine generated .h file that implements the RISC-V circuit and writes it to a file.
pub fn make_circuit(path: &str) -> Result<()> {
    let_cxx_string!(path = path);
    Ok(ffi::make_circuit(&path)?)
}
