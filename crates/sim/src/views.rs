//! Text views of a running sim: the agent's eyes on the town.
use crate::brain::CharacterSet;
use crate::ids::VillagerId;
use crate::protocol::{Act, Attribution, BidAnswer, Emitted};
use crate::sim::Sim;
use content::Pos;
use std::fmt::Write as _;

/// One line per villager: where, doing what, since when.
#[must_use]
pub fn where_is_everyone<C: CharacterSet>(sim: &Sim<C>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", sim.tick().calendar());
    for v in &sim.state().villagers {
        let place = &sim.world().places[v.place.index()].id;
        let moving = v
            .moving
            .as_ref()
            .map(|m| format!(" -> {}", m.then))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "{:8} {:16} ({:2},{:2})  {:12} since {}{}",
            v.name,
            place,
            v.pos.x,
            v.pos.y,
            v.activity,
            v.since.calendar().time,
            moving
        );
    }
    out
}

/// A place with villagers drawn as the first letter of their names (uppercase).
/// A place with the living ground and villagers drawn in. Villagers are the first letter
/// of their names, uppercase. Ground glyphs follow `terrain::Surface`.
#[must_use]
pub fn render_place_now<C: CharacterSet>(sim: &Sim<C>, place: usize) -> String {
    let p = &sim.world().places[place];
    let mut overlay: Vec<(Pos, char)> = Vec::new();
    for y in 0..p.height {
        for x in 0..p.width {
            let pos = Pos { x, y };
            if let Some(surface) = sim.surface(place, pos) {
                let glyph = surface.glyph();
                if glyph != p.cell(x, y).glyph() {
                    overlay.push((pos, glyph));
                }
            }
        }
    }
    overlay.extend(
        sim.state()
            .villagers
            .iter()
            .filter(|v| v.place.index() == place)
            .map(|v| {
                (
                    v.pos,
                    v.name.chars().next().unwrap_or('?').to_ascii_uppercase(),
                )
            }),
    );
    content::render_place(sim.world(), place, &overlay)
}

/// One villager's day from an event list: departures, arrivals, activities, trouble.
#[must_use]
pub fn itinerary<C: CharacterSet>(sim: &Sim<C>, who: VillagerId, events: &[Emitted]) -> String {
    let mut out = String::new();
    let name = &sim.state().villagers[who.index()].name;
    let _ = writeln!(out, "{name}");
    let mut last_activity = String::new();
    for e in events.iter().filter(|e| e.who() == Some(who)) {
        let t = e.tick().calendar().time;
        match e {
            Emitted::Departed { activity, eta, .. } => {
                let _ = writeln!(
                    out,
                    "  {t} leaves for {activity}, eta {}",
                    eta.calendar().time
                );
            }
            Emitted::Activity { activity, .. } if activity != "walking" => {
                if *activity != last_activity {
                    let _ = writeln!(out, "  {t} {activity}");
                }
                last_activity.clone_from(activity);
            }
            Emitted::Moved { place, x, y, .. } => {
                // Only report place changes, not every tile.
                let _ = (place, x, y);
            }
            Emitted::Stuck { reason, .. } => {
                let _ = writeln!(out, "  {t} STUCK: {reason}");
            }
            Emitted::Decided { text, .. } => {
                let _ = writeln!(out, "  {t} [{text}]");
            }
            Emitted::Planned { items, dropped, .. } => {
                plan_lines(&mut out, &t.to_string(), items, dropped);
            }
            // Notes made inside a reaction are already in its `Decided` line.
            Emitted::Noted { text, cause, .. }
                if !matches!(cause, crate::protocol::Cause::Reaction { .. }) =>
            {
                let _ = writeln!(out, "  {t} note: {text}");
            }
            Emitted::Cleared { x, y, tiles, .. } => {
                let _ = writeln!(out, "  {t} clears {tiles} tiles around ({x}, {y})");
            }
            Emitted::Paid {
                from,
                to,
                amount,
                what,
                ..
            } => {
                let name = |p: &Option<VillagerId>| {
                    p.map_or_else(
                        || "outside".to_string(),
                        |v| sim.state().villagers[v.index()].name.clone(),
                    )
                };
                let _ = writeln!(
                    out,
                    "  {t} {} pays {} {amount} for {what}",
                    name(from),
                    name(to)
                );
            }
            Emitted::Stocked {
                object,
                good,
                delta,
                now,
                ..
            } => {
                let key = &sim.world().objects[object.index()].id;
                let _ = writeln!(out, "  {t} {good} {delta:+} at {key} (now {now})");
            }
            Emitted::Need { need, level, .. } => {
                let _ = writeln!(out, "  {t} {} ({level})", need.name());
            }
            Emitted::ConversationStarted { .. }
            | Emitted::ConversationEnded { .. }
            | Emitted::Mishap { .. } => {
                if let Some(line) = conversation_line(sim, e) {
                    let _ = writeln!(out, "  {t} {line}");
                }
            }
            Emitted::Talked {
                with,
                topic,
                minutes,
                ..
            } => {
                let other = &sim.state().villagers[with.index()].name;
                if *minutes == 0 {
                    let _ = writeln!(out, "  {t} greets {other}");
                } else {
                    let _ = writeln!(out, "  {t} talks with {other} about {topic} ({minutes}m)");
                }
            }
            _ => {}
        }
    }
    out
}

