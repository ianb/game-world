# Direction: legibility, lasting change, intentions

Ian's notes from a design conversation, recorded verbatim on 2026-10-30,
followed by where the code stands on each point and a proposed plan. The
plan is a proposal; nothing in it is decided until Ian says so.

---

Context: This is ianb/game-world, a deterministic, tick-based living-world
sim (Rust → wasm, TypeScript viewer). There's a working village: characters
with needs, jobs, schedules, an economy and resources (that part emerged
partly by accident), scored social interactions with feelings, and
per-character behavior routines that find things by type, not hardcoded
coordinates. Below are conclusions from a design conversation. Please read
the codebase, then propose a concrete plan (what to build first, what to
change) before implementing anything big.

1. Legibility comes first
Things happen in the sim, but I can't tell what. It's unclear what the
feelings mean, how they affect behavior, how relationships develop over
time, or even what the steady state of the world looks like. Because of
that, I can't judge whether anything needs improving, since I have no sense
of what "good" looks like. I need ways to see feelings, how they drive
behavior, and what normal looks like versus deviations from it.

2. Exploring needs a lens
I want it to be fun to discover and explore this world. In games, exploring
works because you have some kind of goal. The point isn't necessarily
quests or mysteries. A goal gives you a reason to care about details and to
look at what's happening through a particular lens. It has to go both ways:
what you can do is affected by what's going on in the world.

3. Progressive environmental change
My favorite current feature is desire paths: characters wear in paths by
walking and reinforce them. Right now that's the only way the environment
changes, and otherwise it's fairly static. I want much more of this: a
world that literally changes over time, in a progressive, accumulating way,
not just rhythmically (day/night, seasons). Coming back to a place should
show its history.

4. Characters need to make lasting changes
Give characters the ability to change the world, starting with building
things. Their desires should extend to longer-horizon wants that lead to
projects. They need notions of what can be constructed and what it
requires, tied into the existing resource economy. A construction project is
a plan carried out by someone over time. It can be interrupted, and it fits
the existing custom-routine model (e.g. a house routine that knows what a
house needs and how to build it).

