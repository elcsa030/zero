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

use clap::{Parser, Subcommand};
use semver::Version;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install,
}

impl Commands {
    fn run(&self) {
        match self {
            Commands::Install => self.cmd_install(),
        }
    }

    fn cmd_install(&self) {
        install_solc();
    }
}

fn install_solc() {
    const SOLC_VERSION: Version = Version::new(0, 8, 20);

    println!("svm install {SOLC_VERSION}");
    svm_lib::blocking_install(&SOLC_VERSION).unwrap();
    println!("svm use {SOLC_VERSION}");
    svm_lib::use_version(&SOLC_VERSION).unwrap();
}

fn main() {
    Cli::parse().cmd.run();
}
