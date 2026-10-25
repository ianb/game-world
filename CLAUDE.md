# Working in this repo

A living-town game: a deterministic Rust simulation compiled to wasm, a
TypeScript viewer, web first and mobile probable. Start with
[README.md](./README.md), then [docs/decisions.md](./docs/decisions.md) (the
decision log; read before proposing architecture) and
[docs/design/what-remains.md](./docs/design/what-remains.md) (the build
order). Research reports live in `docs/research/`.

## Commands

`just check` runs format, clippy with `-D warnings`, the test suite with
insta in check mode, the wasm32 check, the generated-files drift check, the
wasm hash smoke in Node, and the TypeScript typecheck, contract test and
viewer build. CI runs `just check viewer-smoke`, which adds the headless
browser smoke. Run `just check` before every commit. If `just` or the cargo
tools are missing, run `scripts/setup.sh` first. `just viewer-smoke` drives the built page in
headless Chromium and prints the scene dump; `just wasm-smoke` runs a week
in Node and compares the hash with the native golden.

Text views are how you see the town: `cargo run -p tools -- day` for a day's
itineraries, `summary` for what was unusual each day, `talks` for
conversation transcripts, `render`, `route`, `travel-matrix`, `where`,
`economy`. The viewer's
`window.__scene()` returns the same views from the browser. To see the
tiles as drawn, build the viewer and run `node scripts/viewer-shot.mjs
--days 20 --place town`, then read the PNG it writes.

Snapshots: run `just snap` (which is `cargo insta test --accept`) locally,
read the `.snap` diff, and commit it with a reason in the message. CI never
writes snapshots and fails on a mismatch or a stale file.

## Determinism rules for `crates/sim` and `crates/characters`

The sim is a pure function of (content, seed, inputs). Enforced by
`crates/sim/clippy.toml` and `#![deny(clippy::float_arithmetic)]`:

- No wall clock, no ambient randomness. Random numbers come from the streams
  in `SimState.rng`; pick the stream that matches the concern.
- No `HashMap` or `HashSet`. Use `BTreeMap`, `BTreeSet`, or a `Vec` indexed
  by a typed id.
- No floats. Ticks, tiles, durations and costs are integers.
- Inputs are the only outside influence, applied at their tick through the
  event queue. Character code reads through `Ctx` and writes by emitting
  commands; it never mutates world state directly.
- Everything that determines the future lives in `SimState` and is
  serialized. Anything not in the snapshot is not in the hash, and a
  snapshot restored at tick t must behave exactly like the live sim.

## Types over strings

Use types as much as possible, so the compiler finds every place a change
has to reach. A closed set is an enum. Where a new variant should force a
decision, match it exhaustively with no `_` arm and put
`#[deny(clippy::wildcard_enum_match_arm)]` on that function (as each
character's `warmth_of(Gesture)` does). Strings are for open sets that
grow with content (topics), and then they are declared in `content/` and
checked, not matched by luck.

## Boundary rules (Rust to TypeScript)

Boundary types live in `crates/sim/src/protocol.rs`. They are integer-only,
carry no `u64`, `usize`, `HashMap` or floats, and use no `serde(with)`.
`PROTOCOL_VERSION` bumps on any change, and `PROTOCOL_VERSION` in
`packages/protocol/src/index.ts` must match. Generated files are never
hand-edited: `packages/protocol/src/generated/` (ts-rs) and
`schemas/content.schema.json` (schemars) are written by `just gen-types`,
committed, and diff-checked. The test run regenerates them as a side effect.
The fixture `fixtures/day0-seed1.json` is regenerated with
`cargo run -p tools -- events` whenever emitted events change; both the Rust
test and the vitest contract test check it.

## Prose

Game text (anything a player reads, anything a villager says) is the
author's voice. Propose it in chat and let Ian write or approve it. Prompts
for any model, if they ever exist, are written flat: plain instructions, no
rhetoric. Engineering prose (comments, docs, commit messages) is fine but
lighter than instinct: no em dashes, no closing aphorisms, no "This is not
X. It is Y."

## Decisions

Record design calls in `docs/decisions.md`: a dated heading, Ian's words,
what was decided, implications. Never rewrite an earlier entry; add a
correcting one.
