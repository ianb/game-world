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

## 2026-10-23: Live multiuser eventually; inputs in, effects out

Ian: eventually live multiuser is wanted. That adds to the benefit of
client/server, with player inputs feeding into the simulation and effects
coming out.

Status: direction, not scheduled. It costs almost nothing to honour now
because the sim is already a deterministic core that consumes an input log
and emits typed events. What it fixes in place from the start:

- Inputs are the only way anything affects the sim, and every input carries
  a player id and is stamped with the tick the server (or the local loop)
  assigns. No viewer-side shortcuts into state.
- The viewer talks to a `Sim` interface (step, apply input, events out,
  snapshot, hash) that can be backed by local wasm or by a network
  connection. The viewer never learns which.
- The sim supports N avatars, not one. The observer-player decision already
  made the avatar optional; this makes it plural.
- Time is a property of the world, not of a player. Pause and fast-forward
  are single-player conveniences; a shared town runs at wall-clock pace on
  the server.
- Snapshot and load are the late-joiner mechanism, so they stay first-class
  rather than a save-game afterthought.
- Events are the wire format as well as the viewer format, so they stay
  small, integer-only and serializable.

See `design/multiplayer.md` for the shape and the choice to make later.

## 2026-10-23: LLM dialogue is an external actor, not part of the sim

Ian: LLM dialogue would probably be another entity external to the sim,
which receives inputs and produces responses asynchronously.

Decided as the shape (the LLM itself is still "later, maybe"):

- The sim is synchronous and pure. Anything slow, external or
  nondeterministic (a model call, a network peer, a future director) lives
  outside it as an actor that subscribes to the event stream and submits
  inputs, exactly like a player. This is the AI Town lesson from the
  research: model calls are asynchronous operations that can only submit
  inputs.
- Replay stays deterministic because the actor's responses enter the input
  log. A replay never calls the model; it replays what the model said.
- Inputs carry provenance (player id, or actor id such as
  `villager-brain`, `director`), so the sim can apply different rules to
  each source and tooling can show who caused what.
- The sim emits request events when it wants something from an external
  actor ("villager X is in situation S with facts F; a line is needed") and
  must cope with a late, missing or invalid response: the conversation may
  have moved on, so the sim validates every input against current state and
  drops what no longer applies.
- The same door serves scripted brains. A villager's decision-making can be
  in-sim (fast, deterministic) or external (slow, model-backed) without the
  sim knowing which, which is how "mostly scripts, LLM for high-plot
  moments" gets implemented.

## 2026-10-23: Characters are code first; data is the baseline

Ian: a lot of ad hoc and eclectic code will be needed. Every character may
have hundreds of lines custom to that character rather than being entirely
data-driven. There is no reason not to give rich code to each character, or
a big toolbox that can be put together in ways for the characters. Might
this be hard with Rust?

Decided:

- A character is a Rust module implementing a `Brain` trait, with as much
  bespoke code as it needs. The schedule data format from the research is
  the baseline a character's code extends or overrides, not the whole model.
- Character code is written against a toolbox: read-only queries through a
  context (`&Ctx`: where am I, who is nearby, time, what I know, relationship
  bucket) and actions as queued commands (go to, wait, say, schedule a
  wake-up, start an exchange). Character code never mutates world state
  directly. This is what keeps the borrow checker quiet, keeps replay
  deterministic, and keeps the toolbox the single place behaviours are
  implemented.
- The workspace gains a `characters` crate (one module per villager plus the
  shared toolbox) beside `content` (places, buildings, baseline schedules as
  data). The toolbox is a first-class deliverable that grows from what
  character code wants to say.
- Escape hatch, only if the week-two spike shows real friction: an embedded
  scripting language inside the sim (Rhai, pure Rust, wasm-capable; or Lua
  via mlua). Either keeps character logic inside the deterministic core.
  External actors are not the place for ordinary character logic, because
  tests and replay would then depend on logged responses instead of code.

Check: in week two, write two characters this way (one mostly data, one
mostly code with eclectic behaviour) and judge the friction before writing
twelve.

## 2026-10-23: Typed struct per character; a functional language if any

Ian: intra-game used a class per distinct entity, which let code touch other
entities and still be type-checked. Is that possible here? And if a language
is embedded, it can and should be functional, since the code has to be more
or less functional to fit the system.

Refinement of the characters decision:

- Each character is its own struct type whose fields are that character's
  sparse attributes (`Thug { anger: u8, .. }`, `Mara { worried_about_rent:
  bool, .. }`). The world holds them as named typed fields
  (`Characters { mara: Mara, thug: Thug, .. }`), so one character's code can
  read another's fields and the compiler checks it.
- Reads across characters are typed field access through `&Ctx`. Writes go
  through commands, which may be typed per character (`ThugCmd::CalmDown`).
  Dispatch over all characters is an enum or generated match, never a trait
  object, so no fields are erased. Shared behaviour is a trait the relevant
  structs implement.
- Adding a character touches one struct field and one match arm.
- A brain is a pure function `(state, ctx) -> commands`, so the system is
  already functional in shape. If an embedded language is ever wanted it
  should be functional too. Unresearched candidates: Steel (Scheme in Rust),
  Fennel over mlua, Koto, Gluon (maintenance unknown). Rhai is imperative
  and drops down the list. Research only if the week-two spike shows need.

## 2026-10-23: Character spike verdict: plain Rust modules, low friction

The week-two check from the characters decision ran early, with Mara (mostly
data) and Tom (eclectic code) in `crates/characters`. Verdict: write the
rest the same way; no scripting layer is needed.

What worked:

