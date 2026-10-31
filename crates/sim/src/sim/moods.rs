//! The mood system: moves each villager's moods from what happens to them, by the sources
//! their content gives each mood. It reads the hour (alone, idle, noise, crowds, dark,
//! money, garden), the night's weather, and each step's events (conversations, payments,
//! harvests, thoughts, getting stuck). A mood that changes level is an event; one that
//! rises to `ACTS_AT` is a trigger for the villager.
use super::Sim;
use crate::brain::{CharacterSet, Trigger};
use crate::fields::WEEDY;
use crate::ids::VillagerId;
use crate::moods::{ACTS_AT, POINTS_PER_LEVEL, Push, Source, apply};
use crate::protocol::{Blame, Emitted, FieldStage, MoodKind};
use crate::time::TICKS_PER_HOUR;
use content::{Delight, Gloom, Irritant, MoodSpec, Pride, Worry};

/// Activities that count as idle, for restlessness.
const IDLE: [&str; 4] = ["idle", "rest", "sit", "wander"];
/// Activities loud enough to bother the noise-shy within `NOISE_TILES`.
const LOUD: [&str; 3] = ["smith", "chop", "make-barrels"];
const NOISE_TILES: u16 = 6;
/// How long a mood that stays strong waits before it is offered to the villager again.
const OFFER_AGAIN: u32 = 3 * TICKS_PER_HOUR;
/// Others within `CROWD_TILES` that make a crowd.
const CROWD: usize = 4;
const CROWD_TILES: u16 = 3;
/// Hours of the night: from `DUSK` to `DAWN`.
const DUSK: u8 = 20;
const DAWN: u8 = 5;
/// Conversations someone must remember with another before they are no stranger.
const KNOWN_AFTER: u16 = 3;

impl<C: CharacterSet> Sim<C> {
    fn specs(&self, who: VillagerId) -> &[MoodSpec] {
        &self.content.villagers[who.index()].moods
    }

    /// Applies a push and tells the world: an event when the level changes, and a trigger
    /// when it rises to where it is acted on.
    fn push_mood(&mut self, who: VillagerId, push: &Push, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let Some((mood, was, level)) = apply(&mut self.state.moods[who.index()], push, tick) else {
            return;
        };
        out.push(Emitted::MoodShifted {
            tick,
            who,
            mood,
            was,
            level,
            cause: push.cause.clone(),
        });
        let busy = matches!(
            self.state.villagers[who.index()].activity.as_str(),
            "sleep" | "talk"
        );
        if level >= ACTS_AT && level > was && !busy {
            self.mark_offered(who, mood);
            self.react(who, Trigger::Mood { mood, level }, out);
        }
    }

    fn lonely_hour(&self, who: VillagerId, after_hours: u8, h: &Hour, pushes: &mut Vec<Push>) {
        if h.among_people {
            pushes.push(Push::by(MoodKind::Lonely, -1, Source::Time, ""));
        } else if !h.asleep && h.alone >= after_hours {
            let since = m_time(self, who);
            pushes.push(Push::by(
                MoodKind::Lonely,
                1,
                Source::Alone,
                format!("no company since {since}"),
            ));
        }
    }

    fn irritable_hour(&self, who: VillagerId, by: &[Irritant], h: &Hour, pushes: &mut Vec<Push>) {
        let mut moved = false;
        if !h.asleep
            && by.contains(&Irritant::Noise)
            && let Some(loud) = self.noise_near(who)
        {
            let name = self.state.villagers[loud.index()].name.clone();
            pushes.push(Push::by(
                MoodKind::Irritable,
                3,
                Source::Noise(loud),
                format!("the noise of {name} at work"),
            ));
            moved = true;
        }
        if !h.asleep && by.contains(&Irritant::Crowds) && self.crowded(who) {
            pushes.push(Push::by(
                MoodKind::Irritable,
                2,
                Source::Crowds,
                "too many people about",
            ));
            moved = true;
        }
        if !moved {
            pushes.push(Push::by(MoodKind::Irritable, -1, Source::Time, ""));
        }
    }

