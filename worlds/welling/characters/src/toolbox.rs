//! What the Welling's character code is written against: the shared library
//! (`behaviour`), re-exported, and the Welling's own errands: water from the cisterns,
//! drinks at the Kettle, gas from the Mire's casks to the stall.
pub use behaviour::*;
use sim::brain::{Command, Ctx, Target};
use sim::ids::ObjectId;
use sim::needs::Need;
use sim::protocol::{Blame, Frame, Mishap, MoodKind};

/// Prices, in coins. Placeholders.
pub const DRINK_PRICE: u32 = 2;
/// What the stall pays the refiners for a measure of refined gas.
pub const GAS_TRADE_PRICE: u32 = 3;

/// The Kettle's barrel and who keeps it.
pub const KETTLE: (&str, &str) = ("barrel_kettle", "lark");
/// Where the Mire keeps its gas, raw and refined, in the hold.
pub const CASKS: &str = "casks_mire";
/// The table the refined gas is sold from, on the yard.
pub const STALL: &str = "stall_gas";
pub const RAW_GAS: &str = "raw-gas";
pub const REFINED_GAS: &str = "refined-gas";

/// The table in someone's home.
#[must_use]
pub fn home_table(ctx: &Ctx<'_>, me: &str) -> Option<ObjectId> {
    let home = &ctx.content.villagers.iter().find(|v| v.id == me)?.home;
    ctx.world
        .objects
        .iter()
        .position(|o| o.kind == "table" && &o.building == home)
        .map(|i| ObjectId(u16::try_from(i).expect("few objects")))
}

/// A drink from the Kettle's barrel, paid to its keeper. None when it is dry or the buyer
/// cannot pay.
#[must_use]
pub fn buy_drink(ctx: &Ctx<'_>, me: &str) -> Option<Vec<Command>> {
    let (barrel, keeper) = KETTLE;
    if stock_of(ctx, barrel, "drink") == 0 {
        return None;
    }
    if me == keeper {
        return Some(vec![stock(ctx, barrel, "drink", -1)]);
    }
    if ctx.coins(villager(ctx, me)) < DRINK_PRICE {
        return None;
    }
    Some(vec![
        stock(ctx, barrel, "drink", -1),
        pay(ctx, me, keeper, DRINK_PRICE, "a drink"),
    ])
}

/// The answer to a pressing need: the privy, a cistern, food at home when starving, and a
/// yawn for sleep, which the schedule handles.
#[must_use]
pub fn satisfy(ctx: &Ctx<'_>, me: &str, need: Need, level: u8) -> Vec<Command> {
    let id = villager(ctx, me);
    match need {
        Need::Bladder => match ctx.nearest_object(id, "privy") {
            Some(privy) => vec![errand(privy, "relieve", 4)],
            None => vec![Command::Note("no privy anywhere".into())],
        },
        Need::Thirst => match ctx.nearest_object(id, "cistern") {
            Some(c) => vec![errand(c, "drink-water", 3)],
            None => vec![Command::Note("nowhere to drink".into())],
        },
        Need::Hunger if level < sim::needs::URGENT => {
            vec![Command::Note("hungry; it can wait for a meal".into())]
        }
        Need::Hunger if meal_soon(ctx, me, 60) => {
            vec![Command::Note("starving, but a meal is coming".into())]
        }
        Need::Hunger => match home_table(ctx, me) {
            Some(t) => vec![
                Command::Note("starving; eating at home".into()),
                errand(t, "eat", 15),
            ],
            None => vec![Command::Note("nothing to eat anywhere".into())],
        },
        Need::Sleep => vec![Command::Note(if level >= sim::needs::URGENT {
            "nodding off".into()
        } else {
            "yawning".into()
        })],
    }
}

/// The topic someone's work gives them to talk about.
#[must_use]
pub fn work_topic(activity: &str) -> Option<&'static str> {
    Some(match activity {
        "refine" | "capture" | "sell" => "gas",
        "fish" => "the-catch",
        "lash" => "the-lashings",
        "serve" => "drink",
        _ => return None,
    })
}

