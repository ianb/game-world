# The Welling: principles

The rules of this world as checkable statements, distilled from Ian's dossier
(`dossier.md`), so the systems and the content stay consistent with each other, and so
the differences between groups stay sharp. Each says whether it is decided (from the
dossier or by Ian), proposed (by the agent, for Ian to confirm) or open, and where it is
built once it is.

The dossier is the source; where this page and the dossier differ, the dossier wins.

## The world

- **Decided.** A city of boats and rafts lashed together, anchored, on a wide shallow
  shelf entirely under water. No land anywhere near.
- **Decided.** Vessels do not slowly sink. Nothing is built on sinking.
- **Decided.** Seep rises from below at many places across the shelf. Seeps are found,
  claimed and lashed onto; old ones decline. This is the main engine of change.
- **Decided.** No cheap death and no resurrection. Relationships and grudges last.
- **Decided.** One mixed city. Groups overlap in bars, markets and families; zones are
  not walled.
- **Decided.** A haunted, unoccupied part of the city that nobody will unmoor (lore
  idea).

## The seep

- **Decided.** The central bargain: people trade a bit of their humanity for the seep,
  which is useful, valuable and intriguing, not only economically necessary. It is used
  locally and sold.
- **Decided.** Kelp, a seep plant, is a mild stimulant chewed all day by nearly
  everyone: background life. Ian, 2026-10-31: a mild mood and energy booster, mildly
  addictive, not a strong thing. Other seep plants have other effects. Bars serve drinks and
  snacks made from them.
- **Decided.** Plant effects map onto the moods the sim already has, so they can be
  tuned and so moods become visible.
- **Decided.** Seep potency survives in raw fish and is lost in cooking. Raw eaters are a
  little odd. A piece of raw fish can have an unplanned effect on someone's mood that
  day, sometimes bad, and a bad batch can hit many at once, traceable through who bought
  where.
- **Decided.** Crust, seep crystallised on the seabed, is mined by divers in hardened
  suits and is the most valuable export. **Open:** the physical cost of crust exposure.
- **Decided.** Gas is captured where bubbles rise. A little raw gas does something;
  refining makes the good stuff. It cannot be burned as fuel. It has versatile uses (over
  a fire, infused into iron for better metal). It is addictive: junkies, dealers, a black
  market. Restricted in most places.

## Every group gives something up

- **Decided.** Each community gives something up for what it has.
- **Decided.** Fish catchers and trainers (not food fishers) catch exotic fish, breed
  them and train worker fish to find things such as seeps. They become fish-like over
  time. A go-between group deals with everyone else for them. Convention: a protected or
  worker fish that is caught is put back; breaking that causes real disputes.
- **Decided.** Divers live long stretches below in bell-jar habitats, taking or wearing
  things that make it possible and that change them. Surfacing is a big deal. The bell
  levels are part of the same city, not a separate society.
- **Decided.** Puritans are a way of life, not a trade: regimented and devotional, they
  cook everything and oppose the corrupting parts. Their fishers do not grow fishy and
  catch worse fish. They are good gas refiners because their discipline keeps them off
  their own supply. Some secretly sell to addicts, against their creed.
- **Decided.** The scrap zone is a shantytown of discards. Its gas refining is cheap and
  poorer, and sold openly off a table even where restricted.
- **Decided.** Fire is the shared danger. Not needed for the first slice (Ian,
  2026-10-31). Cooking fires are normal; heavy uses such as
  smelting may be regulated. Firefighters extort a little: slower if you have not paid.
- **Decided.** Clerks are everywhere, concentrated downtown, perhaps with an exchange for
  crust, gas, live fish and kelp. Prices are a readout of the world.
- **Decided.** The docks: sailors and shipping, outside fleets buying exports, newcomers
  arriving.
- **Decided.** Division of labour (by gender, for instance) is a zone's or a guild's
  convention, never citywide.
- **Open.** Lashers and shipwrights who hold the city together, apothecaries, a weak
  council of zones.

## Proposed

- **Proposed, built.** Every substance is declared with its effects: a lift to spirits
  for some hours, pushes to named moods, sleepiness taken away
  (`crates/sim/src/substances.rs`).
- **Proposed, built.** Addiction is a habit that deepens with use and eases without, and
  between doses a craving grows at a rate the habit sets. A desperate craving lowers
  spirits.
- **Proposed, built.** Every portion of a substance belongs to a batch with an id, made
  by whoever stocked it, carried with the goods from object to object, so a bad batch can
  be followed through the people it reached.
- **Proposed.** A seep's strength is state that rises, holds and declines over weeks,
  and what grows or bubbles near it follows its strength.
