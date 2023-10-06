#!/bin/bash

set -xeuo pipefail

export CUDA_LIBRARY_PATH=/usr/lib/cuda
export RISC0_BUILD_LOCKED=1
export RISC0_SERVER_PATH=$PWD/target/release/r0vm
export RUST_BACKTRACE=full
# export RUSTC_WRAPPER=sccache

echo "--- cargo risczero install"
cargo run --bin cargo-risczero -- risczero install

echo "--- xtasks"
cargo xtask install
cargo xtask gen-receipt

echo "--- build r0vm"
cargo build -p risc0-r0vm --release -F $FEATURE

echo "--- test"
cargo test -F $FEATURE -F profiler
cargo test -p risc0-r0vm -F $FEATURE -F disable-dev-mode
cargo test -p risc0-zkvm -F $FEATURE -F fault-proof -- tests::memory_io tests::memory_access
cargo test -p cargo-risczero -F experimental

echo "--- check"
cargo check -F $FEATURE --benches
cargo check -p risc0-build
cargo check -p risc0-circuit-rv32im -F $FEATURE
cargo check -p risc0-core
cargo check -p risc0-sys -F $FEATURE
cargo check -p risc0-zkp -F $FEATURE
cargo check -p risc0-zkvm -F $FEATURE
cargo check -p substrate-minimal-runtime
