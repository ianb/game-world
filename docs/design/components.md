# Components, systems and modules

How the sim is organised, borrowing the useful parts of an entity-component-system
design without an ECS library. Written 2026-10-31, after Ian asked for "a bit of
thought into that structure, with modules and ECS inspiration (but I agree not to use an
ECS library)". Moods are the first thing built this way.

## The shape

**Entities are ids.** `VillagerId`, `ObjectId`, `PlaceId`, a plot index, a tile `Pos`.
There is no entity object; an id is an index into tables.

**Components are typed tables keyed by id, in `SimState`.** One table per concern,
each a plain `Vec` indexed by id or a `BTreeMap` keyed by it:

| table | keyed by | holds |
| --- | --- | --- |
| `villagers` | villager | where, doing what, needs, coins |
| `moods` | villager | each configured mood's level and why |
| `terrain` | place, tile | wear and growth |
| `fields` | plot, tile | stage, growth, water, weeds, fertility |
| `stock` | object | goods |
| `animals` | animal | where, roosting |
| `characters` | villager | each character's private program state |

A table only exists for the entities that have the component. Moods are kept per
villager but each villager only has the moods their configuration gives them.

**Configuration is data in content, per entity.** A villager's file says which
components they carry and with what parameters: their schedule, income, what they send
by rail, and now which moods they are prone to and what sets each off. Kinds are closed
enums (types over strings), so content cannot name a mood or a source the code does not
handle, and a new variant makes every match decide.

**Systems are functions over tables, run by the event queue.** Not once a frame:
the discrete-event scheduler wakes a system when something it cares about happens or
its own timer fires. `sim/fields.rs` runs nightly; `sim/animals.rs` on each animal's
clock; `sim/railway.rs` on the timetable; the mood system hourly per villager and on
events. A system reads what it needs through the sim and writes only its own tables,
emitting events for the rest.

**Events are the bus between systems.** Every change worth knowing is an `Emitted`
event. Systems that react to other systems' changes read those events after each step
(`keep_record` already does this for recollection; the mood system does the same), so
the fields system never has to know that a good harvest makes Will proud.

**Triggers carry a system's news into a character's program.** A threshold crossed in
a component becomes a `Trigger` delivered to that villager's `react`. The character's
own code answers first; anything it passes on gets the shared answer from the toolbox.
This is how needs work, and moods follow the same path (`Trigger::Mood`).

**Views read tables, never write.** Protocol views, tools and summaries are
observe-only, so they can change freely and be run over old history.

Why not an ECS library: iteration order must never vary (replay), every table must
serialize into the snapshot and the hash, and closed enums give the compiler the job of
finding every place a new kind has to reach. A general component store with run-time
queries gives up all three. What is borrowed is the discipline: data in tables by id,
behaviour in systems, configuration as data, events between them.

## Modules

A module is a reusable piece of a character's inner life (grieving, courting,
building something): a component plus the behaviour that goes with it. It has the same
four parts as everything else:

- a typed state, in a table keyed by villager (the characters crate holds module
  tables beside each character's own state);
- configuration: which villagers carry it from the start, set in content, and with what
  parameters;
- hooks: `plan` (add to the day), `react` (answer triggers), and the conversation hooks
  it wants to adjust;
- the triggers that switch it on.

Dispatch is by flags. A villager's modules are a closed enum set; `Town::react` offers a
trigger to the character's own program, then to each module it carries, then to the
shared toolbox. A core set is switched on by the shared rules when something happens
(grieving when someone they care for dies or leaves), and a switched-on module is an
event, so it replays and summarises.

New module code is Rust written at authoring time. At run time an LLM wake-up, an
input, or a shared rule can only attach an existing module with parameters.

## Moods, the first worked example

Moods are a category of named states, not one number. Each villager is prone to two or
three of them, from a shared library, each with personal sources:

| mood | feels | rises with | eases with |
| --- | --- | --- | --- |
| lonely | bad | hours without company, past their own limit | a conversation |
| restless | bad | idle hours, past their own limit | work, a walk |
| irritable | bad | their irritants: noise, crowds, slights, getting stuck | time |
| anxious | bad | their worries: money below their own mark, the dark, strangers, their garden | the worry passing |
| gloomy | bad | their glooms: rain | time |
| cheerful | good | their delights: rain, a fine day, company, good things that happen | time |
| proud | good | what they take pride in: their harvest, money earned, their own work, their garden | time |

Each has four levels with names (none, then for lonely: missing company, lonely,
desolate), on an internal scale of 12 points (4 a level), so every number shown has a
unit and a meaning. A mood's last cause is kept and shown ("lonely: no company since
9am").

Spirits replace the old bare mood number: the good moods' levels, less the bad ones',
less discomfort from the body's needs, -3 to 3, each step named. Conversations read
spirits where they used to read mood, so a proud smith and an irritable cooper talk
differently.

When a mood rises to its second level, the villager gets `Trigger::Mood`. Their own
program can answer; otherwise the shared answer is small and visible: the lonely look
for someone to talk to, the anxious and the proud think about what moved them, the
restless take a walk. Every change of level is an event (`MoodShifted`) with its cause,
summaries report a mood reaching its second level, and the inspector shows each mood
with its level, scale and cause.

The level names and the configurations are placeholders written from the character
notes; Ian to rename and rewrite.
