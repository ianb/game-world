# game-world

An explorable 2D world whose point is the people in it: buildings, routes,
and characters with schedules, goals, personalities and relationships. The
tone to aim for is a little soap opera and a little Sims, in the vibe of
Stardew Valley, with Ultima VI as an older touchstone.

Most character behavior will be carefully developed scripts, written largely
by a coding agent at authoring time. An LLM may drive behavior in high-plot
situations but is not the baseline runtime. The project is developed entirely
through the cloud coding interface, so everything has to work headless: no GUI
editor, text-first testing, deploy to the web.

## Design proposals

- [docs/design/world-layout.md](docs/design/world-layout.md): a unified
  one-scale world with inline ground floors and detached places for stated
  reasons, authored as structured data, with the validators and generated
  views that make layout testable from text.

- [docs/design/interactions.md](docs/design/interactions.md): proximity
  triggers, greetings and conversations between villagers, and what the
  placeholder routines do with them.
- [docs/design/multiplayer.md](docs/design/multiplayer.md): the shape live
  multiuser would take on the current architecture, the interface invariant
  to keep now, and the same boundary used for LLM dialogue and other
  external actors.
- [docs/design/what-remains.md](docs/design/what-remains.md): the decisions
  and designs still open on the way to a first Stardew-style town, tagged by
  when each is needed, with defaults.

## Try it

Locally, with Rust 1.97, Node 22 and pnpm 10 installed:

```
git clone https://github.com/ianb/game-world && cd game-world
scripts/setup.sh                       # wasm target, just, cargo tools, pnpm install
just build-wasm                        # the sim as wasm, plus the content bundle
pnpm --filter @game-world/viewer dev   # then open the printed localhost URL
```

The page shows the clock, who is where, and each place drawn in tiles,
advancing at six game-minutes per second with pause and speed buttons; the
`text` button adds the text views underneath. The tiles are placeholders
drawn by `scripts/make-tileset.mjs` (`just tileset`); generated art replaces
`packages/viewer/public/tiles.png` later, keeping the names in `tiles.json`.
Without a browser: `cargo run -p tools -- day --seed 7` prints everyone's
day, and `cargo run -p tools -- png --at 2pm` writes a picture to
`target/town.png`. After `pnpm --filter @game-world/viewer build`,
`node scripts/viewer-shot.mjs --days 20` screenshots the town's tiles as
the browser draws them.

A hosted copy can be built with `node scripts/artifact-page.mjs target/artifact`
after `pnpm --filter @game-world/viewer build`; it is the same page with the
document skeleton stripped for hosts that add their own.

## Status

Building. The Rust simulation core exists and runs the first town: a Cargo
workspace (`content`, `sim`, `characters`, `tools`, `sim-wasm`), a
discrete-event scheduler with a calendar, seeded RNG streams, snapshots and
per-subsystem hashes, content as JSON compiled to tile grids with
validators, hierarchical pathfinding, villagers on baseline schedules, text
views, and a wasm facade whose Node smoke test matches the native hash.
On top of it: generated TypeScript types with a drift check, a protocol
package wrapping the wasm behind a `Sim` interface, a vitest contract test
against a committed fixture, a text viewer in the browser driven by
Playwright in CI, and the first three placeholder villagers written as Rust modules
(`crates/characters`), who greet, talk and remember. `just check` runs it all; `cargo run -p tools -- day`
shows a day in the town. Next: more villagers and buildings, then the
renderer spike (see [docs/design/what-remains.md](docs/design/what-remains.md)).

## What is here

- [docs/decisions.md](docs/decisions.md): design calls made so far and what
  they imply. Read this before proposing architecture.
- [docs/research/living-world-game-research.md](docs/research/living-world-game-research.md):
  the research report. Covers shipped games (Stardew, Ultima VI/VII, The Sims,
  Radiant AI, Kingdom Come, Hitman, Dwarf Fortress, RimWorld, CK3), research
  systems (Facade, Comme il Faut / Prom Week / Ensemble, Versu / Praxish, Talk
  of the Town, Kismet, storylets), NPC AI architectures (utility AI, GOAP, HTN,
  behavior trees, schedule and override stacks, smart objects), LLM-driven NPCs,
  and engine and stack options for a headless, agent-driven, web-deployed
  project. Ends with twelve recommendations and eight open decisions.
- [docs/research/generated-game-assets.md](docs/research/generated-game-assets.md):
  research on generating 2D game assets with Google's models and others:
  transparency, reference images, pixel-art tools, a headless Node pipeline,
  costs, licensing. Ends with two candidate pipelines and first experiments.
- [docs/research/world-layout-research.md](docs/research/world-layout-research.md):
  research on how games structure interiors, exteriors and floors, map tooling
  and text formats, procedural layout, presentation, and validation. Includes a
  check of the layout proposal against the evidence.
- [docs/research/engine-deep-dive.md](docs/research/engine-deep-dive.md):
  the renderer and engine question in depth: JS renderers, Unity, Godot,
  Defold and GameMaker run headless, Rust cores and wasm splits, the
  sim/viewer seam and the agent's feedback loop. Recommends Canvas 2D
  verified in Node, gated by a one-week experiment, with PixiJS as fallback.
- [docs/research/rust-stack-and-mobile.md](docs/research/rust-stack-and-mobile.md):
  how to build the Rust simulation crate, keep the Rust/TypeScript boundary
  tight, and reach web and mobile. Ends with a concrete workspace layout,
  crate versions, a CI checklist, a viewer recommendation under the mobile
  constraint, and the first week of work.
- [docs/research/blocked-sources.md](docs/research/blocked-sources.md):
  sources the research could not read, as a checklist to revisit.
- [docs/research/notes/](docs/research/notes/): the raw notes behind the
  report, one file per track, with inline source links and a list of gaps and
  blocked sources per section. See [docs/research/README.md](docs/research/README.md).

## Next

See [docs/design/what-remains.md](docs/design/what-remains.md). The first
code slice (repo setup, discrete-event time with a calendar, places,
buildings, validators, views, then villagers on schedules) is unblocked.
Decided 2026-10-23: the simulation is Rust (compiled to wasm for the
browser), the viewer is TypeScript, web first and mobile probable. The
Rust stack report lays out the workspace, crate versions, boundary protocol
and CI checklist. The viewer library (PixiJS as renderer, Phaser 4 as the
alternative) is decided by a spike that needs a resolution and tile-size
call. The cast and town are placeholders until Ian's own.
