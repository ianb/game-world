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

In a browser at <https://ianb.github.io/game-world/>, published by
`.github/workflows/pages.yml` each time CI passes on main.

Locally, with Rust 1.97, Node 22 and pnpm 10 installed:

```
git clone https://github.com/ianb/game-world && cd game-world
scripts/setup.sh                       # wasm target, just, cargo tools, pnpm install
just build-wasm                        # the sim as wasm, plus the content bundle
pnpm --filter @game-world/viewer dev   # then open the printed localhost URL
```

The page shows the map of one place at a time, filling the window, with
the controls floating over it on wooden panels like a game's: the clock at
the top left (a dial with the sun or moon, the day, pause and four speeds,
also on space and the keys 1 to 4), the places across the top (a `*` marks
one with somebody in it) with the 3D switch, and zoom at the bottom right.
The journal on the right holds the people, days, words and key pages; the
"Journal" button or J folds it away to give the map the whole window, and
on a phone it is a sheet along the bottom. Pan the map with a two-finger
scroll or a drag; zoom with a pinch, ctrl+scroll or the buttons. Zoomed far
out, each person is a dot. The text views are in the tools and in
`window.__scene()`, not on the page. Hover a tile to see what it is in the
card at the bottom left, click to pin it, or click a person (on the map or
in the people page, which filters as you type) to follow them from place to
place: what they are doing and why, coins, needs as bars, character,
thoughts and what they have heard. The typeface is Nunito (SIL Open Font
License), bundled from `@fontsource-variable/nunito`. The days tab pages through
finished days, what was unusual first; click a line to replay the town to
that moment, and "back to now" to return. The words tab lists every topic
and gesture with how often it has come up; open one for who uses it, how
far a gesture carries and how each villager takes it, and each time it
came up. Topics and gestures in the inspector open there too. The 3D
button shows the same place as a lit diorama (three.js): full-height walls,
cut away where they would block the view, villagers as upright sprites that walk smoothly, light that
follows the clock, and speech marks for what is being said. Drag or
two-finger scroll to pan, pinch to zoom, right-drag or shift-drag (or the
turn buttons) to turn. Replaying a moment in 3D moves the camera in close,
and "photograph this day" in the days tab puts a picture beside each
unusual moment. The art is
Liberated Pixel Cup tiles and villagers (CC-BY-SA), composed into
`packages/viewer/public/tiles.png` by `scripts/make-tileset.mjs`
(`just tileset`); sources, credits and how to change an outfit are in
[art/lpc/README.md](./art/lpc/README.md).
Without a browser: `cargo run -p tools -- day --seed 7` prints everyone's
day, `summary --days 14` what was unusual each day, `economy --day 13` the
money, needs and goods, and `png --at 2pm` writes a picture to
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

- [docs/testing.md](docs/testing.md): how behavior tests set up scenes and
  force named chances instead of relying on seeds.
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