- A struct per villager with only that villager's fields (`Tom { mood,
  days_since_day_off, fish_caught }`, `Mara { batches_baked,
  pastries_delivered }`), held by name in `Town`. Mara's code reads
  `tom.mood` as a typed field. Destructuring `Town` into its fields in the
  `CharacterSet` match gives each character `&mut` to itself and `&` to the
  peers it names, with no borrow trouble.
- Peers are explicit arguments (`mara.plan_day(ctx, rng, tom)`), so which
  characters depend on which is visible in the signature and checked by the
  compiler. This is the sparse-attributes idea applied to relationships.
- The toolbox (`Plan::baseline`, `at`, `drop_activity`, `shift`, `go`,
  `go_tile`, `wait_minutes`, `together`, `activity_of`, `pick`) covered both
  characters in about 120 lines. It grows from what characters want to say.
- Reactive behaviour through `react` plus `Wait` commands (Tom's 20:00
  check on the room, fishing in randomized stretches) stayed deterministic
  and snapshot-safe because all randomness comes from the stream handed in.
- Character state is in the snapshot and the hash for free, via the generic
  `SimState<C>`.

Rough edges to fix as characters multiply:

- Dispatch matches on the villager's string id. An enum of villagers
  generated from content, or a const per module, would make a typo a
  compile error.
- Toolbox lookups panic on an unknown object or place id. Acceptable while
  the validator guarantees content ids, but a `Ctx::must_object` with a
  clearer message is cheap.
- `react` cannot tell which `Wait` elapsed when a character has several
  kinds; a tag on `Wait` (or matching on the current activity, as Tom does)
  is needed once that happens.

## 2026-10-23: Tile costs and cowpaths, 12-hour time, richer text views

Ian: "We should probably have a concept of score for traveling on tiles.
That makes paths the correct thing to traverse on, when available, much
like for people. (Should lead to cowpaths too!) I prefer 12 hour time. You
could use more full unicode and colors to make this better, it's not that
easy to parse visually."

Decided:

- Every walkable cell has an integer step cost. Ground is 4, a laid path
  (`Cell::Path`, authored as `paths` rects on a zone) and every indoor cell
  is 1, door crossings are 0. Routing is Dijkstra between door ends and A*
  within a place, both over cost rather than step count, so a villager takes
  the road when it is there and cuts across grass only when that is cheaper.
- Footfall wears the ground. Each step onto a Ground tile adds one to that
  tile's wear; at 24 it is a track (cost 2), at 80 a worn path (cost 1), and
  it loses 2 a day. Wear lives in `SimState.wear` and is in the hash, so
  cowpaths are deterministic and replay. A path villagers choose makes
  itself cheaper, which is how a cowpath forms.
- Time is shown as 12-hour everywhere (`7:15am`, `11:30pm`). Content may
  write times either way (`7:15am`, `19:15`).
- Text views use box and shade glyphs (`·` ground, `░` path, `∙` track,
  `▒` worn path, `♣` blocked, `█` wall, `▫` opening, `▪` door) and colour:
  ANSI in the tools when stdout is a terminal (or `--color`), CSS spans by
  glyph in the viewer. Plain text is still the source of truth; colour is
  added at the edge, so snapshots and the scene dump stay plain.

Implications: the protocol version is 7 (`Cell` gained `Path`). Costs
are integers because the sim has no floats. Wear thresholds are constants
in `crates/sim/src/sim.rs`; tuning them changes every golden hash.

## 2026-10-23: Grass grows back, and somebody has to cut it

Ian: "Seems like grass and possibly weeds should grow day to day, recovering
the cowpaths potentially, and leading to maintenance needs as things may
overgrow."

Decided:

- Every outdoor tile carries two counters in `SimState.terrain`: `wear`
  (footfall, as before) and `growth` (days since the tile was last trampled
  or cleared). Growth rises by 1 a day on grass and laid paths, drops by 4
  per step, and is capped at 60. At 7 grass is tall (cost 6) and a path is
  weedy (cost 2); at 28 both are overgrown (brush, cost 9; an overgrown
  path, cost 4). Growth wins over wear: a disused cowpath grows over in a
  week even though its wear takes longer to fade.
- Maintenance is a command. `Command::Clear { radius }` zeroes growth
  around the villager and emits `Cleared`; the character pairs it with a
  `Wait` for the time the work takes. `Ctx::overgrown_tiles` and
  `Ctx::nearest_overgrown(place, area, from)` let a character see what
  needs doing. Tom clears the tavern's yard before opening, a few patches a
  morning; the rest of the green is nobody's job and goes wild, which is the
  intended look: trodden routes and kept yards, meadow beyond.
- Surfaces are a boundary type (`protocol::Surface`) with a `TerrainView`
  per place, so a graphical renderer gets the same picture the text views
  do. Glyphs: `"` tall grass, `┈` weedy path, `♠` brush or an overgrown
  path.

Implications: protocol 8, snapshot format 3, character layout 2. The
counters are dense per zone (two bytes a tile), which is fine at this scale
and would want a sparse form for a very large outdoors. Pip's random
roaming tramples the green into a patchwork; a wandering routine that
prefers paths would change the look, and is a character decision, not a
terrain one.

## 2026-10-23: Real tiles in the viewer, placeholder art first

Ian: "Also, we need to make some real tiles and stuff, not just text
characters."

Decided:

- The viewer draws each place on a canvas from the sim's views: cells and
  surfaces as 16 px tiles, objects and villagers as sprites, names as
  labels. Canvas 2D at native size with integer CSS upscaling and
  `image-rendering: pixelated`; no renderer library yet. This is the
  renderer spike the design doc deferred, done small; whether PixiJS is
  needed later is decided when lighting, roofs or many sprites ask for it.
- The art is a placeholder tileset drawn procedurally by
  `scripts/make-tileset.mjs` into `packages/viewer/public/tiles.png` with
  an atlas in `tiles.json`. Both are committed. The atlas names (`grass`,
  `tall_grass`, `cowpath`, `obj_bed`, `villager_0`, ...) are the contract
  with the art pipeline: generated art replaces the PNG and keeps the names.
- Zones gained `water` rects (`Cell::Water`, glyph `≈`), impassable like
  `blocked` but drawn as water. The pond is water; `blocked` draws as trees.
- The text views stay the source of truth. The browser smoke checks a
  canvas exists per place and reads the text scene through
  `window.__scene()`; nothing asserts on pixels.

Implications: protocol 9 (`Cell` gained `Water`, and the version was not
bumped for the terrain change, so this bump covers both). Each frame redraws
every tile of every place; fine at this size, and a dirty-tile cache is the
first optimisation when it is not. Villager sprites snap to tiles; the view
has `next_tile` for interpolation when walking should look smooth.

## 2026-10-23: Auto-shaped tiles over a background; no player yet

Ian: "Can we put some auto-shaping on the tiles? Like the lake shouldn't
have square corners. This might require a sense of 'background', like the
background of the lake is the grass, in a building the background is a
floor. And now that we're using images, can we make the tiles square? I'm
comfortable with simulation without a player for now, so we'll keep going
that way."

Decided:

- Every cell has a background: grass outdoors, floor indoors. Features
  (water, the canopy that `blocked` draws as, paths of every kind, tall
  grass, brush) draw over it in four 8 px quadrants, each shaped by whether
  the feature continues to that quadrant's two cardinal neighbours and the
  diagonal: full, an edge, an outer corner (rounded) or an inner corner
  (notched). Features that join (`path` with `cowpath` and `track`, `tall
  grass` with `brush`) share a group. Off the map counts as the feature, so
  the border forest runs off the edge. Walls, floors and doors stay plain.
- The tileset carries five shape tiles per feature (`water_full`,
  `water_h`, `water_v`, `water_out`, `water_in`), each holding its four
  quadrant variants; the renderer reads sub-rects. Generated art fills the
  same names.
- Tiles were already square in the atlas; the page squashed them because
  the canvas kept a fixed CSS height while its width shrank. The canvas now
  sets only its width and the places span the full page width.
- `scripts/viewer-shot.mjs` screenshots one place after running the sim
  forward some days, at 2x, through a `window.__advance(days)` hook. It is
  how the agent looks at the tiles.
- No player for now: the game is a simulation to watch. Player walking,
  click-to-walk and roof hiding by the player's room come off the near list.

