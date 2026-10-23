# Decisions

Design calls Ian has made. Each one is a short statement, who made it, and
what it implies. Add to the bottom; do not rewrite earlier entries, strike
them and add a new one if a call is reversed.

## 2026-10-22: NPC attributes are diverse but sparse

Ian: there is a place for diverse but sparse NPC attributes. The thug might
track anger a lot, but anger need not be universally accounted for.

Implications:

- There is no fixed, universal stat block that every character carries. A
  character has the attributes that matter to that character's behavior and
  story, and nothing else.
- Rules and scripts that read an attribute must tolerate its absence (treat a
  missing attribute as "this character does not care about this").
- This is the opposite of the Dwarf Fortress approach (every dwarf has all 50
  facets) and closer to Crusader Kings' small trait sets. The research notes
  that the many-facet approach is illegible to players; sparse attributes
  also keep each character's script small enough to read.

## 2026-10-22: Inventory and resource contention only where it is interesting

Ian: we can be a little sloppy on inventory and stuff. A character may need to
get supplies for an activity, but we can conjure things in and out of
existence. If contention is not interesting, do not implement it.

Implications:

- "Fetch supplies" is a schedule beat (go to the shop, come back), not a
  tracked transfer of items. Items exist when a script needs them.
- Capacity and contention are opt-in per place or object, used only when a
  conflict is something the player can notice and care about (two people want
  the same seat at the bar because that is a scene). The research's canonical
  contention bugs (Shenmue's shop queue, Kingdom Come running out of beds) are
  a reason to leave contention off by default, not a reason to model it
  everywhere.

## 2026-10-22: Text rendition first

Ian: definitely a text rendition of things. Easy to test, easy for the agent
to inspect.

Implications:

- The simulation has a text view from the start: a map dump, a per-character
  itinerary for the day, a "where is everyone at 14:00" table, and a log of
  what happened and why. These are the primary debugging and testing surface.
- Tests assert on text and on invariants, not on pixels. Graphics come later
  and sit on top of the same state.
- This matches the report's stack recommendation (dependency-free simulation
  core, tested in Node, rendered by a thin layer) and the testing and
  inspection approach described there.

## 2026-10-22: Investigate generated art, starting with Google's models

Ian: game assets are hard. Investigate using Google's image models.
Transparency (alpha output) is very useful. Reference images for consistency
are very important.

Status: direction, not yet a commitment. The research is in
`docs/research/generated-game-assets.md`. Its findings that bear on the
direction:

- No Google model outputs alpha; the working recipe is a solid chroma
  background plus a light outline, keyed in Node. Reference images are well
  supported (up to 14 on the current default model).
- Pixel-native tools (Retro Diffusion, PixelLab) cover the things Google is
  weakest at: pixel grids, seamless tiles, walk cycles.
- No image API key exists in the development container. A Gemini key must be
  added as an environment secret before the listed experiments can run.

## 2026-10-23: Simulate everyone on real tiles now; scaling tricks later

Ian: for scaling up the size we would need things like teleport, or even
just abbreviating all action when no one is around. That can probably be
added later.

Implications:

- The first build simulates every NPC on real tiles in every place every
  tick, as Stardew does.
- The architecture keeps the door for later LOD open: the travel-time matrix
  and the place graph already exist, so "teleport along the route after the
  travel time" or "skip ticks in places with no observer" can be added
  without re-authoring content.

## 2026-10-23: Source is for the agent, not humans; tooling is unlimited

Ian: source legibility is only for the agent, not humans, and can include
any amount of tooling to understand. Template-string grids are cute but not
necessary or particularly helpful for agent authoring.

Implications:

- The map source is structured, typed data (rooms as rectangles, doors and
  objects as records with coordinates), not hand-drawn ASCII. The agent is
  better at writing coordinates and rectangles than at counting characters
  in a grid.
- ASCII, tables, itineraries and PNGs are generated views for inspection,
  never the source of truth. Any number of such tools can exist.
