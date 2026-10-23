# World layout: places, doors, floors

Status: proposal, revised 2026-10-23 after the research in
[world-layout-research.md](../research/world-layout-research.md). Not
decided. Responds to the need for a way to lay out buildings and the town,
test the layout, adjust it, and handle interiors, exteriors and multiple
floors, all from a text-only workflow.

## The model: a graph of places joined by doors

Every shipped game the research examined ends up with the same two-level
structure: a graph of places, and a grid search inside each place. The
difference is whether the graph is authored (Stardew, Pokémon, Ocarina of
Time) or derived from geometry (The Sims, RimWorld, Dwarf Fortress, Shadows
of Doubt). Authored graphs fail by omission at load time, which a validator
catches. Derived graphs fail by geometry at runtime (Sims routing failures
across floors are chronic). Ultima VII's continuous world with a Z axis left
its upper floors empty because the schedule format had no Z field.

Proposal: an authored graph.

- A **place** is one grid: an exterior zone, one floor of a building, a
  cellar. Places have a stable path-style id (`bakery/upstairs`), a declared
  floor index, a default arrival tile, and camera bounds.
- A **door** is one record owned by the building or the town, never by a
  single place: `from` place and tile, `to` place and tile, optional
  `oneWay`, optional conditions (hours, owner, lock). The reverse direction
  is implied unless `oneWay` is set. Stairs are doors.
- A **building** is a footprint on the exterior grid plus an ordered list of
  interior places. The footprint's door and the ground floor's door are the
  same record.
- **Exterior zones** are places joined by edge records carrying a direction
  and offset (the Pokémon map-connection shape), even if there is only one
  zone at first. This keeps a stitched continuous exterior available later
  without re-authoring.
- Interiors can be any size. Stardew interiors run 2.7x to 7.4x their
  footprints and no game enforces a fit. The render reports the ratio; no
  validator fails on it.
- Same tile size indoors and out. Only the map dimensions change.

What was dropped from the first draft: the claim that off-screen NPCs can
move at the place level only. Stardew updates every NPC in every location
every tick, and no 2D town game that teleports NPCs on a travel-time estimate
was found. At a few dozen NPCs and places, simulate everyone on real tiles.
Add place-level LOD only if a profile shows a need.

## Pathfinding

Two levels. A route table over the place graph, shortest-path weighted by
precomputed door-to-door distances inside each place (the HPA* shape), then
grid A* inside each place for the current leg. Stardew's route table is
first-found rather than shortest and depends on declaration order; a test
here shuffles the door list and asserts the route does not change. Any
iteration cap on the grid search is asserted in tests rather than failing
silently (Stardew caps at 30,000 and returns null).

Schedules address a place id plus a tile or object, never a building id.
That rule is the local form of the Ultima VII lesson.

## Authoring form: ASCII in TypeScript

Each place is an ASCII block with a legend, written as a TypeScript literal:
a template string for the grid, a legend object beside it, and door, object
and stair declarations as typed data. NetHack 3.7 moved its level grammar
into Lua in exactly this shape, and it gives types without writing a parser
for the outer structure. The grid parser is small.

