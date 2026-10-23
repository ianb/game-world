# World layout: places, doors, floors

Status: proposal, revised 2026-10-23 after the research in
[world-layout-research.md](../research/world-layout-research.md) and the
decisions of the same date in [decisions.md](../decisions.md). Not
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
  interior places. A floor is inline (on the exterior grid at footprint
  scale, the default for ground floors) or `detached: "<reason>"` (its own
  grid). The footprint's door and the ground floor's door are the same
  record.
- **Exterior zones** are places joined by edge records carrying a direction
  and offset (the Pokémon map-connection shape), even if there is only one
  zone at first. This keeps a stitched continuous exterior available later
  without re-authoring.
- Interiors may be larger than their footprints, as in Stardew (2.7x to
  7.4x there). No validator enforces a fit; the render may report the ratio.
- Single scale across town and countryside. One tile means the same
  distance everywhere, and there is no Ultima IV-style overworld map at a
  different scale. Same tile size indoors and out.
- Outdoors is therefore large, and is one place or a few, split only at
  natural edges (a river, a tree line, the road out of town) into places
  joined by edge records. Zones are otherwise interiors and floors.

What was dropped from the first draft: the claim that off-screen NPCs can
move at the place level only. Stardew updates every NPC in every location
every tick, and no 2D town game that teleports NPCs on a travel-time estimate
was found. At a few dozen NPCs and places, simulate everyone on real tiles,
driven by a discrete-event scheduler (each villager wakes on its own next
event, such as arriving at the next tile) rather than polled every tick.
Scaling tricks (coarsening events in places with no observer) are a later
event policy; the place graph and travel-time matrix leave room for them.

## Unified versus detached interiors (decided: inline by default)

Ian, 2026-10-23: a unified scale and layout as in Ultima VI could be nice.
It allows forms like a carport or a porch without trouble. The scale is
awkward, though: Ultima towns felt large and sprawling next to the
countryside, and you spend a lot of time walking around the outside of
buildings. True in real life, but not in a fun way.

The two ends:

| | Unified (Ultima VI/VII) | Detached (Stardew) |
|---|---|---|
| Interiors | On the exterior grid at footprint scale; roofs hide when inside | Separate grids, scene swap at the door |
| Open forms (carport, porch, market stall, open barn) | Natural | Awkward; either exterior decoration or a tiny interior |
| Seeing NPCs move between buildings | Continuous, visible through doors and windows | They vanish at doors; legibility needs map markers |
| Town size | Buildings must hold their rooms, so towns sprawl and walks are long | Compact exteriors, roomy interiors |
| Floors | Need a Z layer on the same grid, or stairs that swap scenes | Each floor is its own grid |
| Renderer | Roof groups per building, occlusion rules | Camera bounds and a fade |

The data model does not have to pick one. A building's floors can be
declared `inline` (rendered on the exterior grid at footprint scale, roof
hidden when the player is inside) or `detached` (its own grid, scene swap).
Both are places in the door graph. Mixed layouts follow:

- Ground floors inline, upper floors detached. Walking in and out of a shop
  is continuous; stairs cut to a new scene. This is also how the Ultima VII
  Z problem is avoided without a Z axis.
- Open structures inline (carport, porch, stall, barn), closed houses
  detached.
- A whole town unified, a farmhouse detached, chosen per building.

The walking problem is a pacing problem, and it is tunable separately from
the model: town density (buildings sharing walls, alleys, back doors),
outdoor walking speed or a run, gates and cut-throughs, and click-to-walk
pathfinding so the player watches a walk rather than steering it. Ultima VII
had single-door buildings and no auto-walk, which is much of why its towns
felt like a chore.

Decision (2026-10-23, see decisions.md): inline by default. Ground floors
sit on the exterior grid at footprint scale with roof hiding. A floor is
detached only for a stated reason: upper floors and cellars, an interior that
must be larger than its footprint for gameplay, a space that is a different
world in feel, or something the renderer cannot show inline. Each detached
floor carries `detached: "<reason>"`, and the validator warns on a detached
ground floor without one. The town is built dense (shared walls, alleys,
back doors, gates, cut-throughs), and the travel-time matrix and itinerary
tools tune density and walking speed.

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

## Authoring form: structured data, with generated views

The source is for the agent, not for people, so it should be whatever the
agent writes most reliably: typed, structured data with coordinates. Rooms
are rectangles. Walls are derived from room edges (the Sims model: walls on
tile edges, rooms by flood fill). Doors, stairs and objects are records with
tile positions. The agent writes `{ x: 3, y: 5 }` more reliably than it
counts characters across a grid, and a schema validates every field.