- The strict-parser concerns from the research (ragged rows, unknown glyphs)
  mostly disappear; what remains is schema validation of the structured
  data.

## 2026-10-23: Zones yes, single scale, outdoors mostly continuous

Ian: zones are fine, but I want a single scale. Outdoors have limited zone
opportunities.

Reading taken here, corrected the same day (see the next entry): "single
scale" was misread as interiors fitting their footprints. It means one tile
scale across town and countryside.

Implications:

- Buildings are sized to hold their interiors. The footprint-fit check comes
  back as a default validator, not a reported ratio.
- Interior places exist for the door graph and scene swap, not to gain
  space. A small cottage is a small room.
- Outdoors is one large place, or a few, split only where geography gives a
  natural edge (a river, a forest line, a road out of town). Zones are
  mainly interiors and floors.
- This makes a later continuous exterior with visible interiors (the Ultima
  VII look) cheaper, since interiors already match the footprints.

## 2026-10-23: Correction: larger-inside interiors are fine; no overworld scale

Ian: tesseracts (interiors larger than their footprints) are fine. What is
not wanted is Ultima IV's two scales, a town map at one scale and a
countryside map where a tile is a mile.

Implications:

- Interiors may exceed their footprints, as in Stardew. No footprint-fit
  validator. The render may report the ratio for interest.
- The countryside is at the same tile scale as the town. There is no
  overworld map and no scale change when leaving town. Walking to the next
  farm is walking the tiles.
- This makes the outdoor world large in tiles, which is the reason outdoor
  zones exist at all: a river, a forest line or a road can split the
  outdoors into places joined by edge records, all at one scale.

## 2026-10-23: Open: unified (Ultima VI) versus detached (Stardew) interiors

Ian: an actual unified scale and layout as in Ultima VI could be nice and
allows forms like a carport without trouble. The scale is awkward: Ultima
towns felt large and sprawling next to the countryside, and you spend a lot
of time walking around the outside of buildings. True in real life, but not
in a fun way.

Status: decided the same day; see the next entry.

## 2026-10-23: Inline by default; detached for specific reasons

Ian: that sounds like a good approach. Keep detached, but use it for
specific reasons.

Decision: the world is unified at one scale. Ground floors are inline on the
exterior grid at footprint scale, with roofs hidden when the player is
inside. Detached places remain in the model and are used only where there is
a reason, each reason stated in the building's source.

Reasons that justify a detached place:

- Upper floors and cellars (stairs cut to a new scene; avoids a Z axis).
- An interior that must be larger than its footprint for gameplay (a tavern
  hall, a workshop floor), where the exterior would otherwise sprawl.
- A space that is a different world in feel (a cave, a dream, a shrine).
- Anything the renderer cannot show inline without special work.

Implications:

- The exterior is one continuous grid per outdoor zone, and the town is
  built dense: shared walls, alleys, back doors, gates and cut-throughs.
- The renderer owes roof hiding per building from the first visual build.
  Scene swap stays for detached places.
- Every detached floor carries a `detached: "<reason>"` field, so a reviewer
  can see why it is not inline. The validator warns on a detached ground
  floor with no reason.
- NPC legibility is mostly solved by the continuous exterior; map markers
  or a notebook are now a nice-to-have rather than a first-playable budget
  item.
- The travel-time matrix and itinerary tools still measure the walking
  cost; they tune density and speed rather than decide the model.

## 2026-10-23: First-town decisions

Ian, answering the list in `design/what-remains.md`:

**Time.** A calendar, of course. Deterministic ticks. Probably a scheduler
with events scheduled for the future; time could be by the second, but the
simulation does not revisit every entity every second.

Implications:

- The simulation core is a discrete-event scheduler: a priority queue of
  future events keyed by tick, with a sequence number as tie-breaker so
  ordering is deterministic. Entities schedule their own next wake-up
  (arrive at the next tile, finish eating, shop opens) rather than being
  polled. Player input enters as an event at a tick.