/// An itinerary line for the start or end of a conversation, or a mishap.
fn conversation_line<C: CharacterSet>(sim: &Sim<C>, e: &Emitted) -> Option<String> {
    let name = |v: &VillagerId| sim.state().villagers[v.index()].name.clone();
    match e {
        Emitted::ConversationStarted {
            with, frame, topic, ..
        } => Some(format!(
            "starts {} with {} about {topic}",
            frame.name(),
            name(with)
        )),
        Emitted::ConversationEnded {
            with,
            frame,
            my_peak,
            my_end,
            ..
        } => Some(format!(
            "done talking with {} ({}, peak {my_peak:+}, end {my_end:+})",
            name(with),
            frame.name()
        )),
        Emitted::Mishap { what, x, y, .. } => Some(format!(
            "{} at ({x}, {y})",
            format!("{what:?}").to_lowercase()
        )),
        _ => None,
    }
}

/// Place changes for one villager (door crossings), from an event list.
#[must_use]
pub fn place_changes<C: CharacterSet>(sim: &Sim<C>, who: VillagerId, events: &[Emitted]) -> String {
    let mut out = String::new();
    let mut last: Option<u16> = None;
    for e in events.iter().filter(|e| e.who() == Some(who)) {
        if let Emitted::Moved {
            tick, place, x, y, ..
        } = e
            && last != Some(place.0)
        {
            let _ = writeln!(
                out,
                "  {} enters {} at ({x}, {y})",
                tick.calendar().time,
                sim.world().places[place.index()].id
            );
            last = Some(place.0);
        }
    }
    out
}

/// Money and goods: every villager's coins and needs, then every stocked object.
#[must_use]
pub fn economy<C: CharacterSet>(sim: &Sim<C>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", sim.tick().calendar());
    for v in &sim.state().villagers {
        let n = &v.needs;
        let _ = writeln!(
            out,
            "{:<8} {:>4} coins  hunger {:>3} thirst {:>3} sleep {:>3} bladder {:>3}",
            v.name, v.coins, n.hunger, n.thirst, n.sleep, n.bladder
        );
    }
    for s in crate::protocol::stock_views(sim) {
        let _ = writeln!(out, "{:<18} {:<8} {}", s.key, s.good, s.count);
    }
    out
}

/// The day-plan lines of an itinerary. `*` marks items the character added or changed;
/// the rest are the schedule's.
fn plan_lines(
    out: &mut String,
    t: &str,
    items: &[crate::protocol::PlanItem],
    dropped: &[crate::protocol::PlanItem],
) {
    let plan: Vec<String> = items
        .iter()
        .map(|i| format!("{}{} {}", if i.schedule { "" } else { "*" }, i.at, i.text))
        .collect();
    let _ = writeln!(out, "  {t} [plan: {}]", plan.join("; "));
    if !dropped.is_empty() {
        let gone: Vec<String> = dropped
            .iter()
            .map(|i| format!("{} {}", i.at, i.text))
            .collect();
        let _ = writeln!(out, "  {t} [dropped from schedule: {}]", gone.join("; "));
    }
}

