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

use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use risc0_crates_validator::{
    profiles::{Profiles, Repo},
    ProfileConfig, RunStatus, ValidatorBuilder,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to json version of [ProfileConfig]
    #[arg(short = 'p', long, default_value = "./profiles/primary.json")]
    profiles_path: PathBuf,

    /// Run just a single crate from the [ProfileConfig]
    #[arg(short = 'c', long)]
    crate_name: Option<String>,

    /// Output location to write temporary projects
    ///
    /// Defaults value is a new TempDir
    #[arg(short = 'o', long)]
    out_dir: Option<PathBuf>,

    /// Write profile data with results
    ///
    /// Optional: write out the profile data with all the validation results
    #[arg(short = 'j', long)]
    json_output: Option<PathBuf>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let file = File::open(args.profiles_path).context("Failed to open profiles_path file")?;
    let reader = BufReader::new(file);
    let mut profiles: Profiles = serde_yaml::from_reader(reader)?;
    profiles.iter_mut().for_each(|p| {
        if p.settings.repo.is_none() {
            p.settings.repo = Some(Repo::Local("/Users/mcs/git/github/risc0/risc0/".into()));
        }
    });

    info!("Starting run of {} profiles...", profiles.len());
    let validator = ValidatorBuilder::new(/* context */)
        .out_dir(args.out_dir)
        .build()?;

    let results = if let Some(crate_name) = args.crate_name {
        validator.run_single(&crate_name, &mut profiles)?
    } else {
        validator.run_all(&mut profiles)?
    };

    info!("Test results:");
    colored::control::set_override(true);
    let mut successful = 0;
    for (idx, (name, result)) in results.iter().enumerate() {
        let result_str = result.status.as_ref();
        info!(
            "{idx}: {} - {}",
            name,
            match result.status {
                RunStatus::Success => {
                    successful += 1;
                    result_str.green()
                }
                RunStatus::Skipped => result_str.yellow(),
                RunStatus::BuildFail => result_str.red(),
                RunStatus::RunFail => result_str.red(),
            }
        );
    }

    colored::control::unset_override();
    info!("{}/{} successful", successful, results.len());

    if let Some(out_path) = args.json_output {
        std::fs::write(
            out_path,
            serde_json::to_string(&results).context("Failed to serialize Validator context")?,
        )
        .context("Failed to write output json file")?;
    }

    Ok(())
}