    fn anxious_hour(&self, who: VillagerId, about: &[Worry], h: &Hour, pushes: &mut Vec<Push>) {
        // Worry fades unless something keeps it up.
        pushes.push(Push::by(MoodKind::Anxious, -1, Source::Time, ""));
        for worry in about {
            match worry {
                Worry::Money(mark) => {
                    let coins = self.state.villagers[who.index()].coins;
                    if let Some(level) = money_level(coins, *mark) {
                        pushes.push(Push {
                            kind: MoodKind::Anxious,
                            by: 0,
                            floor: Some(level * POINTS_PER_LEVEL),
                            source: Source::Money,
                            cause: format!("{coins} coins, wants {mark}"),
                        });
                    }
                }
                Worry::Dark if !h.asleep && self.out_in_the_dark(who) => {
                    pushes.push(Push::by(
                        MoodKind::Anxious,
                        3,
                        Source::Dark,
                        "out after dark",
                    ));
                }
                Worry::Garden if !h.asleep && self.garden_suffering(who) => {
                    pushes.push(Push::by(
                        MoodKind::Anxious,
                        2,
                        Source::Garden,
                        "the garden going to weeds",
                    ));
                }
                Worry::Dark | Worry::Garden | Worry::Strangers => {}
            }
        }
    }

    fn proud_hour(&self, who: VillagerId, of: &[Pride], h: &Hour, pushes: &mut Vec<Push>) {
        let doing = self.state.villagers[who.index()].activity.clone();
        let at_work = of
            .iter()
            .any(|p| matches!(p, Pride::Work(a) if *a == doing));
        if at_work {
            pushes.push(Push::by(
                MoodKind::Proud,
                1,
                Source::Work,
                format!("good work: {doing}"),
            ));
        } else if of.contains(&Pride::Garden) && self.garden_thriving(who) {
            pushes.push(Push::by(
                MoodKind::Proud,
                1,
                Source::Garden,
                "the garden doing well",
            ));
        } else if h.hours.is_multiple_of(2) {
            pushes.push(Push::by(MoodKind::Proud, -1, Source::Time, ""));
        }
    }

    fn mark_offered(&mut self, who: VillagerId, mood: MoodKind) {
        let tick = self.state.tick;
        if let Some(m) = self.state.moods[who.index()]
            .moods
            .iter_mut()
            .find(|m| m.kind == mood)
        {
            m.offered = tick;
        }
    }

    /// A mood that stays strong is offered to the villager again every few hours, so
    /// someone still lonely goes on looking for company.
    fn offer_again(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let busy = matches!(
            self.state.villagers[who.index()].activity.as_str(),
            "sleep" | "talk"
        );
        if busy {
            return;
        }
        let due: Vec<(MoodKind, u8)> = self.state.moods[who.index()]
            .moods
            .iter()
            .filter(|m| m.level() >= ACTS_AT && tick.0 >= m.offered.0 + OFFER_AGAIN)
            .map(|m| (m.kind, m.level()))
            .collect();
        for (mood, level) in due {
            self.mark_offered(who, mood);
            self.react(who, Trigger::Mood { mood, level }, out);
        }
    }

    /// The hourly reading, called from the body's hour.
    pub(super) fn mood_hour(&mut self, who: VillagerId, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let v = &self.state.villagers[who.index()];
        let asleep = v.activity == "sleep";
        let idle = IDLE.contains(&v.activity.as_str());
        let m = &mut self.state.moods[who.index()];
        m.hours += 1;
        m.idle_hours = if idle {
            m.idle_hours.saturating_add(1)
        } else {
            0
        };
        let lately = tick.0.saturating_sub(m.last_company.0) < TICKS_PER_HOUR;
        if lately {
            m.alone_hours = 0;
        } else if !asleep {
            m.alone_hours = m.alone_hours.saturating_add(1);
        }
        let hour = Hour {
            asleep,
            idle,
            idle_hours: m.idle_hours,
            alone: m.alone_hours,
            hours: m.hours,
            among_people: !asleep && self.people_near(who) >= 2,
        };
        let mut pushes = Vec::new();
        for spec in self.specs(who).to_vec() {
            match spec {
                MoodSpec::Lonely { after_hours } => {
                    self.lonely_hour(who, after_hours, &hour, &mut pushes);
                }
                MoodSpec::Restless { after_hours } => {
                    restless_hour(after_hours, &hour, &mut pushes);
                }
                MoodSpec::Irritable { by } => self.irritable_hour(who, &by, &hour, &mut pushes),
                MoodSpec::Anxious { about } => self.anxious_hour(who, &about, &hour, &mut pushes),
                MoodSpec::Gloomy { .. } => {
                    pushes.push(Push::by(MoodKind::Gloomy, -1, Source::Time, ""));
                }
                MoodSpec::Cheerful { .. } => {
                    pushes.push(Push::by(MoodKind::Cheerful, -1, Source::Time, ""));
                }
                MoodSpec::Proud { of } => self.proud_hour(who, &of, &hour, &mut pushes),
            }
        }
        for p in &combined(pushes) {
            self.push_mood(who, p, out);
        }
        self.offer_again(who, out);
    }

