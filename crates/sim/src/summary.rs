//! Day summaries derived from the event stream. Observe-only: nothing here feeds back into
//! the sim, so the summarizer can be changed and re-run over any history the sim can
//! regenerate.
//!
//! The rule of thumb is to report deviations from routine. Each line has a key ("Tom skipped
//! tend-bar", "Mara and Tom talked about fishing"); a key seen on enough of the previous
//! week's days, or on the same weekday last week, is routine. A habit that did not happen
//! today is news too.
use crate::ids::VillagerId;
use crate::protocol::{Cause, DaySummary, Emitted, PlanItem, SummaryItem, SummaryKind};
use crate::time::{TICKS_PER_DAY, TICKS_PER_MINUTE, Tick};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

mod talk;

/// How many previous days decide what is routine.
const LOOKBACK: usize = 7;
/// Seen on this many of them: routine.
const ROUTINE_DAYS: usize = 3;
/// Seen on this many of them and missing today: news.
const HABIT_DAYS: usize = 5;
/// A planned activity that has not started this long after its time was skipped.
const SKIP_GRACE: u32 = 90 * TICKS_PER_MINUTE;
/// Activities that are not news by themselves.
const QUIET: [&str; 3] = ["walking", "talk", "idle"];
/// Needs, by the name `Need::name` gives them, as they appear in reaction triggers.
const NEED_WORDS: [&str; 4] = ["hungry", "thirsty", "sleepy", "needs the privy"];

#[derive(Debug)]
struct Draft {
    tick: Tick,
    who: Option<VillagerId>,
    kind: SummaryKind,
    text: String,
    cause: Option<Cause>,
    /// The keys deciding routine; a draft is routine when all its keys are.
    keys: Vec<String>,
}

#[derive(Debug, Default)]
struct Day {
    drafts: Vec<Draft>,
    /// (who, activity) started under a reaction that was not about a need, with the first
    /// tick, a count and the first cause.
    unplanned: BTreeMap<(u16, String), (Tick, u16, Cause)>,
    /// Activities in each villager's plan today.
    planned: BTreeMap<u16, BTreeSet<String>>,
    /// Errands of the body per villager: activity -> count.
    body: BTreeMap<u16, BTreeMap<String, u16>>,
    /// Money per villager: (signed amount, what), and coins left after the last payment.
    money: BTreeMap<u16, Vec<(Tick, i64, String)>>,
    left: BTreeMap<u16, (Tick, u32)>,
    /// Keys that only feed habits: single payments ("Nell buys bread"), pairs who talked
    /// about anything.
    habit_keys: BTreeSet<String>,
    greetings: u16,
    cleared: BTreeMap<u16, (Tick, u32)>,
    urgent: BTreeSet<(u16, String)>,
}

#[derive(Debug)]
pub struct Summarizer {
    names: Vec<String>,
    objects: Vec<String>,
    open: BTreeMap<u32, Day>,
    /// Planned activities waiting to see whether they start: (who, due, activity, cause).
    checks: Vec<(VillagerId, Tick, String, Cause)>,
    /// Recent activity starts, for the checks: (who, tick, activity).
    starts: Vec<(u16, Tick, String)>,
    days: Vec<DaySummary>,
    /// Keys seen on each finished day, in day order.
    seen: Vec<BTreeSet<String>>,
    /// A readable phrase per habit key, for "did not happen" lines.
    habits: BTreeMap<String, (Option<VillagerId>, String)>,
    /// Conversations in progress.
    talks: BTreeMap<u32, talk::Talk>,
    /// Topic glosses by key, for showing what conversations were about.
    glosses: BTreeMap<String, String>,
}

fn time(tick: Tick) -> String {
    tick.calendar().time.to_string()
}

