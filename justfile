# One entry point. CI runs the same recipes.

default: check

# Everything that must pass before a commit.
check: fmt-check lint test wasm-check gen-types-check ts

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo nextest run --workspace --all-features
    cargo test --workspace --all-features --doc
    cargo insta test --workspace --all-features --check --unreferenced=reject

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
    echo '{ "type": "commonjs" }' > packages/sim-wasm/pkg-node/package.json
    cargo run -q -p tools -- bundle > target/content-bundle.json
    mkdir -p packages/viewer/public && cp target/content-bundle.json packages/viewer/public/content.json

# Build the wasm, run a week in Node, and compare the hash with the native golden snapshot.
wasm-smoke: build-wasm
    node packages/sim-wasm/smoke.mjs target/content-bundle.json \
        "$(grep -v '^---' crates/sim/tests/snapshots/town__golden_hash_after_a_week.snap | grep -v '^$' | grep -v '^source' | grep -v '^expression' | tail -1)"

# Regenerate the TypeScript bindings from the Rust boundary types.
gen-types:
    rm -rf packages/protocol/src/generated
    cargo test -q -p sim --features schema export_bindings

# Fail if the committed bindings do not match the Rust types.
gen-types-check: gen-types
    git diff --exit-code --stat packages/protocol/src/generated || (echo "generated TypeScript is stale: run just gen-types and commit" && exit 1)

# TypeScript: typecheck, the wasm contract test, and the viewer build. Needs build-wasm first.
ts: build-wasm
    pnpm install --frozen-lockfile
    pnpm -r run typecheck
    pnpm --filter @game-world/protocol test
    pnpm --filter @game-world/viewer build

# Serve the built viewer and drive it with headless Chromium; prints the scene dump.
viewer-smoke: ts
    (cd packages/viewer && pnpm exec vite preview --port 4173 --strictPort > /dev/null 2>&1 &) ; sleep 3; \
        CHROMIUM_PATH="${CHROMIUM_PATH:-/opt/pw-browsers/chromium}" node scripts/viewer-smoke.mjs http://localhost:4173/ ; \
        status=$?; pkill -f "vite preview" > /dev/null 2>&1 || true; exit $status
