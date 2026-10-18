#!/usr/bin/env bash
set -euo pipefail
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p ghosh-core --example export_contracts -- --check
npm run build
npm run test:ui
