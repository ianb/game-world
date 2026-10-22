# World layout: places, doors, floors

Status: proposal, 2026-10-22. Not decided. Responds to the need for a way to
lay out buildings and the town, test the layout, adjust it, and handle
interiors, exteriors and multiple floors, all from a text-only workflow.

## The two models on the table

The two touchstone games solved interiors in opposite ways.

| | Stardew Valley (separate locations) | Ultima VII (one continuous world) |
|---|---|---|
| Interiors | Each building interior is its own map. Door tiles warp the player between maps. | Interiors are on the same grid as the exterior. Roofs are drawn on top and hidden when you walk inside. |
| Floors | Another separate map (farmhouse upstairs), reached by a stair warp. | A Z axis (lift 0 to 15). Stairs change Z. Schedules could not set Z, which the Exult forum lists as a known limit for multi-floor routines. |
| Scale | Interiors are larger than the building's exterior footprint. Players accept this. | Interiors fit the footprint exactly, so buildings are big and towns are sparse. |
| NPC travel | Pathfind within a map; crossing a door is a discrete hop. | Pathfind across the whole world, including through doors. |
| Authoring | Each map is a small independent file. | Every edit touches one huge map; roof layers must be kept consistent. |

Recommendation: the Stardew model, with one addition. The world is a graph of
places connected by doors. A place is a grid (an exterior zone, one floor of
a building, a basement). A door is a directed link from a tile in one place
to a tile in another, with an optional condition (locked, hours, owner).
Stairs are doors. The addition is that a building is a first-class thing: a
footprint on the exterior grid plus a list of interior places, so the town
map places buildings by id and the building carries its own floors.

Why this and not the continuous world:

- Schedules in the research all address places ("be at the bakery 07:00 to
  12:00"), and the place graph makes "where is everyone" a lookup rather than
  a geometry question.
- Pathfinding is hierarchical: A* over the place graph (which doors to take),
  then grid A* inside each place. Off-screen NPCs can move at the place level
  only, which is the simulation-LOD pattern from Assassin's Creed and
  Kingdom Come.
- Each place is a small text block. The agent can read one building without
  loading the town.
- Multiple floors are free: a floor is a place, a stair is a door. The
  Ultima VII Z-axis problem does not arise.
- Roof hiding and interior occlusion disappear as rendering problems.
  Entering a building is a scene cut, as in Stardew.

The cost is the "bigger on the inside" convention and the scene cut on every
door. Both are accepted by the genre. If a later design wants a continuous
exterior with visible interiors, the place graph still works: exterior
places become one large place and interiors stay separate.

## Authoring form: ASCII is the source

The agent writes text, and the text rendition is already decided as the
primary inspection surface, so the map source should be the same thing the
agent reads back. Proposal: each place is an ASCII block plus a legend, in a
TypeScript or YAML file, and buildings bundle their places.

```
building bakery
  footprint (on town grid, 6x4, door at bottom row col 2):
    ######
    #....#
    #....#
    ##D###

  place bakery/ground  (interior, 12x8)
    legend: # wall  . floor  C counter  O oven  T table  S stair-up  D door
    ############
    #OO.....T..#
    #CC.....T..#
    #..........#
    #..........#
    #S.........#
    #..........#
    #####D######
    objects:
      oven      at O*   capacity 1   owner mara
      counter   at C*   capacity 1   activity tend-shop
      table     at T*   capacity 2   activity eat
    doors:
      D  -> town/square at (14, 22)   hours 06:00-20:00 unless owner
      S  -> bakery/upstairs at S

  place bakery/upstairs (interior, 12x6)
    ...
```

Rules the format needs:

- A legend per place (or per building, inherited). Letters mean tile types
  and object kinds; objects then get properties in a list below the map.
- Doors are declared twice, once in each direction, or declared once and the
  reverse is generated. Either way a validator checks the pair.
- A building's footprint door and its ground-floor door are the same door,
  so the town map and the interior stay in sync.
- Place ids are paths (`bakery/upstairs`), so schedules and dialogue can
  address them.
- Grid coordinates are tile coordinates. No pixels anywhere in the source.

The format compiles to whatever the renderer wants later (LDtk or Tiled
JSON, or the renderer's own tile layers). The compiler is one function. The
ASCII stays the source of truth.

## The test-and-adjust loop

Editing is editing text. Testing is running validators and renders, all in
Node, no browser. The loop is: edit, run, read the output, edit again.

Validators (fail the test suite):

- Every door has a target place and tile, the target tile is walkable, and a
  reverse door exists.
- Every place is reachable from a root (the town square) through doors.
- Every scheduled destination in every NPC's schedule is a place or object
  that exists and is reachable from the NPC's previous location within the
  schedule's travel window at walking speed.
- Every object a schedule names is reachable from the place's doors (not
  walled in).
- Every sleeping NPC has a bed, and a bed has at most one owner (this is the
  Kingdom Come "ran out of beds" check).
- Stairs match: a stair-up on floor N has a stair-down on floor N+1, and the
  upper floor's footprint fits within the building's footprint (or the
  mismatch is declared).
- Building footprints do not overlap on the town grid, and each footprint
  door opens onto a walkable town tile.
- Capacity declared only where contention is wanted (see the decision on
  loose inventory); the validator warns on undeclared capacity for places
  schedules send more than one NPC to at once.

Renders (read by a person or the agent, snapshot-tested):

- A place at time T with NPCs drawn as letters and a legend of who is where.
- The town map at time T with a count per building.
- A travel-time matrix between places, so a schedule author can see that
  the mill to the chapel is nine minutes.
- An itinerary trace for one NPC for one day: departure, route as a list of
  doors, arrival, with any waits.
- A "doors" listing per building, so the graph can be checked by eye.

Adjustments the loop supports: move a door and rerun; widen a room; add a
floor; add a building and see which schedules now reach it faster. Because
everything is text, a diff shows the change and the snapshot diff shows the
effect.

## Multiple floors and other vertical cases

- Floors are places. Stairs are doors. A building declares its floors in
  order so renders can stack them.
- Basements and cellars are floors with negative index.
- Roof access, balconies and ladders are the same mechanism.
- Windows are decoration unless a design needs "seen through a window," in
  which case they are a one-way visibility link, not a door. Leave out until
  wanted.
- The exterior can be one place or several zones (town, farm, woods) joined
  by edge doors. Start with one; split when the ASCII block gets unwieldy.

## What to build first

A thin vertical slice, all in the text-first simulation package:

1. Types for Place, Door, Building, Object, and the ASCII parser with a
   legend.
2. The validators above, as tests.
3. Three buildings (a house with two floors, a shop, a tavern) and a town
   square, authored in the format.
4. Hierarchical pathfinding and the travel-time matrix.
5. The time-T render and the one-NPC itinerary trace.

Only after that, schedules on top, then a renderer that reads the compiled
output.

## Open questions

1. Tile size and whether interiors use the same tile size as exteriors
   (Stardew does; it only changes the map dimensions).
2. Whether the format is YAML, a TypeScript literal, or its own small
   grammar. TypeScript gives types and no parser; a grammar reads cleaner.
3. Whether interiors can be larger than footprints without declaration, or
   whether the author states the ratio.
4. Whether the exterior starts as one place.
5. Whether door transitions are instant for the player (Stardew) or whether
   a later continuous-exterior design is wanted, which argues for keeping
   exterior places joined at shared edges from the start.