Implications: the renderer does nine neighbour lookups per feature tile,
still cheap at this size. Inner corners are notched rather than rounded
inward, which reads fine for water and paths. Wall shaping (a top face,
corners) is the next thing that would make buildings look right.

## 2026-10-23: An inspector in the viewer

Ian: "I'd like to be able to mouse over or maybe click things and learn
about them (type of tile, any stats)." And: "should be able to click on a
name and see info on the person too."

Decided:

- Hovering a tile on any place shows its cell or surface, step cost, wear
  and growth, any object on it with its goods, and any villager on it.
  Clicking pins: a villager if one stands there, else the tile. The
  who-is-where list is buttons; clicking a name pins that person, and the
  panel follows them. The pinned or hovered tile is outlined on the map.
- The data comes from the views, not from the page: `TerrainView` now
  carries wear, growth and cost per tile, `VillagerView` carries coins and
  needs, and `economy()` lists goods on objects. Anything the inspector
  shows is something the sim already exposes across the boundary.

Implications: protocol 10. The inspector reads the whole terrain view of a
place on every frame it is open; fine here, a per-tile call if places get
large. Touch works the same as a click (pointer events); there is no hover
on a phone, so pinning is the way in.

## 2026-10-23: A small economy, and a town big enough to hold it

Ian: "We need to expand the town some so it can have some kind of economy.
Even if it's like... pensioners that receive money, some services, some
supplies entering."

Decided:

- Money is a count of coins on each villager, in the state and the hash.
  Goods are counts by name on objects (`SimState.stock`), sloppy by design:
  no weights, no capacities, no inventory on people. Content declares
  starting coins on villagers and starting stock on objects.
- Two commands move things: `Pay { from, to, amount, what }` between
  villagers or the world outside the town (`Party::Outside`), paying what
  the payer has when short; `Stock { object, good, delta }`, never below
  zero. Both emit events (`Paid`, `Stocked`) that the itinerary prints and
  the `economy` text view and the inspector summarise. Character code
  decides when to pay and what to stock; the sim only keeps the books.
- The loop: a pension (from outside) to Nell weekly; Nell buys bread from
  Mara and ale from Tom; Hal the carter goes out along the road twice a
  week, pays the market (to outside) for flour and ale, sells them to Mara
  and Tom; Mara bakes flour into bread when the shelf is low and buys ale;
  Tom buys bread for the bar and sells ale and lunch; Pip gets a bun free.
  Money in equals money out at 20 a week, so the town's total holds; what
  drifts is who has it. Prices are constants in the character modules.
- The town grew to 48 by 36 with a cottage (Nell), a cart yard (Hal), a
  well and a bench on the square, three privies, a copse, and the street
  continued east to the edge as the road out. Zones can place objects
  directly (`objects` in zone coordinates, on ground or path).
- Nell, Hal, the cottage and the cart yard are placeholders in my words,
  like Mara, Tom and Pip. Rename and rewrite at will.

Implications: tuning is by watching `cargo run -p tools -- economy --day N`
over weeks, not by reasoning about it. The first version collapsed in four
weeks (Hal could not afford loads once Tom was short, and the shortfall
cascaded); half loads when short, a margin on ale and a cheaper lunch made
it hold for twelve weeks of seed 1. Stock still creeps up slowly, which a
bake that skips when the shelf is full only partly answers. Snapshot 4,
character layout 3.

## 2026-10-23: Bodies

Ian: "Let's put in some basic physical functions too, food, water,
sleepiness, bathroom. A bit stochastic."

Decided:

- Four needs on every villager (hunger, thirst, sleep, bladder), 0 to 100,
  in the state and the hash. Once an hour each body ticks, staggered a few
  minutes apart per villager; each need rises by its base rate plus a roll
  from a new `body` RNG stream, half as fast while asleep. Sleep repays
  while sleeping. Eating zeroes hunger, drinking zeroes thirst, the privy
  zeroes bladder, applied when the activity starts.
- At 70 and 90 the character gets `Trigger::Need`; hunger, thirst and
  bladder keep asking hourly while above 90, so a need that got urgent
  overnight is dealt with in the morning. The sim never acts on a need by
  itself; the toolbox's `satisfy` is the baseline answer (the nearest
  privy, the well, a loaf from the bakery or the larder at home when urgent
  and broke, a yawn for sleep) and every character currently uses it.
- Errands come back. `Command::Interlude { target, activity, minutes }`
  goes somewhere, does the thing for a while, walks back to the spot and
  activity the villager left, re-arms any wait it interrupted, and fires
  `Resumed` instead of `Arrived` so purchase-on-arrival code does not run
  twice. Privies and the well are zone objects next to buildings.

Implications: needs are sim-owned because they are universal, unlike
Tom's mood; a character can still override `satisfy`. Rates are constants
in `crates/sim/src/needs.rs`. The hourly body tick is the first thing in
the sim that runs on a clock rather than on a schedule entry; it is one
event per villager per hour, so cheap.

## 2026-10-24: Summaries derived from events, with causes and replay

Ian brought design notes from another session (now in
`docs/design/living-world.md`): the next step is summaries, derived from
the event series rather than logged, reporting deviations from routine,
with causes on events because those cannot be added later, and with a way
to go back and watch a flagged moment. Replay needs per-character RNG
streams, behavior versioning alongside snapshots, and observe-only
instrumentation.

Decided:

- Summaries are code over events (`summary.rs`), fed by a `History` that
  lives outside the sim's state. Routine is decided by key frequency over
  the previous week and the same weekday last week; habits that did not
  happen are news. Summary text is flat tool output, not game text; if it
  ever reaches a player it needs Ian's voice.
- Events carry a `Cause`. Day plans are recorded with schedule items marked
  and dropped schedule items listed, so "Tom changed the day: no tend-bar;
  added fish" comes straight from the event.
- `History` snapshots each day start; the viewer and the tools can rewind to
  any summarised moment and step back to the frontier. Inputs are refused
  while in the past.
- RNG streams are per villager (mind, body) and seeded by villager id.
- A build-time fingerprint of the behavior code goes in every snapshot, and
  a snapshot from other code is refused.

Implications: protocol 12, snapshot 5. Every golden hash changed because
the streams changed. `Cause` is externally tagged because postcard cannot
decode internally tagged enums and causes are in the snapshot. Chained
causes (goal, plot) wait for goals and plots to exist. The fingerprint
changes on any edit to those crates, comments included; that is the
conservative choice.

## 2026-10-24: No zooming out

Ian: "Don't do any of the zoom out stuff."

Decided: no levels of detail. Every villager in every place is simulated
at full detail. Nothing coarsens unobserved places, teleports travellers on
a travel-time estimate, stands in closed-form outcomes for a zoomed-out
region, or calibrates such outcomes against fine runs. This replaces the
"scaling tricks later" door left open in "Simulate everyone on real tiles
now; scaling tricks later" (2026-10-23) and the scale section of
`docs/design/living-world.md`.

