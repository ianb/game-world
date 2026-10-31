# One entry point. CI runs the same recipes.

default: check

# Everything that must pass before a commit.
check: fmt-check lint test wasm-check gen-types-check wasm-smoke ts

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# One run of the suite under nextest, with insta in check mode so a changed or stale
# snapshot fails here as it does in CI.
test:
    cargo insta test --workspace --all-features --test-runner nextest --check --unreferenced=reject
    cargo test --workspace --all-features --doc

# Review and accept insta snapshots locally; CI never writes them.
snap:
    cargo insta test --workspace --all-features --accept

# The sim must compile for wasm with no ambient-randomness or wall-clock crates in the graph.
wasm-check:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo check -p sim-wasm --target wasm32-unknown-unknown
    tree=$(cargo tree -p sim-wasm --target wasm32-unknown-unknown -e normal)
    if grep -E 'getrandom|ahash|instant|web-time' <<< "$tree"; then
        echo "forbidden crate in the wasm dependency graph"; exit 1
    fi

# Build the wasm package for the viewer (bundler) and for vitest (nodejs).
# Builds the tileset in packages/viewer/public from the LPC art in art/lpc (commit it).
tileset:
    node scripts/make-tileset.mjs

build-wasm: bundles
    cargo build -p sim-wasm --release --target wasm32-unknown-unknown
    wasm-bindgen --target bundler --out-dir packages/sim-wasm/pkg target/wasm32-unknown-unknown/release/sim_wasm.wasm
    wasm-bindgen --target nodejs --out-dir packages/sim-wasm/pkg-node target/wasm32-unknown-unknown/release/sim_wasm.wasm
    echo '{ "type": "commonjs" }' > packages/sim-wasm/pkg-node/package.json

# Each world's content as one JSON document, for the wasm smoke, vitest and the viewer.
bundles:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p target/bundles
    for world in valley; do
        cargo run -q -p tools -- --world "$world" bundle > "target/bundles/$world.json"
        mkdir -p "packages/viewer/public/worlds/$world"
        cp "target/bundles/$world.json" "packages/viewer/public/worlds/$world/content.json"
    done

# Build the wasm, run a week in Node, and compare the hash with the native golden snapshot.
wasm-smoke: build-wasm
    node packages/sim-wasm/smoke.mjs valley target/bundles/valley.json \
        "$(grep -v '^---' worlds/valley/characters/tests/snapshots/town__golden_hash_after_a_week_matches_the_wasm_smoke.snap | grep -v '^$' | grep -v '^source' | grep -v '^expression' | tail -1)"

# Regenerate the TypeScript bindings and the content JSON Schema from the Rust types.
gen-types:
    rm -rf packages/protocol/src/generated
    cargo test -q -p sim --features schema export_bindings
    cargo test -q -p content --features schema export_schema

# Fail if the committed bindings or schema do not match the Rust types. Untracked files
# count as drift too.
gen-types-check: gen-types
    #!/usr/bin/env bash
    set -euo pipefail
    untracked=$(git ls-files --others --exclude-standard packages/protocol/src/generated schemas)
    if [ -n "$untracked" ] || ! git diff --exit-code --stat packages/protocol/src/generated schemas; then
        echo "generated files are stale: run just gen-types and commit"; exit 1
    fi

# TypeScript: typecheck, the wasm contract test, and the viewer build. Needs build-wasm first.
ts: build-wasm
    pnpm install --frozen-lockfile
    pnpm -r run typecheck
    pnpm --filter @game-world/protocol test
    pnpm --filter @game-world/viewer build

# Serve the built viewer and drive it with headless Chromium; prints the scene dump.
viewer-smoke: ts
    #!/usr/bin/env bash
    set -uo pipefail
    log=target/viewer-preview.log
    (cd packages/viewer && exec pnpm exec vite preview --port 4173 --strictPort) > "$log" 2>&1 &
    server=$!
    for _ in $(seq 1 50); do
        if curl -sf -o /dev/null http://localhost:4173/; then break; fi
        sleep 0.2
    done
    node scripts/viewer-smoke.mjs http://localhost:4173/
    status=$?
    kill "$server" 2>/dev/null; wait "$server" 2>/dev/null
    if [ "$status" -ne 0 ]; then echo "--- preview server log"; cat "$log"; fi
    exit $status
