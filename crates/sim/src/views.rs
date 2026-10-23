//! Text views of a running sim: the agent's eyes on the town.
use crate::brain::CharacterSet;
use crate::ids::VillagerId;
use crate::protocol::Emitted;
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
