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
#[must_use]
pub fn render_place_now<C: CharacterSet>(sim: &Sim<C>, place: usize) -> String {
    let overlay: Vec<(Pos, char)> = sim
        .state()
        .villagers
        .iter()
        .filter(|v| v.place.index() == place)
        .map(|v| {
            (
                v.pos,
                v.name.chars().next().unwrap_or('?').to_ascii_uppercase(),
            )
        })
        .collect();
    content::render_place(sim.world(), place, &overlay)
}

/// One villager's day from an event list: departures, arrivals, activities, trouble.
#[must_use]
pub fn itinerary<C: CharacterSet>(sim: &Sim<C>, who: VillagerId, events: &[Emitted]) -> String {
    let mut out = String::new();
    let name = &sim.state().villagers[who.index()].name;
    let _ = writeln!(out, "{name}");
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
                let v = &sim.state().villagers[who.index()];
                let _ = v;
                let _ = writeln!(out, "  {t} {activity}");
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
