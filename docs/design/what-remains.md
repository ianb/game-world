# What remains to decide and design

Written 2026-10-23, for a first target of a simple Stardew-style town at an
easy scale. Each item has a tag:

- **now**: blocks the first code slice (the layout and simulation core).
- **soon**: blocks the first playable (a town you can walk around in a
  browser with villagers on schedules).
- **later**: can wait until the first playable exists.

Where there is a sensible default I state it. Items marked **Ian** are the
ones only the author can call; the rest can proceed on the default and be
revisited.

## 1. Scope of the first town (soon, Ian)

- How many villagers and buildings. Default: 8 to 12 villagers, 8 to 10
  buildings (homes, a shop, a tavern, a workplace or two, a chapel or hall),
  one outdoor zone with a square, a road out, a bit of countryside.
- What the player can do. Default for the first playable: walk, enter
  buildings, talk (state-keyed lines), observe the clock and the villagers.
  No farming, no inventory, no money.
- Who the player is in the town (see section 7).

## 2. Stack (now)

Already recommended by the research; needs a yes.

- Simulation: a dependency-free TypeScript package, integer ticks, one
  seeded RNG passed explicitly, no wall clock or DOM, per-tick state hash,
  input log as save and replay. Tests in Node with vitest.
- Renderer: PixiJS v8 behind a snapshot interface, or Excalibur pinned to
  0.32.x. Default: PixiJS, because the renderer is thin and the project owns
  its map format anyway. Decide in the first renderer week.
- Package layout: `sim` (world, places, NPCs, time, pathfinding), `content`
  (buildings, villagers, schedules, lines, as typed data), `tools` (views,
  validators, playtest harness), `web` (renderer and UI). pnpm workspace.
- Deployment: static files on Cloudflare Pages. No server until something
  needs one. Keep `sim` free of browser APIs so a Durable Object can run it
  later.

## 3. Time model (now, Ian)

The one decision that touches every schedule and every test.

- Tick length. Default: one tick is one game-second, with 60 ticks per
  game-minute; schedules are written in minutes, movement in ticks.
- Clock speed. Stardew runs a game day (6:00 to 2:00) in about 14 real
  minutes. Default: one game-minute per real second, tunable.
- Day span and sleep. Default: the clock runs 6:00 to midnight; the player
  sleeping or the clock passing midnight ends the day; NPC days are computed
  at dawn as in Stardew.
- Calendar. Default: days of the week only, so schedules can vary by
  weekday. Seasons and festivals later.
- Determinism: the sim never reads real time; the renderer feeds ticks.

## 4. Movement and space (now)

- Tile size in pixels is an art decision (16 or 32). The sim does not care.
- Walking speed. Default: one tile per 10 ticks for NPCs (6 tiles per
  game-minute), player slightly faster. Tune with the travel-time matrix.
- Grid movement, 4-connected for NPCs (Stardew), 8-connected or free for the
  player. Default: tile-to-tile for both, with the player able to change
  direction mid-tile visually.
- Player control (soon, Ian): direct keyboard walking, click-to-walk with
  pathfinding, or both. Default: both; click-to-walk is what makes a
  unified-scale town bearable.
- Doors: walk into the door tile. Locked doors show a message.

## 5. Layout content (now)

Specified in `world-layout.md`. Still to pin down:

- Room shapes: rectangles only for the first slice. L-shapes come when a
  building needs one.
- Door conditions: hours and owner only for the first slice.
- Roof regions: derived from the inline floor's rooms unless overridden.
- Outdoor zone: one place containing the town and nearby countryside.

## 6. Villager model (now for schedules, soon for the rest)

From the research and the sparse-attributes decision.

- Identity: id, name, home place, bed, workplace, a short description, and
  only the attributes that character uses.
- Schedule format: per villager, a list of entries `(day pattern, start
  time, place or object, activity verb, optional dialogue key, optional
  conditions)`, resolved once at dawn by most-specific-key-wins with a
  required fallback. Travel time is computed, so "arrive by" is a flag on
  the entry.
- Activity verbs (now): the engine code each verb runs. Default first set:
  `sleep`, `eat`, `work` (at an object), `tend-shop`, `wander` (within a
  room or place), `visit` (stand near another villager), `idle`. Add verbs
  when a schedule needs one.
- Interruptions (soon): a priority stack with resume. First interrupts:
  the player talking, a locked door, an occupied object.
- Needs and utility (later): the Sims-style gap filler for unscheduled
  time. Not in the first playable; `wander` and `idle` cover the gaps.
- Relationships (soon, minimal): a per-pair ledger of named, timed
  modifiers, bucketed before use. First playable needs only the player's
  standing with each villager, if even that.
