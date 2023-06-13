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

#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(feature = "deny_warnings", deny(warnings))]

use clap::{Parser, Subcommand};

use crate::commands::{build::BuildCommand, new::NewCommand};

/// Implementations of the commands
pub mod commands {
    /// Build a crate for RISC Zero.
    pub mod build;
    /// Create a new RISC Zero project.
    pub mod new;
}

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
/// Main cargo command
pub enum Cargo {
    /// The `risczero` command
    Risczero(Risczero),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
/// `cargo risczero`
pub struct Risczero {
    #[clap(subcommand)]
    /// Which `risczero` command to run
    pub command: RisczeroCmd,
}

#[derive(Subcommand)]
/// Primary commands  of `cargo risczero`.
pub enum RisczeroCmd {
    /// Creates a new risczero starter project.
    New(NewCommand),
    /// Build a crate for RISC Zero.
    Build(BuildCommand),
    /// Build and test a crate for RISC Zero.
    Test(BuildCommand),
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_app() {
        Cargo::command().debug_assert();
    }
}
