//! What character code is written against. Grows from what characters want to say.
use rand::RngExt;
use rand_pcg::Pcg32;
use sim::brain::{Command, Ctx, Party, Planned, Target, Trigger};
use sim::ids::{ObjectId, VillagerId};
use sim::needs::Need;
use sim::protocol::{Blame, Emitted, Frame, Mishap, Thought};
use sim::recall::Perceived;
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
    let k = villager(ctx, keeper);
    let (mine, theirs) = (ctx.villager(villager(ctx, me)), ctx.villager(k));
    let free = theirs.place == mine.place
        && theirs.moving.is_none()
        && !matches!(theirs.activity.as_str(), "talk" | "sleep");
    free.then(|| Command::Talk {
        with: k,
        frame: Frame::Transaction,
        topic: ctx.topic(topic),
        minutes: COUNTER_MINUTES,
    })
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
/// and how it felt.
#[must_use]
pub fn think(about: &str, blame: Blame, lasting: bool, everywhere: bool, feeling: i8) -> Command {
    Command::Think(Thought {
        about: about.into(),
        blame,
        lasting,
        everywhere,
        feeling,
    })
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
        } => vec![think("tripped", Blame::Luck, false, false, -1)],
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
