//! The railway: goods trains on a timetable from content. Where a train is follows from
//! the clock alone, so only what changes hands is state: goods left on the platform to be
//! sent, and who left them.
use crate::ids::VillagerId;
use crate::time::{Tick, Weekday};
use content::{Rail, RailwaySpec, TrackEnd, TrainRun};
use serde::{Deserialize, Serialize};

/// Seconds a train takes to cross one tile.
pub const TICKS_PER_TILE: u32 = 4;
/// Tiles a train covers: an engine and three wagons, two tiles each.
pub const TRAIN_TILES: u32 = 8;

/// Goods someone has left on the platform for the next train.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consignment {
    pub who: VillagerId,
    pub good: String,
    pub count: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RailState {
    pub consigned: Vec<Consignment>,
}

/// A train's day: when it enters the track, reaches the stop, leaves it, and is gone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    pub enter: Tick,
    pub arrive: Tick,
    pub depart: Tick,
    pub exit: Tick,
    pub from: TrackEnd,
    pub stops: bool,
}

/// Today's runs, in timetable order, for a day starting at `day_start`.
#[must_use]
pub fn timings(spec: &RailwaySpec, rail: &Rail, day_start: Tick, weekday: Weekday) -> Vec<Timing> {
    spec.runs
        .iter()
        .filter(|r| crate::brain::day_applies(&r.days, weekday))
        .filter_map(|r| timing(r, rail, day_start))
        .collect()
}

fn timing(run: &TrainRun, rail: &Rail, day_start: Tick) -> Option<Timing> {
    let arrive = day_start.plus(content::validate::parse_time(&run.arrive)?);
    let depart = match (&run.depart, run.stops) {
        (Some(d), true) => day_start.plus(content::validate::parse_time(d)?),
        _ => arrive,
    };
    let last = u32::try_from(rail.tiles.len().saturating_sub(1)).ok()?;
    let stop = u32::try_from(rail.stop).ok()?;
    let (before, after) = match run.from {
        TrackEnd::Start => (stop, last - stop),
        TrackEnd::End => (last - stop, stop),
    };
    Some(Timing {
        enter: arrive.minus(before * TICKS_PER_TILE),
        arrive,
        depart: depart.max(arrive),
        exit: depart
            .max(arrive)
            .plus((after + TRAIN_TILES) * TICKS_PER_TILE),
        from: run.from,
        stops: run.stops,
    })
}

/// Where the front of a train is at `tick`, in sixteenths of a track tile from the start,
/// or `None` when it is not on the track.
#[must_use]
pub fn head_at(t: &Timing, rail: &Rail, tick: Tick) -> Option<i32> {
    if tick < t.enter || tick >= t.exit {
        return None;
    }
    let stop = i32::try_from(rail.stop).ok()? * 16;
    let per = i32::try_from(TICKS_PER_TILE).ok()?;
    let since = |a: Tick| i32::try_from(tick.0 - a.0).unwrap_or(0) * 16 / per;
    let until = |a: Tick| i32::try_from(a.0 - tick.0).unwrap_or(0) * 16 / per;
    let toward_end = matches!(t.from, TrackEnd::Start);
    let offset = if tick < t.arrive {
        -until(t.arrive)
    } else if tick < t.depart {
        0
    } else {
        since(t.depart)
    };
    Some(if toward_end {
        stop + offset
    } else {
        stop - offset
    })
}
