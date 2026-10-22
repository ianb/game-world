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

## Status

Research phase. Nothing is decided about the stack yet. There is no code.

## What is here

- [docs/research/living-world-game-research.md](docs/research/living-world-game-research.md):
  the research report. Covers shipped games (Stardew, Ultima VI/VII, The Sims,
  Radiant AI, Kingdom Come, Hitman, Dwarf Fortress, RimWorld, CK3), research
  systems (Facade, Comme il Faut / Prom Week / Ensemble, Versu / Praxish, Talk
  of the Town, Kismet, storylets), NPC AI architectures (utility AI, GOAP, HTN,
  behavior trees, schedule and override stacks, smart objects), LLM-driven NPCs,
  and engine and stack options for a headless, agent-driven, web-deployed
  project. Ends with twelve recommendations and eight open decisions.
- [docs/research/blocked-sources.md](docs/research/blocked-sources.md):
  sources the research could not read, as a checklist to revisit.
- [docs/research/notes/](docs/research/notes/): the raw notes behind the
  report, one file per track, with inline source links and a list of gaps and
  blocked sources per section. See [docs/research/README.md](docs/research/README.md).

## Next

Read the recommendations and open decisions at the end of the report and
decide the first few: renderer, schedule authoring unit, and where the LLM
boundary sits. Then the first milestone is a simulation core with text-only
tests, before any rendering.
