# The valley

Status: built 2026-10-30. The map, the buildings and 40 new villagers are
placeholders written to get a town of this size running. Names, people and
places are all open to rewriting.

The town grew to a valley about ten times its area: one outdoor zone of 150 by
115 tiles (the old town was 48 by 36) with 32 buildings and 45 villagers. The
old town sits in the middle, unchanged inside, moved by (40, 36).

## Map

North is up. Coordinates are zone tiles.

- **The old town** (x 42 to 86, y 38 to 69): the bakery, the tavern, the house,
  Nell's cottage, Hal's cart yard, the square with its well and bench, and the
  pond. The valley road runs through it along y = 47.
- **The west end**: the chapel and its graveyard, the vicarage, the school, and
  the sexton's cottage.
- **South of the old town**: the Coopers' house and workshop, Ned's shack, and
  the herbalist's cottage at the wood's edge.
- **The farms** (north): the Hollins farmhouse and barn, the Ashby farmhouse and
  barn, their fields, and the boundary between them. The shepherd's hut and
  pasture are on the north-west hill.
- **The river** runs north to south along x = 100 to 102. The valley road
  bridges it, and a second bridge in the south leads to the fisher's shack. The
  mill is on the west bank.
- **The south woods**, with three clearings: the woodcutter, the charcoal
  burner, and the hermit. Tracks cut through to each.
- **The hamlet** (east of the river): the smithy, the store, the alehouse, the
  Reed house (weaver), the Dory house (potter), Clem's lodging, and a green.
- **The manor** (north-east): the house, its garden, and the gardener's lodge.
- **The town hall** is north of the road on the old town's east side: a hall
  with benches and a lectern, and a records room with Pell's desk.
- **The road out** leaves the valley at its east edge (147, 47). Hal goes to
  market that way, and Lottie walks there to look down it.

Named areas (`areas` in the zone) give character code places to work without
coordinates: `square`, `tavern-yard`, `pond-bank`, `road-out`, `graveyard`,
`pasture`, `hollins-fields`, `boundary`, `ashby-fields`, `herb-garden`,
`wood-edge`, `abel-clearing`, `rook-clearing`, `hermit-wood`, `riverbank`,
`manor-garden`, `hamlet-green` and `hiding-spots`. `Ctx::area` panics on an
undeclared id, the way `Ctx::topic` does.

## People

Each villager has a file in [docs/characters](../characters/README.md) with a
description and design notes, and a module in `crates/characters/src` with at
least one behavior of their own. The old town's five are unchanged except that
their hard-coded coordinates became areas.

Households: the Coopers (Rob, Jenna, Kit, Sal), the Pikes (Hester, Lottie), the
Hollinses (Will, Ada, Jory, Tilly), the Ashbys (Silas, Martha, and Dan their
hand), the mill (Gid, Nan), the fisher's shack (Fen, Ottilie), the smithy (Bram,
Greta), the Reeds (Amos, Iris), the Dorys (Penn, Rosa, Benny), the alehouse
(Dot, Wat), and the manor (Celia, Pell, Josie). Alone: Agnes, Elias, Bess,
Wynn, Ned, Moss, Abel, Rook, Ivo, Clem and Thorn.

Programs that involve more than one person:

- Agnes notes late pupils and tells their parents; Kit is late when he looks
  for Pip first.
- Celia calls on Elias and Agnes and asks them to tea; they come on Thursday if
  they heard her.
- Pell collects rent from Will and Silas for Celia.
- Will holds it against Silas when he finds Ashby's people at the boundary.
- Jory seeks out Lottie on Sundays.
- Dot finds the keg short and has it out with Wat, who drinks while she is not
  at the bar.
- Rosa finds Benny when he hides at bedtime.
- Greta asks debtors for what is owed wherever she meets them.
- Rook supplies the smithy with charcoal and Bram burns it.
- Gid and Martha sell flour and honey to Mara; Abel sells firewood to Tom and
  Nell; Rob sells barrels to Tom; Fen sells fish to Dot.
- Hester, Jenna and Clem pass on what they know; Martha says it aloud to her
  bees.

## Saturdays

- The moot meets at the town hall at half past ten: Pell at the lectern, and
  Will, Silas, Rob, Bram, Gid, Amos and Hester on the benches. At half past
  eleven they go back to work and Pell stays on with the accounts.
- Martha, Penn and Iris keep stalls on the square from eight until dinner,
  beside Ada's cheese stall.

Dot brews in the alehouse back room on weekday mornings.

## Materials

Rooms declare a `flooring` (planks, dark planks, flagstone, cobbles, checker,
earth) and buildings their `walls` (timber, stone, brick, boards). Only the
views read them: the 2D map draws each room's floor and each building's wall
faces, and the 3D view uses the matching full-height wall strips. The chapel,
town hall, school, vicarage, smithy, mill and sexton's cottage are stone; the
manor, its lodge and the store are brick; barns, sheds and woodland huts are
boards. Workshops have cobbles, kitchens flagstone, the manor hall checker.

Rooms are furnished with things that say what happens in them (bookshelves,
sacks of flour, barrels, a weapon rack at the old soldier's, a clock in the
hall), and the fields have scarecrows and the road has lamps. Every object kind
in content has its own sprite.

## Economy

Money enters from outside on schedules in content (pensions, wages, stipends,
farm produce, flour, cloth, pots, ironwork, timber, fish, rents to the manor)
and leaves through Hal's market runs, Greta's and Dot's restocking, and the two
bars. It has not been tuned; `cargo run -p tools -- economy --day N` shows
where it drifts.

## Scale

A week of the valley takes about half a second to simulate natively. Routing
was rewritten for it: one A* over every tile of every place with doors as
edges (see `crates/content/src/nav.rs`).

The viewer's 2D map is 4800 by 3680 pixels, which is slightly over the canvas
area iPhone Safari allows. The still parts are cached in a layer repainted
every few seconds.
