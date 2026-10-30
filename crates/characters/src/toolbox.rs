//! What character code is written against. Grows from what characters want to say.
use rand::RngExt;
use rand_pcg::Pcg32;
use sim::brain::{Command, Ctx, Party, Planned, Target, Trigger};
use sim::ids::{ObjectId, VillagerId};
use sim::needs::Need;
use sim::protocol::{Blame, Cause, Emitted, Frame, Mishap, Thought, Topic};
use sim::recall::Perceived;
use sim::social::Interlocutor;
use sim::time::{TICKS_PER_MINUTE, Tick, TimeOfDay, Weekday};

/// A day's intentions, built up and handed back to the sim.
#[derive(Debug, Default)]
pub struct Plan {
    items: Vec<Planned>,
}

impl Plan {
    #[must_use]
    pub fn new() -> Plan {
        Plan::default()
    }

    /// Start from the content schedule and modify it.
    #[must_use]
    pub fn baseline(who: VillagerId, ctx: &Ctx<'_>) -> Plan {
        Plan {
            items: sim::brain::baseline_plan(who, ctx),
        }
    }

    pub fn at(&mut self, hour: u32, minute: u32, command: Command) -> &mut Plan {
        self.items.push(Planned {
            at: TimeOfDay::new(hour, minute, 0),
            command,
        });
        self
    }

    /// Drop every planned item whose activity is `activity`.
    pub fn drop_activity(&mut self, activity: &str) -> &mut Plan {
        self.items.retain(|p| !matches!(&p.command, Command::GoTo { activity: a, .. } | Command::Wait { activity: a, .. } if a == activity));
        self
    }

    /// Move the item at `from` to `to` (same day), if present.
    pub fn shift(&mut self, from: (u32, u32), to: (u32, u32)) -> &mut Plan {
        let from = TimeOfDay::new(from.0, from.1, 0);
        for p in &mut self.items {
            if p.at == from {
                p.at = TimeOfDay::new(to.0, to.1, 0);
            }
        }
        self
    }

    #[must_use]
    pub fn build(self) -> Vec<Planned> {
        self.items
    }
}

/// Walk to an object and do something there. Panics on an unknown object id, which the
/// content validator would have caught; character code may name only real objects.
#[must_use]
pub fn go(ctx: &Ctx<'_>, object: &str, activity: &str) -> Command {
    let id = ctx
        .object_id(object)
        .unwrap_or_else(|| panic!("no object {object:?}"));
    Command::GoTo {
        target: Target::Object(id),
        activity: activity.to_string(),
    }
}

#[must_use]
pub fn go_place(ctx: &Ctx<'_>, place: &str, activity: &str) -> Command {
    let id = ctx
        .place_id(place)
        .unwrap_or_else(|| panic!("no place {place:?}"));
    Command::GoTo {
        target: Target::Place(id),
        activity: activity.to_string(),
    }
}

#[must_use]
pub fn go_tile(ctx: &Ctx<'_>, place: &str, x: u16, y: u16, activity: &str) -> Command {
    let id = ctx
        .place_id(place)
        .unwrap_or_else(|| panic!("no place {place:?}"));
    Command::GoTo {
        target: Target::Tile {
            place: id,
            pos: content::Pos { x, y },
        },
        activity: activity.to_string(),
    }
}

/// Walk to a random spot in a declared area and do `activity` there. `None` when the area
/// has no free tile.
#[must_use]
pub fn go_area(ctx: &Ctx<'_>, area: &str, activity: &str, rng: &mut Pcg32) -> Option<Command> {
    ctx.random_tile_in(area, rng)
        .map(|(place, pos)| Command::GoTo {
            target: Target::Tile { place, pos },
            activity: activity.to_string(),
        })
}

/// Whether a villager is standing in a declared area.
#[must_use]
pub fn in_area(ctx: &Ctx<'_>, who: VillagerId, area: &str) -> bool {
    let (place, rect) = ctx.area(area);
    let v = ctx.villager(who);
    v.place == place && rect.contains(v.pos.x, v.pos.y)
}

#[must_use]
pub fn wait_minutes(minutes: u32, activity: &str) -> Command {
    Command::Wait {
        ticks: minutes * TICKS_PER_MINUTE,
        activity: activity.to_string(),
    }
}

#[must_use]
pub fn is_weekend(ctx: &Ctx<'_>) -> bool {
    ctx.calendar.weekday.is_weekend()
}

#[must_use]
pub fn weekday(ctx: &Ctx<'_>) -> Weekday {
    ctx.calendar.weekday
}