```ts
export const bakery = building("bakery", {
  footprint: {
    grid: `
      ######
      #....#
      #....#
      ##D###`,
    door: "D",
  },
  places: {
    ground: place({
      floor: 0,
      legend: { "#": wall, ".": floor, C: counter, O: oven, T: table, S: stairUp, D: door },
      grid: `
        ############
        #OO.....T..#
        #CC.....T..#
        #..........#
        #..........#
        #S.........#
        #..........#
        #####D######`,
      objects: [
        { glyph: "O", kind: "oven", owner: "mara" },
        { glyph: "C", kind: "counter", activity: "tend-shop" },
        { glyph: "T", kind: "table", activity: "eat", capacity: 2 },
      ],
      arrival: "D",
    }),
    upstairs: place({ floor: 1, /* ... */ }),
  },
  doors: [
    { glyph: "D", to: { place: "town/square", tile: [14, 22] }, hours: "06:00-20:00", unlessOwner: true },
    { glyph: "S", to: { place: "bakery/upstairs", glyph: "S" } },
  ],
});
```

Format rules the research argued for:

- The legend is per place and may inherit from the building. One glyph can
  bind to a stack (tile plus object plus NPC), which is how DCSS and
  PuzzleScript avoid running out of letters.
- The parser errors on ragged rows, unknown glyphs and characters that are
  not single-column. DCSS silently defaults unknown glyphs to floor and
  that is a documented source of bugs.
- Door tiles sit on a place boundary (a grid edge or against a wall), never
  on a through-path. Stardew's warp tiles are both triggers and barriers,
  and warps on walkways cause accidental teleports and blocked NPC paths.
- Arrival tiles are at least one tile inside the destination and off any
  door tile, to prevent ping-pong between places.
- Place ids are stable. A `formerIds` list is kept on renames (Stardew's
  `FormerLocationNames`), so schedules and dialogue do not break.
- Tile coordinates only. Pixels belong to the compiler.

The format compiles to the renderer's own tile layers. The autotiler is
written in the project: a Wang-set resolver for ground and floors and a small
pattern matcher for walls. LDtk and Tiled export can be added later if a
person wants to open a map in an editor, with two caveats: LDtk bakes
autotile results in its editor so a code-written file has no tiles until
re-saved, and there is no Node parser for Tiled `.world` files.

## The test-and-adjust loop

Editing is editing text. Testing is running validators and renders in Node.
The loop is edit, run, read, edit.

Validators (fail the suite):

| Check | Failure it prevents |
|---|---|
| Every door record resolves both endpoints to walkable tiles; one-way only when flagged | Dangling or drifting door pairs |
| Door tiles on boundaries; arrival tiles inside and off doors; every place has a default arrival | Accidental warps, ping-pong, debug warps into the void |
| Every place reachable from the root (town square) | Orphaned places |
| Flood-fill walkable components per place; every component contains a door; components reported as rooms | Walled-off rooms and unreachable objects |
| Every object reachable from the place's doors; furniture leaves a free path between every pair of doors and does not block another object's access tiles | Blocked counters and doors |
| Every schedule destination is a place id (not a building id) plus tile or object that exists | The Ultima VII empty-upstairs failure |
| Every schedule hop through a door is permitted for that NPC at that time (hours, owner, lock) | NPC stuck at a locked door |
| Schedule points sorted by time with gaps at least the computed travel time | Unreachable "arrive by" entries |
| Route table independent of declaration order (shuffle test); grid search cap asserted | Order-dependent routes, silent null paths |
| Every sleeping NPC has a bed; a bed has at most one owner | Kingdom Come's "ran out of beds" |
| Stairs pair across floors; each place declares its floor index | Mismatched stairs |
| Footprints do not overlap on the town grid; each footprint door opens onto a walkable town tile | Buildings in buildings |
| Exterior edge records are mirrored on both zones | Future stitching impossible |
| Capacity declared only where contention is wanted; warn where schedules send more than one NPC to an undeclared place at once | Unintended contention, per the loose-inventory decision |

Renders (snapshot-tested as text files, read by a person or the agent):

- A place at time T with NPCs as letters and a who-is-where legend.
- The town at time T with a count per building and the footprint-to-interior
  ratio per building.
- A travel-time matrix between places.
- One NPC's day as an itinerary: departures, doors taken, arrivals, waits.
- The door listing per building.

Vitest file snapshots for the text renders, CI never writes snapshots. A PNG
composited with sharp for occasional human review.

Procedural generation is a drafting tool, not a shipping one: a treemap room
split and a greedy furniture pass that respects door paths and wall
adjacency can draft a building, and the result is written out as ASCII and
edited by hand.

## Presentation

Scene swap on doors, as in Stardew. It commits the renderer to the least:
camera bounds per place, an arrival tile, a fade. Roof fading (Ultima VII)
and floor slicing (Sims, Project Zomboid) both add per-building authoring and
both have documented readability failures, so neither is planned.

The cost players name under scene swap is losing track of people. Stardew
left villager map markers to a mod. Budget NPC legibility into the first
playable: villager markers on the town map, or a people-against-time
notebook in the Majora's Mask shape. Lit windows at night and door hours are
the cheap exterior cues.

## What to build first

A thin vertical slice in the text-first simulation package:

1. Types for Place, Door, Building, Object; the grid parser with legend and
   the strict error cases.
2. The validators above, as tests.
3. Three buildings (a two-floor house, a shop, a tavern) and a town square.
4. The route table with door-to-door weights, grid A*, and the travel-time
   matrix.
5. The time-T render and the one-NPC itinerary trace, snapshot-tested.

Schedules go on top after that. The renderer reads the compiled output
later.

## Open decisions

1. Tile size (16 or 32 px) is an art decision, carried over from the asset
   research; the layout does not depend on it.
2. Whether exterior zones start as one place or several.
3. How much of the legibility budget (map markers, notebook) is in the first
   playable.
4. Whether to adopt Stardew's hour-and-owner door conditions as the only
   conditions, or allow arbitrary predicates from the start.
5. Whether procedural drafting of buildings is worth building before a
   handful of buildings exist by hand.
