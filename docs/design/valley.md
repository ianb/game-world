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

## The railway

A goods line runs east to west along y = 34, on a bridge over the river. The
station is north of it, off the lane from the square: a goods shed and an
office, with the platform beside the track. Trains come from the west on
Tuesday and Friday, stop from ten to twenty past ten, and leave east; on other
weekdays and Saturday one passes through at three without stopping.

Where a train is follows from the clock (`sim::railway::head_at`), so the
only state is what waits on the platform and whose it is. A stopping train
tops the shed up to a fixed stock (tea, salt, candles, nails, iron, malt) and,
when it leaves, takes everything consigned and pays the senders.

Villagers declare in content what they send (`sends`: Penn's pots, Iris's
cloth, Martha's honey, Rob's barrels, Ada's cheese and eggs, Dot's eggs) and
what they fetch (`fetches`: Greta's shop goods, Bram's iron, Dot's malt). A
fetch says how much to `keep` where it goes, and they take and pay for only the
difference. On a day a train stops their plan gains a walk to the platform half
an hour before it comes, or to the shed ten minutes after, and then back to
what they were doing. Lottie goes to watch the stopping trains and stays until
one has gone.

Dot brews from the malt when the keg runs low, and buys ale in only when there
is none. Greta orders candles from outside on Mondays only when the shelf is
low.

Money comes into the valley from outside (wages, pensions, sales at market and
by rail) and leaves through the railway: what the store, the smithy and the
alehouse buy, and luxuries. Anyone with more than 40 coins goes to the store
once a week, on a weekday afternoon that follows from who they are, and spends
up to half of what they have above 40 on tea, sugar and tobacco, at most 12
things a trip. These are used up. The store pays the railway a coin less for
each than it charges. Greta keeps 200 coins besides to restock with, and
spends half of anything above that on her own luxuries, bought straight from
the goods shed an hour after a train. Over twelve weeks (seed 1) the best-off
level off at around 60 to 130 coins, and Greta at around 330. Tom's tavern
still runs him down to nothing within a month: he is bad at business.

## Animals

Flocks are declared in the zone: six hens of Ada's in a yard north of the
Hollins farmhouse, five of Dot's on the hamlet green, and Moss's eight sheep
in the pasture. Each animal wanders its area on its own clock. Hens roost in
their coop at dusk and come out at dawn, when each may lay (the chance
`hen-lays`); the eggs wait in the coop until someone sends them by rail.

When a villager comes within a tile of a flock they notice it (once in four
hours per flock), and how they take it is theirs: every character module
says how fond it is of each kind of animal, and a new kind makes every one of
them decide. The shared response: delight is said aloud, fondness and
wariness are kept quiet, and annoyance and fear are said aloud and laid at the
owner's door, where the owner may hear of it.

Children who are free (Pip, Kit, Tilly, Benny) run after hens they come
across, and hens run from anyone chasing them within two tiles.

## Farms and gardens

Fields are state, tile by tile (`sim::fields`): a stage (fallow, tilled, sown,
sprouting, growing, ripe, spoiling, stubble), growth toward the crop's need,
water, weeds and fertility, each on a stated scale. Every night the land
changes on its own: water drops a day's worth unless it rains (a nightly
chance by season, `rains`), weeds come up, crops grow 2 a day watered and
weeded, 1 a day weedy or on poor soil, and wilt back 1 a day dry. Growth is
fast: one to two weeks from sowing to ripe. A ripe crop stands 5 days, then
spoils over 3 and is lost; stubble goes fallow; fallow ground gets a point of
fertility back every 4 days.

Farmers work one tile at a time (`toolbox::tend_fields`), most pressing job
first: harvest, weed, water, sow, till. A harvest takes a point of fertility
(beans give one back), and spent ground is rested, not tilled, until it
recovers, so a worked field shows resting patches and its yield falls as it
tires. The harvest goes into the farm's sacks (Hollins wheat, potatoes and
beans; Ashby corn, cabbages and carrots) and Will and Silas send it by rail;
the fixed grain income is gone, so the farms' money follows the land.

Gardens belong to the household that sleeps in the building they are named
for (`household` in content). One of them, in turn, works the garden for 40
minutes from half past five (`toolbox::garden_turn`). A household with no
time for it lets it go to weeds; Thorn's at the lodge does.

`cargo run -p tools -- fields --day N` prints every plot's stages and mean
water, weeds and fertility.

The barns have wide doors and a hay loft up a ladder; Dan sleeps in the
Ashby loft. Couples share a bed.

## The ground

The river winds, three or four tiles wide, held off the buildings along its
banks. The woods have ragged edges. Outdoors are scattered flowers, stones,
bushes, ferns and mushrooms near the woods, reeds along the water and lilies
on it, and the woods mix several kinds of tree, with pines and firs to the
south. Paths over water are drawn as bridges, doorways have sills, and barns
have their doors standing open.

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
