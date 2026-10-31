# Worlds

The repository holds more than one world on one engine. A world is chosen when the page
opens; after that it has its own map, people, content, character programs, art and
docs, and nothing crosses between worlds: no shared timeline, people or saves.

- **valley**: the village this project started with. Code and content in
  `../../worlds/valley/`. Docs: `../design/valley.md`, `../characters/`.
- **welling**: The Welling, a city of lashed boats over a seabed that seeps. Docs:
  `welling/`, starting with Ian's `welling/dossier.md`.

Status: agreed 2026-10-31. Step 1 below is built; steps 2 and 3 are next.

## What is shared and what is not

**Shared engine.** The parts that make any world run, and that a third world (a
generation ship has been mentioned) would want too:

- time, the event queue, RNG streams, named odds, inputs, snapshots, history and replay,
  the state hash;
- the protocol and the viewer shell (map, 3D view, journal, inspector, summaries);
- movement and navigation, needs, conversation, perception and knowledge, moods,
  thoughts, summaries;
- the general systems, each configured per world in content: ground that wears and
  grows, beds of plants that grow and are worked, flocks, timetabled arrivals (the
  valley's train, the Welling's boats), goods, money and stock, the luxuries sink;
- the shared behaviour library: satisfying needs, conversation helpers, mood reactions,
  tending beds, sending and fetching goods. This is where modules will live.

**Per world.** Everything that gives a world its character:

- content: map, buildings or vessels, people, schedules, topics, goods, moods and their
  sources, crops or plants, animals;
- a characters crate: one program per person, and that world's own behaviours;
- art: tileset, dressing, building or vessel styles;
- docs: the dossier, principles, people;
- world-specific systems, when a mechanic really belongs to one world. The rule is to
  try the general version first.

## Layout

**Layout.** One repository and one Cargo workspace. Shared crates stay under `crates/`
(engine, content model, tools, wasm). Each world gets `worlds/<id>/content/` and a
crate `worlds/<id>/characters/`. Docs under `docs/worlds/<id>/`. The valley's existing
content and characters move into `worlds/valley/` in one mechanical commit.

**The content model loses its valley words.** `Crop`, `AnimalKind`, `Flooring`,
`Walling` and the railway are valley vocabulary in the shared model. Each becomes a kind
declared in the world's content, with its rules as data (a crop's growth and yield, an
animal's movement), checked by the validator like topics already are. Mechanics stay
closed enums in the engine (field stages, field work, moods, gestures). Where a
character must decide about every kind (how they take each animal), the world's crate
keeps its own exhaustive enum and checks it against its content when it loads.

**One wasm module, a choice of world.** `sim-wasm` builds every world and dispatches on
the id given at start (`World::Valley(History<valley::Town>)`, then `World::Welling(...)`).
Each world's content bundle is served from `worlds/<id>/content.json`. The viewer opens
the world named by `?world=<id>`, else the last one opened, else the first built.

**Checks per world.** `just check` validates, tests and snapshots every world. The golden
hash and the fixtures are per world (`worlds/<id>/fixtures/`, each world's own snapshots).

**Adding a world** touches: the workspace picks up `worlds/<id>/characters` by itself;
`World` in `crates/tools/src/main.rs` and in `crates/sim-wasm/src/lib.rs` (with
`worlds()`) each gain a variant; `just bundles` loops over the world ids.

## Order

1. Move the valley into `worlds/valley/` and make the wasm and viewer world-aware, with
   the valley as the only world. Nothing visible changes; the snapshots must not move.
   Done 2026-10-31: the characters crate is now `valley`.
2. Generalise the valley words in the content model into declared kinds. Snapshots must
   not move.
3. Add the Welling as a second, small world (see `welling/plan.md`).
