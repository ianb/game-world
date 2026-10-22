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
