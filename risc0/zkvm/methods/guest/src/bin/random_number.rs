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

#![no_std]
#![no_main]

use risc0_zkvm::guest::env;
use risc0_zkvm_platform::syscall::sys_rand;

risc0_zkvm::entry!(main);

pub fn main() {
    let mut buf = [0u32; 32];
    unsafe {
        sys_rand(buf.as_mut_ptr(), 32);
        env::commit_slice(&buf);
    }
}