5. Intentions as first-class, visible objects
To stop characters from mindlessly working against each other (one builds,
another tears down), make intentions explicit and acknowledged, which is
lighter than ownership: "I'm building a house here." Others can see an
intention and work toward it, ignore it, or work against it. Working against
a known intention is a deliberate act, which makes it a genuine conflict
that can feed the feelings/drama layer, instead of constant accidental
interference. Visible intentions also help legibility (#1): "she's trying
to build a mill and he opposes it" is something a viewer can read.

6. Group intentions → eventually government
A government is a group intention. To get there, we need a way to express a
proposed intention that hasn't been carried out yet, and a way to gather
others' preferences about it to decide whether it goes ahead. Start simple;
something like elections can come later.

7. Character growth via an extensible module library
Characters' behavior is little programs (motivations, neuroses, etc.).
Sometimes a character really has something new going on and needs new
intentions and understandings. Rather than rewriting the character's
program, they include modules from a shared library (e.g. grieving,
courting). When an agent/LLM is available, the library can be extended with
new modules when nothing existing fits. When no agent is available, the sim
runs with existing modules. Modules should be reusable by other characters
and by entirely different simulations of the world later. This fits the
existing plan of occasional LLM wake-ups via triggers, not LLM calls every
tick.

8. Perturbations ("hand grenades") and robustness tests
Dropping in disruptions (a location destroyed, a drought, a newcomer) is
part of the fun. We need tests for how routines cope when the map changes or
a location disappears entirely. Use determinism: run the same seed with and
without the perturbation and compare. Characters that get stuck or loop are
routines that need fixing.

9. Setting (open question, don't build yet)
The current Stardew Valley-style setting is too cozy: building stuff with
no stakes isn't the game I'm after. I want something punchier, with a tense
surface. Paranoia (clones, the Computer, secret societies) is appealing
partly because cheap death lowers existential stakes while keeping the
tension, and because secret intentions would contrast interestingly with
#5. Keep the systems setting-agnostic so this can change later.

Dwarf Fortress is the reference point for depth, but I want more personal
autonomy for the characters (in DF fortress mode, the player designates most
jobs) and much better legibility.

---

## Where the code stands

**Feelings.** A conversation is a run of gestures; each one a person takes
in has a warmth from -3 to 3 (`Interlocutor::warmth_of`, exhaustive per
character). A conversation leaves each side an `Episode` with its peak, end
and how the other seemed (`social.rs`). `Memory::regard` reads a feeling
about someone, -3 to 3, from the last 8 episodes with them, bad ones
counting double. It is computed when asked and never stored. Mood is the
body (pressing needs) plus the character's own adjustments. Thoughts
(`Emitted::Thought`: a topic, who is blamed, a feeling) are events and
recollections, but they do not feed regard. About a dozen call sites read
regard to decide something (greeting, gossip, kindness, the triangle), and
no event records that a feeling changed a choice. So the answer to "how do
feelings affect behavior" is: a little, invisibly.

**Normal versus deviation.** The summaries already work this way for
events: a line seen on 3 of the previous 7 days is routine, and only
departures are shown. Nothing describes the steady state itself: what a
typical Tuesday looks like for Mara, how many conversations the town has a
day, how regard is distributed and whether it drifts. The `economy` tool is
the only view that shows a trend.

**The map.** `content::World` is compiled from content when the sim starts.
It is not in `SimState`, not hashed and not snapshotted; it cannot change.
The only accumulating change is `terrain.rs`: per-tile wear (tracks and
cowpaths) and growth (tall grass, brush, overgrown paths), in the state and
the hash. Crops and seasons are drawn from the calendar, not simulated.

**Finding things.** Needs, eating, drinking and clearing find objects by
kind (`Ctx::nearest_object`). Most character code and every content
schedule name objects by id: about 56 `go(ctx, "...")` and 93 stock calls
with literal ids. `go` panics on an unknown id. Removing a building today
would crash the sim, not inconvenience a villager.

**Programs.** Each villager is a Rust module behind the `Character` trait
(`plan_day`, `react`, `fondness`) and `Interlocutor` (persona, warmth,
conversation hooks with research-based defaults). The toolbox is already a
shared library in all but name: `chase_hens`, `meet_animals`, `kindness`,
`shopping_trip`, `work_area` are behaviours any character calls. Character
state is a typed struct per villager, serialized in the snapshot.

**Outside influence.** `InputKind` has `Noop`, `Marker` and `Teleport`.
Inputs are the only outside influence and replay with the seed, so an LLM's
output, a perturbation, or a player's action would arrive as a new kind.
Triggers for LLM wake-ups (living-world.md) are not built.

**Robustness signals that exist.** `Stuck` events (no route), summaries of
planned activities that never started, urgent needs. No loop detector, no
"no progress" detector, no with/without comparison.

## Proposed plan

Five steps, in order. Each ends with something Ian can look at in the
viewer and judge. The first is small and changes no behaviour; the second
is the foundation the rest need.

### Step 1: See feelings and see normal (legibility)

Observe-only, so it can run over any history and change freely.

- **Norms.** A tool and a viewer page that describe the steady state from N
  days: for each villager, an hour-by-hour ribbon of what they usually do
  (activity by share of days) beside today's; town-wide counts per day
  (conversations by frame, coins in and out, meals, urgent needs) with
  their usual range. A day outside the range is flagged. This is the
  missing "what good looks like".
- **Relationships over time.** Regard for every pair that has talked,
  sampled each day: a matrix for now, a sparkline per pair, and the
  episodes behind a number (click a feeling, see the conversations that
  made it, with their peak and end). On the person card: who they like,
  who they don't, what changed this week.
- **Why, recorded.** Where character code or the shared defaults use a
  feeling or the mood to choose (greet or not, gossip or not, accept a bid,
  help), record it: a `Cause::Feeling { about, value }` on the resulting
  event, and a summary line when a feeling flipped a choice that is usually
  the other way ("Mara walked past Hal; she usually greets him; feeling -2
  after Tuesday"). Feelings stop being a number in the inspector and become
  reasons attached to what people did.
- **What the numbers mean.** A short legend in the journal: the -3 to 3
  scale, what moves it, what reads it. Generated from the code's constants
  where it can be, so it does not go stale.

Changes no behaviour, so no snapshot churn. Roughly one session for the
tools and data, one for the viewer pages.

### Step 2: A world that can change (foundation for 3, 4 and 8)

- **World edits in the state.** Keep content as the base and add an
  overlay to `SimState`: placed and removed objects, changed cells (wall,
  floor, door, rubble), with the derived `World` and `Nav` rebuilt when an
  edit lands. Edits are hashed and snapshotted like terrain. Snapshot and
  protocol versions bump; the viewer gets a world-changed event and drops
  its cached chunks.
- **Find by role, not id.** Replace literal object ids in character code
  and schedules with roles that resolve at run time ("my bed", "the store
  counter", "the nearest privy", "where I work"). Content declares the
  roles; a role whose object is gone resolves to nothing, and the character
  copes (another one, go home, complain) instead of the sim panicking.
  Roughly 150 call sites; mechanical but wide.
- **Perturbations as inputs.** `InputKind::Destroy { building }`,
  `Block { place, rect }`, `Drought { days }`, and `Arrive { villager }`
  for a newcomer (declared dormant in content, woken by the input, since
  villagers are fixed at start). They replay like any input.
- **The robustness harness.** `tools -- perturb --seed 1 --day 3 --destroy
  store --days 7` runs with and without and prints the difference in norms
  (step 1), plus every villager who got stuck, looped (the same failed
  errand three times), or made no progress on a need. A test runs it for
  every building in turn and fails on a panic, a stuck villager, or a
  missed night's sleep. This is where routines that cannot cope show up.

Two to three sessions. Worth doing before building, because building is a
world edit made by a character instead of an input.

### Step 3: Accumulating change (history you can see)

Generalise what terrain does for wear and growth to other things that
accumulate and are tied to the economy:

- **Trees as a resource.** Abel's timber and firewood come from felling
  real trees, which leave stumps; saplings regrow over seasons. Woods thin
  where people cut, clearings appear near Abel's woodpile and grow back where
  they stop.
- **Condition.** Buildings and objects carry a condition that drops with
  use and weather and rises with repair, which costs materials and time.
  A cared-for cottage and a neglected one look different, and an abandoned
  one decays to rubble (a world edit).
- **Marks of use.** Worn floors, stained counters, a bench polished by
  sitting: cheap to show, and they make a place's history readable.

The rule for all of them: a counter in the state, changed only by what
happens, never by the calendar alone, shown in the viewer. One to two
sessions for trees and condition.

### Step 4: Intentions and the first project

- **Intentions as state.** `SimState.intentions`: who holds it (a villager
  or later a group), what it is (`Build { blueprint, site }`, `Repair`,
  `Fell { area }`, `Oppose { intention }`, more later), when it was
  declared, progress, and status (proposed, active, paused, done,
  abandoned). An intention is known by those who have seen the site or
  heard about it (the existing heard-memory and topic machinery), not by
  everyone.
- **Stances.** A new `Interlocutor`-style hook with a default: how someone
  takes an intention they know about (support, ignore, oppose), from their
  persona, their regard for the holder, and their interests (it is on my
  field, it blocks my path, it competes with my trade). Supporting can mean
  helping (hauling, lending materials); opposing is a deliberate act with a
  cause that cites the intention, and it lands as an episode and a thought
  for both sides. Accidental interference stays possible but stops being
  the default.
- **Blueprints in content.** A house, a shed, a fence, a footbridge: a
  footprint template, materials by stage (timber, stone, nails), labour per
  stage. Validated like the rest of content.
- **A builder module.** Knows how to turn a blueprint into errands: get
  materials (buy at the store, by rail, fell a tree), haul them to the site,
  work a stage, stop for needs and the day's plan, resume. Interrupted work
  stays as a visible half-built site.
- **The first project.** Dan sleeps in the Ashby barn loft. A long-horizon
  want ("a place of my own") becomes an intention to build a cottage on a
  plot near the Ashby farm, saved for over weeks, with Ada and others taking
  stances. One story, end to end, readable in the summaries and on the map.

Three or four sessions. `Cause` gains `Intention { id }`, so summaries can
say why someone was hauling timber.

### Step 5: Modules, then group intentions

- **Modules.** Formalise the toolbox pattern: a `Module` is a typed piece of
  state plus hooks (plan, react, stance, warmth adjustments) that a
  character includes alongside its own program. Modules are an exhaustive
  enum (types over strings), so adding one forces every place that must
  decide about it. The builder from step 4 is the first; grieving and
  courting next. Attaching a module to a character at run time is an input,
  so it replays.
- **On LLM-written modules.** New module code is Rust, so a new module is a
  commit made by an agent at authoring time, which changes the behaviour
  fingerprint like any code change. At run time an LLM wake-up can only
  attach existing modules with parameters, recorded as inputs. If modules
  ever need to be created while a world is running, that needs an
  interpreted module language; not proposed now.
- **Proposals and the moot.** A proposal is an intention in the proposed
  state, held by a group. The Saturday moot in the town hall already
  exists: proposals are raised there, each attendee's stance hook answers,
  and a simple majority decides. Carried proposals become group intentions
  with work handed out; elections are a proposal about who holds a role,
  later.

### Not yet: the lens (2) and the setting (9)

The lens needs intentions to exist, because they are the natural thing to
hook a player's goal onto: back someone's project, stop one, broker
between two, find out who is quietly opposing whom. That is also two-way:
what you can do depends on who holds which intentions and who knows. Worth
designing after step 4, with a player input or two ("put in a word",
"lend materials") rather than an avatar first.

Setting: keep the systems free of village assumptions. In practice that
means roles, blueprints, goods, intentions and perturbations are all
declared in content, and nothing in `sim` names a cottage or a cow. Secret
intentions (known only to the holder and those told) fall out of step 4's
"known by those who saw or heard" rule, which suits a paranoid setting
without building one now.

## Questions for Ian

1. Step 1 first, before any new behaviour? It is the cheapest and it is
   what lets you judge the rest.
2. Find-by-role in step 2 touches most character modules. Fine to do as one
   sweeping change, or would you rather it go module by module as each is
   next touched?
3. Is Dan's cottage the right first project, or is there a project you
   would rather see (a mill, a bridge, something contested)?
4. Agreed that new modules are code written at authoring time, and run time
   only attaches them?

## Revised after Ian's answer (2026-10-30)

Ian's answer, verbatim:

> It was supposed to be more varied than just warmth too, if that's the only
> real emotion. Mostly people could have their own internal feeling
> vocabularies, only the interactions have to be more normalized. One person
> might keep track of resentments, another person might keep track of
> flirting and attraction, another might keep track of intimacy, etc. A
> person might keep track of 2 things or something, but we don't have to
> fill out every attribute for every person; the differences are about
> different focuses and encourage diversity.
>
> Also the emotions are often not with clear units. E.g., Tom mood 0->2...
> 0 to 2 out of what? What kind of mood? Happiness? Mood is a category of
> attributes.
>
> I think for disasters we should mostly treat them as hand-coded (by you)
> events that affect things. They might use randomness, but like a flood
> doesn't place water anywhere, it places it by the stream.
>
> The character probably has their own flags that basically dispatch to
> known modules. Maybe there's a core set, like grieving is a module that is
> invoked if there's something that causes grieving.
>
> Let's not worry about intention and contention right now, just change.
> Maybe an easy place is farming, where the farmer actually affects their
> land, and the land changes automatically. Of course we should probably
> have accelerated growth, waiting months for stuff to resolve is too long.

What changes in the plan:

- **Intentions, contention and group decisions (old steps 4 and 5) are
  set aside.** The next work is change in the world.
- **First: living fields.** Each field tile is state: tilled or not, what
  is sown, how far it has grown, water, weeds, fertility. Growth, drying,
  weeds and rot happen daily on their own, and fast (a crop ripens in one to
  two weeks, not a season). Farmers till, sow, water, weed and harvest tile
  by tile, and the harvest is goods that go into the economy. A neglected
  field goes to weeds and back to grass; a tended one shows it. Gardens
  nobody tends show that too.
- **Disasters are hand-written events,** each with its own logic for where
  it lands: a flood rises along the stream, a drought dries the fields and
  the pond, a storm fells trees at the wood's edge. They may roll dice, but
  within their own geography. The with/without harness from step 2 still
  applies. Drought comes first because it acts on the fields.
- **Feelings become per-person vocabularies over a normalized interaction
  layer.** Interactions (gestures, help, slights, gifts, disclosures) emit a
  small fixed set of normalized signals. Each person keeps only the feelings
  their character cares about, two or three each, from a shared library of
  feeling kinds (resentment, attraction, intimacy, gratitude, trust,
  rivalry, fear, and so on), each with named levels instead of bare numbers
  ("sore, 2 of 4: took ale without paying, Tuesday"). Mood stops being one
  number; it is a category with named members (contentment from the body,
  loneliness, stress), each with its own scale. Every level change is an
  event with a cause, which is the legibility half of step 1.
- **Modules are dispatched by the character's own flags,** with a core set
  the shared rules invoke when something happens (grieving when someone they
  care about dies or leaves). Same exhaustive-enum shape as before.
- **Order now:** living fields; drought; feeling vocabularies and the norms
  views (old step 1); the changeable map and find-by-role (old step 2) as
  disasters that move things need it; trees and building condition (old
  step 3).
