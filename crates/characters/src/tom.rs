//! Tom, who keeps the tavern. The eclectic one: a mood only he tracks, Sundays off at the
//! pond, late openings when he is low, and an evening check on whether anyone came.
//! Starving, he eats the bar's bread; otherwise his body behaves like anyone's
//! (`toolbox::human`). In talk he is blunt and teases his regulars, not always well; his
//! mood colours everything, and each conversation moves it a little. A new fish is the
//! first thing he mentions. When things go wrong he blames the town, for good and for
//! everything; the grievance builds through the day and, past a point, the next person he
//! talks to gets a dig whatever they did. He grumbles aloud.
use crate::toolbox::{
    MEAL_WINDOW_MINUTES, Plan, activity_of, eat_something, errand, go_tile, meal_soon, object, pay,
    pick, seen_doing, stock, stock_of, think_aloud, villager, wait_minutes, weekday,
};
use content::{Pos, Rect};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use sim::brain::{Command, Ctx, Planned, Trigger};
use sim::ids::VillagerId;
use sim::needs::{Need, URGENT};
use sim::protocol::{Act, Blame, Frame, Gesture, Mishap, Topic};
use sim::social::{
    Episode, Interlocutor, Memory, Persona, Situation, act, default_next_act, needs_mood,
};
use sim::time::Weekday;

/// Fishing spots on the pond's west bank, in building-free ground.
const FISHING_SPOTS: [(u16, u16); 3] = [(29, 21), (29, 22), (29, 23)];

/// The tavern's yard: the building, the lane along its south side and a margin round it.
/// Tom keeps this clear of weeds; the rest of the green can go wild.
pub const YARD: Rect = Rect {
    x: 2,
    y: 12,
    w: 17,
    h: 13,
};
/// How many patches he will clear in one morning before opening.
const WEEDING_PATCHES: u8 = 4;
const WEEDING_RADIUS: u16 = 2;
/// Hunger at which he will not go to bed without eating; he checks while closing up,
/// so he can eat at the bar.
const BEDTIME_HUNGER: u8 = crate::toolbox::BEDTIME_HUNGER;
/// Grievance at which he takes it out on whoever he talks to next.
pub const BOILING: u8 = 2;
/// A loaf a day for the bar.
pub const BAR_BREAD_PRICE: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tom {
    /// Negative is grumpy. Nudged by how the evening went.
    pub mood: i8,
    pub fish_caught: u16,
    /// Fish he has told someone about.
    pub fish_told: u16,
    /// Patches cleared so far this morning.
    pub weeded_today: u8,
    /// The day he last bought bread for the bar, so he buys once a day at most.
    pub bar_bread_day: u32,
    /// The day Mara last brought him a pastry.
    pub pastry_day: u32,
    /// Things that went wrong and that he put down to the town, building toward an outburst.
    pub grievance: u8,
    /// The day he last noticed the bar out of bread.
    pub bare_bar_day: u32,
    pub mind: Memory,
}

impl Default for Tom {
    fn default() -> Self {
        Tom {
            mood: -1,
            fish_caught: 0,
            fish_told: 0,
            weeded_today: 0,
            bar_bread_day: u32::MAX,
            pastry_day: u32::MAX,
            grievance: 0,
            bare_bar_day: u32::MAX,
            mind: Memory::default(),
        }
    }
}

impl Tom {
    /// Bread from behind the bar, eaten at the bar.
    fn bar_meal(ctx: &Ctx<'_>) -> Vec<Command> {
        vec![
            Command::Note("starving; bread from behind the bar".into()),
            stock(ctx, "bar_tavern", "bread", -1),
            errand(object(ctx, "bar_tavern"), "eat", 10),
        ]
    }

    /// Closing up hungry: he eats something before bed rather than wake up starving.
    fn closing_up(ctx: &Ctx<'_>) -> Vec<Command> {
        let me = ctx.villager_id("tom").expect("tom exists");
        let hunger = ctx.needs(me).hunger;
        if hunger < BEDTIME_HUNGER {
            return Vec::new();
        }
        let mut cmds = vec![Command::Note(format!("too hungry to sleep ({hunger})"))];
        if stock_of(ctx, "bar_tavern", "bread") > 0 {
            cmds.extend(Tom::bar_meal(ctx));
        } else {
            cmds.extend(eat_something(ctx, "tom", true));
        }
        cmds
    }

    /// A loaf for the bar when it has fewer than two, if the bakery has one and he can pay.
    fn stock_the_bar(ctx: &Ctx<'_>) -> Vec<Command> {
        let me = ctx.villager_id("tom").expect("tom exists");
        if stock_of(ctx, "bar_tavern", "bread") >= 2
            || stock_of(ctx, "shelf_bakery", "bread") == 0
            || ctx.coins(me) < BAR_BREAD_PRICE
        {
            return Vec::new();
        }
        vec![
            pay(ctx, "tom", "mara", BAR_BREAD_PRICE, "bread for the bar"),
            stock(ctx, "shelf_bakery", "bread", -1),
            stock(ctx, "bar_tavern", "bread", 1),
        ]
    }