/// Is `other` in the same place as `me` right now?
#[must_use]
pub fn together(ctx: &Ctx<'_>, me: &str, other: &str) -> bool {
    match (ctx.villager_id(me), ctx.villager_id(other)) {
        (Some(a), Some(b)) => ctx.villager(a).place == ctx.villager(b).place,
        _ => false,
    }
}

/// The current activity of a villager by content id, or "" if unknown.
#[must_use]
pub fn activity_of<'a>(ctx: &'a Ctx<'_>, id: &str) -> &'a str {
    ctx.villager_id(id)
        .map_or("", |v| ctx.villager(v).activity.as_str())
}

/// One of `choices`, uniformly, from the given stream.
pub fn pick<'a, T>(rng: &mut Pcg32, choices: &'a [T]) -> &'a T {
    let n = u32::try_from(choices.len()).expect("small choice list");
    &choices[rng.random_range(0..n) as usize]
}

/// Prices, in coins. One table so the whole economy can be retuned in one place.
pub const BREAD_PRICE: u32 = 2;
pub const ALE_PRICE: u32 = 3;

#[must_use]
pub fn villager(ctx: &Ctx<'_>, id: &str) -> VillagerId {
    ctx.villager_id(id)
        .unwrap_or_else(|| panic!("no villager {id:?}"))
}

#[must_use]
pub fn object(ctx: &Ctx<'_>, id: &str) -> ObjectId {
    ctx.object_id(id)
        .unwrap_or_else(|| panic!("no object {id:?}"))
}

/// `from` pays `to`; either may be "outside" for the world beyond the town.
#[must_use]
pub fn pay(ctx: &Ctx<'_>, from: &str, to: &str, amount: u32, what: &str) -> Command {
    let party = |id: &str| {
        if id == "outside" {
            Party::Outside
        } else {
            Party::Villager(villager(ctx, id))
        }
    };
    Command::Pay {
        from: party(from),
        to: party(to),
        amount,
        what: what.to_string(),
    }
}

#[must_use]
pub fn stock(ctx: &Ctx<'_>, object_id: &str, good: &str, delta: i32) -> Command {
    Command::Stock {
        object: object(ctx, object_id),
        good: good.to_string(),
        delta,
    }
}

#[must_use]
pub fn stock_of(ctx: &Ctx<'_>, object_id: &str, good: &str) -> u16 {
    ctx.stock(object(ctx, object_id), good)
}

/// A short errand at an object, then back to whatever was going on.
#[must_use]
pub fn errand(target: ObjectId, activity: &str, minutes: u32) -> Command {
    Command::Interlude {
        target: Target::Object(target),
        activity: activity.to_string(),
        minutes,
    }
}

/// Buy and eat a loaf at the bakery shelf: the stock drops, the baker is paid, the buyer
/// eats there. None when there is no bread or no money.
#[must_use]
pub fn buy_bread(ctx: &Ctx<'_>, me: &str) -> Option<Vec<Command>> {
    if stock_of(ctx, "shelf_bakery", "bread") == 0 || ctx.coins(villager(ctx, me)) < BREAD_PRICE {
        return None;
    }
    let mut cmds = vec![stock(ctx, "shelf_bakery", "bread", -1)];
    if me != "mara" {
        cmds.push(pay(ctx, me, "mara", BREAD_PRICE, "bread"));
    }
    Some(cmds)
}

/// Pay for a drink from the tavern barrel. None when the barrel is dry or the buyer broke.
#[must_use]
pub fn buy_ale(ctx: &Ctx<'_>, me: &str) -> Option<Vec<Command>> {
    if stock_of(ctx, "barrel_tavern", "ale") == 0 || ctx.coins(villager(ctx, me)) < ALE_PRICE {
        return None;
    }
    Some(vec![
        stock(ctx, "barrel_tavern", "ale", -1),
        pay(ctx, me, "tom", ALE_PRICE, "ale"),
    ])
}

/// How long a word at the counter is booked for; it can end sooner.
pub const COUNTER_MINUTES: u32 = 15;

/// A conversation at the counter about the topic `topic` with whoever keeps it, if they are
/// here, still and free.
/// `None` when there is nobody to talk to, so the caller can carry on alone.
#[must_use]
pub fn counter_talk(ctx: &Ctx<'_>, me: &str, keeper: &str, topic: &str) -> Option<Command> {
    if me == keeper {
        return None;
    }
    chat(
        ctx,
        villager(ctx, me),
        villager(ctx, keeper),
        Frame::Transaction,
        topic,
        COUNTER_MINUTES,
    )
}

