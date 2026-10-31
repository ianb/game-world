//! What the valley's character code is written against: the shared library
//! (`behaviour`), re-exported, and the valley's own errands.
pub use behaviour::*;
use content::Rect;
use rand_pcg::Pcg32;
use sim::brain::{Command, Ctx, Party, Planned, Target, Trigger};
use sim::ids::{ObjectId, PlaceId, VillagerId};
use sim::needs::Need;
use sim::protocol::{Blame, Cause, Emitted, Frame, Mishap, MoodKind};
use sim::recall::Perceived;
use sim::social::Interlocutor;
use sim::time::Weekday;

/// Prices, in coins. One table so the whole economy can be retuned in one place.
pub const BREAD_PRICE: u32 = 2;
pub const ALE_PRICE: u32 = 3;

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

/// A child's answer to hens close by when they are free to: run at them for a few
/// minutes, delighted, then back to whatever they were doing. Hens run from a chaser.
#[must_use]
pub fn chase_hens(ctx: &Ctx<'_>, me: &str, trigger: &Trigger) -> Option<Vec<Command>> {
    let Trigger::Animals {
        kind,
        owner,
        place,
        at,
    } = trigger
    else {
        return None;
    };
    if crate::Animal::of(ctx, *kind) != crate::Animal::Chicken {
        return None;
    }
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
    kind: sim::ids::AnimalKindId,
    owner: VillagerId,
    fondness: crate::Fondness,
) -> Vec<Command> {
    use crate::Fondness as F;
    if me == owner {
        return Vec::new();
    }
    let topic = ctx.topic(sim::brain::animal_plural(ctx, kind)).about(owner);
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

/// What every villager does unless their own code says otherwise: answer needs, shrug off
/// a stumble, and not go to sleep starving. Character code returns `None` for a trigger to
/// land here.
#[must_use]
pub fn human(trigger: &Trigger, ctx: &Ctx<'_>, me: &str) -> Vec<Command> {
    match trigger {
        Trigger::Mood { mood, .. } => feel_mood(ctx, me, *mood),
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
        Trigger::Arrived | Trigger::Waited if activity_of(ctx, me) == "tend-garden" => {
            work_garden(ctx, me)
        }
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

/// The topic someone's work gives them to talk about, for the activities that have one.
fn work_topic(activity: &str) -> Option<&'static str> {
    Some(match activity {
        "smith" => "ironwork",
        "chop" => "firewood",
        "make-barrels" => "barrels",
        "weave" => "cloth",
        "throw-pots" | "fire-pots" => "pots",
        "bake" => "bread",
        "fish" => "the-catch",
        "burn" => "charcoal",
        "tend-bees" | "jar-honey" => "honey",
        "dairy" => "cheese",
        _ => return None,
    })
}

/// The shared answer to a mood strong enough to act on: the lonely look for someone to
/// talk to, the restless take a walk, the anxious, the irritable and the proud think
/// about what moved them (the proud, and the irritable at noise, aloud). Cheer and gloom
/// show only in how they talk.
#[must_use]
#[deny(clippy::wildcard_enum_match_arm)]
pub fn feel_mood(ctx: &Ctx<'_>, me: &str, mood: MoodKind) -> Vec<Command> {
    use sim::moods::Source as S;
    let who = villager(ctx, me);
    let Some(m) = ctx.moods(who).get(mood) else {
        return Vec::new();
    };
    let mine = |key: &str| ctx.topic(key).about(who);
    match (mood, m.source) {
        (MoodKind::Lonely, _) => {
            let company = around(ctx, who, 8)
                .into_iter()
                .find_map(|o| chat(ctx, who, o, Frame::SmallTalk, "how-they-are", 10));
            if let Some(talk) = company {
                return vec![talk];
            }
            // Nobody about: a while at the nearest bar, where there usually is someone.
            let v = ctx.villager(who);
            let bar = BARS
                .iter()
                .filter_map(|(keg, _)| ctx.object_id(keg))
                .min_by_key(|o| {
                    let at = ctx.world.objects[o.index()].at;
                    at.x.abs_diff(v.pos.x) + at.y.abs_diff(v.pos.y)
                });
            let mut cmds = vec![think(mine("nobody-came"), Blame::World, false, false, -1)];
            if let Some(bar) = bar {
                cmds.push(Command::Interlude {
                    target: Target::Object(bar),
                    activity: "company".into(),
                    minutes: 45,
                });
            }
            cmds
        }
        (MoodKind::Restless, _) => {
            let v = ctx.villager(who);
            // A short walk, in a direction that follows from the hour, to somewhere
            // passable.
            let steps: [(i32, i32); 4] = [(6, 0), (0, 6), (-6, 0), (0, -6)];
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
        (MoodKind::Anxious | MoodKind::Proud, S::Garden) => {
            let feeling = if mood == MoodKind::Proud { 2 } else { -1 };
            vec![think(mine("the-garden"), Blame::Me, false, false, feeling)]
        }
        (MoodKind::Irritable, S::Noise(loud)) => {
            let topic = work_topic(&ctx.villager(loud).activity).unwrap_or("the-news");
            vec![think_aloud(
                ctx.topic(topic).about(loud),
                Blame::Them(loud),
                false,
                false,
                -2,
            )]
        }
        (MoodKind::Irritable, S::Slight(them)) => vec![think(
            ctx.topic("what-was-said").about(them),
            Blame::Them(them),
            false,
            false,
            -2,
        )],
        (MoodKind::Proud, S::Harvest) => {
            vec![think_aloud(mine("the-garden"), Blame::Me, false, false, 2)]
        }
        // Pride in earning is about custom: "money" is the topic of money trouble, and a
        // listener would take it as a call for help.
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

/// Field work inside `rect` of `place`, one tile at a time: work the tile underfoot if it
/// wants anything, else walk to the most pressing job there (`Ctx::field_job`). `None`
/// when the fields there want nothing.
#[must_use]
pub fn tend_in(
    ctx: &Ctx<'_>,
    who: VillagerId,
    place: PlaceId,
    rect: Rect,
    activity: &str,
) -> Option<Vec<Command>> {
    let v = ctx.villager(who);
    let here = (v.place == place && rect.contains(v.pos.x, v.pos.y))
        .then(|| ctx.field_at(v.place, v.pos))
        .flatten()
        .and_then(|(_, t)| sim::fields::wants(t, ctx.calendar.season));
    if let Some(work) = here {
        return Some(vec![
            Command::Tend { work },
            wait_minutes(sim::fields::minutes(work), activity),
        ]);
    }
    let (pos, _) = ctx.field_job(place, rect, v.pos)?;
    Some(vec![Command::GoTo {
        target: Target::Tile { place, pos },
        activity: activity.to_string(),
    }])
}

/// Farm work over the fields in an area (`tend_in`), and when they want nothing, the
/// old round of the area pulling weeds. Call on `Arrived` and `Waited` for the activity.
#[must_use]
pub fn tend_fields(
    ctx: &Ctx<'_>,
    me: &str,
    trigger: &Trigger,
    area: &str,
    activity: &str,
    rng: &mut Pcg32,
) -> Vec<Command> {
    let (place, rect) = ctx.area(area);
    tend_in(ctx, villager(ctx, me), place, rect, activity)
        .unwrap_or_else(|| work_area(ctx, me, trigger, area, activity, 40, 2, rng))
}

/// The gardens kept by the household `who` sleeps in.
fn gardens<'a>(ctx: &Ctx<'a>, who: VillagerId) -> Vec<&'a sim::fields::Plot> {
    let Some(v) = ctx.content.villagers.get(who.index()) else {
        return Vec::new();
    };
    let Some(home) = ctx
        .world
        .objects
        .iter()
        .find(|o| o.id == v.bed)
        .map(|o| o.building.as_str())
    else {
        return Vec::new();
    };
    let households: Vec<&str> = ctx
        .content
        .zones
        .iter()
        .flat_map(|z| z.plots.iter())
        .map(|p| p.household.as_deref().unwrap_or(""))
        .collect();
    ctx.fields
        .plots
        .iter()
        .zip(households)
        .filter(|(_, h)| *h == home)
        .map(|(p, _)| p)
        .collect()
}

/// How long an evening in the garden lasts.
const GARDEN_MINUTES: u32 = 40;

/// The people of a household take turns in their garden in the early evening: one of
/// them a day, by who they are and the day, from half past five.
pub fn garden_turn(ctx: &Ctx<'_>, who: VillagerId, plan: &mut Vec<Planned>) {
    let Some(garden) = gardens(ctx, who).into_iter().next() else {
        return;
    };
    let home = ctx
        .world
        .objects
        .iter()
        .find(|o| o.id == ctx.content.villagers[who.index()].bed)
        .map(|o| o.building.as_str());
    let household: Vec<usize> = ctx
        .content
        .villagers
        .iter()
        .enumerate()
        .filter(|(_, v)| {
            ctx.world
                .objects
                .iter()
                .find(|o| o.id == v.bed)
                .map(|o| o.building.as_str())
                == home
        })
        .map(|(i, _)| i)
        .collect();
    let day = usize::try_from(ctx.calendar.day_index).unwrap_or(0);
    if household.get(day % household.len().max(1)) != Some(&who.index()) {
        return;
    }
    let at = (17 * 60 + 30) * 60;
    let target = Target::Tile {
        place: garden.place,
        pos: garden.pos(0),
    };
    sim::brain::add_errand(plan, at, target, "tend-garden", GARDEN_MINUTES);
}

/// In the garden: the next job it wants, or a stand in it when it wants nothing.
#[must_use]
pub fn work_garden(ctx: &Ctx<'_>, me: &str) -> Vec<Command> {
    let who = villager(ctx, me);
    let Some(garden) = gardens(ctx, who).into_iter().next() else {
        return Vec::new();
    };
    tend_in(ctx, who, garden.place, garden.rect, "tend-garden")
        .unwrap_or_else(|| vec![wait_minutes(10, "tend-garden")])
}
