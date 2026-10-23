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

Reading taken here (correct it if wrong): "single scale" means one
consistent world scale, so an interior fits inside the footprint of its
building rather than being larger on the inside. Not just one tile size.

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