/// A conversation with `other` if they are here, still and free.
#[must_use]
pub fn chat(
    ctx: &Ctx<'_>,
    me: VillagerId,
    other: VillagerId,
    frame: Frame,
    topic: &str,
    minutes: u32,
) -> Option<Command> {
    let (mine, theirs) = (ctx.villager(me), ctx.villager(other));
    let free = me != other
        && theirs.place == mine.place
        && theirs.moving.is_none()
        && mine.moving.is_none()
        && !matches!(theirs.activity.as_str(), "talk" | "sleep");
    free.then(|| Command::Talk {
        with: other,
        frame,
        topic: ctx.topic(topic),
        minutes,
    })
}

/// Today's trains, in timetable order.
#[must_use]
pub fn trains_today(ctx: &Ctx<'_>) -> Vec<sim::railway::Timing> {
    let (Some(rail), Some(spec)) = (
        ctx.world.rail.as_ref(),
        ctx.content.zones.iter().find_map(|z| z.railway.as_ref()),
    ) else {
        return Vec::new();
    };
    sim::railway::timings(spec, rail, ctx.tick.day_start(), ctx.calendar.weekday)
}

/// The railway platform's object id, if there is a railway.
#[must_use]
pub fn platform<'a>(ctx: &Ctx<'a>) -> Option<&'a str> {
    let rail = ctx.world.rail.as_ref()?;
    Some(ctx.world.objects[rail.platform].id.as_str())
}

/// At the platform: everything they send by rail, from where they keep it.
#[must_use]
pub fn send_goods(ctx: &Ctx<'_>, me: &str) -> Vec<Command> {
    let Some(v) = ctx.content.villagers.iter().find(|v| v.id == me) else {
        return Vec::new();
    };
    v.sends
        .iter()
        .filter(|x| stock_of(ctx, &x.object, &x.good) > 0)
        .map(|x| Command::Consign {
            from: object(ctx, &x.object),
            good: x.good.clone(),
            count: u16::MAX,
        })
        .collect()
}

/// At the goods shed: what they fetch, as much as is there, paid for and carried to where
/// it goes.
#[must_use]
pub fn fetch_goods(ctx: &Ctx<'_>, me: &str) -> Vec<Command> {
    let (Some(v), Some(rail)) = (
        ctx.content.villagers.iter().find(|v| v.id == me),
        ctx.world.rail.as_ref(),
    ) else {
        return Vec::new();
    };
    let shed = ctx.world.objects[rail.shed].id.clone();
    let who = villager(ctx, me);
    let mut cmds = Vec::new();
    for f in &v.fetches {
        let wanted = f.keep.saturating_sub(stock_of(ctx, &f.to, &f.good));
        let n = stock_of(ctx, &shed, &f.good).min(wanted);
        let afford = ctx.coins(who) / f.price.max(1);
        let n = u32::from(n).min(afford);
        if n == 0 {
            continue;
        }
        let n = i32::try_from(n).unwrap_or(0);
        cmds.push(stock(ctx, &shed, &f.good, -n));
        cmds.push(stock(ctx, &f.to, &f.good, n));
        cmds.push(Command::Pay {
            from: Party::Villager(who),
            to: Party::Outside,
            amount: f.price * n.unsigned_abs(),
            what: f.good.clone(),
        });
    }
    cmds
}

/// A child's answer to hens close by when they are free to: run at them for a few
/// minutes, delighted, then back to whatever they were doing. Hens run from a chaser.
#[must_use]
pub fn chase_hens(ctx: &Ctx<'_>, me: &str, trigger: &Trigger) -> Option<Vec<Command>> {
    let Trigger::Animals {
        kind: content::AnimalKind::Chicken,
        owner,
        place,
        at,
    } = trigger
    else {
        return None;
    };
    let busy = matches!(
        activity_of(ctx, me),
        "school" | "sleep" | "talk" | "service" | "chase-hens"
    );
    if busy {
        return None;
    }
    Some(vec![
        think_aloud(
            ctx.topic("hens").about(*owner),
            Blame::Them(*owner),
            false,
            false,
            2,
        ),
        Command::Interlude {
            target: Target::Tile {
                place: *place,
                pos: *at,
            },
            activity: "chase-hens".into(),
            minutes: 4,
        },
        Command::Note("after the hens".into()),
    ])
}