/// The shared answer to a mood strong enough to act on: the lonely look for company and
/// go to the Kettle if there is none, the restless walk, the anxious, the irritable and
/// the proud think about what moved them (the proud aloud).
#[must_use]
pub fn feel_mood(ctx: &Ctx<'_>, me: &str, mood: MoodKind) -> Vec<Command> {
    use sim::moods::Source as S;
    let who = villager(ctx, me);
    let Some(m) = ctx.moods(who).get(mood) else {
        return Vec::new();
    };
    let mine = |key: &str| ctx.topic(key).about(who);
    match (mood, m.source) {
        (MoodKind::Lonely, _) => {
            if let Some(talk) = around(ctx, who, 8)
                .into_iter()
                .find_map(|o| chat(ctx, who, o, Frame::SmallTalk, "how-they-are", 10))
            {
                return vec![talk];
            }
            let mut cmds = vec![think(mine("nobody-came"), Blame::World, false, false, -1)];
            if let Some(counter) = ctx.object_id("counter_kettle") {
                cmds.push(Command::Interlude {
                    target: Target::Object(counter),
                    activity: "company".into(),
                    minutes: 45,
                });
            }
            cmds
        }
        (MoodKind::Restless, _) => {
            let v = ctx.villager(who);
            let steps: [(i32, i32); 4] = [(5, 0), (0, 5), (-5, 0), (0, -5)];
            let start = usize::try_from(ctx.tick.0 / sim::time::TICKS_PER_HOUR).unwrap_or(0);
            (0..4)
                .map(|k| steps[(start + k) % 4])
                .find_map(|(dx, dy)| {
                    let x = u16::try_from(i32::from(v.pos.x) + dx).ok()?;
                    let y = u16::try_from(i32::from(v.pos.y) + dy).ok()?;
                    ctx.world.passable(v.place.index(), x, y).then(|| {
                        vec![Command::Interlude {
                            target: Target::Tile {
                                place: v.place,
                                pos: content::Pos { x, y },
                            },
                            activity: "wander".into(),
                            minutes: 20,
                        }]
                    })
                })
                .unwrap_or_default()
        }
        (MoodKind::Anxious, S::Money) => {
            vec![think(mine("money"), Blame::World, false, false, -1)]
        }
        (MoodKind::Anxious, S::Dark) => {
            vec![think(mine("the-dark"), Blame::World, false, false, -2)]
        }
        (MoodKind::Irritable, S::Slight(them)) => vec![think(
            ctx.topic("what-was-said").about(them),
            Blame::Them(them),
            false,
            false,
            -2,
        )],
        (MoodKind::Proud, S::Earning) => {
            vec![think_aloud(mine("custom"), Blame::Me, false, false, 1)]
        }
        (MoodKind::Proud, S::Work) => work_topic(&ctx.villager(who).activity)
            .map(|t| vec![think_aloud(mine(t), Blame::Me, false, false, 2)])
            .unwrap_or_default(),
        (
            MoodKind::Anxious
            | MoodKind::Irritable
            | MoodKind::Proud
            | MoodKind::Cheerful
            | MoodKind::Gloomy,
            _,
        ) => Vec::new(),
    }
}

/// What everyone in the Welling does unless their own code says otherwise.
#[must_use]
pub fn human(trigger: &sim::brain::Trigger, ctx: &Ctx<'_>, me: &str) -> Vec<Command> {
    use sim::brain::Trigger;
    match trigger {
        Trigger::Mood { mood, .. } => feel_mood(ctx, me, *mood),
        Trigger::Need { need, level } => satisfy(ctx, me, *need, *level),
        Trigger::Mishap {
            what: Mishap::Tripped,
        } => vec![think(
            ctx.topic("tripped").about(villager(ctx, me)),
            Blame::Luck,
            false,
            false,
            -1,
        )],
        Trigger::Arrived if activity_of(ctx, me) == "drink" => {
            buy_drink(ctx, me).unwrap_or_else(|| vec![Command::Note("no drink to be had".into())])
        }
        _ => Vec::new(),
    }
}

/// Work at a refining bench: two measures of raw gas from the casks make one of refined,
/// then another spell. Call on `Arrived` and `Waited` for "refine".
#[must_use]
pub fn refine(ctx: &Ctx<'_>) -> Vec<Command> {
    let mut cmds = Vec::new();
    if stock_of(ctx, CASKS, RAW_GAS) >= 2 {
        cmds.push(stock(ctx, CASKS, RAW_GAS, -2));
        cmds.push(stock(ctx, CASKS, REFINED_GAS, 1));
    } else {
        cmds.push(Command::Note("no raw gas to work".into()));
    }
    cmds.push(wait_minutes(90, "refine"));
    cmds
}