    /// The night's weather, for everyone whose moods it moves.
    pub(super) fn moods_weather(&mut self, rained: bool, out: &mut Vec<Emitted>) {
        for i in 0..self.state.villagers.len() {
            let who = VillagerId(u16::try_from(i).expect("few villagers"));
            let mut pushes = Vec::new();
            for spec in self.specs(who) {
                match spec {
                    MoodSpec::Gloomy { by } if rained && by.contains(&Gloom::Rain) => {
                        pushes.push(Push::by(MoodKind::Gloomy, 6, Source::Rain, "rain"));
                    }
                    MoodSpec::Cheerful { by } if rained && by.contains(&Delight::Rain) => {
                        pushes.push(Push::by(MoodKind::Cheerful, 6, Source::Rain, "rain"));
                    }
                    MoodSpec::Cheerful { by } if !rained && by.contains(&Delight::FineDays) => {
                        pushes.push(Push::by(
                            MoodKind::Cheerful,
                            5,
                            Source::FineDay,
                            "a fine day",
                        ));
                    }
                    _ => {}
                }
            }
            for p in &pushes {
                self.push_mood(who, p, out);
            }
        }
    }

    /// Moves moods from the events of a step, from `start` on, including those the moods
    /// themselves set off.
    pub(super) fn feel_events(&mut self, start: usize, out: &mut Vec<Emitted>) {
        let mut i = start;
        while i < out.len() {
            let pushes = self.pushes_for(&out[i]);
            i += 1;
            for (who, p) in pushes {
                self.push_mood(who, &p, out);
            }
        }
    }

    /// Whether `who` is prone to a mood that `f` picks out.
    fn prone(&self, who: VillagerId, f: impl Fn(&MoodSpec) -> bool) -> bool {
        self.specs(who).iter().any(f)
    }

    /// What one event does to whose moods. Conversations also mark company.
    fn pushes_for(&mut self, e: &Emitted) -> Vec<(VillagerId, Push)> {
        let tick = self.state.tick;
        match e {
            Emitted::ConversationStarted { who, with, .. } => {
                for v in [who, with] {
                    self.state.moods[v.index()].last_company = tick;
                    self.state.moods[v.index()].alone_hours = 0;
                }
                Vec::new()
            }
            Emitted::ConversationEnded {
                who,
                with,
                my_peak,
                my_end,
                ..
            } => {
                self.state.moods[who.index()].last_company = tick;
                self.state.moods[who.index()].alone_hours = 0;
                self.talk_pushes(*who, *with, *my_peak, *my_end)
                    .into_iter()
                    .map(|p| (*who, p))
                    .collect()
            }
            Emitted::Paid {
                to: Some(to),
                amount,
                what,
                ..
            } if *amount > 0
                && self.prone(
                    *to,
                    |s| matches!(s, MoodSpec::Proud { of } if of.contains(&Pride::Earning)),
                ) =>
            {
                let cause = format!("paid {amount} for {what}");
                vec![(*to, Push::by(MoodKind::Proud, 2, Source::Earning, cause))]
            }
            Emitted::Tended {
                who,
                stored: Some(g),
                ..
            } if self.prone(
                *who,
                |s| matches!(s, MoodSpec::Proud { of } if of.contains(&Pride::Harvest)),
            ) =>
            {
                let cause = format!("bringing in the {}", g.good);
                vec![(*who, Push::by(MoodKind::Proud, 1, Source::Harvest, cause))]
            }
            Emitted::Thought { who, thought, .. } => self
                .thought_pushes(*who, thought)
                .into_iter()
                .map(|p| (*who, p))
                .collect(),
            Emitted::Stuck { who, reason, .. }
                if self.prone(
                    *who,
                    |s| matches!(s, MoodSpec::Irritable { by } if by.contains(&Irritant::Stuck)),
                ) =>
            {
                vec![(
                    *who,
                    Push::by(MoodKind::Irritable, 3, Source::Stuck, reason.clone()),
                )]
            }
            _ => Vec::new(),
        }
    }

