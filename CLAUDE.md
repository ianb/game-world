# Working in this repo

A living-town game: a deterministic Rust simulation compiled to wasm, a
TypeScript viewer, web first and mobile probable. Start with
[README.md](./README.md), then [docs/decisions.md](./docs/decisions.md) (the
decision log; read before proposing architecture) and
[docs/design/what-remains.md](./docs/design/what-remains.md) (the build
order). Research reports live in `docs/research/`.

## Commands

`just check` runs everything CI runs: format, clippy with `-D warnings`,
tests, insta snapshots, and the wasm32 check. Run it before every commit. If
`just` or the cargo tools are missing, run `scripts/setup.sh` first.

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

## Boundary rules (Rust to TypeScript)

Boundary types live in `crates/sim/src/protocol.rs`. They are integer-only,
carry no `u64`, `usize`, `HashMap` or floats, and use no `serde(with)`.
`PROTOCOL_VERSION` bumps on any change. Generated TypeScript under
`packages/protocol/src/generated/` and schemas under `schemas/` are never
hand-edited; regenerate and commit.

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