    /// 20:00 on a working day: did anyone come in for a drink this evening?
    fn evening_check(&mut self, ctx: &Ctx<'_>) -> Vec<Command> {
        let since = ctx.tick.day_start().at_time(19, 0);
        let came = seen_doing(ctx, "tom", "drink", since);
        let company = !came.is_empty();
        let delivered_today = self.pastry_day == ctx.calendar.day_index;
        let before = self.mood;
        self.mood = if company || delivered_today {
            self.mood.saturating_add(2).min(5)
        } else {
            self.mood.saturating_sub(1).max(-5)
        };
        let why = match (company, delivered_today) {
            (true, _) => format!("{} came in", ctx.villager(came[0]).name),
            (false, true) => "nobody here, but a pastry came by".into(),
            (false, false) => "nobody came".into(),
        };
        // No further wait: he keeps the `tend-bar` activity until the 22:00 item.
        let mut cmds = vec![Command::Note(format!(
            "mood {before} -> {} ({why})",
            self.mood
        ))];
        if !company && !delivered_today {
            self.grievance = self.grievance.saturating_add(1);
            let me = villager(ctx, "tom");
            cmds.push(think_aloud(
                ctx.topic("nobody-came").about(me),
                Blame::World,
                true,
                true,
                -1,
            ));
        }
        cmds
    }

    pub fn plan_day(&mut self, ctx: &Ctx<'_>, rng: &mut Pcg32) -> Vec<Planned> {
        let me = ctx.villager_id("tom").expect("tom exists");
        let mut plan = Plan::baseline(me, ctx);
        // A night's sleep takes the edge off one grievance.
        self.grievance = self.grievance.saturating_sub(1);

        if weekday(ctx) == Weekday::Sun {
            // Day off: no bar. Fish from late morning, then home for a long evening.
            let (x, y) = *pick(rng, &FISHING_SPOTS);
            plan.drop_activity("tend-bar").drop_activity("close-up");
            plan.at(10, 30, go_tile(ctx, "town", x, y, "fish"));
            plan.at(16, 0, crate::toolbox::go(ctx, "table_house", "rest"));
            return plan.build();
        }

        // Before opening: if the yard has gone to weed, go and clear the nearest patch.
        // Arriving there starts the loop in `react`.
        self.weeded_today = 0;
        let town = ctx.place_id("town").expect("town exists");
        let tavern_door = Pos { x: 9, y: 22 };
        if let Some(patch) = ctx.nearest_overgrown(town, YARD, tavern_door) {
            plan.at(
                8,
                30,
                Command::Note(format!(
                    "the yard needs weeding ({} tiles)",
                    ctx.overgrown_tiles(town)
                        .iter()
                        .filter(|p| YARD.contains(p.x, p.y))
                        .count()
                )),
            );
            plan.at(8, 31, go_tile(ctx, "town", patch.x, patch.y, "weeding"));
        }

        if self.mood <= -3 {
            plan.shift((10, 0), (11, 0));
            plan.at(
                0,
                1,
                Command::Note(format!("opening late, mood {}", self.mood)),
            );
        }
        // Mid-evening check on the room.
        plan.at(19, 30, wait_minutes(30, "tend-bar"));
        plan.build()
    }