/// An act as a transcript line.
#[must_use]
pub fn describe_act(act: &Act) -> String {
    match act {
        Act::Move { kind, subject } if subject.is_empty() => kind.name().to_string(),
        Act::Move { kind, subject } => format!("{} \"{subject}\"", kind.name()),
        Act::Bid { frame, subject } => format!("bids for {} about \"{subject}\"", frame.name()),
        Act::Answer { answer } => match answer {
            BidAnswer::Toward => "turns toward it".into(),
            BidAnswer::Away { account: true } => "turns away, with a reason".into(),
            BidAnswer::Away { account: false } => "turns away".into(),
            BidAnswer::Against => "turns against it".into(),
        },
    }
}

/// Every conversation in `events`, turn by turn: what each speaker meant and did, and how
/// each listener took it. Lines are grouped by conversation in the order they started.
#[must_use]
pub fn transcripts<C: CharacterSet>(sim: &Sim<C>, events: &[Emitted]) -> String {
    let mut order: Vec<u32> = Vec::new();
    let mut lines: std::collections::BTreeMap<u32, String> = std::collections::BTreeMap::new();
    for e in events {
        if let Emitted::ConversationStarted { convo, .. } = e {
            order.push(*convo);
        }
        if let Some((convo, line)) = transcript_line(sim, e) {
            let l = lines.entry(convo).or_default();
            l.push_str(&line);
            l.push('\n');
        }
    }
    let mut out = String::new();
    for id in order {
        if let Some(l) = lines.remove(&id) {
            out.push_str(&l);
            out.push('\n');
        }
    }
    out
}

/// One transcript line and the conversation it belongs to.
fn transcript_line<C: CharacterSet>(sim: &Sim<C>, e: &Emitted) -> Option<(u32, String)> {
    let name = |v: VillagerId| sim.state().villagers[v.index()].name.clone();
    let t = e.tick().calendar().time;
    Some(match e {
        Emitted::ConversationStarted {
            convo,
            who,
            with,
            place,
            frame,
            topic,
            ..
        } => {
            let cal = e.tick().calendar();
            let place = &sim.world().places[place.index()].id;
            let (a, b) = (name(*who), name(*with));
            let frame = frame.name();
            let day = cal.day_index;
            let weekday = cal.weekday;
            let head = format!("day {day} {weekday:?} {t} {a} and {b} ({place}): {frame}");
            (*convo, format!("{head} about \"{topic}\""))
        }
        Emitted::Beat {
            convo,
            who,
            stance,
            meant,
            did,
            breach,
            ..
        } => {
            let mut line = format!("  {t} {:<5} {}", name(*who), describe_act(did));
            if meant != did {
                let _ = write!(line, "  (meant: {})", describe_act(meant));
            }
            if *breach {
                line.push_str("  [out of frame]");
            }
            let _ = write!(line, "  <{stance:?}>");
            (*convo, line)
        }
        Emitted::TookUp {
            convo,
            who,
            did,
            taken_as,
            warmth,
            attribution,
            overheard,
            ..
        } => {
            let verb = if *overheard { "overhears" } else { "takes it" };
            let reading = if taken_as == did {
                String::new()
            } else {
                format!(" as {}", taken_as.name())
            };
            let blame = match (*warmth < 0, attribution) {
                (true, Attribution::Character) => ", puts it down to them",
                (true, Attribution::Situation) => ", puts it down to the moment",
                _ => "",
            };
            let who = name(*who);
            (
                *convo,
                format!("          {who} {verb}{reading} ({warmth:+}{blame})"),
            )
        }
        Emitted::FrameChanged {
            convo, from, to, ..
        } => (*convo, format!("  {t} -- {} to {}", from.name(), to.name())),
        Emitted::ConversationEnded {
            convo,
            who,
            my_peak,
            my_end,
            ended_by,
            ..
        } => {
            let by = ended_by
                .map(|v| format!(", {} left", name(v)))
                .unwrap_or_default();
            let who = name(*who);
            (
                *convo,
                format!("  {t} {who} done: peak {my_peak:+}, end {my_end:+}{by}"),
            )
        }
        _ => return None,
    })
}
