//! What character code is written against. Grows from what characters want to say.
use rand::RngExt;
use rand_pcg::Pcg32;
use sim::brain::{Command, Ctx, Planned, Target};
use sim::ids::VillagerId;
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
