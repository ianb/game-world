# The Welling: how to get there

From Ian's dossier (`dossier.md`) and the valley as built, 2026-10-31. Ian agreed the
scrap zone first, the repository layout, and keeping the valley playable, and named the
world The Welling. Other names are placeholders.

## What carries over

Most of what the valley built is setting-agnostic already, and some of it maps onto the
Welling closely:

| valley | the Welling | change needed |
| --- | --- | --- |
| the train on a timetable, buying exports and bringing goods | ships calling at the docks: outside fleets buying crust, gas and live fish, bringing what the shelf cannot make; later, newcomers | timetabled arrivals become a general system with a berth instead of a track |
| fields: tile state, growth, water, weeds, fertility, worked tile by tile | seep plant beds (kelp and others) under and beside the rafts, growing fast near a seep and slower as it declines | plants declared in content; "water" becomes the local seep's strength |
| flocks wandering an area, roosting | fish: shoals off the rafts, pens of captive fish; worker fish later | animal kinds declared in content; swimming instead of walking |
| ground wear (cowpaths) and growth (overgrowth) | deck wear along the routes people take, weed and growth on decks nobody uses | none; it is the same mechanic on planks |
| the store, the bars, the luxuries sink | stalls and bars selling kelp, snacks, fish, and gas off a table | goods and prices are content already |
| moods, configured per person | moods moved by what people chew, drink and eat | substances (below) |
| needs (hunger, thirst, sleep, bladder) | the same, plus craving for anything addictive | a craving need per substance |
| conversation, knowledge, gossip, summaries, replay, the inspector | the same | none |

## What is new

In the order each is needed:

1. **Substances.** Anything consumed (kelp, a bar's drinks, raw fish, gas) has effects
   declared in content as mood pushes and need changes: kelp lifts mood and energy a
   little and leaves a mild craving, a drink calms the anxious, gas is a strong high and leaves a craving. Each
   portion comes from a batch with an id, so effects can vary within a batch (sushi's
   unplanned effects) and a bad batch can be traced through who bought it where. Raw
   keeps the seep in fish, cooking removes it. Addiction is a craving need that grows
   with use and fades with abstinence. This is the dossier's "map plant effects onto the
   moods" and it makes moods visible. General: a generation ship has its own substances.
2. **Seeps as sources.** A seep is a point on the map with a strength that rises,
   holds and declines over weeks, feeding the plant beds and gas around it. New seeps
   can appear (found by prospectors, later by worker fish). Strength is state, so the
   map's best places move over time: the main engine of progressive change the dossier
   names. General: a source of anything that waxes and wanes.
3. **A map that can change.** Rafts lash on near new seeps and unmoor from dead ones;
   fire damages decks. This is the changeable-map step already planned
   (`../../design/direction.md`, step 2): world edits in the state, the map and routes
   rebuilt when they land, people finding things by role rather than by id.
4. **Fire.** A spreading danger on a wooden city, with firefighters who come slower to
   those who have not paid. A hand-written disaster with its own geography (it spreads
   along planks, stops at water) and the first perturbation test for this world.
5. **Prices.** A small exchange where crust, gas, live fish and kelp change price with
   supply and the ships' demand, shown as a readout of the city's state.

Later, as the valley's plan already had them: claims on seeps as visible intentions,
the weak council as group intentions, modules for what changes people (fishiness,
diving and the bends, addiction, devotion), secret intentions (the puritans who sell),
the bell-jar levels as places below reached by diving, the haunted quarter.

## The first slice: the scrap zone

One zone, small, to prove the second world end to end before anything else is built.

- **Map.** A shantytown of hulks and rafts lashed together on open water: a dozen or so
  vessels, gangplanks and rope bridges between them, a few cabins with below-deck
  rooms, open decks for work and drinking, a berth where a boat calls. A gas vent rises
  a short way off, with a capture hood moored over it.
- **People.** Eight to ten, each with a program: gas refiners working a table, a stall
  keeper selling refined gas openly, a food fisher who cooks and one who eats raw, a
  junkie, a lasher who keeps the rafts tied, someone who runs drinks. Names and
  descriptions are placeholders for Ian.
- **What happens.** Gas is captured at the hood and refined at tables (work turning raw
  into refined goods); the refined gas sells off the stall to visitors arriving by boat
  and to the local junkie; everyone chews kelp through the day, and its effect shows in
  their moods; fish are caught off the edges, some cooked, some eaten raw with a chance
  of a strange day; decks wear where people walk.
- **What it needs.** Steps 1 and 2 of `../README.md` (the valley moved and generalised),
  substances (kelp, raw and cooked fish, raw and refined gas, craving), the docks as
  timetabled arrivals, a raft tileset (water, planks, hulks, tarps, rope), and a content
  generator for lashed vessels. Seeps can start as a fixed vent; their waxing and waning
  comes next.
- **Not in it.** Fire, the exchange, claims, the council, divers, fish trainers, other
  zones.

## Answered

Ian, 2026-10-31: "sounds good. Fire isn't necessary to start. Kelp isn't like a super
strong thing, just a mood/energy booster, mildly addictive. The repo sounds fine. Keep
valley playable throughout. For the name of the world, let's call it The Welling".
