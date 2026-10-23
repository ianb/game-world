# Interactions between villagers

Status: built 2026-10-23 as an experiment in authored routines that interact.
The mechanics are in the sim; the routines are placeholder characters in
`crates/characters` (Mara, Tom, Pip) whose text is not final.

## Mechanics

The sim notices proximity and hands it to character code as triggers.
Character code answers with commands. Nothing talks unless a character
decides to.

| Trigger | When | Cooldown |
|---|---|---|
| `Encounter { other }` | One villager comes to rest and another resting villager is together with them: same room, or both outdoors within 6 tiles. Fired to both. Not fired to or about anyone mid-conversation. | 20 minutes per pair |
| `Passing { other }` | A walking villager steps within one tile of another. Fired to both. | 20 minutes per pair, separate from encounters |
| `TalkedTo { other, topic }` | Someone greeted you or started a conversation with you. | none |
| `TalkEnded { other, topic }` | A conversation finished; your previous activity is already restored. | none |

| Command | Effect |
|---|---|
| `Greet { with }` | Emits `Talked` with topic `greeting` and zero minutes, gives the other a `TalkedTo`. Works while walking. Changes no state. |
| `Talk { with, topic, minutes }` | Both must be in the same place, standing still, and not already talking. Both switch to the `talk` activity, their previous activity is saved, and a `TalkEnd` event restores it and fires `TalkEnded` to each. The other gets `TalkedTo` at the start. If the conditions fail, a `Decided` line says so and nothing happens. |

Rooms come from the compiled content (`World::room_at`), so "together" means
the bakery shop, not the whole zone the bakery sits in.

## What the placeholder routines do with it

- Mara greets anyone she passes unless baking or asleep. When she settles at
  the tavern with Tom there, she talks with him for ten minutes, about
  fishing if he has caught something she has not heard about, otherwise about
  the day. She remembers how many fish she has heard of.
- Tom greets back and is cheered a little by any conversation. His 20:00
  check on the room sees Mara drinking and raises his mood; that mood decides
  whether Mara brings a pastry the next day.
- Pip roams the square between random ground tiles, greets everyone, and at
  noon walks to the bakery counter and talks with Mara about sweets for five
  minutes. He then decides for himself whether he got a bun (Mara has baked
  today), because character code never writes another character's state.

## Rough edges found

- Greetings echo: a greeting back to a greeting is a second `Talked` event,
  which reads fine in the itinerary but doubles the noise. A character that
  wants to greet back should check it was not the one who started it.
- Sleeping villagers had to opt out of greetings in their own code. A sim-level
  rule (no proximity triggers to anyone asleep) may be better once "asleep"
  is a state rather than an activity string.
- Conversations are two-party and have no content. Lines belong to the
  author; the sim only knows topics and durations.
- `Talk` across a counter (Pip outside the shop room? No: the counter is in
  the shop room, so this worked) will need a notion of adjacency to objects
  once rooms are bigger.
