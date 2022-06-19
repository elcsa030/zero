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

#![no_std]
// TODO: WIP porting pure rust prover impl.
#![allow(unused)]
#![allow(dead_code)]

extern crate alloc;

mod zkp;
pub mod zkvm;

const OUTPUT_REGS: usize = 9;
const ACCUM_MIX_GLOBAL_SIZE: usize = 20;
