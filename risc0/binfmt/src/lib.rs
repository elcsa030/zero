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

//! Manages formatted binaries used by the RISC Zero zkVM

#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

mod elf;
mod hash;
#[cfg(not(target_os = "zkvm"))]
mod image;
mod sys_state;

#[cfg(not(target_os = "zkvm"))]
pub use crate::image::{compute_image_id, MemoryImage, PageTableInfo};
pub use crate::{
    elf::Program,
    hash::{tagged_list, tagged_list_cons, tagged_struct, Digestible},
    sys_state::{read_sha_halfs, write_sha_halfs, SystemState},
};