- Tick resolution is one game-second. Nothing runs per tick unless an event
  is due. The renderer interpolates movement between tile-arrival events.
- Calendar from the start: date, weekday, season, year. Schedules key on
  all of them, most-specific wins.
- "Simulate everyone on real tiles" stands, but by events, not by polling.
  Scaling tricks (skip or coarsen events in unobserved places) slot in as
  event policies later.

**Player.** Mostly a newcomer observer. It might end up like The Sims, an
omniscient observer.

Implications:

- The camera and the player avatar are separate things from the first
  renderer build. The camera can follow the avatar, a villager, or a place.
- The avatar is optional in the sim: the world runs without one. "Talk" and
  other player verbs attach to the avatar when present.
- Legibility tools (where is everyone, what are they doing) matter more
  than under a pure avatar model and should be in the first playable as
  views, even before they have UI.

**Scope.** 8 to 12 villagers, 8 to 10 buildings, one outdoor zone. Fine.

**Player control.** Both keyboard walking and click-to-walk.

**Art.** Pixel art to start. That selects the pixel-native pipeline (A) in
the asset research for generated art, and placeholder pixel packs before
that.

**NPC action and decisions.** Developing action and villager decision-making
(needs, utility, interruptions, drama) come later. First playable is
schedules and movement.

**Stack.** The underlying simulation design is agreed. The renderer is not:
PixiJS is a maybe. Renderer choice stays open until the renderer step, with
raw Canvas 2D at integer scale added to the candidates since pixel art
needs little from a library.

**Dialogue.** Comes later, and might be LLM-driven from the start rather
than a rule database. When it comes, the research constraints apply: the
engine owns state, the model returns text plus a closed set of flags,
grounded in the fact bag, with a per-scene budget.

**Headless testing.** Lots of options; no decision needed.

## 2026-10-23: Rust simulation, TypeScript viewer, web for sure, mobile probably

Ian: Rust makes sense for the simulation, and it is an experiment worth
running since there has been no agent project in Rust before. The
two-language barrier is not a big worry; it needs processes to keep it tight
and consistent, which is doable. Rust sim with a TS viewer is reasonable.
Deploy to web for sure, and probably mobile.

Background: Ian's stated taste is for highly typed, deterministic systems
rather than libraries built for casual experimental development, and the
impression (self-described as based on little) that PixiJS is clever and
simple rather than robust. The agent-benchmark evidence in
`research/engine-deep-dive.md` (Rust at or above TypeScript for coding
agents, 2025 figures) points the same way.

Decided:

- The simulation core is a Rust crate: no wall clock, no ambient
  randomness, ordered collections, one seeded portable RNG, a discrete-event
  scheduler, serde for content and snapshots, `insta` and `proptest` for
  tests. It compiles to wasm for the browser and runs natively for tests
  and tools.
- The viewer is TypeScript. Which library is still open; the Canvas 2D
  recommendation is withdrawn in favour of whatever best serves web and
  mobile (see the next research track).
- The Rust/TS boundary is a first-class engineering concern with explicit
  processes: generated types (ts-rs or typeshare), a typed event stream as
  the only direction of flow from sim to viewer and typed inputs the other
  way, shared fixtures checked by both `insta` and vitest, pinned toolchain
  versions, one build entry point, and CI that fails on drift.
- Deployment: web first. Mobile is probable, so the viewer and input design
  assume touch from the start (click-to-walk is already the primary
  control), integer-scaled pixel art on small screens, and a packaging path
  (PWA, Capacitor, or a native shell) to be chosen from research.

Implications for the build order in `design/what-remains.md`: steps 1 to 4
(repo setup, time and calendar, places and buildings with validators and
views, villagers on schedules) move to Rust. The text views and playtest
harness are Rust binaries. The content format is serde-backed (RON or JSON
with schemas) rather than TypeScript literals.
