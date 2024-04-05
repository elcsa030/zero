// Copyright 2024 RISC Zero, Inc.
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

// This is based on cargo-wasix: https://github.com/wasix-org/cargo-wasix

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};
use fs_extra::dir::CopyOptions;
use risc0_build::risc0_data;

use crate::utils::CommandExt;

pub enum ToolchainRepo {
    Rust,
    C,
}

impl ToolchainRepo {
    pub const fn url(&self) -> &str {
        match self {
            Self::Rust => "https://github.com/risc0/rust.git",
            Self::C => "https://github.com/risc0/toolchain.git",
        }
    }

    pub const fn language(&self) -> &str {
        match self {
            Self::Rust => "rust",
            Self::C => "c",
        }
    }

    pub fn asset_name(&self, target: &str) -> String {
        match self {
            Self::Rust => format!("rust-toolchain-{target}.tar.gz"),
            Self::C => match target {
                "aarch64-apple-darwin" => "riscv32im-osx-arm64.tar.xz".to_string(),
                "x86_64-unknown-linux-gnu" => "riscv32im-linux-x86_64.tar.xz".to_string(),
                _ => panic!("binaries for {target} are not available"),
            },
        }
    }
}

/// Branch to use in the custom Rust repo.
pub const RUST_BRANCH: &str = "risc0";

/// The name of the rustup toolchain
pub const RUSTUP_TOOLCHAIN_NAME: &str = "risc0";

/// A rustup toolchain manager
#[derive(Clone, Debug)]
pub struct RustupToolchain {
    /// The name of the rustup toolchain
    pub name: String,

    /// The path of the rustup toolchain
    pub path: PathBuf,
}

impl RustupToolchain {
    /// Verify if the toolchain is present in rustup.
    ///
    /// Returns the path to the toolchain.
    fn find_by_name(name: &str) -> Result<Option<Self>, anyhow::Error> {
        let out = Command::new("rustup")
            .args(["toolchain", "list", "--verbose"])
            .capture_stdout()?;
        let path_raw = out
            .lines()
            .find(|line| line.trim().starts_with(name))
            .and_then(|line| line.strip_prefix(name))
            .map(|line| line.trim());

        if let Some(path) = path_raw {
            Ok(Some(Self {
                name: name.to_string(),
                path: path.into(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Link the toolchain to a local directory via rustup.
    pub fn link(name: &str, dir: &Path) -> Result<Self> {
        eprintln!(
            "Activating rustup toolchain {} at {}...",
            name,
            dir.display()
        );

        // Small sanity check.
        #[cfg(not(target_os = "windows"))]
        let rustc_exe = "rustc";
        #[cfg(target_os = "windows")]
        let rustc_exe = "rustc.exe";

        let rustc_path = dir.join("bin").join(rustc_exe);
        if !rustc_path.is_file() {
            bail!(
                "Invalid toolchain directory: rustc executable not found at {}",
                rustc_path.display()
            );
        }

        // If already present, unlink first.
        // This is required because otherwise rustup can get in a buggy state.
        if Self::find_by_name(name)?.is_some() {
            Command::new("rustup")
                .args(["toolchain", "remove", name])
                .run()
                .context("Could not remove existing toolchain")?;
        }

        Command::new("rustup")
            .args(["toolchain", "link", name])
            .arg(dir)
            .run_verbose()
            .context("Could not link toolchain: rustup not installed?")?;

        eprintln!("rustup toolchain {name} was linked and is now available!");

        Ok(Self {
            name: name.to_string(),
            path: dir.into(),
        })
    }
}

/// A rustup toolchain manager
#[derive(Clone, Debug)]
pub struct CToolchain {
    /// The path of the rustup toolchain
    pub path: PathBuf,
}

impl CToolchain {
    fn get_subdir(path: &Path) -> Result<PathBuf> {
        let sub_dir: Vec<std::result::Result<std::fs::DirEntry, std::io::Error>> =
            std::fs::read_dir(path)?.into_iter().collect();
        if sub_dir.len() != 1 {
            bail!(
                "Expected {} to only have 1 subdirectory, found {}",
                path.display(),
                sub_dir.len()
            );
        }
        let entry = sub_dir[0].as_ref().unwrap();
        Ok(entry.path())
    }

    pub fn link(path: &Path) -> Result<Self> {
        let c_download_dir = Self::get_subdir(path)?;
        let r0_data = risc0_data()?;
        fs_extra::dir::copy(
            c_download_dir.clone(),
            &r0_data,
            &CopyOptions::new().overwrite(true).copy_inside(true),
        )?;

        // for c, we will keep the toolchains in the r0_data directory for now
        let c_install_dir = &r0_data.join("c");
        if c_install_dir.exists() {
            fs::remove_dir_all(c_install_dir)?;
        }
        fs::rename(
            r0_data.join(c_download_dir.file_name().unwrap()),
            c_install_dir,
        )?;

        let gcc_script_path = c_install_dir.join("r0-gcc");
        let mut gcc_script = File::create(gcc_script_path.clone())?;
        write!(
            gcc_script,
            "#!/bin/bash\n\"{}\" -I \"{}\" \"$@\"\n",
            c_install_dir.join("bin/riscv32-unknown-elf-gcc").display(),
            c_install_dir.join("picolibc/include").display()
        )
        .unwrap();

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = gcc_script.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(gcc_script_path, perms)?;
        }

        Ok(Self {
            path: c_install_dir.into(),
        })
    }
}