/// The shared response to animals close by, by how the person takes to them. Their own
/// animals get no remark. Pleasure is quiet unless delight; annoyance and fear are said
/// aloud and laid at the owner's door, so the owner may hear of it.
#[must_use]
pub fn meet_animals(
    ctx: &Ctx<'_>,
    me: VillagerId,
    kind: content::AnimalKind,
    owner: VillagerId,
    fondness: crate::Fondness,
) -> Vec<Command> {
    use crate::Fondness as F;
    if me == owner {
        return Vec::new();
    }
    let topic = ctx.topic(sim::brain::animal_plural(kind)).about(owner);
    match fondness {
        F::Delighted => vec![think_aloud(topic, Blame::Them(owner), false, false, 2)],
        F::Fond => vec![think(topic, Blame::Them(owner), false, false, 1)],
        F::Indifferent => Vec::new(),
        F::Wary => vec![think(topic, Blame::Luck, false, false, -1)],
        F::Annoyed => vec![think_aloud(topic, Blame::Them(owner), false, false, -1)],
        F::Afraid => vec![think_aloud(topic, Blame::Them(owner), false, false, -2)],
    }
}

/// Coins someone keeps back before they will give any away.
pub const SPARE_COINS: u32 = 12;
/// What a kindness gives.
pub const KINDNESS_COINS: u32 = 3;

/// The shared rule for helping someone you know is in trouble. You heard, in words and in
/// the last week, a topic whose content marks trouble (`content/topics`) about `other`;
/// you like them (feeling 1 or more) and are warm by nature (warmth 1 or more); you have
/// coins to spare; and you have not helped them since you heard it. You give a few coins,
/// and the payment cites what you heard. Run when you have just talked with or met them.
#[must_use]
pub fn kindness(
    ctx: &Ctx<'_>,
    me: VillagerId,
    other: VillagerId,
    mind: &dyn Interlocutor,
) -> Vec<Command> {
    if mind.persona().warmth < 1
        || mind.memory().regard(other).feeling < 1
        || ctx.coins(me) < SPARE_COINS
    {
        return Vec::new();
    }
    let week = ctx.tick.minus(7 * sim::time::TICKS_PER_DAY);
    let trouble = mind
        .memory()
        .heard()
        .words()
        .about(other)
        .since(week)
        .iter()
        .filter(|h| {
            h.topic
                .as_ref()
                .and_then(|t| ctx.content.topic(&t.key))
                .is_some_and(|d| d.trouble.is_some())
        })
        .last();
    let Some(h) = trouble else {
        return Vec::new();
    };
    let helped = ctx.recall(me, h.tick).any(|(r, p)| {
        p == Perceived::Did
            && matches!(&r.event, Emitted::Paid { to: Some(to), cause: Cause::Knew { .. }, .. } if *to == other)
    });
    if helped {
        return Vec::new();
    }
    vec![
        Command::Because {
            from: h.speaker,
            topic: h.topic.clone().expect("words were caught"),
            heard_at: h.tick,
        },
        Command::Pay {
            from: Party::Villager(me),
            to: Party::Villager(other),
            amount: KINDNESS_COINS,
            what: "help".into(),
        },
    ]
}

/// Hunger at which anyone eats something before going to sleep.
pub const BEDTIME_HUNGER: u8 = 80;
/// A meal due within this many minutes is worth waiting for, even starving.
pub const MEAL_WINDOW_MINUTES: u32 = 60;

/// The table in the villager's home, which stands in for the larder.
#[must_use]
pub fn home_table(ctx: &Ctx<'_>, me: &str) -> Option<ObjectId> {
    let home = &ctx.content.villagers.iter().find(|v| v.id == me)?.home;
    ctx.world
        .objects
        .iter()
        .position(|o| o.kind == "table" && &o.building == home)
        .map(|i| ObjectId(u16::try_from(i).expect("few objects")))
}

/// Eats something now: a loaf from the bakery if they can pay, else the larder at home.
/// `home_first` turns that round, for a snack at bedtime when the shop is shut.
#[must_use]
pub fn eat_something(ctx: &Ctx<'_>, me: &str, home_first: bool) -> Vec<Command> {
    let larder = || {
        home_table(ctx, me).map(|t| {
            vec![
                Command::Note("something from the larder".into()),
                errand(t, "eat", 15),
            ]
        })
    };
    let bakery = || {
        buy_bread(ctx, me).map(|mut cmds| {
            cmds.push(errand(object(ctx, "shelf_bakery"), "eat", 10));
            cmds
        })
    };
    let food = if home_first {
        larder().or_else(bakery)
    } else {
        bakery().or_else(larder)
    };
    food.unwrap_or_else(|| vec![Command::Note("nothing to eat anywhere".into())])
}

