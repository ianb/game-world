#!/usr/bin/env bash
# Installs what a fresh cloud session needs beyond rustup, Node and pnpm.
# Idempotent: skips anything already present.
set -euo pipefail

rustup target add wasm32-unknown-unknown
rustup component add rustfmt clippy

if ! command -v just >/dev/null; then cargo install --locked just; fi
if ! command -v cargo-nextest >/dev/null; then cargo install --locked cargo-nextest; fi
if ! command -v cargo-insta >/dev/null; then cargo install --locked cargo-insta; fi
if ! command -v wasm-bindgen >/dev/null || ! wasm-bindgen --version | grep -q '0.2.129'; then
  cargo install --locked wasm-bindgen-cli --version 0.2.129
fi
pnpm install --frozen-lockfile
echo "setup complete"