    /// A conversation's end, for `who`: company eases loneliness; it may cheer, irritate,
    /// or, with someone they hardly know, unsettle them.
    fn talk_pushes(&self, who: VillagerId, with: VillagerId, peak: i8, end: i8) -> Vec<Push> {
        let name = self.state.villagers[with.index()].name.clone();
        let mut pushes = vec![Push::by(
            MoodKind::Lonely,
            -8,
            Source::Company(with),
            format!("talked with {name}"),
        )];
        if end > 0
            && self.prone(
                who,
                |s| matches!(s, MoodSpec::Cheerful { by } if by.contains(&Delight::Company)),
            )
        {
            pushes.push(Push::by(
                MoodKind::Cheerful,
                2 * end,
                Source::Company(with),
                format!("a good talk with {name}"),
            ));
        }
        if peak < 0
            && self.prone(
                who,
                |s| matches!(s, MoodSpec::Irritable { by } if by.contains(&Irritant::Slights)),
            )
        {
            pushes.push(Push::by(
                MoodKind::Irritable,
                -2 * peak,
                Source::Slight(with),
                format!("words with {name}"),
            ));
        }
        if self.prone(
            who,
            |s| matches!(s, MoodSpec::Anxious { about } if about.contains(&Worry::Strangers)),
        ) && self.familiarity(who, with) < KNOWN_AFTER
        {
            pushes.push(Push::by(
                MoodKind::Anxious,
                4,
                Source::Stranger(with),
                format!("talking with {name}, whom they hardly know"),
            ));
        }
        pushes
    }

    /// A thought of `who`'s: a good one cheers those who take delight in good things;
    /// a gloomy one about the world or themselves feeds anxiety; one blaming someone
    /// irritates those who mind slights.
    fn thought_pushes(&self, who: VillagerId, thought: &crate::protocol::Thought) -> Vec<Push> {
        let about = thought.about.key.replace('-', " ");
        if thought.feeling > 0
            && self.prone(
                who,
                |s| matches!(s, MoodSpec::Cheerful { by } if by.contains(&Delight::GoodThings)),
            )
        {
            return vec![Push::by(MoodKind::Cheerful, 2, Source::GoodThing, about)];
        }
        if thought.feeling < 0
            && matches!(thought.blame, Blame::World | Blame::Me)
            && self.prone(who, |s| matches!(s, MoodSpec::Anxious { .. }))
        {
            return vec![Push::by(
                MoodKind::Anxious,
                -2 * thought.feeling,
                Source::Time,
                about,
            )];
        }
        if let Blame::Them(them) = thought.blame
            && thought.feeling < 0
            && self.prone(
                who,
                |s| matches!(s, MoodSpec::Irritable { by } if by.contains(&Irritant::Slights)),
            )
        {
            let name = self.state.villagers[them.index()].name.clone();
            return vec![Push::by(
                MoodKind::Irritable,
                2,
                Source::Slight(them),
                format!("{name}, over {about}"),
            )];
        }
        Vec::new()
    }

    fn familiarity(&self, who: VillagerId, other: VillagerId) -> u16 {
        let ctx = self.ctx();
        self.state
            .characters
            .interlocutor_ref(who, &ctx)
            .map_or(KNOWN_AFTER, |m| m.memory().regard(other).familiarity)
    }

    /// Someone at loud work close by, if anyone.
    fn noise_near(&self, who: VillagerId) -> Option<VillagerId> {
        let me = &self.state.villagers[who.index()];
        self.state.villagers.iter().enumerate().find_map(|(i, o)| {
            let near = o.place == me.place
                && o.pos.x.abs_diff(me.pos.x) <= NOISE_TILES
                && o.pos.y.abs_diff(me.pos.y) <= NOISE_TILES;
            (i != who.index() && near && LOUD.contains(&o.activity.as_str()))
                .then(|| VillagerId(u16::try_from(i).expect("few villagers")))
        })
    }

    /// Others awake within `CROWD_TILES`.
    fn people_near(&self, who: VillagerId) -> usize {
        let me = &self.state.villagers[who.index()];
        self.state
            .villagers
            .iter()
            .enumerate()
            .filter(|(i, o)| {
                *i != who.index()
                    && o.place == me.place
                    && o.activity != "sleep"
                    && o.pos.x.abs_diff(me.pos.x) <= CROWD_TILES
                    && o.pos.y.abs_diff(me.pos.y) <= CROWD_TILES
            })
            .count()
    }

    fn crowded(&self, who: VillagerId) -> bool {
        self.people_near(who) >= CROWD
    }