/// The human default for hunger: keep to the schedule while merely hungry; when starving,
/// wait for a meal that is under way or due within the hour, else eat something now.
#[must_use]
pub fn hunger(ctx: &Ctx<'_>, me: &str, level: u8) -> Vec<Command> {
    if level < sim::needs::URGENT {
        return vec![Command::Note("hungry; it can wait for a meal".into())];
    }
    if meal_soon(ctx, me, MEAL_WINDOW_MINUTES) {
        return vec![Command::Note("starving, but a meal is coming".into())];
    }
    let mut cmds = vec![Command::Note("starving; eating now".into())];
    cmds.extend(eat_something(ctx, me, false));
    cmds
}

/// The baseline answer to a pressing need: the privy, the well, hunger as `hunger` says,
/// and a yawn for sleep, which the schedule handles.
#[must_use]
pub fn satisfy(ctx: &Ctx<'_>, me: &str, need: Need, level: u8) -> Vec<Command> {
    let id = villager(ctx, me);
    match need {
        Need::Bladder => match ctx.nearest_object(id, "privy") {
            Some(privy) => vec![errand(privy, "relieve", 4)],
            None => vec![Command::Note("no privy anywhere".into())],
        },
        Need::Thirst => match ctx.nearest_object(id, "well") {
            Some(well) => vec![errand(well, "drink-water", 3)],
            None => vec![Command::Note("nowhere to drink".into())],
        },
        Need::Hunger => hunger(ctx, me, level),
        Need::Sleep => vec![Command::Note(if level >= sim::needs::URGENT {
            "nodding off".into()
        } else {
            "yawning".into()
        })],
    }
}

/// A thought: what it was about, whose doing, whether it will last, how far it reaches,
/// and how it felt. Kept to oneself.
#[must_use]
pub fn think(about: Topic, blame: Blame, lasting: bool, everywhere: bool, feeling: i8) -> Command {
    Command::Think(Thought {
        about,
        blame,
        lasting,
        everywhere,
        feeling,
        aloud: false,
    })
}

/// The same thought, said aloud for anyone in earshot.
#[must_use]
pub fn think_aloud(
    about: Topic,
    blame: Blame,
    lasting: bool,
    everywhere: bool,
    feeling: i8,
) -> Command {
    match think(about, blame, lasting, everywhere, feeling) {
        Command::Think(t) => Command::Think(Thought { aloud: true, ..t }),
        other => other,
    }
}

/// Others `me` saw or heard start `activity` since `since`.
#[must_use]
pub fn seen_doing(ctx: &Ctx<'_>, me: &str, activity: &str, since: Tick) -> Vec<VillagerId> {
    let me = villager(ctx, me);
    let mut who: Vec<VillagerId> = ctx
        .recall(me, since)
        .filter(|(_, p)| *p != Perceived::Did)
        .filter_map(|(r, _)| match &r.event {
            Emitted::Activity {
                who, activity: a, ..
            } if a == activity => Some(*who),
            _ => None,
        })
        .collect();
    who.sort_unstable();
    who.dedup();
    who
}

/// Talks `me` saw `other` start with someone else since `since`, other than the business at
/// a counter: the people `other` sought out. As (when, with whom), oldest first.
#[must_use]
pub fn seen_seeking_out(
    ctx: &Ctx<'_>,
    me: VillagerId,
    other: VillagerId,
    since: Tick,
) -> Vec<(Tick, VillagerId)> {
    ctx.recall(me, since)
        .filter(|(_, p)| *p == Perceived::Saw)
        .filter_map(|(r, _)| match &r.event {
            Emitted::ConversationStarted {
                tick,
                who,
                with,
                frame,
                ..
            } if *who == other && *with != me && *frame != Frame::Transaction => {
                Some((*tick, *with))
            }
            _ => None,
        })
        .collect()
}

/// What every villager does unless their own code says otherwise: answer needs, shrug off
/// a stumble, and not go to sleep starving. Character code returns `None` for a trigger to
/// land here.
#[must_use]
pub fn human(trigger: &Trigger, ctx: &Ctx<'_>, me: &str) -> Vec<Command> {
    match trigger {
        Trigger::Need { need, level } => satisfy(ctx, me, *need, *level),
        // Most people put a stumble down to bad luck and forget it.
        Trigger::Mishap {
            what: Mishap::Tripped,
        } => vec![think(
            ctx.topic("tripped").about(villager(ctx, me)),
            Blame::Luck,
            false,
            false,
            -1,
        )],
        // A drink at whichever bar this is.
        Trigger::Arrived if activity_of(ctx, me) == "send-goods" => send_goods(ctx, me),
        Trigger::Arrived if activity_of(ctx, me) == "fetch-goods" => fetch_goods(ctx, me),
        Trigger::Arrived if activity_of(ctx, me) == "buy-luxuries" => buy_luxuries(ctx, me),
        Trigger::Arrived if activity_of(ctx, me) == "drink" => buy_ale_nearest(ctx, me)
            .unwrap_or_else(|| vec![Command::Note("no drink to be had".into())]),
        Trigger::Arrived if activity_of(ctx, me) == "sleep" => {
            let hunger = ctx.needs(villager(ctx, me)).hunger;
            if hunger < BEDTIME_HUNGER {
                return Vec::new();
            }
            let mut cmds = vec![Command::Note(format!("too hungry to sleep ({hunger})"))];
            cmds.extend(eat_something(ctx, me, true));
            cmds
        }
        _ => Vec::new(),
    }
}

