# Living world: design notes

Ian's notes from a design conversation in another session, recorded here
verbatim on 2026-10-24. They describe direction, not settled specs. The
section after them maps each point to the code.

---

## Current state (exists)

- A tick-based, fully deterministic world simulation that uses seeded PRNG.
- The world is a low-drama, Stardew Valley–style village.
- Characters have:
  - base needs (hunger, thirst, etc.)
  - jobs and schedules
  - income, plus the ability to pay for and buy things
- Each character has individual behavior code, and much of it is written by the coding agent.
- Behavior is rule-based. There is no LLM in the runtime yet.
- The world currently settles into a stable equilibrium. This is expected and is the intended baseline.
- There are no summaries yet.

## Vision

- A world that feels alive, inspired by Ultima VI. The focus is the world itself, not a player.
- The world is large and has depth, not a single cubicle town.
- The real goal is **stories**: quality soap-opera-style plots that come out of the simulation's details. Emergent sims produce incidents (an affair) but not plots, so plots need deliberate authoring.

## Immediate next step: summaries

Problem: when a day passes, thousands of tick-level events happen, and there is no way to appreciate what happened. This is especially true for indirect LLM influence later on.

Approach:
- **No hand-written log.** You can't fix a log's contents after the fact.
- Instead, write **summary code that derives summaries from the event series**. Because the sim is deterministic, events can be regenerated, so the summarizer can be improved and re-run over old history.
- Events should carry **causal information** where it isn't otherwise recoverable: why a state change happened (e.g. "skipped work because of goal X, which came from plot/trigger Y"). This is the one thing that can't be added retroactively.
- Useful heuristic: summarize **deviations from routine** rather than routine events. "Tom ate breakfast" is noise, and "Tom skipped work" is news.
- Summaries should flag interesting happenings so the user can then **replay and inspect the world at that point in the past** ("go spy on what happened").

## Replay / determinism requirements

- Keep the sim fully deterministic. Replay works from periodic snapshots plus the seed.
- **Use separate RNG streams per subsystem or per character**, not one global stream. That way, adding a random call in one place doesn't shift every later roll.
- **Version the behavior code alongside snapshots.** Agent-written character code changes often, and a replay must run against the rules that were live at the time.
- **Record LLM outputs as inputs to the sim.** Replays should read the recorded responses instead of calling the model again.
- Instrumentation added later must be observe-only, so it can be run over old history.

## Future: LLM plot layer

- The LLM is **not** called every tick. It intervenes occasionally to drive plot.
- This is a two-way loop: LLM plots shape the procedural sim, and sim state informs the LLM.
- **Triggers** are the interface. The LLM registers conditions such as "when these two characters meet, wake me up," and the sim checks them cheaply each tick. When a trigger fires, the LLM decides what happens next, which may mean rewriting character code, creating a scene, or setting new triggers.
  - Open question: how expressive triggers should be. A reasonable start is a small fixed vocabulary of conditions (co-located, alone, bought item, time of day) combined with and/or, widened only when real plots need more. Triggers should probably expire.