    pub fn react(
        &mut self,
        trigger: Trigger,
        ctx: &Ctx<'_>,
        rng: &mut Pcg32,
    ) -> Option<Vec<Command>> {
        Some(match trigger {
            Trigger::Arrived if activity_of(ctx, "tom") == "weeding" => {
                vec![
                    Command::Clear {
                        radius: WEEDING_RADIUS,
                    },
                    wait_minutes(12, "weeding"),
                ]
            }
            Trigger::Waited if activity_of(ctx, "tom") == "weeding" => {
                self.weeded_today += 1;
                let town = ctx.place_id("town").expect("town exists");
                let here = ctx
                    .villager(ctx.villager_id("tom").expect("tom exists"))
                    .pos;
                match ctx.nearest_overgrown(town, YARD, here) {
                    Some(next) if self.weeded_today < WEEDING_PATCHES => {
                        vec![go_tile(ctx, "town", next.x, next.y, "weeding")]
                    }
                    Some(_) => vec![
                        Command::Note("enough weeding for one morning".into()),
                        Command::Idle,
                    ],
                    None => vec![Command::Note("the yard is clear".into()), Command::Idle],
                }
            }
            Trigger::Arrived if activity_of(ctx, "tom") == "fish" => {
                // A fishing afternoon: wait a while, maybe catch something, repeat.
                vec![wait_minutes(*pick(rng, &[40u32, 55, 70]), "fish")]
            }
            Trigger::Waited if activity_of(ctx, "tom") == "fish" => {
                if *pick(rng, &[true, false, false]) {
                    self.fish_caught += 1;
                }
                self.mood = self.mood.saturating_add(1).min(5);
                vec![wait_minutes(*pick(rng, &[40u32, 55, 70]), "fish")]
            }
            Trigger::Waited => self.evening_check(ctx),
            // Behind the bar he keeps an eye on the bread.
            Trigger::Look { .. }
                if activity_of(ctx, "tom") == "tend-bar"
                    && stock_of(ctx, "bar_tavern", "bread") == 0
                    && self.bare_bar_day != ctx.calendar.day_index =>
            {
                self.bare_bar_day = ctx.calendar.day_index;
                self.grievance = self.grievance.saturating_add(1);
                vec![think_aloud(
                    ctx.topic("bar-out-of-bread").about(villager(ctx, "tom")),
                    Blame::World,
                    true,
                    true,
                    -1,
                )]
            }
            Trigger::Mishap {
                what: Mishap::Tripped,
            } => {
                self.grievance = self.grievance.saturating_add(1);
                vec![think_aloud(
                    ctx.topic("tripped").about(villager(ctx, "tom")),
                    Blame::World,
                    true,
                    true,
                    -1,
                )]
            }
            Trigger::TalkEnded { topic, .. } if topic.is("pastry") => {
                self.pastry_day = ctx.calendar.day_index;
                vec![Command::Note("a pastry from Mara".into())]
            }
            // Starving behind the bar, he eats the bar's bread. Anything else about needs is
            // the human default.
            Trigger::Need {
                need: Need::Hunger,
                level,
            } if level >= URGENT
                && !meal_soon(ctx, "tom", MEAL_WINDOW_MINUTES)
                && stock_of(ctx, "bar_tavern", "bread") > 0 =>
            {
                Tom::bar_meal(ctx)
            }
            Trigger::Arrived if activity_of(ctx, "tom") == "close-up" => Tom::closing_up(ctx),
            // Opening the bar: top up its bread if it is low and the bakery has some.
            Trigger::Arrived
                if activity_of(ctx, "tom") == "tend-bar"
                    && self.bar_bread_day != ctx.calendar.day_index =>
            {
                self.bar_bread_day = ctx.calendar.day_index;
                Tom::stock_the_bar(ctx)
            }
            Trigger::Passing { other } if activity_of(ctx, "tom") != "sleep" => {
                vec![Command::Greet { with: other }]
            }
            _ => return None,
        })
    }
}

impl Interlocutor for Tom {
    fn persona(&self) -> Persona {
        Persona {
            sociable: 1,
            warmth: 0,
            skill: 4,
            rejection_sensitivity: 4,
            charity: -1,
        }
    }
    fn memory(&self) -> &Memory {
        &self.mind
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mind
    }

    /// What each gesture means to Tom: he likes banter, even rough, and has no use for pity
    /// or praise; a complaint is just bar talk. Every gesture is listed: a new one makes this decide.
    #[deny(clippy::wildcard_enum_match_arm)]
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        use Gesture as G;
        match gesture {
            G::Tease => 2,
            G::Jab | G::Sympathise => -1,
            G::Compliment | G::Complain => 0,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Request
            | G::Offer
            | G::Thank
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Rebuff
            | G::Pry => gesture.warmth(),
        }
    }

    fn mood(&self, ctx: &Ctx<'_>, me: VillagerId) -> i8 {
        (needs_mood(ctx, me) + self.mood / 2).clamp(-3, 3)
    }

    fn worry(&self, ctx: &Ctx<'_>, me: VillagerId) -> Option<Topic> {
        (self.mood <= -3).then(|| ctx.topic("empty-bar").about(me))
    }

    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        // Boiling over: whoever is in front of him gets it.
        if self.grievance >= BOILING {
            self.grievance = 0;
            return act(Gesture::Jab, s.topic);
        }
        let fish = self.fish_caught > self.fish_told;
        match s.frame {
            Frame::SmallTalk if fish => {
                self.fish_told = self.fish_caught;
                act(Gesture::Tell, &s.ctx.topic("fish"))
            }
            // Past the business, a new fish is worth keeping someone for.
            Frame::Transaction if fish && s.turn >= 1 && !s.i_bid(Frame::SmallTalk) => Act::Bid {
                frame: Frame::SmallTalk,
                topic: s.ctx.topic("fish"),
            },
            _ => default_next_act(self, s, rng),
        }
    }

    /// Each conversation moves his mood by how it went for him; a bad one adds to the
    /// grievance.
    fn remember(&mut self, e: Episode) {
        let felt = e.peak + e.end;
        self.mood = (self.mood + felt.signum()).clamp(-5, 5);
        // A talk that went badly is one more thing the town did to him.
        if felt < 0 {
            self.grievance = self.grievance.saturating_add(1);
        }
        self.mind.remember(e);
    }
}