- Memory and gossip (later): the tagged world event log plus bounded
  impressions. Not in the first playable.

## 7. The player's place in the town (soon, Ian)

The vision says the point is the living world, and the research says
players need something to do in it. Options:

- Newcomer resident with a house and a daily life of their own (Stardew
  without the farm).
- Observer with a job that sends them everywhere (postal carrier, doctor,
  reporter), which makes watching the town the gameplay.
- Catalyst with limited means to nudge (deliver messages, introduce
  people, carry gossip), the soap-opera participant.

This decides what verbs the player has and what the first storylets are
about. Default if no call is made: newcomer resident, with a small errand
list that gives reasons to visit every building in the first day.

## 8. Dialogue (soon)

- Barks and state-keyed lines: a rule database over a fact bag (place,
  time, activity, weather later, relationship bucket, recent events), most
  specific rule wins, JSON the project owns, schema-validated.
- Authored scenes: ink via inkjs, compiled and played in CI. Not needed
  for the first playable unless the player role (section 7) wants one.
- Voice: Ian writes or approves lines; the agent drafts into the schema.
  Carry over the prose rule from the intra-game repo: game text is the
  author's voice.
- Minimum for the first playable: each villager has a greeting per activity
  and a couple of state-keyed variants.

## 9. Drama between villagers (later, Ian decides when)

Authored, role-agnostic exchanges (ask out, confide, snub, make up) that
cast eligible pairs, with volition from a short rule list and outcomes that
plant hooks. Dozens of rules per exchange, not thousands. A drama throttle
in the Left 4 Dead intensity shape. Decision needed only on whether any of
it is in the first playable. Default: none; the first playable is schedules,
movement and talk.

## 10. LLM boundary (decided enough)

Zero runtime calls in the first playable. Authoring-time generation into
schemas with validators and a human voice read. Revisit when authored
scenes exist.

## 11. Views, tests and the playtest harness (now)

- Views: place at time T, town at time T, travel-time matrix, one-villager
  itinerary, door listing, roof regions. Text, snapshot-tested.
- Playtest harness: run N days headless from a seed, dump every villager's
  itinerary and every invariant violation. This is the thing the agent runs
  after every content change.
- Invariants: the validator table in `world-layout.md`, plus "every
  villager sleeps in their own bed every night" and "no villager is idle
  more than X minutes unless scheduled."
- Browser checks (soon): a few Playwright screenshots gated on a WebGL
  context assertion.

## 12. Art (soon, Ian)

From the asset research.

- Style: pixel art at 16 or 32 px, or painted. Decides Pipeline A or B.
- First playable: placeholder art. Default: Kenney CC0 tiles and
  characters, or LPC bodies for walk cycles, so the town is walkable before
  any generation runs.
- Palette: a fixed project palette before generation starts.
- Generation experiments need a Gemini key (and optionally a Retro
  Diffusion balance) as environment secrets.

## 13. Rendering details (soon)

- Camera: follows the player, clamped to place bounds; snaps on scene swap
  with a fade.
- Depth: tile layers below and above, actors and tall props sorted by foot
  Y as sprites.
- Roof hiding: one alpha group per inline building, toggled by the player's
  room.
- UI: a clock, a day label, a dialogue box, and a debug overlay that shows
  the text views in the browser (the agent cannot see pixels, so the overlay
  is also for Playwright).

## 14. Repo conventions (now)

- A `CLAUDE.md` for this repo: the prose rules (game text is the author's
  voice; prompts, if any, written flat; engineering prose light), how
  decisions are recorded, what to run before committing.
- `docs/decisions.md` stays the log. Design docs per area under
  `docs/design/`.
- Tests and typecheck before every commit; the playtest harness before
  every content commit.

## Suggested order

1. Repo setup: pnpm workspace, `sim` and `tools` packages, vitest,
   `CLAUDE.md`.
2. Time, places, rooms, doors, buildings, the compiler to tiles, the
   validators, the views.
3. Three buildings and a square; the travel-time matrix and itinerary.
4. Villagers with schedules and the first activity verbs; the playtest
   harness; a full town of 8 to 12.
5. Renderer with placeholder art, roof hiding, player walking and
   click-to-walk, the debug overlay; deploy to Cloudflare Pages.
6. Talk: the rule database and first lines (Ian's voice).
7. Then art generation experiments, relationships, interruptions, and the
   first drama exchange, in whatever order the playable suggests.

Steps 1 to 4 are text-only and need no decisions beyond the time model and
the stack yes. Step 5 needs the art style and player control. Step 6 needs
the player's role.