```ts
export const bakery = building("bakery", {
  at: [12, 18],                       // footprint origin on town/square
  floors: [
    {
      id: "ground",
      rooms: [
        { id: "shop",    rect: [0, 0, 8, 6] },
        { id: "kitchen", rect: [8, 0, 4, 6] },
      ],
      openings: [
        { between: ["shop", "kitchen"], at: [8, 3] },          // interior doorway
      ],
      objects: [
        { kind: "counter", room: "shop",    at: [1, 2], activity: "tend-shop" },
        { kind: "oven",    room: "kitchen", at: [9, 1], owner: "mara" },
        { kind: "table",   room: "shop",    at: [6, 1], activity: "eat", capacity: 2 },
      ],
      arrival: [4, 5],
    },
    { id: "upstairs", rooms: [{ id: "bedroom", rect: [0, 0, 6, 6] }], objects: [
        { kind: "bed", room: "bedroom", at: [1, 1], owner: "mara" },
      ], arrival: [3, 4] },
  ],
  doors: [
    { id: "front", from: { floor: "ground", at: [4, 6] }, to: { place: "town/square", at: [16, 25] },
      hours: "06:00-20:00", unlessOwner: true },
    { id: "stairs", from: { floor: "ground", at: [0, 5] }, to: { floor: "upstairs", at: [0, 5] } },
  ],
});
```

Rules carried over from the research, restated for structured source:

- A door is one record with both endpoints and an optional `oneWay`. The
  reverse is implied otherwise.
- Door tiles sit on a room edge, never inside a room's walkable interior.
  Arrival tiles are at least one tile inside the destination and off any
  door tile.
- Place ids are stable paths (`bakery/upstairs`); a `formerIds` list is kept
  on renames.
- Tile coordinates only. Pixels belong to the compiler.
- Schedules name a place id plus a tile or object, never a building id.

Views are generated, never edited: an ASCII render per place (with a
legend), the town overview, tables and itineraries. Any amount of tooling
to produce views is fine. An ASCII-input path can be added as a convenience
if a case ever wants it, but it is not the source.

The compiler turns rooms and openings into wall and floor tiles and runs the
autotiler (a Wang-set resolver for ground and floors, a small pattern
matcher for walls), producing the renderer's own layers. LDtk or Tiled
export can come later if a person wants to open a map in an editor.

## The test-and-adjust loop

Editing is editing text. Testing is running validators and renders in Node.
The loop is edit, run, read, edit.

Validators (fail the suite). Schema validation of the structured source
comes first and catches the typo class the research worried about in ASCII
sources.

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
| Rooms on a floor do not overlap; every room has at least one opening or door | Sealed or doubled rooms |
| Footprints do not overlap on the town grid; each footprint door opens onto a walkable town tile | Buildings in buildings |
| Exterior edge records are mirrored on both zones | Future stitching impossible |
| Inline floors fit their footprint; detached floors carry a reason, and a detached ground floor without one warns | Sprawl by accident, detachment by habit |
| Every inline floor has a roof region covering its rooms and nothing else | Roofs lifting in the garden, or never lifting |
| Capacity declared only where contention is wanted; warn where schedules send more than one NPC to an undeclared place at once | Unintended contention, per the loose-inventory decision |

Renders (snapshot-tested as text files, read by a person or the agent):

- A place at time T with NPCs as letters and a who-is-where legend.
- The town at time T with a count per building and the footprint-to-interior
  ratio per building, for interest.
- A travel-time matrix between places.
- One NPC's day as an itinerary: departures, doors taken, arrivals, waits.
- The door listing per building.

Vitest file snapshots for the text renders, CI never writes snapshots. A PNG
composited with sharp for occasional human review.

Procedural generation is a drafting tool, not a shipping one: a treemap room
split and a greedy furniture pass that respects door paths and wall
adjacency can draft a building, and the result is written out as the same
structured source and edited there.

## Presentation

Inline floors need roof hiding per building from the first visual build.
Ultima VII's single lift cutoff is the simplest form; the documented failure
modes are roofs lifting in the garden or never lifting, which the roof-region
validator above targets. Detached places use scene swap, as in Stardew:
camera bounds per place, an arrival tile, a fade. Floor slicing (Sims,
Project Zomboid) is not planned; upper floors are detached.

With a continuous exterior, most NPC movement is visible, which is the
legibility cost scene swap pays and this layout mostly avoids. Lit windows at
night and door hours remain cheap cues; map markers or a notebook are a
later nice-to-have.

## What to build first

A thin vertical slice in the text-first simulation package:

1. Types for Building, Floor, Room, Door, Object with schema validation;
   the compiler from rooms and openings to tiles.
2. The validators above, as tests.
3. Three buildings (a two-floor house with a detached upstairs, a shop, a
   tavern) inline on a dense town square, with roof regions.
4. The route table with door-to-door weights, grid A*, and the travel-time
   matrix.
5. The time-T render and the one-NPC itinerary trace, snapshot-tested.

Schedules go on top after that. The renderer reads the compiled output
later.

## Open decisions

1. Tile size (16 or 32 px) is an art decision, carried over from the asset
   research; the layout does not depend on it.
2. Where the outdoor place splits, if at all, and whether the first town
   and its surroundings are one outdoor place.
3. Whether map markers or a notebook are wanted at all now that the exterior
   is continuous.
4. Whether door conditions are only hours and owner, or arbitrary
   predicates from the start.
5. Whether rooms are rectangles only, or rectangles plus L-shapes and
   polygons from the start.
6. Whether procedural drafting of buildings is worth building before a
   handful exist by hand.