/// Whether `me` is eating, walking to eat, or has a meal in the plan within `minutes`.
#[must_use]
pub fn meal_soon(ctx: &Ctx<'_>, me: &str, minutes: u32) -> bool {
    let v = ctx.villager(villager(ctx, me));
    // Eating, walking to eat, or on an errand that will go back to eating.
    if v.activity == "eat"
        || v.moving.as_ref().is_some_and(|m| m.then == "eat")
        || v.interlude.as_ref().is_some_and(|i| {
            i.back_activity == "eat" || i.queued.iter().any(|e| e.activity == "eat")
        })
    {
        return true;
    }
    let until = ctx.tick.plus(minutes * TICKS_PER_MINUTE);
    v.plan.range(ctx.tick..=until).any(|(_, cmds)| {
        cmds.iter().any(|(_, c)| {
            matches!(c, Command::GoTo { activity, .. } | Command::Wait { activity, .. } | Command::Interlude { activity, .. } if activity == "eat")
        })
    })
}

// ---- Helpers for the valley's villagers ---------------------------------------------

/// The hour of the day, 0 to 23.
#[must_use]
pub fn hour(ctx: &Ctx<'_>) -> u32 {
    u32::from(ctx.calendar.time.hour)
}

/// Minutes since midnight.
#[must_use]
pub fn minute_of_day(ctx: &Ctx<'_>) -> u32 {
    u32::from(ctx.calendar.time.hour) * 60 + u32::from(ctx.calendar.time.minute)
}

/// Whether the clock is in `[from, to)`, as (hour, minute) pairs on the same day.
#[must_use]
pub fn between(ctx: &Ctx<'_>, from: (u32, u32), to: (u32, u32)) -> bool {
    let now = minute_of_day(ctx);
    now >= from.0 * 60 + from.1 && now < to.0 * 60 + to.1
}

/// Days since the start of the run.
#[must_use]
pub fn day_number(ctx: &Ctx<'_>) -> u32 {
    ctx.calendar.day_index
}

/// Whether two villagers are in the same place within `tiles` of each other (the larger of
/// the two axis distances).
#[must_use]
pub fn near(ctx: &Ctx<'_>, a: VillagerId, b: VillagerId, tiles: u16) -> bool {
    let (va, vb) = (ctx.villager(a), ctx.villager(b));
    va.place == vb.place && va.pos.x.abs_diff(vb.pos.x).max(va.pos.y.abs_diff(vb.pos.y)) <= tiles
}

/// Everyone awake in the same place within `tiles`, nearest first, ties by id.
#[must_use]
pub fn around(ctx: &Ctx<'_>, me: VillagerId, tiles: u16) -> Vec<VillagerId> {
    let here = ctx.villager(me);
    let mut out: Vec<(u16, VillagerId)> = ctx
        .villagers
        .iter()
        .enumerate()
        .filter(|(i, v)| *i != me.index() && v.place == here.place && v.activity != "sleep")
        .map(|(i, v)| {
            let d = v
                .pos
                .x
                .abs_diff(here.pos.x)
                .max(v.pos.y.abs_diff(here.pos.y));
            (d, VillagerId(u16::try_from(i).expect("few villagers")))
        })
        .filter(|(d, _)| *d <= tiles)
        .collect();
    out.sort();
    out.into_iter().map(|(_, v)| v).collect()
}

/// Whether a villager stands inside a building's footprint (its walls included).
#[must_use]
pub fn in_building(ctx: &Ctx<'_>, who: VillagerId, building: &str) -> bool {
    let v = ctx.villager(who);
    ctx.world.roofs.iter().any(|r| {
        r.building == building && r.place == v.place.index() && r.rect.contains(v.pos.x, v.pos.y)
    })
}