    /// Outdoors, not under any roof, at night.
    fn out_in_the_dark(&self, who: VillagerId) -> bool {
        let hour = self.state.tick.calendar().time.hour;
        if (DAWN..DUSK).contains(&hour) {
            return false;
        }
        let v = &self.state.villagers[who.index()];
        let outdoors = self.world.places[v.place.index()].kind == content::PlaceKind::Zone;
        let roofed = self
            .world
            .roofs
            .iter()
            .any(|r| r.place == v.place.index() && r.rect.contains(v.pos.x, v.pos.y));
        outdoors && !roofed
    }

    /// The plots that are `who`'s to care about: their household's garden, and any they
    /// keep as their work.
    fn garden_plots(&self, who: VillagerId) -> Vec<usize> {
        let v = &self.content.villagers[who.index()];
        let home = self
            .world
            .objects
            .iter()
            .find(|o| o.id == v.bed)
            .map(|o| o.building.as_str());
        self.content
            .zones
            .iter()
            .flat_map(|z| z.plots.iter())
            .enumerate()
            .filter(|(_, p)| {
                p.keeper.as_deref() == Some(v.id.as_str())
                    || (home.is_some() && p.household.as_deref() == home)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// A quarter or more of their garden spoiling or choked with weeds, and at least two
    /// tiles of it.
    fn garden_suffering(&self, who: VillagerId) -> bool {
        let (mut bad, mut all) = (0usize, 0usize);
        for p in self.garden_plots(who) {
            for t in &self.state.fields.plots[p].tiles {
                all += 1;
                if t.stage == FieldStage::Spoiling || (t.planted() && t.weeds >= WEEDY) {
                    bad += 1;
                }
            }
        }
        bad >= 2 && bad * 4 >= all
    }

    fn garden_thriving(&self, who: VillagerId) -> bool {
        let plots = self.garden_plots(who);
        !plots.is_empty()
            && plots.iter().all(|&p| {
                let tiles = &self.state.fields.plots[p].tiles;
                tiles
                    .iter()
                    .all(|t| t.weeds < WEEDY && t.stage != FieldStage::Spoiling)
                    && tiles
                        .iter()
                        .any(|t| matches!(t.stage, FieldStage::Growing | FieldStage::Ripe))
            })
    }
}

/// One push per mood from an hour's several: the changes added, the highest floor kept,
/// and the cause of the biggest rise.
fn combined(pushes: Vec<Push>) -> Vec<Push> {
    let mut out: Vec<Push> = Vec::new();
    for p in pushes {
        match out.iter_mut().find(|o| o.kind == p.kind) {
            Some(o) => {
                if p.by > o.by.max(0) || (p.floor > o.floor && p.floor.is_some()) {
                    o.source = p.source;
                    o.cause.clone_from(&p.cause);
                }
                o.by = o.by.saturating_add(p.by);
                o.floor = o.floor.max(p.floor);
            }
            None => out.push(p),
        }
    }
    out
}

/// When `who` last had company, as a time of day.
fn m_time<C: CharacterSet>(sim: &Sim<C>, who: VillagerId) -> String {
    let t = sim.state.moods[who.index()].last_company.calendar().time;
    let h = t.hour % 12;
    format!(
        "{}:{:02}{}",
        if h == 0 { 12 } else { h },
        t.minute,
        if t.hour < 12 { "am" } else { "pm" }
    )
}

/// How anxious `coins` against a `mark` makes someone, if at all: 1 below the mark, 2
/// below a quarter of it, 3 with nothing.
#[must_use]
pub fn money_level(coins: u32, mark: u32) -> Option<u8> {
    if coins >= mark {
        None
    } else if coins == 0 {
        Some(3)
    } else if coins * 4 >= mark {
        Some(1)
    } else {
        Some(2)
    }
}

fn restless_hour(after_hours: u8, h: &Hour, pushes: &mut Vec<Push>) {
    if !h.asleep && h.idle_hours >= after_hours {
        pushes.push(Push::by(
            MoodKind::Restless,
            3,
            Source::Idle,
            format!("nothing to do for {} hours", h.idle_hours),
        ));
    } else if !h.idle {
        pushes.push(Push::by(MoodKind::Restless, -3, Source::Time, ""));
    }
}

/// What an hour was like for someone, for the hourly mood rules.
struct Hour {
    asleep: bool,
    idle: bool,
    idle_hours: u8,
    alone: u8,
    hours: u32,
    among_people: bool,
}