Implications: the size of the world is bounded by what the full sim can run
at the speeds the viewer and the summaries need. The LLM plot layer, when
it comes, works at one granularity. If the world outgrows full detail, that
is a new conversation, not an existing plan to pick up.

## 2026-10-24: Tom keeps his schedule unless he is starving

Ian: "Would be great if Tom reacted to extreme hunger, while usually
sticking to a schedule."

Decided:

- Tom has his own hunger rule instead of the toolbox default (which buys
  bread as soon as hunger reaches 70). Below 90 he notes it and keeps to his
  schedule. At 90 or more, unless a meal is already under way or planned
  within the hour, he eats: bread kept behind the bar, else a loaf from the
  bakery, else the larder at home. Closing up at 10pm with hunger of 80 or
  more, he eats before bed rather than sleep through starving.
- The bar keeps bread now. Opening the bar, Tom buys a loaf from the bakery
  when the bar has fewer than two, instead of paying for one every day.
- Needs are asked on waking: anything that got pressing overnight triggers
  when the villager stops sleeping, not at the next hourly tick. This
  applies to everyone.
- Errands queue. Several needs at once (the privy, the well, food) are done
  one after another before going back, instead of each replacing the last.
  A wait that ends in the same minute as a conversation now waits for the
  conversation to wrap up, which had been losing errands' return points.

Implications: snapshot 6, character layout 4. `meal_soon` in the toolbox is
there for any other character who should hold out for a meal. With Tom
buying less bread, money shifts toward Mara; on seed 1 Hal now runs out by
the fourth week, which the summaries flag.

## 2026-10-24: Holding out for meals is human, not Tom's

Ian, on Tom's hunger rule: "Probably this is a general human trait, that
all characters should inherit."

Decided:

- The rule moves to the toolbox as the default for everyone
  (`toolbox::hunger`): below 90, keep to the schedule; at 90 or more, wait
  for a meal under way or due within the hour, else eat now (a loaf from
  the bakery if they can pay, else the larder at home). Anyone arriving at
  bed with hunger of 80 or more eats from their own larder first.
- Characters inherit it. A character's `react` now returns `None` for
  triggers it does not handle, and the town's dispatch hands those to
  `toolbox::human`, which answers needs and the bedtime check. Characters
  override only what is theirs: Tom, starving behind the bar, eats the
  bar's bread, and checks his hunger while closing up so he can eat there.
- The larder is the table in the villager's home building, found from
  content rather than named in each character.

Implications: this supersedes the toolbox default in "Bodies" (which
bought bread at 70) and narrows "Tom keeps his schedule unless he is
starving" to Tom's bar. New shared behaviour goes in `toolbox::human`;
a character opts out of a piece by handling that trigger itself. No
character struct changed shape, so the layout version stays.

## 2026-10-24: Interiority as per-character programs; conversations as negotiated frames

Ian, over several messages: "I want something a little more thought out
than just like a bunch of scores of warmth and such... a self-concept and
concept of others as a model. And that we will typically write a program
for each character that represents this interiority." "Maybe what we need
is things like, how warm is a conversation, not what is the warmth between
two people." "There might be a bit of negotiation in a conversation... The
negotiation establishes what can occur. (Outside of socially inappropriate
behavior, which exists but is an exception)... there can still be
intention and actual received behavior... it's just landing wrong." "You
could probably base the set on some research... out of frame is possible
and might require another handler. If we have a player people love to
experiment with breaking convention, so it's worth making that a typical
thing you implement in a character."

Decided (direction; not built yet):

- No relationship scores. Conversations are recorded with observable parts;
  each participant reads them through their own program and keeps episodes;
  feelings about a person are read from memory by that person's program.
- Conversations run in shared, negotiated frames drawn from research
  (`docs/research/conversation-frames.md`): greeting, transaction, small
  talk, catching up, favour, confiding, courting, quarrel. Shifting frame
  is a bid answered toward, away or against.
- Each move records intent, performance and each hearer's uptake.
- Breaking frame is its own event, and every character implements an
  `out_of_frame` handler on top of a shared default.
- Characters know each other only through perception; direct reads of
  another character's private state are removed.

Implications: the full proposal is `docs/design/conversation.md`. The first
spike is four frames, about ten moves, shared defaults for every hook, and
placeholder personal programs for two or three villagers, judged by whether
the summaries can tell the bakery-counter scene in that document. Prom Week
and Versu both found authoring the hard part; inherited defaults with small
personal overrides are how this repo keeps it manageable.

## 2026-10-24: Ground interiority in psychology research too

Ian: "Psych research is also appropriate."