/// Who `me` could have seen doing anything since `since`, most recent first, once each.
#[must_use]
pub fn seen_since(ctx: &Ctx<'_>, me: VillagerId, since: Tick) -> Vec<VillagerId> {
    let mut out: Vec<VillagerId> = Vec::new();
    let mut seen: Vec<(Tick, VillagerId)> = ctx
        .recall(me, since)
        .filter(|(_, p)| *p == Perceived::Saw)
        .filter_map(|(r, _)| r.event.who().map(|w| (r.event.tick(), w)))
        .filter(|(_, w)| *w != me)
        .collect();
    seen.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (_, w) in seen {
        if !out.contains(&w) {
            out.push(w);
        }
    }
    out
}

/// A conversation with `other` wherever they are in the same place: the sim walks `me`
/// over first when they are out of reach. `None` when they are asleep, talking, or
/// somewhere else.
#[must_use]
pub fn seek(
    ctx: &Ctx<'_>,
    me: VillagerId,
    other: VillagerId,
    frame: Frame,
    topic: Topic,
    minutes: u32,
) -> Option<Command> {
    let (mine, theirs) = (ctx.villager(me), ctx.villager(other));
    let free = me != other
        && theirs.place == mine.place
        && !matches!(theirs.activity.as_str(), "talk" | "sleep");
    free.then_some(Command::Talk {
        with: other,
        frame,
        topic,
        minutes,
    })
}

/// The bars, their kegs and keepers: the tavern in the old town and the alehouse in the
/// hamlet.
pub const BARS: [(&str, &str); 2] = [("barrel_tavern", "tom"), ("bar_alehouse", "dot")];

/// The store: its shelf, its counter and who keeps it.
pub const STORE: (&str, &str, &str) = ("shelf_store", "counter_store", "greta");
/// What people with money to spare buy at the store for themselves, with the price of
/// each. All of it gets used up.
pub const LUXURIES: [(&str, u32); 3] = [("tea", 4), ("sugar", 3), ("tobacco", 5)];
/// What the store makes on each luxury: it pays the railway this much less (Greta's
/// `fetches` in content).
pub const MARKUP: u32 = 1;
/// Coins people keep by them. Half of anything above this goes on luxuries.
pub const COMFORT_COINS: u32 = 40;
/// What the store keeper holds back besides, to restock the shelves when the train comes.
pub const KEEPER_RESERVE: u32 = 200;

/// What `who` keeps back before spending on luxuries.
fn keeps_back(ctx: &Ctx<'_>, who: VillagerId) -> u32 {
    if ctx.villager(who).id == STORE.2 {
        COMFORT_COINS + KEEPER_RESERVE
    } else {
        COMFORT_COINS
    }
}
/// The most luxuries one person buys on a trip; the store keeper, buying from the shed,
/// takes up to `KEEPER_MOST_LUXURIES`.
pub const MOST_LUXURIES: u32 = 12;
pub const KEEPER_MOST_LUXURIES: u32 = 36;
/// How long a trip to the store keeps them before they go back to what they were doing.
const SHOP_MINUTES: u32 = 30;
/// Minutes after a stopping train arrives that the store keeper buys her own luxuries from
/// the shed: after the store's fetch and her walk back, so the one errand does not cut the
/// other short.
const KEEPER_SHOPS_AFTER_MINUTES: u32 = 60;

/// Someone with more than `COMFORT_COINS` goes to the store once a week, on a weekday and
/// at an hour in the afternoon that follow from who they are. On Saturday the store keeper
/// is at the market. The keeper herself buys hers from the railway, at the goods shed
/// after each stopping train.
pub fn shopping_trip(ctx: &Ctx<'_>, who: VillagerId, plan: &mut Vec<Planned>) {
    if ctx.coins(who) <= keeps_back(ctx, who) {
        return;
    }
    if ctx.villager(who).id == STORE.2 {
        let (Some(rail), Some(train)) = (
            ctx.world.rail.as_ref(),
            trains_today(ctx).into_iter().find(|t| t.stops),
        ) else {
            return;
        };
        let at = train.arrive.seconds_into_day() + KEEPER_SHOPS_AFTER_MINUTES * 60;
        let target = Target::Object(ObjectId(u16::try_from(rail.shed).expect("few objects")));
        sim::brain::add_errand(plan, at, target, "buy-luxuries", SHOP_MINUTES);
        return;
    }
    let n = usize::from(who.0);
    if Weekday::ALL[n % 5] != ctx.calendar.weekday {
        return;
    }
    let minutes = 14 * 60 + u32::try_from(n % 5).unwrap_or(0) * 30;
    let target = Target::Object(object(ctx, STORE.1));
    sim::brain::add_errand(plan, minutes * 60, target, "buy-luxuries", SHOP_MINUTES);
}

