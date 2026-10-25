# Viv: what it does, and what it suggests for character knowledge

2026-10-25. Read for the question raised in conversation: knowing where a
belief came from needs some representation of knowledge, and this repo has
none. Viv is James Ryan's emergent-narrative engine, the successor to
Talk of the Town and Hennepin. Read from its documentation and source at
`github.com/siftystudio/viv` (tour, language reference chapters 9 to 20,
and `runtimes/js/src/knowledge-manager`).

## What it is

- A language for authoring character actions, compiled to a JSON content
  bundle, plus a JavaScript runtime that a host game calls through an
  adapter. Symbolic, no models at runtime. Ryan rewrote it as a standalone
  system in 2025.
- License: source-available, free for non-commercial use only. Commercial
  use, including inside a for-profit organization, needs a license from
  Sifty. We can learn from it; we should not copy its code into this repo.
- The runtime is JavaScript with async adapter calls, floats for salience,
  and `Math.random` unless the host supplies an RNG. It could not run inside
  our deterministic Rust sim as is, so the useful part is its design.

## The model

- **Actions are the unit of everything.** An action has roles (initiator,
  partner, recipient, bystander, items, locations, and past actions),
  conditions, effects on host state, and reactions that queue later actions
  or plans. Every performed action goes into the *chronicle*.
- **Knowledge is memory of actions, nothing else.** A character's memory
  holds: the action's ID, when the memory formed, a *salience* (how much it
  matters to them, which is also its strength), *associations* (subjective
  tags such as `betrayal` or `guilt`), the *sources* it came through, and
  whether it has been forgotten. Facts about the world's state (mood,
  personality, where things are) are not known subjectively; conditions
  read them straight from the host.
- **Provenance is a list of actions.** `sources` starts as the action itself
  for a witness, or the relaying action for someone who heard. It grows each
  time they hear of it again or retell it.
- **Knowledge spreads by reference.** When an action casts a past action in
  a role ("gossip about X", "tell a parable made of X, Y, Z"), everyone
  present for it learns of the past action. Items can be *inscribed* with
  actions and later *inspected*, so a note or a monument carries knowledge
  across time.
- **Salience and associations are computed per character, per action,** by
  the action's author: by role (the victim rates it higher than a witness)
  or by a custom block that reads the character. Hearing of it again later
  re-computes associations, so a past event can be reinterpreted.
- **Forgetting** is salience decay over story time, with a threshold below
  which the memory is excluded from queries.
- **Causal bookkeeping.** The runtime records which actions caused which:
  an action casting a past action in a role, a reaction leading to an
  action, relayed knowledge or an inspection prompting a reaction, or the
  host forcing a link. Each action keeps its transitive ancestors.
- **Story sifting.** A sifting pattern names roles and a set of actions with
  causal conditions between them ("an offense caused a vow, the vow caused
  the payback"). Patterns can be run over the whole chronicle or over one
  character's memories, and patterns can be built from other patterns.
  Running a pattern over a character's memories inside an action is how a
  character "knows a story" and can retell it.
- Other constructs: *tropes* (named reusable conditions), *plans* (phased
  sequences of queued actions with waits and timeouts), *selectors*
  (weighted or utility choice between actions or plans), *embargoes* (an
  action cannot repeat for given roles, place or time).

What it does not do: no false beliefs, lies or distortion in retelling.
Talk of the Town had those (mutation, confabulation, lies, transference);
Viv keeps true memories of real actions, held by some and not others.

## What it suggests here

- **An action-centric knowledge representation fits this sim.** We already
  log actions: conversation beats, payments, deliveries, clearing weeds.
  A belief like "Mara is short of money" does not need a fact vocabulary if
  it is the memory of the beat where Mara confided it, with its source.
  Characters' programs read meaning from remembered actions, the way
  `Memory::regard` reads feeling from episodes.
- **We already have the part Viv lacks.** A hearer's uptake records what
  they took a move as (`taken_as`), which can differ from what was done. If
  a memory stores the reading rather than the act, a retelling relays the
  reading, and distortion spreads without a separate mutation mechanism.
  Pip retelling what he overheard would pass on what he understood.
- **Provenance as a list of action IDs is cheap.** It needs actions to have
  stable IDs. Conversations have IDs and beats have indexes; other events
  have a tick and a villager.
- **Salience and associations are where self-talk would live.** Viv computes
  them from the action's definition; here the character's program would
  compute them (Tom attaching "the town is against me" to a trip in the
  yard). That is the per-character appraisal discussed for self-talk.
- **Causes on events are a weak form of causal bookkeeping.** `Cause` now
  holds a trigger string. Pointing causes at the action that caused them
  (the beat Hal misread, the conversation that made him touchier) would turn
  the event log into a causal graph.
- **The summarizer could sift.** It now reports deviations from routine.
  With causal links it could also match authored patterns: a confidence
  misread, then a grudge, then a snub. That is the "summaries tell stories"
  goal approached from the structure side, and it would give a later model
  layer something concrete to narrate.

## Open questions

- Knowledge of state: Viv lets conditions read host state directly. Do we
  accept that for some things (shop hours, where people live) and require
  memory of an action for others (who is broke, who is fishing)?
- Memory size: Viv keeps every memory until it fades. Our `Memory` caps
  episodes at 60 and heard items at 40. Salience-based forgetting would
  replace the caps with something characters can be wrong about.
- Sifting cost: patterns over a large causal graph are search. At our scale
  (five villagers, hundreds of actions a week) it is small; a town of
  hundreds would need incremental matching (Ryan's Felt and Kreminski's
  Winnow address this).
