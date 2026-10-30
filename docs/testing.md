# Testing

`just check` runs everything CI runs except the browser smoke. This page is
about how behavior tests are written.

## Scenes, not seeds

A behavior test sets up what the behavior needs and forces the chances that
decide it. It does not search for a seed where it happens to occur. A seed that
produces some event today stops producing it the next time anything in the
town changes, and the test then fails for no reason connected to what it
checks.

Two pieces make this possible.

**Named chances.** Every roll that decides whether something happens goes
through `Ctx::chance(rng, who, label, percent)` (`crates/sim/src/odds.rs`).
The label is a `pub const` next to the code that rolls it: `kit::DAWDLES`,
`benny::HIDES`, `sim::social::chances::CONFIDE`, `sim::sim::TRIPS`. A test
forces a label for one villager or everyone; a forced chance returns its
outcome without drawing from the stream. In a normal run nothing is forced.
The forced outcomes are part of `SimState`, so a snapshot carries them.

Choices among options (which tile, which topic, how long to fish) still use
the streams directly. Force those by setting up the state instead.

**`characters::scene::Scene`.** A sim on seed 1 with helpers to set a scene:

- `force` and `force_for` for chances
- `set_coins`, `needs`, `memory`, and `acquaint` (pleasant past conversations,
  so two people know and like each other)
- `put`, `put_by` (next to an object) and `put_beside` (next to a villager);
  a teleport counts as coming to rest, so it fires encounters
- `cue` (a command in a villager's plan at a time today) and `talk`
- `run_until`, which returns what was emitted and keeps all of it in `events`

Day plans are made at midnight, and day 0's when the sim is built. A chance
read while planning (Kit dawdling, Hal calling at the bakery) takes effect from
day 1.

`crates/characters/tests/scenes.rs` has the examples: a confidence taken up and
acted on, news passed on to someone who acts on it, errands queuing, Agnes and
a late pupil, Benny hiding, Penn forgetting. Each behavior with a scene also
has its negative where that is cheap (the same set-up with the chance forced
the other way).

## Long runs

Some tests run the ordinary town for days or weeks and check properties that
should hold on any run: every gift cites words the giver caught, misreadings
stay a minority, nobody is stranded by a conversation. These use seed 1 and
force nothing. If one of them needs a particular event to occur, it should be
a scene instead.

## Snapshots and fixtures

`just snap` accepts insta snapshots; read the diff and give the reason in the
commit. `cargo run -p tools -- events` regenerates `fixtures/day0-seed1.json`.
The golden hash in `golden_hash_after_a_week_matches_the_wasm_smoke` is the native and wasm
determinism check.