- **Thumb on the scale**: build explicit bias hooks into the sim's choice points (who goes where, whether someone notices, etc.) so a plot can push hard toward its outcomes while everything still happens for valid in-world reasons. Biased choices should record which plot nudged them, which feeds the causal summaries.
- Good scenes are **built backwards**: decide the payoff scene, work out its preconditions, then arrange for them. It's best to mine the sim for situations that are already half-true, so the plot feels earned.
- Modeling **knowledge and secrets** (who saw what, who suspects, who's lying) is likely important, because soap opera runs on who knows what.

## Future: scale and levels of detail

- Distant or less interesting regions are simulated more coarsely. Some shortcuts are simple, like teleporting travelers instead of pathfinding.
- LLM plotting also runs at coarser granularity further out.
- **Expand granularity one level at a time**: first a moment (they flirt at a time and place), then a relationship arc, and so on. Each level should rest on the one below it working.
- Levels should **look the same structurally** and differ in fidelity. Zoomed out is closed form: "flirting occurred, with X chance of producing this circumstance."
- Layers must stay consistent, so the world shouldn't behave differently when you zoom in. Coarse probabilities can be **calibrated by running the fine-grained version many times** across seeds.
- Zooming into an already-decided coarse outcome can't exactly reproduce it. One acceptable option is to sample fine-grained runs (or apply biases) until one naturally produces the decided outcome. This amounts to a mild retcon, which is acceptable.

---

## Where the code stands (2026-10-24)

### Summaries

- Derived, not logged: `crates/sim/src/summary.rs` reads the event stream
  and nothing else. `History` (`crates/sim/src/history.rs`) feeds it and
  keeps it outside the sim's state and hash, so it can change freely and be
  re-run by regenerating events from the seed and inputs.
- Deviations first. Each line has keys; a key seen on 3 of the previous 7
  days, or on the same weekday last week, is routine. A habit seen on 5 of
  7 days that did not happen today is its own line, unless it was also
  missing on this weekday last week (Tom's Sunday). Lines that repeat
  within a day fold into one with a count. Digits are ignored when matching
  ("after 6 days" matches "after 5 days").
- What becomes a line: plan changes against the content schedule, planned
  activities that never started (90 minutes' grace), activities started by
  a reaction that were not in the day's plan, conversations, character
  notes, money per villager, running out of money, goods running out,
  urgent needs, getting stuck. Errands of the body and greetings are always
  routine.
- Replay: every summary line carries its tick and who it is about. The
  viewer's days panel replays to that tick from the nearest day snapshot
  and pins the person. While in the past, inputs are refused (they would
  fork history) and a "back to now" button steps forward to the furthest
  point reached.
- Tooling: `cargo run -p tools -- summary --days 14` prints the same thing.

### Causes on events

- `protocol::Cause` is carried by `Activity`, `Departed`, `Stuck`,
  `Talked`, `Cleared`, `Paid`, `Stocked` and `Noted`, and shown in the
  inspector. Values: `Schedule` (the content schedule, unchanged), `Plan`
  (character code added or changed it), `Reaction { trigger }`, `Return`
  (walking back from an errand), `Input { seq }`, `Sim`.
- `Planned` events record each day's plan with which items came from the
  schedule and which schedule items the character dropped.
- Not yet: causes that chain ("skipped work because of goal X, which came
  from plot Y"). There are no goals or plots to point at. When they exist,
  `Cause` gains `Goal { id }` and `Plot { id }` and the chain is a lookup
  from those ids, recorded at the moment the goal or plot is made.

### Replay and determinism

- Snapshots at every day start, plus the seed and the input log, replay any
  moment. A test rewinds, checks inputs are refused, steps back to the
  frontier, and requires the same hash.
- RNG: one `mind` and one `body` stream per villager, seeded from the
  villager's id so adding or reordering villagers never shifts anyone
  else's rolls. One shared `social` stream is reserved.
- Behavior versioning: `characters/build.rs` fingerprints the source of the
  sim, the content compiler and the characters. Snapshots record it and a
  snapshot from different code is refused. Replaying old history under old
  rules means building the commit that made it; mapping fingerprints to
  commits is not automated yet.
- LLM outputs as inputs: inputs already exist as the only outside
  influence (`InputKind`); an LLM response becomes an input kind when there
  is an LLM. Nothing to build until then.
- Observe-only instrumentation: the summarizer is the first instance.

### Not started

- The LLM plot layer: triggers, bias hooks, scenes built backwards,
  knowledge and secrets.
- Levels of detail beyond the existing note on teleports.

### Things the first summaries turned up

Reading a month of summaries found two sim bugs, now fixed: weeding could
target a tile under a privy, and an errand interrupted by a planned item
left a stale return point that a later errand walked back to. It also
shows behaviour that is wrong rather than broken:

- People wake up urgently needing the privy, because needs only ask the
  character hourly and not on waking.
- Tom has no supper in his schedule and is often urgently hungry at night.
- Tom runs out of money every week or two: his bar's margin does not cover
  his buying. That is a story as much as a bug, and the summaries flag it.
