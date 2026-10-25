# Perception, recollection and self-talk

2026-10-25. First spike built. Direction from Ian over several messages (see
the decision log); background in `docs/research/viv.md` and
`docs/research/interiority-psychology.md`.

## Shape

- **Recollection** is what the town keeps of what happened: `SimState.record`
  (`crates/sim/src/recall.rs`). Footsteps, plans and private notes are not
  kept. Most kept events last a day; conversation outcomes, money,
  accidents and thoughts last 28 days. Each kept event records where it
  happened and who was awake and close enough to see it (8 tiles) or hear
  it (16 tiles). Thoughts and how a turn landed are private.
- **Attention** is character code. Every quarter hour an awake villager gets
  `Trigger::Look { since }`, and their program checks what it cares about:
  the state in front of them through `Ctx`, and what they could have
  perceived through `Ctx::recall`. People notice what they look for.
- **Intrusions** reach a villager whatever they attend to. The first is the
  body: tripping in long grass or weeds (`Trigger::Mishap`).
- **Self-talk** is `Command::Think(Thought)`: what it was about, who or what
  is to blame (themself, someone, the world, luck), whether it will last and
  whether it reaches everything (explanatory style, after Seligman), and how
  it felt. It changes nothing by itself; the character's own state does
  that.

Nothing is a knowledge store. A character keeps what it wants in its own
struct; `recall` is the honest way to ask "what could I have perceived".
`Ctx` still shows the whole town, for logistics and common knowledge.

## The placeholder programs

- Tom blames the town, for good and for everything. A trip, a night the
  bar runs out of bread, an evening nobody comes, or a talk that went badly
  each add to a grievance; a night's sleep takes one off. At two he gives a
  dig to whoever he talks to next. His 8pm check now asks what he saw, not
  where Mara is.
- Mara watches the shelf while minding the shop and bakes again if it is
  bare and there is flour. It almost never fires: the shelf only empties at
  6:35am, when she clears yesterday's bread.
- Nell blames herself and expects it to last. Hal blames whoever was near,
  or Tom when it was Tom's yard. Pip does not give a fall a thought. Anyone
  without a style puts it down to luck.

## Found while tuning

- Paths avoid overgrowth, so at first nobody tripped. Even at 15% a step in
  brush, only Tom trips, while weeding.
- Halving the grievance each night meant it never reached three. One off a
  night and a threshold of two gives 0 to 3 outbursts per four weeks.
- Seed 2, Spring 19: Tom trips in the yard and blames the world, misreads
  Mara twice at 7pm, and at 8pm gives Hal a dig instead of his ale. Hal,
  Mara and Nell all take it at -2 and blame Tom; the next evening Hal
  misreads Tom twice.

## Not done

Hearing your own name, raised voices, noticing an absence from what you
expected, relaying what you perceived to others (Pip's blurting), and
thoughts in Ian's words rather than tags.