Decided: the defaults behind each character's inner program come from named
psychology findings, written up in `docs/research/interiority-psychology.md`
and mapped in `docs/design/conversation.md`. Personality is modelled as
if-then behavioural signatures (Mischel and Shoda's CAPS), which supports
writing a program per character; traits are parameters to shared defaults.
Self-concept is a belief that can be wrong; views of others are built by
attribution; "what they think of me" defaults to one's own self-view;
feelings come from appraisal; warmth comes from disclosure answered with
perceived responsiveness; memory keeps peaks and endings and weighs the bad
more.

Implications: findings with mixed later support (self-discrepancy's
emotion mapping) are defaults to override, not rules. Still direction only;
nothing built.

## 2026-10-24: First conversation spike

Ian: "I think some randomness gives an opportunity for misinterpretation
(with per-character biased). So… are you ready to experiment with these
characters?"

Decided and built:

- Conversations are records in `SimState.conversations`, advanced one beat
  every two minutes. Each beat records the speaker's stance, what they meant
  and what came out; each listener's uptake records what they took it as,
  how warm it felt and whether they blamed the person or the moment.
- Misreading is a biased roll in the shared `take_up` default: rejection
  sensitivity, mood, a poor view of the speaker and a curt answer push it
  worse; charity and a good view push it better. A frame's scripted moves
  (the request and offer at a counter) are rarely misread. Misfires come
  from skill and footing in `perform`.
- Every villager has a placeholder program (`impl Interlocutor` in their
  module) for Ian to rewrite: Mara friendly and discreet, Tom blunt with his
  mood moved by each talk, Pip ignoring frames, Nell anxious with anyone
  unfamiliar, Hal quick to hear a rebuff.
- Perception only: no character reads another's struct. Mara decides on the
  pastry from how Tom seemed when they last talked, learns about his fish
  when he tells her, and Pip takes a bun only if he understood Mara to offer
  one.
- Purchases, deliveries and lunch open a transaction at the counter when the
  keeper is there and free. An accepted bid extends the talk by 12 minutes.
- `talks` prints transcripts; day summaries get one line per conversation
  with anything beyond its business; the inspector shows each villager's
  persona, mood, worry, view of others and recent memory.

Implications: the bakery-counter scene in `docs/design/conversation.md`
turns up with different people. On seed 2, day 36, Mara bids to confide
about money to Tom at the tavern with Nell there, Tom turns away with a
reason, Mara takes it as a rebuff, and Nell overhears "money". Hal never has
money trouble in this economy (he ends weeks richer, and Mara poorer), so
the original casting does not occur. The numbers (misread and misfire rates,
the 7-day retelling gap, feeling thresholds) were tuned by reading six weeks
of transcripts on ten seeds, not by any score.

## 2026-10-24: Self-concept as changeable state in a character's program (experiment)

Ian, on whether self-concept is more than hidden variables: "if you believe
that their self-concept can really change, then you would do hidden
variables that can change, and then change the code based on that." Then:
"Yes, treating it as an experiment."

Decided: no self-concept system. A character's program owns whatever
private state it needs, reads it, and updates it from experience. The
first case is Hal: a `touchiness` field (0 to 100, starting at 80) that
each remembered conversation moves, 8 up per point a talk went below zero
and 2 down after a good one. His rejection sensitivity is a tenth of it,
computed in `persona()`. A change is noted at the start of the next day.

Findings over six weeks on ten seeds: no runaway and no collapse. It moves
as a sawtooth, slow easing and sudden jumps, mostly between 7 and 10; on
seed 7 he settled lower (5 to 7) and misread less (4 times against 9 with
a fixed 8). The jumps follow readable causes: on seed 1, Spring 18, Mara
confided her money trouble to him, he took it as complaining and blamed
her, misread Tom at lunch, and was touchier the next morning. The effect
on behavior is small, because the shared default weighs sensitivity
lightly (a misreading chance of 26% at 8 against 32% at 10).

Implications: where the set point lands is mostly the ratio of the two
constants, not his history. Open: whether a character needs a second
layer (what they believe about themselves, separate from what is true) so
they can be wrong; nothing here needs it yet.

## 2026-10-25: Perception as recollection plus attention, and self-talk

Ian, over several messages: self-talk "should build on perception"; "lots of
knowledge is indirect (memory of patterns, phone calls, etc). But there's
perceptions that intrude as well"; "we don't have to create an action log.
We can just allow the character programs to read actions from the context
... with helpers for ... would have perceived"; filtering events into "some
worth keeping a day, some worth keeping a month ... That's about
recollection, though ... That's not about attention"; and the watch list
"maybe it's just a tick thing. Maybe it's just a pattern."

Decided and built (`docs/design/perception.md`):

- The town keeps worthwhile events for a day or a month, each with who was
  close enough to see or hear it. Character code reads it through
  `Ctx::recall`. No central knowledge store.
- Attention is a quarter-hourly `Look` trigger and each character's own
  checks. Intrusions (so far, tripping in weeds) are pushed as triggers.
- Self-talk is a `Think` command carrying explanatory style; each villager
  has a placeholder style. Tom's grievance builds from it and boils over.

Implications: kept events go into the snapshot as JSON text, because the
event type is internally tagged for the viewer and postcard cannot decode
that. Knowledge representation stays action-centric, per the Viv note:
what a character knows is what it perceived of actions plus what it chose
to keep.

## 2026-10-25: Viewer controls: absolute speed, skip to morning, no text view

Ian: "I'd rather select an absolute speed than faster/slower. And the
layout changes as some of the values change size (like time). And at night
I want to bump up to 6am or something, or maybe just a fast forward to
morning button in general (instead of +1 day?) We can also kill the text
version."

Decided: a speed list (game-minutes per real second, with how long a day
takes) replaces faster and slower; "to 6am" replaces "+1 day" and jumps to
the next 6am, later the same day when it is still night. The clock and the
pause button have fixed widths. The page no longer has the text view; the
text views stay in `sim-tools` and in `window.__scene()`, which the browser
smoke still checks.

## 2026-10-25: Gestures are exhaustive per character; topics are declared, not exhaustive

Ian: "I almost want an exhaustive switch for those cases in characters."
Then: "I suspect exhaustive topic handling is too hard. Move kind is maybe
not the best term. But that's the only one I want exhaustive. Topics grow
easily, moves don't." And: "Let's do gesture. Later maybe we could do a
topic adjacency list, but not yet."

Decided and built:

- `MoveKind` is now `Gesture`, and a turn is `Act::Gesture { gesture, topic }`.
- Every character must implement `warmth_of(Gesture)`: what each gesture
  means to them. It is written as a match under
  `#[deny(clippy::wildcard_enum_match_arm)]`, so it names every gesture
  and a new one fails the build until each character decides. Arms can be
  grouped and fall back to the shared `Gesture::warmth`. The placeholders:
  Tom likes banter and dislikes pity, Nell is hurt by teasing and warmed by
  sympathy, Pip loves jokes and hates a no, Hal takes refusals and prying
  hard, Mara values being confided in.
- Topics are declared in `content/topics/` with an id and a gloss, checked
  by content validation. `Ctx::topic(id)` panics on an undeclared id, so a
  typo fails the first test that reaches it. A topic can say whom it
  concerns ("money" about Mara). Characters match only the topics they
  care about. Views show the gloss.
- Hal's delivery talks are routed by his activity, not by matching a topic
  string.

Implications: what a topic implies for the listener (knowledge, gossip) is
still to come; this only makes topics typed and declared.

## 2026-10-25: Types over strings, as a general pattern

Ian: "As a general pattern, I like to make use of types as much as
possible, often to support exhaustive updates."

Decided: recorded as a rule in `CLAUDE.md` ("Types over strings"). Closed
sets are enums, matched exhaustively without a wildcard where a new
variant should force a decision. Strings are kept for open, content-grown
sets, which are declared in content and validated.

Implications: existing string-typed sets are candidates for conversion
(activities, goods, villager and object keys in character code). Each is
its own change.

## 2026-10-25: Character state is for behavior; income is content

Ian, on Nell's pension living in her code and struct: "just odd that
there's a pension attribute or something, not just coins and the
schedule", and "They shouldn't invent attributes to form as comments or
text. We should have fields to explain the behavior, but not weird
attributes."

Decided:

- Scheduled income is content: a villager's file can list `income`
  (days, time, amount, what), and the baseline plan pays it from outside.
  Nell's pension moved there; her code no longer mentions it.
- Character structs keep only state their behavior reads. Removed:
  Mara's batches baked and pastries delivered, Tom's days since a day
  off, hunger breaks and outbursts, Pip's buns and greetings, Nell's
  pensions received and loaves bought, Hal's trips. Tests that counted
  them now count events.
- Recorded as a rule in `CLAUDE.md` ("Character state is for behavior").

## 2026-10-25: Overhearing follows the gesture; heard memories are queryable

Ian: "I feel like gesture is attached to overhearing." Then: "the carry
makes sense just folding in with the gesture", and on memory: "person
talking, who they're talking to, which might be to themselves, and the
topic they're talking about, and the way they're talking about it ...
they're able to query those in programming code just to do general
queries."

Decided and built:

- Each gesture has a carry (`Gesture::carry`, exhaustive): low (confiding,
  sympathy, and a bid to confide), ordinary, carrying (a joke), loud (a
  complaint, a dig, a rebuff). It replaces the fixed list of "notable"
  gestures.
- What a listener in the same room (or outdoors) catches depends on their
  distance from the speaker against that carry: the words (`Close`), the
  tone and any name in it (`Gist`), or only raised voices (`Noise`).
- A heard memory is: when, speaker, to whom (when they could tell), the
  gesture as they took it, the topic (only if they caught the words),
  whether it was about them, and how much they caught. Character code
  queries it: `mind.heard().from(tom).topic("fish").last()`.
- Hearing your own name, or something loud, from someone else's talk is an
  intrusive `Overheard` trigger. Hal assumes a sharp remark was meant for
  him (he blames the speaker and grows touchier); Nell, hearing her name,
  blames herself.

Implications: talking to oneself is not recorded yet; the memory has room
for it (the speaker as listener). Loud talk does not yet carry through a
doorway. Hal is rarely near anyone else's conversation, so his reaction
seldom fires.

## 2026-10-25: Knowledge into action through shared rules; actions cite their cause

Ian: "can nell act on any knowledge, or anyone else?" and "behaviors
aren't really organic, right? They each are programmed specifically",
then the worry that "the specific memories are smuggled into numbers and
the numbers turn into logic and the logic turn into action". Then:
"you can experiment and make conclusions and act on them".

Decided and built (details in `docs/design/knowledge.md`):

- Thoughts can be said aloud (`Thought.aloud`). Tom grumbles aloud;
  anyone in earshot hears him complain to himself, and Mara takes him a
  pastry when she has.
- Shared rules in the characters toolbox read specific heard memories and
  return commands; persona decides who runs them. The first is
  `kindness`: help someone you like after hearing words about their
  trouble. Topics mark trouble in content (`trouble: money | food`).
- `Command::Because` makes the next event's cause `Cause::Knew {from,
  topic, heard_at}`. Summaries and the viewer show the chain.
- News about third parties passes on by default in conversation, gated
  by a new `Persona.discretion`, and not to someone already heard
  talking about it. Pip stops to chat when he meets people.

Implications: numbers stay for aggregates (feeling, familiarity) and
thresholds; facts travel as memories to the rule that acts on them. One
rule exists; the next few will show whether `Trouble` stays a closed
enum. News is retold verbatim, with no distortion yet. Protocol 17,
snapshot 10.

## 2026-10-25: A triangle among the existing cast

Ian: "Most self-talk is only in someone's mind, so it can't be heard,
overheard. But it's kind of interesting that you can. Seems like maybe we
could have other stuff too, like maybe we either need to expand the world
or something, so that there's some like social tension here, love
triangle or something." Offered the existing cast or new villagers, he
said: "sure, use them".

Decided and built (details in `docs/design/triangle.md`):

- Hal is sweet on Mara, Mara is fond of Tom, Tom does not notice. This is
  code in each character's file.
- Thoughts stay silent by default; saying one aloud is a per-character
  habit (Tom grumbling, Hal after a drink when jealous).
- `Topic` gains a second person (`with`) for news between two people.
  Gossip tells the second person only at discretion 2 or less.
- Hal's jealousy grows from what he sees in the town's record: Mara
  starting conversations that are not business. It drives muttering
  aloud, digs at the rival, and confiding.
- Mara, once she knows, keeps Hal at a distance. Tom takes complaints
  about him to heart without asking why.

Implications: Hal's calls at the bakery are without buying, because
buying kept Mara out of money trouble and removed the confidings and
gifts that depend on it. Protocol 18, snapshot 11, layout 9.

## 2026-10-25: Art from the Liberated Pixel Cup

Ian: "We should really have much better tiles. Can you look around for
better tile sets". After a comparison (Ninja Adventure, Serene Village,
Kenney, LPC, paid packs): "this is no problem licensing-wise. I'm totally
fine with share-alike. So let's go with LPC".

Decided and built:

- The viewer's tiles and villagers come from LPC art, 32px tiles, kept
  under `art/lpc` with each pack's credits. `tiles.png` is a derivative and
  is CC-BY-SA 3.0 / GPL 3.0; the viewer links the credits.
- Ruled out on the way: Mana Seed (its license forbids use alongside
  AI-generated code or writing), and paid packs that forbid
  redistribution (the repository is public).
- `scripts/make-tileset.mjs` composes the atlas. Terrain uses the LPC
  3x6 blocks, cut into the renderer's existing quadrant shapes. Wall tops
  are built from the LPC beam; a wall shows its panelled face to a room
  below it and an outside face to the street.
- Each villager's look is a list of generator items and colors in
  `art/lpc/characters.json`; `scripts/lpc-vendor.mjs` copies the layers
  from a generator checkout and the tileset build recolors and composes
  them. Villagers face the way they last moved and step while walking.
- The procedural placeholder tileset is gone.

Implications: anything we draw by editing LPC art is share-alike too.
Sleeping villagers are shown by name only, since LPC has no lying-down
frames. Roofs are still not drawn; interiors stay visible from above.

## 2026-10-26: Conversations need reach

Ian, replaying a moment from the words tab: "I clicked on 'Pip and Mara:
greeting about a greeting' But they don't seem to be near each other", then
"Oh wait, maybe Pip and Mara are actually on top of each other in the
flashback".

Both were true. At the bakery counter Pip and Mara stood on the same tile and
were drawn as one. Elsewhere, an encounter (same room, or within 6 tiles
outdoors) started a conversation where it was, so people talked across the
square.

Decided and built:

- A conversation starts only within `TALK_REACH` (2 tiles, same place).
  Further than that, the one who started it walks over first and starts it
  on arrival. Encounters still fire at 6 tiles; reach applies to the talk.
- Villagers on one tile are drawn side by side, talkers face each other, and
  their names stack.
- A conversation whose frame is its own topic is summarized as just the
  frame ("greeting", not "greeting about a greeting").

Implications: encounter chats take a little walking time, and the walk can
be interrupted like any other. Snapshot 12.

## 2026-10-30: A 3D view, as an experiment

Ian: "Is there a way to make this world feel more... embodied? 3D? Camera
images of events? I'm not sure. I have a bunch of quota to burn, so we can
try something..."

Built as a trial beside the 2D view, which stays the default:

- `packages/viewer/src/diorama.ts`, loaded only when the 3D button is
  pressed. three.js, with the same LPC art: the 2D renderer paints the
  ground onto a plane, walls are raised one tile from it, and objects,
  trees and villagers are upright sprites turned toward the camera, in the
  style of "HD-2D" games.
- Villagers ease between tiles instead of jumping, and pick their facing
  relative to the camera.
- Light follows the clock: a sun from east to west, dusk colours, a dim
  moon, hearths that glow, and a lamp in each building in the evening.
  The town sits in woods.
- Speech marks show each gesture and topic over the speaker for a few
  seconds. They use the sim's own vocabulary, not authored text.
- Replaying a moment in 3D frames the people involved from the side.
  "Photograph this day" replays each unusual moment of a day and puts a
  picture of it beside the line, then returns the town to where it was.

Found on the way: replaying any moment of the first day failed, because
the first snapshot held the events of making the sim, and postcard cannot
decode the tagged `Emitted` enum. Those events are now stored as JSON, as
the record already did. Snapshot 13. The browser smoke test now replays a
first-day moment and draws the 3D view.

Implications: the renderer choice (see "Stack") now has a working 3D
candidate to compare against Canvas 2D. Nothing in the sim changed for the
view; it reads the same views and events as the 2D one. LPC has no
lying-down or sitting frames, and furniture art is drawn for a fixed
angle, so turning the camera far from south shows the limits of the art.

## 2026-10-30: Full-height walls in the 3D view, cut away where they block

Ian, on the 3D view: "The viewer is not great, but the actual images are
surprisingly good! The cut-off walls are a bit distracting, though I can
see why it'd be very hard to make these shots without them".

Changed: walls are three tiles high, from the LPC tall wall art (wood
panelling inside, half-timbered brick outside), and only walls that block
the view are cut down to a stub:

- In a close shot, the near walls of the building being looked into come
  away, as on a stage set, along with any other wall the line of sight
  would pass through. Other buildings keep their walls.
- Further out, every wall with a room behind it, as seen from the camera,
  is cut, as in a dollhouse.
- Doorways have wall above them. Outside walls have windows on some cells,
  lit in the evening.
- A shot is taken from the south side of the people in it unless much
  more stands in the way there, never end on. Objects and trees still in
  the way are hidden.

## 2026-10-30: A valley ten times the size, with a program for everyone

Ian: "OK, now we just need to make the world quite a bit larger. Like maybe
10x...?" Asked what to scale, he chose area and people together and one big
outdoor zone. On the villagers: "The mix and configuration of each character
should somewhat unique (some have jobs, some don't, some are neurotic in
certain ways, etc). And probably at least one unique program per character
too, something that makes them special and unique. Probably all characters
should have both a description, and design notes you use now and in the
future to guide their programming development".

Decided and built (see `docs/design/valley.md`):

- One zone of 150 by 115 tiles with the old town in the middle, farms, a mill
  on a river, woods with clearings, a hamlet across the river and a manor.
  31 buildings, 45 villagers.
- Every villager has `docs/characters/<id>.md` (a placeholder description and
  design notes) and a module with at least one behavior of their own. The 40
  new ones are placeholders like the first five.
- Zones declare named areas, and character code refers to them instead of
  coordinates. Paths are laid after water and woods, so a road across water is
  a bridge.
- A `Character` trait and a `town!` macro replace the hand-written dispatch:
  adding a villager is a module and a line.
- Routing became one A* over dense tiles before any of this, which made the
  size affordable (a week in about half a second).
- The villager sheets are packed four to a row so the atlas stays within
  common texture limits, and the 2D map caches its still layer.

Implications: tests that named seeds or coordinates moved to areas, wider
seed searches, or properties that hold on any seed. Confiding is rarer with 45
people (warm pairs are thinner), so the four-week conversation test checks
that people try to confide, and a separate test that one takes. The economy
has not been tuned. Layout 10.

## 2026-10-30: Tests set up scenes and force chances

Ian, on tests that searched seeds for an event: "using seeds to get a test to
pass is crazy! You can and should build testability in, maybe in this case
forcing a certain "random" order of things".

Decided and built (see `docs/testing.md`):

- Every roll that decides whether something happens is a named chance,
  `Ctx::chance(rng, who, label, percent)`, with the label a `pub const` beside
  the code. A test can force a label for one villager or everyone. Forced
  outcomes live in `SimState` (snapshot 14) and are empty in a normal run.
- `characters::scene::Scene` sets a moment up: forced chances, coins, needs,
  memories, positions, and cued commands (`Sim::cue`, `Sim::state_mut`).
- A teleport now fires encounters, since the villager has come to rest.
- The seed searches and seed-specific tests became scenes. Tests that run the
  ordinary town for weeks all use seed 1 and check properties that hold on any
  run.

This corrects the implication in the previous entry: there are no wider seed
searches.

## 2026-10-30: Town hall, Saturdays, and materials

Ian, on the valley: "make sure you add more variety too, town halls,
workplaces, etc. Not just more cottages or something. Use lots more textures
too for floors or whatever".

Decided and built (see `docs/design/valley.md`):

- A town hall with a Saturday moot that eight villagers attend, a Saturday
  market on the square with three new stalls, and brewing at the alehouse.
  These are schedule entries; no character code changed.
- Rooms have a `flooring` and buildings `walls`, both closed enums in content
  that only the views read. The world view carries rooms and wall materials
  to the viewer (protocol 20).
- Six floors, four wall styles inside and out, and sprites for every object
  kind, all from the LPC sheets already vendored.

Implications: a new material is a variant in `Flooring` or `Walling` and a
tile in `scripts/make-tileset.mjs`. The furniture is decoration and blocks a
tile; content validation catches a piece that seals part of a room.

## 2026-10-30: A railway, animals, farms, and a more organic valley

Ian, after looking at the valley: "Some parts should be more organic. Like the
stream should be curvy, not straight. Seems like there should be something
like a farm or garden. Maybe chickens too (interesting to have a new entity)
... Also there should be like barn doors and a loft ... A train that comes
through, maybe delivering items, would be cool. Also taking items, maybe there
can be some cottage industry of sorts here. Not sure how to model the train,
but you can figure it out. For now no people going on or off the train, just
goods." And: "Probably the chickens make some people happy and some people sad
(or other emotional reactions, annoyed, etc)." And: "a bridge like this should
look like a bridge".

Decided and built (see `docs/design/valley.md`):

- Animals are a new kind of entity in the sim: flocks declared in content,
  each animal's position in the state, moving on its own random stream.
  Chickens roost and lay; sheep graze. They have no minds.
- Every character says how fond it is of each kind of animal
  (`Character::fondness`, an exhaustive match), and the shared response turns
  that into thoughts said aloud or kept quiet. Annoyance is laid at the
  owner's door.
- The railway is a timetable in content. A train's position is a function of
  the clock, so it needs no state of its own; consignments do. Villagers send
  and fetch goods by declaring it in content, and the baseline plan adds the
  trips on days a train stops.
- Crops in plots, gardens, barn doors and lofts, a winding river, ragged
  woods, more kinds of tree and ground decoration. All view-only except the
  river, woods and lofts, which change the map.

Implications: protocol 22, snapshot 15. A new animal kind is an `AnimalKind`
variant, which every character must then answer for. Trains carry no
passengers; if they ever do, a train becomes a place villagers can be in.

## 2026-10-30: Money leaves through things that get used up

Ian: "for money, probably something that doesn't last that they can spend
money on. Rent adds a complexity that could shift the balance again."

Decided and built:

- Anyone with more than 40 coins makes one trip a week to the store and
  spends up to half of the excess on tea, sugar and tobacco, which are gone
  once bought. No rent.
- The store buys these from the railway at the price it sells them for, so
  the money leaves the valley instead of collecting with the shopkeeper.
- Fetches carry a `keep` target and the train tops the goods shed up to a
  fixed stock, so nothing piles up unused.

Implications: wealth now levels off where weekly income matches half the
excess over 40. Tuning is in `toolbox::{COMFORT_COINS, LUXURIES,
MOST_LUXURIES}` and the railway's `delivers`. Tom's tavern still loses money,
which predates this.

## 2026-10-30: The store takes a markup; Tom stays bad at business

Ian: "sure for a markup. Poor Tom is running a bad business. That's okay in a
sense, some people are bad at business!"

Decided and built:

- The store makes a coin on each tea, sugar and tobacco (`toolbox::MARKUP`).
  This corrects the earlier entry, where it sold them at cost.
- So the shopkeeper does not just accumulate, Greta spends like anyone else,
  but keeps 200 coins back to restock with and buys her own luxuries from
  the goods shed an hour after a stopping train, up to 36 a trip. The train
  brings 36 of each so there is some left after the store's fetch.
- Tom's tavern losing money is left alone.

Implications: Greta levels off around 330 coins, the richest in the valley.
The reserve matters: without it she spent her margin on herself and could
not restock, and the shelves went bare.

## 2026-10-30: Change before contention; feelings as personal vocabularies

Ian, on the direction plan (`design/direction.md`): "Let's not worry about
intention and contention right now, just change. Maybe an easy place is
farming, where the farmer actually affects their land, and the land changes
automatically. Of course we should probably have accelerated growth." On
feelings: "Mostly people could have their own internal feeling
vocabularies, only the interactions have to be more normalized ... the
differences are about different focuses and encourage diversity." And:
"Mood is a category of attributes." On disasters: "hand-coded (by you)
events that affect things ... a flood doesn't place water anywhere, it
places it by the stream." On modules: "The character probably has their own
flags that basically dispatch to known modules ... grieving is a module
that is invoked if there's something that causes grieving."

Decided:

- Intentions, contention and group decisions wait.
- Next is living fields: field tiles as state that grows, dries, weeds and
  rots on its own, fast, and that farmers work tile by tile, with the
  harvest feeding the economy.
- Disasters are hand-written events with their own geography; drought
  first, since it acts on the fields.
- Feelings will be per-character vocabularies drawn from a shared library of
  feeling kinds with named levels, over a fixed set of normalized
  interaction signals. Mood becomes a category of named states. No bare
  numbers without a scale and a meaning.
- Modules are dispatched by the character's flags, with a core set the
  shared rules invoke.

Implications: the order in `design/direction.md` is revised at its end.
The feelings change replaces `warmth_of` as the only felt quantity, so it
will touch every character module.

## 2026-10-31: Components and systems; moods as personal configuration

Ian: "Put a bit of thought into that structure, with modules and ECS inspiration (but i
agree not to use an ECS library). And I want you to create a more diverse mood system.
Like, people have specific personal attributes that personally effect them. You might
use a combination of these module/ecs ideas, and configuration essentially on a
character basis, to make this happen."

Decided and built (see `design/components.md`):

- The sim is organised as ECS-style tables without a library: entities are ids,
  components are typed tables keyed by id in `SimState`, configuration is per-entity
  data in content with closed enums, systems are functions the event queue runs, and
  systems hear each other through the emitted events. Modules for characters follow the
  same shape when they are built.
- Moods are a component: a library of seven (lonely, restless, irritable, anxious,
  gloomy, cheerful, proud), each villager prone to two or three, configured in their
  content file with their own sources (Bess anxious about the dark, Silas about money
  below 120, Wynn cheered by rain, Rob irritated by slights, Fen proud of his fishing).
  Levels have names and every change has a cause.
- Spirits replace the bare mood number: good moods less bad ones less the body, -3 to
  3, named. Conversations read it.
- A mood at its second level is a trigger; the shared answer is small and visible (the
  lonely seek company or go to a bar, the restless walk, the anxious and the proud think
  about it).

Implications: protocol 26, snapshot 17. The level names and every villager's mood
configuration are placeholders from the character notes, for Ian to rewrite. Pride in
earning talks about "custom", not "money", because "money" is the topic of money
trouble and a listener took a proud shopkeeper's remark as a call for help.

## 2026-10-31: A second world

Ian: "I want to support a second world. There will presumably be a lot of work just to
support two worlds, while keeping a lot of engine stuff shared between them. But they are
also two codebases, as there are a lot of qualitative differences between them.
Presumably you select the world, and all the interface/sim comes after that. There's no
crossover, no shared timeline, no nothing, just completely different." And: "We should
keep the principles and concepts well documented so there's a certain consistency (and
diversity!) in the world. Start with the basics, we'll focus on just one zone."

Decided:

- A second world, the raft city (working name), from Ian's dossier
  (`worlds/raft/dossier.md`, verbatim). The world is chosen first; nothing is shared
  between worlds at run time.
- The engine is shared; content, character programs, art and docs are per world.
- The world's rules are kept as checkable statements in `worlds/raft/principles.md`.

Proposed, for Ian to confirm: the layout and order in `worlds/README.md` (move the valley
into `worlds/valley/`, generalise the valley words in the content model, then add the
raft world), and the first slice in `worlds/raft/plan.md` (the scrap zone).

## 2026-10-31: The Welling, and the valley moves into `worlds/`

Ian, on the second-world proposal: "sounds good. Fire isn't necessary to start. Kelp
isn't like a super strong thing, just a mood/energy booster, mildly addictive. The repo
sounds fine. Keep valley playable throughout. For the name of the world, let's call it
The Welling".

Decided:

- The second world is The Welling, id `welling`. Its docs are in `docs/worlds/welling/`
  (the previous entry's `worlds/raft/` paths now live there).
- The layout in `docs/worlds/README.md` stands. The valley's content, characters crate
  (renamed `valley`) and fixture are under `worlds/valley/`.
- The tools take `--world`, the wasm `Sim` takes a world id, and the viewer opens the
  world named by `?world=`, else the last one opened.
- The first slice is the scrap zone, without fire. Kelp is a mild mood and energy
  booster with a mild craving.
- The valley stays playable at every step.

## 2026-10-31: The Welling's first slice

Following the entry above.

Decided (by the agent, within what Ian agreed):

- The scrap zone is open water with decks laid on it (`Cell::Deck`). Vessels are buildings
  marked `afloat`, which may stand on water; the Mire has a hold below the waterline.
- Eight placeholder people, in `docs/worlds/welling/people.md`. All names, roles, moods
  and programs are for Ian to rewrite.
- Both worlds' characters are written against `crates/behaviour`, the world-agnostic half
  of the valley's old toolbox. World-specific errands stay in each world's crate.
- Thirst is met at cisterns, since there are no wells at sea.
- The viewer chooses a world with a picker in the top bar. People without their own sprite
  sheet are drawn with one of the valley's, chosen by their id, until they get their own.
