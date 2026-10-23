# One entry point. CI runs the same recipes.

default: check

# Everything that must pass before a commit.
check: fmt-check lint test wasm-check

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo nextest run --workspace --all-features
    cargo test --workspace --all-features --doc

# Review and accept insta snapshots locally; CI never writes them.
snap:
    cargo insta test --workspace --all-features --accept

# The sim must compile for wasm with no ambient-randomness or wall-clock crates in the graph.
wasm-check:
    cargo check -p sim-wasm --target wasm32-unknown-unknown
    @! cargo tree -p sim-wasm --target wasm32-unknown-unknown -e normal | grep -E 'getrandom|ahash|instant|web-time' \
        || (echo "forbidden crate in the wasm dependency graph" && exit 1)

# Build the wasm package for the viewer (bundler) and for vitest (nodejs).
build-wasm:
    cargo build -p sim-wasm --release --target wasm32-unknown-unknown
    wasm-bindgen --target bundler --out-dir packages/sim-wasm/pkg target/wasm32-unknown-unknown/release/sim_wasm.wasm
    wasm-bindgen --target nodejs --out-dir packages/sim-wasm/pkg-node target/wasm32-unknown-unknown/release/sim_wasm.wasm