/// Digits do not make a line new: "Sunday off after 6 days" and "after 5 days" match.
fn normalize(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_digit() {
                '#'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

fn need_trigger(cause: &Cause) -> bool {
    matches!(cause, Cause::Reaction { trigger } if NEED_WORDS.iter().any(|w| trigger.starts_with(w)))
}

impl Summarizer {
    #[must_use]
    pub fn new(
        names: Vec<String>,
        objects: Vec<String>,
        glosses: BTreeMap<String, String>,
    ) -> Summarizer {
        Summarizer {
            names,
            objects,
            open: BTreeMap::new(),
            checks: Vec::new(),
            starts: Vec::new(),
            days: Vec::new(),
            seen: Vec::new(),
            habits: BTreeMap::new(),
            talks: BTreeMap::new(),
            glosses,
        }
    }

    /// Finished days, oldest first.
    #[must_use]
    pub fn days(&self) -> &[DaySummary] {
        &self.days
    }

    fn name(&self, who: VillagerId) -> &str {
        self.names.get(who.index()).map_or("?", String::as_str)
    }

    /// A topic's gloss and whose: "money (Mara)".
    fn topic(&self, t: &crate::protocol::Topic) -> String {
        let gloss = self
            .glosses
            .get(&t.key)
            .map_or(t.key.as_str(), String::as_str);
        match t.about {
            Some(v) => format!("{gloss} ({})", self.name(v)),
            None => gloss.to_string(),
        }
    }

    fn day(&mut self, tick: Tick) -> &mut Day {
        self.open.entry(tick.0 / TICKS_PER_DAY).or_default()
    }

    fn draft(&mut self, tick: Tick, d: Draft) {
        self.day(tick).drafts.push(d);
    }

    pub fn ingest(&mut self, e: &Emitted) {
        let tick = e.tick();
        match e {
            Emitted::Planned {
                who,
                items,
                dropped,
                ..
            } => self.planned(tick, *who, items, dropped),
            Emitted::Activity {
                who,
                activity,
                cause,
                ..
            } => self.started(tick, *who, activity, cause),
            Emitted::Talked {
                who,
                with,
                topic,
                minutes,
                cause,
                ..
            } => self.talked(tick, *who, *with, topic, *minutes, cause),
            Emitted::Paid {
                from,
                to,
                amount,
                what,
                payer_left,
                ..
            } => self.paid(tick, *from, *to, *amount, what, *payer_left),
            Emitted::Cleared { who, tiles, .. } => {
                let entry = self.day(tick).cleared.entry(who.0).or_insert((tick, 0));
                entry.1 += u32::from(*tiles);
            }
            Emitted::ConversationStarted { .. }
            | Emitted::Beat { .. }
            | Emitted::TookUp { .. }
            | Emitted::ConversationEnded { .. } => self.conversation(e),
            other => self.incident(other),
        }
        self.advance_to(tick);
    }

    /// One-off lines: notes, shortages, urgent needs, getting stuck.
    fn incident(&mut self, e: &Emitted) {
        let tick = e.tick();
        match e {
            Emitted::Noted {
                who, text, cause, ..
            } if !need_trigger(cause) => {
                let name = self.name(*who).to_string();
                let key = format!("note:{}:{}", name.to_lowercase(), normalize(text));
                let text = format!("{name}: \"{text}\"");
                self.line(
                    tick,
                    *who,
                    SummaryKind::Note,
                    text,
                    Some(cause.clone()),
                    key,
                );
            }
            Emitted::Stocked {
                object,
                good,
                delta,
                now,
                who,
                cause,
                ..
            } if *now == 0 && *delta < 0 => {
                let obj = self
                    .objects
                    .get(object.index())
                    .cloned()
                    .unwrap_or_default();
                let key = format!("out:{obj}:{good}");
                let text = format!("{good} ran out at {obj}");
                self.line(
                    tick,
                    *who,
                    SummaryKind::Shortage,
                    text,
                    Some(cause.clone()),
                    key,
                );
            }
            Emitted::Need {
                who, need, level, ..
            } if *level >= crate::needs::URGENT => {
                let name = self.name(*who).to_string();
                let word = need.name();
                if self.day(tick).urgent.insert((who.0, word.to_string())) {
                    let key = format!("need:{}:{word}", name.to_lowercase());
                    let text = if word.starts_with("needs") {
                        format!("{name} urgently {word} ({level})")
                    } else {
                        format!("{name} is urgently {word} ({level})")
                    };
                    self.line(tick, *who, SummaryKind::Need, text, None, key);
                }
            }
            Emitted::Thought { who, thought, .. } => {
                let name = self.name(*who).to_string();
                let blame = match thought.blame {
                    crate::protocol::Blame::Me => "blames themself".to_string(),
                    crate::protocol::Blame::Them(v) => format!("blames {}", self.name(v)),
                    crate::protocol::Blame::World => "blames the world".into(),
                    crate::protocol::Blame::Luck => "puts it down to luck".into(),
                };
                let span = match (thought.lasting, thought.everywhere) {
                    (true, true) => ", for good and for everything",
                    (true, false) => ", for good",
                    (false, true) => ", for everything",
                    (false, false) => "",
                };
                let key = format!(
                    "thought:{}:{}:{blame}",
                    name.to_lowercase(),
                    thought.about.key
                );
                let aloud = if thought.aloud { " (aloud)" } else { "" };
                let text = format!(
                    "{name} {blame}{span}: {}{aloud}",
                    self.topic(&thought.about)
                );
                self.line(tick, *who, SummaryKind::Thought, text, None, key);
            }
            Emitted::Stuck {
                who, reason, cause, ..
            } => {
                let name = self.name(*who).to_string();
                // Never routine: a fresh key every time.
                let key = format!("stuck:{}:{}", tick.0, who.0);
                let text = format!("{name} got stuck: {reason}");
                self.line(
                    tick,
                    *who,
                    SummaryKind::Stuck,
                    text,
                    Some(cause.clone()),
                    key,
                );
            }
            _ => {}
        }
    }

    /// A one-key line about one villager.
    fn line(
        &mut self,
        tick: Tick,
        who: VillagerId,
        kind: SummaryKind,
        text: String,
        cause: Option<Cause>,
        key: String,
    ) {
        self.draft(
            tick,
            Draft {
                tick,
                who: Some(who),
                kind,
                text,
                cause,
                keys: vec![key],
            },
        );
    }

    fn talked(
        &mut self,
        tick: Tick,
        who: VillagerId,
        with: VillagerId,
        topic: &str,
        minutes: u32,
        cause: &Cause,
    ) {
        if minutes == 0 {
            self.day(tick).greetings += 1;
            return;
        }
        let (a, b) = (self.name(who).to_string(), self.name(with).to_string());
        let (lo, hi) = if who.0 <= with.0 { (&a, &b) } else { (&b, &a) };
        let key = format!("talk:{}:{}:{topic}", lo.to_lowercase(), hi.to_lowercase());
        // The habit is the pair talking at all; a different topic still counts.
        let pair = format!("talks:{}:{}", lo.to_lowercase(), hi.to_lowercase());
        self.habits
            .insert(pair.clone(), (Some(who), format!("{a} and {b} talk")));
        self.day(tick).habit_keys.insert(pair);
        let text = format!("{a} and {b} talked about {topic} ({minutes}m)");
        self.line(tick, who, SummaryKind::Talk, text, Some(cause.clone()), key);
    }

    fn planned(&mut self, tick: Tick, who: VillagerId, items: &[PlanItem], dropped: &[PlanItem]) {
        let day_start = tick.day_start();
        let acts: BTreeSet<String> = items.iter().filter_map(|i| i.activity.clone()).collect();
        self.day(tick).planned.insert(who.0, acts);
        for i in items {
            if let Some(activity) = &i.activity
                && !QUIET.contains(&activity.as_str())
            {
                let due = day_start.plus(i.at.seconds());
                if due >= tick {
                    let cause = if i.schedule {
                        Cause::Schedule
                    } else {
                        Cause::Plan
                    };
                    self.checks.push((who, due, activity.clone(), cause));
                }
            }
        }
        let gone: Vec<&str> = dropped
            .iter()
            .filter_map(|i| i.activity.as_deref())
            .collect();
        let added: Vec<&str> = items
            .iter()
            .filter(|i| !i.schedule)
            .filter_map(|i| i.activity.as_deref())
            .collect();
        if gone.is_empty() && added.is_empty() {
            return;
        }
        let name = self.name(who).to_string();
        let mut text = format!("{name} changed the day:");
        if !gone.is_empty() {
            let _ = write!(text, " no {}", gone.join(", "));
        }
        if !added.is_empty() {
            let _ = write!(
                text,
                "{} added {}",
                if gone.is_empty() { "" } else { ";" },
                added.join(", ")
            );
        }
        let key = format!(
            "plan:{}:-{}+{}",
            name.to_lowercase(),
            gone.join(","),
            added.join(",")
        );
        self.draft(
            tick,
            Draft {
                tick,
                who: Some(who),
                kind: SummaryKind::PlanChange,
                text,
                cause: Some(Cause::Plan),
                keys: vec![key],
            },
        );
    }

    fn started(&mut self, tick: Tick, who: VillagerId, activity: &str, cause: &Cause) {
        if QUIET.contains(&activity) {
            return;
        }
        self.starts.push((who.0, tick, activity.to_string()));
        if need_trigger(cause) || matches!(cause, Cause::Return) {
            let day = self.day(tick);
            *day.body
                .entry(who.0)
                .or_default()
                .entry(activity.to_string())
                .or_insert(0) += 1;
            return;
        }
        if let Cause::Reaction { .. } = cause {
            let day = self.day(tick);
            let in_plan = day
                .planned
                .get(&who.0)
                .is_some_and(|acts| acts.contains(activity));
            if !in_plan {
                let entry = day
                    .unplanned
                    .entry((who.0, activity.to_string()))
                    .or_insert((tick, 0, cause.clone()));
                entry.1 += 1;
            }
        }
    }

    fn paid(
        &mut self,
        tick: Tick,
        from: Option<VillagerId>,
        to: Option<VillagerId>,
        amount: u32,
        what: &str,
        payer_left: u32,
    ) {
        let name_of = |s: &Self, p: Option<VillagerId>| {
            p.map_or_else(|| "outside".to_string(), |v| s.name(v).to_string())
        };
        let (giver, taker) = (name_of(self, from), name_of(self, to));
        let key = format!(
            "pay:{}:{}:{}",
            giver.to_lowercase(),
            taker.to_lowercase(),
            normalize(what)
        );
        self.habits.insert(
            key.clone(),
            (from.or(to), format!("{giver} pays {taker} for {what}")),
        );
        let day = self.day(tick);
        if let Some(v) = from {
            day.left.insert(v.0, (tick, payer_left));
        }
        // A payment of nothing (the payer was broke) moves no money; `left` notes the 0.
        if amount == 0 {
            return;
        }
        day.habit_keys.insert(key);
        let amount = i64::from(amount);
        if let Some(v) = from {
            day.money
                .entry(v.0)
                .or_default()
                .push((tick, -amount, what.to_string()));
            day.left.insert(v.0, (tick, payer_left));
        }
        if let Some(v) = to {
            day.money
                .entry(v.0)
                .or_default()
                .push((tick, amount, what.to_string()));
        }
    }

    /// Settles plan checks whose grace has passed, and closes days that are fully settled.
    /// Call with the sim's tick after each step, so quiet stretches still settle.
    pub fn advance_to(&mut self, now: Tick) {
        let mut due = Vec::new();
        self.checks.retain(|c| {
            if now.0 > c.1.0 + SKIP_GRACE {
                due.push(c.clone());
                false
            } else {
                true
            }
        });
        for (who, at, activity, cause) in due {
            let happened = self.starts.iter().any(|(w, t, a)| {
                *w == who.0 && *a == activity && t.0 >= at.0 && t.0 <= at.0 + SKIP_GRACE
            });
            if !happened {
                let name = self.name(who).to_string();
                let key = format!("skip:{}:{activity}", name.to_lowercase());
                self.draft(
                    at,
                    Draft {
                        tick: at,
                        who: Some(who),
                        kind: SummaryKind::Skipped,
                        text: format!("{name} skipped {activity} (planned for {})", time(at)),
                        cause: Some(cause),
                        keys: vec![key],
                    },
                );
            }
        }
        let keep_from = now.0.saturating_sub(2 * TICKS_PER_DAY);
        self.starts.retain(|s| s.1.0 >= keep_from);
        let closable: Vec<u32> = self
            .open
            .keys()
            .copied()
            .filter(|d| now.0 > (d + 1) * TICKS_PER_DAY + SKIP_GRACE)
            .collect();
        for d in closable {
            if let Some(day) = self.open.remove(&d) {
                self.close(d, day);
            }
        }
    }

    fn close(&mut self, d: u32, mut day: Day) {
        let start = Tick(d * TICKS_PER_DAY);
        self.close_activity(start, &mut day);
        self.close_money(start, &mut day);
        self.classify(d, start, day);
    }

    /// Unplanned activities and errands of the body become lines.
    fn close_activity(&self, start: Tick, day: &mut Day) {
        for ((who, activity), (tick, count, cause)) in std::mem::take(&mut day.unplanned) {
            let name = self.name(VillagerId(who)).to_string();
            let times = if count > 1 {
                format!(" ×{count}")
            } else {
                String::new()
            };
            day.drafts.push(Draft {
                tick,
                who: Some(VillagerId(who)),
                kind: SummaryKind::Unplanned,
                text: format!("{name}: {activity}{times}, not in the day's plan"),
                cause: Some(cause),
                keys: vec![format!("act:{}:{activity}", name.to_lowercase())],
            });
        }
        for (who, errands) in std::mem::take(&mut day.body) {
            let name = self.name(VillagerId(who)).to_string();
            let list: Vec<String> = errands
                .iter()
                .map(|(a, n)| {
                    if *n > 1 {
                        format!("{a} ×{n}")
                    } else {
                        a.clone()
                    }
                })
                .collect();
            day.drafts.push(Draft {
                tick: start,
                who: Some(VillagerId(who)),
                kind: SummaryKind::Body,
                text: format!("{name}: {}", list.join(", ")),
                cause: None,
                keys: Vec::new(),
            });
        }
    }

    /// Money, running out of it, weeding and greetings become lines.
    fn close_money(&self, start: Tick, day: &mut Day) {
        for (who, entries) in std::mem::take(&mut day.money) {
            let name = self.name(VillagerId(who)).to_string();
            let mut by: BTreeMap<(bool, String), (i64, u16)> = BTreeMap::new();
            for (_, amount, what) in &entries {
                let e = by.entry((*amount >= 0, what.clone())).or_insert((0, 0));
                e.0 += amount;
                e.1 += 1;
            }
            let parts: Vec<String> = by
                .iter()
                .map(|((_, what), (sum, n))| {
                    let times = if *n > 1 {
                        format!(" ×{n}")
                    } else {
                        String::new()
                    };
                    format!("{sum:+} {what}{times}")
                })
                .collect();
            let net: i64 = entries.iter().map(|(_, a, _)| a).sum();
            let first = entries.first().map_or(start, |(t, _, _)| *t);
            let left = day.left.get(&who).map(|(_, c)| *c);
            let mut text = format!("{name}: {} (net {net:+})", parts.join(", "));
            if let Some(c) = left {
                let _ = write!(text, ", {c} left");
            }
            let mut keys: Vec<String> = by
                .keys()
                .map(|(earn, what)| {
                    format!(
                        "money:{}:{}{}",
                        name.to_lowercase(),
                        if *earn { "+" } else { "-" },
                        normalize(what)
                    )
                })
                .collect();
            keys.sort();
            day.drafts.push(Draft {
                tick: first,
                who: Some(VillagerId(who)),
                kind: SummaryKind::Money,
                text,
                cause: None,
                keys,
            });
        }
        for (&who, &(tick, coins)) in &day.left {
            if coins == 0 {
                let name = self.name(VillagerId(who)).to_string();
                day.drafts.push(Draft {
                    tick,
                    who: Some(VillagerId(who)),
                    kind: SummaryKind::Money,
                    text: format!("{name} is out of money"),
                    cause: None,
                    keys: vec![format!("broke:{}", name.to_lowercase())],
                });
            }
        }
        for (who, (tick, tiles)) in std::mem::take(&mut day.cleared) {
            let name = self.name(VillagerId(who)).to_string();
            day.drafts.push(Draft {
                tick,
                who: Some(VillagerId(who)),
                kind: SummaryKind::Other,
                text: format!("{name} cleared {tiles} tiles of weeds"),
                cause: None,
                keys: vec![format!("clear:{}", name.to_lowercase())],
            });
        }
        if day.greetings > 0 {
            day.drafts.push(Draft {
                tick: start,
                who: None,
                kind: SummaryKind::Other,
                text: format!("{} greetings in passing", day.greetings),
                cause: None,
                keys: Vec::new(),
            });
        }
    }

    /// Splits the day's lines into news and routine, adds habits that did not happen, and
    /// files the day.
    fn classify(&mut self, d: u32, start: Tick, day: Day) {
        // Routine or news, by how often the same keys came up in the previous week.
        let history: Vec<&BTreeSet<String>> = self.seen.iter().rev().take(LOOKBACK).collect();
        let last_week = self
            .seen
            .len()
            .checked_sub(LOOKBACK)
            .and_then(|i| self.seen.get(i));
        let seen_days = |key: &str| history.iter().filter(|s| s.contains(key)).count();
        let mut today: BTreeSet<String> = BTreeSet::new();
        let mut news = Vec::new();
        let mut routine = Vec::new();
        for d in merge_repeats(day.drafts) {
            today.extend(d.keys.iter().cloned());
            let counts: Vec<usize> = d.keys.iter().map(|k| seen_days(k)).collect();
            let weekly = !d.keys.is_empty()
                && d.keys
                    .iter()
                    .all(|k| last_week.is_some_and(|s| s.contains(k)));
            let is_routine = d.kind == SummaryKind::Body
                || (d.kind == SummaryKind::Other && d.keys.is_empty())
                || (!d.keys.is_empty() && counts.iter().all(|&c| c >= ROUTINE_DAYS))
                || weekly;
            let item = SummaryItem {
                tick: d.tick,
                who: d.who,
                kind: d.kind,
                text: d.text,
                cause: d.cause,
                seen_days: u8::try_from(counts.iter().copied().min().unwrap_or(0))
                    .unwrap_or(u8::MAX),
            };
            if is_routine {
                routine.push(item);
            } else {
                news.push(item);
            }
        }
        today.extend(day.habit_keys.iter().cloned());
        // Habits that did not happen.
        for (key, (who, phrase)) in &self.habits {
            let n = seen_days(key);
            // Not news if it was also missing on this weekday last week (Tom's Sunday).
            let weekly_gap = last_week.is_some_and(|s| !s.contains(key));
            if n >= HABIT_DAYS && !today.contains(key) && !weekly_gap {
                news.push(SummaryItem {
                    tick: start,
                    who: *who,
                    kind: SummaryKind::Absent,
                    text: format!("{phrase} on most days ({n} of the last {LOOKBACK}), not today"),
                    cause: None,
                    seen_days: u8::try_from(n).unwrap_or(u8::MAX),
                });
            }
        }
        news.sort_by_key(|i| (i.tick, i.kind));
        routine.sort_by_key(|i| (i.tick, i.kind));
        self.seen.push(today);
        self.days.push(DaySummary {
            day: d,
            calendar: start.calendar(),
            news,
            routine,
        });
    }
}

/// Folds lines with the same single key into the first, with a count: a note made three
/// times in a day is one line.
fn merge_repeats(drafts: Vec<Draft>) -> Vec<Draft> {
    let mut out: Vec<Draft> = Vec::new();
    let mut counts: Vec<u16> = Vec::new();
    for d in drafts {
        let same = (d.keys.len() == 1)
            .then(|| out.iter().position(|o| o.keys == d.keys))
            .flatten();
        if let Some(i) = same {
            counts[i] += 1;
        } else {
            out.push(d);
            counts.push(1);
        }
    }
    for (d, n) in out.iter_mut().zip(counts) {
        if n > 1 {
            let _ = write!(d.text, " ×{n}");
        }
    }
    out
}

/// A day summary as text, for the tools.
#[must_use]
pub fn render(day: &DaySummary, with_routine: bool) -> String {
    let mut out = String::new();
    let c = &day.calendar;
    let _ = writeln!(
        out,
        "Y{} {} {} ({})",
        c.year,
        c.season.name(),
        c.day_of_season,
        c.weekday.name()
    );
    if day.news.is_empty() {
        let _ = writeln!(out, "  nothing out of the ordinary");
    }
    for i in &day.news {
        let _ = writeln!(
            out,
            "  {:>7}  {}",
            i.tick.calendar().time.to_string(),
            i.text
        );
    }
    if with_routine {
        for i in &day.routine {
            let _ = writeln!(
                out,
                "  {:>7}  ({})",
                i.tick.calendar().time.to_string(),
                i.text
            );
        }
    } else if !day.routine.is_empty() {
        let _ = writeln!(out, "  and {} routine things", day.routine.len());
    }
    out
}