/// At the store counter: luxuries, one of each in turn, while the shelf has them, the
/// trip's limit allows, and half their coins above `COMFORT_COINS` covers them. The
/// keeper buys hers at the goods shed instead, at the railway's price.
#[must_use]
pub fn buy_luxuries(ctx: &Ctx<'_>, me: &str) -> Vec<Command> {
    let (_, _, keeper) = STORE;
    let keeper_shed = ctx
        .world
        .rail
        .as_ref()
        .map(|r| ctx.world.objects[r.shed].id.as_str());
    let (shelf, seller, markup, most) = match (me == keeper, keeper_shed) {
        (true, Some(shed)) => (shed, "outside", MARKUP, KEEPER_MOST_LUXURIES),
        (true, None) => return Vec::new(),
        (false, _) => (STORE.0, keeper, 0, MOST_LUXURIES),
    };
    let open = in_building(ctx, villager(ctx, keeper), "store");
    if me != keeper && !open {
        return vec![Command::Note("the store was shut".into())];
    }
    let who = villager(ctx, me);
    let mut budget = ctx.coins(who).saturating_sub(keeps_back(ctx, who)) / 2;
    let mut taken = [0u16; LUXURIES.len()];
    let mut bought = 0;
    let mut more = true;
    while more && bought < most {
        more = false;
        for (i, (good, price)) in LUXURIES.iter().enumerate() {
            let price = price - markup;
            if bought < most && budget >= price && stock_of(ctx, shelf, good) > taken[i] {
                taken[i] += 1;
                budget -= price;
                bought += 1;
                more = true;
            }
        }
    }
    let mut cmds = Vec::new();
    for (i, (good, price)) in LUXURIES.iter().enumerate() {
        if taken[i] == 0 {
            continue;
        }
        cmds.push(stock(ctx, shelf, good, -i32::from(taken[i])));
        cmds.push(pay(
            ctx,
            me,
            seller,
            (price - markup) * u32::from(taken[i]),
            good,
        ));
    }
    cmds
}

/// Pays for a drink at whichever bar is nearer. None when that bar is dry or the buyer is
/// broke.
#[must_use]
pub fn buy_ale_nearest(ctx: &Ctx<'_>, me: &str) -> Option<Vec<Command>> {
    let v = ctx.villager(villager(ctx, me));
    let (keg, keeper) = BARS.iter().copied().min_by_key(|(keg, _)| {
        let o = &ctx.world.objects[object(ctx, keg).index()];
        o.at.x.abs_diff(v.pos.x) + o.at.y.abs_diff(v.pos.y)
    })?;
    if stock_of(ctx, keg, "ale") == 0 || ctx.coins(villager(ctx, me)) < ALE_PRICE {
        return None;
    }
    let mut cmds = vec![stock(ctx, keg, "ale", -1)];
    if me != keeper {
        cmds.push(pay(ctx, me, keeper, ALE_PRICE, "ale"));
    }
    Some(cmds)
}

/// Loop of work over an area: go to a random spot in it, do `activity` there for
/// `minutes`, clearing within `clear` tiles if it is more than zero, and go on to another
/// spot when the wait is over. Call on `Arrived` and `Waited` for that activity; arriving
/// for it somewhere outside the area (at the object the schedule names) sets off for it.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn work_area(
    ctx: &Ctx<'_>,
    me: &str,
    trigger: &Trigger,
    area: &str,
    activity: &str,
    minutes: u32,
    clear: u16,
    rng: &mut Pcg32,
) -> Vec<Command> {
    match trigger {
        Trigger::Arrived if in_area(ctx, villager(ctx, me), area) => {
            let mut cmds = Vec::new();
            let v = ctx.villager(villager(ctx, me));
            let weeds = clear > 0
                && ctx
                    .overgrown_tiles(v.place)
                    .iter()
                    .any(|p| p.x.abs_diff(v.pos.x) <= clear && p.y.abs_diff(v.pos.y) <= clear);
            if weeds {
                cmds.push(Command::Clear { radius: clear });
            }
            cmds.push(wait_minutes(minutes, activity));
            cmds
        }
        _ => vec![
            go_area(ctx, area, activity, rng).unwrap_or_else(|| wait_minutes(minutes, activity)),
        ],
    }
}

/// Whether `from` said something on `topic` within earshot of `mind`'s owner, catching the
/// words, in the last `days` days.
#[must_use]
pub fn heard_lately(
    ctx: &Ctx<'_>,
    mind: &sim::social::Memory,
    from: &str,
    topic: &str,
    days: u32,
) -> bool {
    mind.heard()
        .from(villager(ctx, from))
        .topic(topic)
        .since(ctx.tick.minus(days * sim::time::TICKS_PER_DAY))
        .words()
        .any()
}
