//! What character code is written against. Grows from what characters want to say.
use rand::RngExt;
use rand_pcg::Pcg32;
use sim::brain::{Command, Ctx, Party, Planned, Target};
use sim::ids::{ObjectId, VillagerId};
use sim::needs::Need;
use sim::time::{TICKS_PER_MINUTE, TimeOfDay, Weekday};

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

/// The baseline answer to a pressing need, for any villager: the privy, the well, a loaf
/// from the bakery (or the larder at home when urgent and broke), and a yawn for sleep,
/// which the schedule handles. Characters call this unless they have something better.
#[must_use]
pub fn satisfy(ctx: &Ctx<'_>, me: &str, need: Need, level: u8, home_table: &str) -> Vec<Command> {
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
        Need::Hunger => {
            if let Some(mut cmds) = buy_bread(ctx, me) {
                cmds.push(errand(object(ctx, "shelf_bakery"), "eat", 10));
                cmds
            } else if level >= sim::needs::URGENT {
                vec![
                    Command::Note("nothing to buy; something from the larder".into()),
                    errand(object(ctx, home_table), "eat", 15),
                ]
            } else {
                vec![Command::Note("hungry; the next meal will do".into())]
            }
        }
        Need::Sleep => vec![Command::Note(if level >= sim::needs::URGENT {
            "nodding off".into()
        } else {
            "yawning".into()
        })],
    }
}
