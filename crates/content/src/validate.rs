//! Referential and geometric checks from the validator table in `docs/design/world-layout.md`.
//! Every error names the thing and the field, because the agent fixes from this output.
use crate::compile::{Cell, World, compile};
use crate::model::{Building, Content, Floor, Pos, Rect};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ContentError {
    pub severity: Severity,
    pub subject: String,
    pub message: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Severity {
    Warning,
    Error,
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = match self.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(f, "{sev}: {}: {}", self.subject, self.message)
    }
}

struct Report {
    errors: Vec<ContentError>,
}

impl Report {
    fn error(&mut self, subject: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ContentError {
            severity: Severity::Error,
            subject: subject.into(),
            message: message.into(),
        });
    }
    fn warn(&mut self, subject: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ContentError {
            severity: Severity::Warning,
            subject: subject.into(),
            message: message.into(),
        });
    }
}

/// Validates content. Returns every problem found; the caller decides whether warnings fail.
#[must_use]
pub fn validate(content: &Content) -> Vec<ContentError> {
    let mut r = Report { errors: Vec::new() };
    check_ids(content, &mut r);
    check_buildings(content, &mut r);
    if r.errors.iter().any(|e| e.severity == Severity::Error) {
        // Compilation assumes references resolve; stop here with the referential errors.
        return r.errors;
    }
    let world = compile(content);
    check_openings(&world, &mut r);
    check_doors(&world, &mut r);
    check_reachability(&world, &mut r);
    check_objects(&world, &mut r);
    check_arrivals(&world, &mut r);
    check_roofs(content, &world, &mut r);
    check_villagers(content, &world, &mut r);
    r.errors.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.subject.cmp(&b.subject))
    });
    r.errors
}

fn check_ids(content: &Content, r: &mut Report) {
    let mut seen = BTreeSet::new();
    for z in &content.zones {
        if !seen.insert(z.id.clone()) {
            r.error(format!("zones/{}", z.id), "duplicate zone id");
        }
    }
    let mut seen = BTreeSet::new();
    for b in &content.buildings {
        if !seen.insert(b.id.clone()) {
            r.error(format!("buildings/{}", b.id), "duplicate building id");
        }
    }
    let mut seen = BTreeSet::new();
    for v in &content.villagers {
        if !seen.insert(v.id.clone()) {
            r.error(format!("villagers/{}", v.id), "duplicate villager id");
        }
    }
    let mut objects = BTreeSet::new();
    for b in &content.buildings {
        for f in &b.floors {
            for o in &f.objects {
                if !objects.insert(o.id.clone()) {
                    r.error(
                        format!("buildings/{}: object {}", b.id, o.id),
                        "duplicate object id",
                    );
                }
            }
        }
    }
    for z in &content.zones {
        for o in &z.objects {
            if !objects.insert(o.id.clone()) {
                r.error(
                    format!("zones/{}: object {}", z.id, o.id),
                    "duplicate object id",
                );
            }
            if !o.room.is_empty() {
                r.error(
                    format!("zones/{}: object {}", z.id, o.id),
                    "a zone object has no room",
                );
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn check_buildings(content: &Content, r: &mut Report) {
    let zones: BTreeMap<&str, &crate::model::Zone> =
        content.zones.iter().map(|z| (z.id.as_str(), z)).collect();
    let mut footprints: Vec<(String, String, Rect)> = Vec::new();

    for b in &content.buildings {
        let subj = |s: &str| format!("buildings/{}: {s}", b.id);
        let Some(zone) = zones.get(b.zone.as_str()) else {
            r.error(subj("zone"), format!("zone {:?} not found", b.zone));
            continue;
        };
        let mut floor_ids = BTreeSet::new();
        let mut ground_floors = 0;
        for f in &b.floors {
            if !floor_ids.insert(f.id.clone()) {
                r.error(subj(&format!("floor {}", f.id)), "duplicate floor id");
            }
            if f.level == 0 && f.detached.is_none() {
                ground_floors += 1;
            }
            if f.level == 0 && f.detached.as_deref().is_some_and(str::is_empty) {
                r.warn(
                    subj(&format!("floor {}", f.id)),
                    "detached ground floor without a stated reason",
                );
            }
            if f.level != 0 && f.detached.is_none() {
                r.warn(
                    subj(&format!("floor {}", f.id)),
                    "non-ground floor is detached by level; state the reason in `detached`",
                );
            }
            check_floor(b, f, r);
            if f.level == 0
                && f.detached.is_none()
                && let Some(bbox) = crate::compile::bbox_with_walls(f)
            {
                let rect = Rect {
                    x: bbox.x + b.at.x,
                    y: bbox.y + b.at.y,
                    w: bbox.w,
                    h: bbox.h,
                };
                if rect.right() > zone.width || rect.bottom() > zone.height {
                    r.error(
                        subj(&format!("floor {}", f.id)),
                        format!(
                            "footprint {rect:?} exceeds zone {}x{}",
                            zone.width, zone.height
                        ),
                    );
                }
                for z in zone.blocked.iter().chain(&zone.water) {
                    if z.overlaps(&rect) {
                        r.error(
                            subj(&format!("floor {}", f.id)),
                            format!("footprint overlaps blocked terrain {z:?}"),
                        );
                    }
                }
                footprints.push((b.id.clone(), b.zone.clone(), rect));
            }
        }
        if ground_floors > 1 {
            r.error(subj("floors"), "more than one inline ground floor");
        }
        if ground_floors == 0 && b.floors.iter().all(|f| f.detached.is_none()) {
            r.warn(subj("floors"), "no ground floor");
        }
        for d in &b.doors {
            for (label, end) in [("from", &d.from), ("to", &d.to)] {
                match (&end.floor, &end.zone) {
                    (Some(f), None) => {
                        if !b.floors.iter().any(|fl| &fl.id == f) {
                            r.error(
                                subj(&format!("door {} {label}", d.id)),
                                format!("floor {f:?} not found in this building"),
                            );
                        }
                    }
                    (None, Some(z)) => {
                        if !zones.contains_key(z.as_str()) {
                            r.error(
                                subj(&format!("door {} {label}", d.id)),
                                format!("zone {z:?} not found"),
                            );
                        }
                    }
                    _ => r.error(
                        subj(&format!("door {} {label}", d.id)),
                        "set exactly one of `floor` or `zone`",
                    ),
                }
            }
        }
    }

    // Footprints include walls. Two buildings may share a wall line, so the test is whether
    // either footprint reaches into the other's interior (its footprint minus the walls).
    let interior = |f: &Rect| Rect {
        x: f.x + 1,
        y: f.y + 1,
        w: f.w.saturating_sub(2),
        h: f.h.saturating_sub(2),
    };
    for (i, (a_id, a_zone, a)) in footprints.iter().enumerate() {
        for (b_id, b_zone, b) in footprints.iter().skip(i + 1) {
            if a_zone == b_zone && (a.overlaps(&interior(b)) || b.overlaps(&interior(a))) {
                r.error(
                    format!("buildings/{a_id}"),
                    format!("footprint overlaps building {b_id}"),
                );
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn check_floor(b: &Building, f: &Floor, r: &mut Report) {
    let subj = |s: &str| format!("buildings/{}: floor {}: {s}", b.id, f.id);
    let mut room_ids = BTreeSet::new();
    for room in &f.rooms {
        if !room_ids.insert(room.id.clone()) {
            r.error(subj(&format!("room {}", room.id)), "duplicate room id");
        }
        if room.rect.w == 0 || room.rect.h == 0 {
            r.error(subj(&format!("room {}", room.id)), "empty rect");
        }
        if room.rect.x == 0 || room.rect.y == 0 {
            r.error(
                subj(&format!("room {}", room.id)),
                "rooms need x >= 1 and y >= 1 to leave space for walls",
            );
        }
    }
    for (i, a) in f.rooms.iter().enumerate() {
        for bb in f.rooms.iter().skip(i + 1) {
            if a.rect.overlaps(&bb.rect) {
                r.error(
                    subj(&format!("room {}", a.id)),
                    format!("overlaps room {}", bb.id),
                );
            } else if a.rect.with_walls().overlaps(&bb.rect) {
                r.error(
                    subj(&format!("room {}", a.id)),
                    format!(
                        "touches room {}; leave one wall tile between rooms, or make them one room",
                        bb.id
                    ),
                );
            }
        }
    }
    for o in &f.openings {
        if o.between.is_empty() || o.between.len() > 2 {
            r.error(
                subj(&format!("opening at {:?}", o.at)),
                "`between` names one room (to outside) or two rooms",
            );
        }
        for id in &o.between {
            if !room_ids.contains(id) {
                r.error(
                    subj(&format!("opening at {:?}", o.at)),
                    format!("room {id:?} not found"),
                );
            }
        }
        // The opening must be a wall tile of every room it names, and not a corner.
        for id in &o.between {
            if let Some(room) = f.rooms.iter().find(|rm| &rm.id == id) {
                let walls = room.rect.with_walls();
                let on_wall = walls.contains(o.at.x, o.at.y) && !room.rect.contains(o.at.x, o.at.y);
                if !on_wall {
                    r.error(
                        subj(&format!("opening at {:?}", o.at)),
                        format!("not on a wall of room {id}"),
                    );
                    continue;
                }
                let corner_x = o.at.x == walls.x || o.at.x + 1 == walls.right();
                let corner_y = o.at.y == walls.y || o.at.y + 1 == walls.bottom();
                if corner_x && corner_y {
                    r.error(
                        subj(&format!("opening at {:?}", o.at)),
                        format!("on a corner of room {id}; nothing can pass"),
                    );
                }
            }
        }
    }
    let mut room_count: BTreeMap<&str, usize> = BTreeMap::new();
    for o in &f.openings {
        for id in &o.between {
            *room_count.entry(id.as_str()).or_default() += 1;
        }
    }
    for o in &f.objects {
        match f.rooms.iter().find(|rm| rm.id == o.room) {
            None => r.error(
                subj(&format!("object {}", o.id)),
                format!("room {:?} not found", o.room),
            ),
            Some(room) => {
                if !room.rect.contains(o.at.x, o.at.y) {
                    r.error(
                        subj(&format!("object {}", o.id)),
                        format!("at {:?} is outside room {}", o.at, room.id),
                    );
                }
            }
        }
    }
    if !f
        .rooms
        .iter()
        .any(|rm| rm.rect.contains(f.arrival.x, f.arrival.y))
    {
        r.error(
            subj("arrival"),
            format!("{:?} is not inside a room", f.arrival),
        );
    }
}

/// Every opening must have passable tiles on two opposite sides (it is a gap in a wall).
fn check_openings(world: &World, r: &mut Report) {
    for (pi, p) in world.places.iter().enumerate() {
        for y in 0..p.height {
            for x in 0..p.width {
                if p.cell(x, y) != Cell::Opening {
                    continue;
                }
                let north = world.passable(pi, x, y.wrapping_sub(1));
                let south = world.passable(pi, x, y + 1);
                let west = world.passable(pi, x.wrapping_sub(1), y);
                let east = world.passable(pi, x + 1, y);
                if !((north && south) || (west && east)) {
                    r.error(
                        format!("place {}", p.id),
                        format!("opening at ({x}, {y}) does not connect two passable tiles"),
                    );
                }
            }
        }
    }
}

fn check_doors(world: &World, r: &mut Report) {
    for d in &world.doors {
        let subj = format!("buildings/{}: door {}", d.building, d.id);
        for (label, place, at) in [("from", d.from_place, d.from), ("to", d.to_place, d.to)] {
            let p = &world.places[place];
            if !p.in_bounds(at.x, at.y) {
                r.error(&subj, format!("{label} {at:?} is outside place {}", p.id));
                continue;
            }
            if !p.walkable(at.x, at.y) {
                r.error(
                    &subj,
                    format!(
                        "{label} {at:?} in {} is not walkable ({:?})",
                        p.id,
                        p.cell(at.x, at.y)
                    ),
                );
            }
            if world.blocked_by_object(place, at.x, at.y) {
                r.error(
                    &subj,
                    format!("{label} {at:?} in {} is covered by an object", p.id),
                );
            }
        }
        if d.from_place == d.to_place && d.from == d.to {
            r.error(&subj, "both ends are the same tile");
        }
        if let Some(h) = &d.hours
            && parse_hours(h).is_none()
        {
            r.error(&subj, format!("hours {h:?} is not HH:MM-HH:MM"));
        }
    }
}

/// Parses a time of day into seconds: `7:15am`, `7:15 PM`, `12:00am` (midnight), or the
/// 24-hour `19:15`.
#[must_use]
pub fn parse_time(text: &str) -> Option<u32> {
    let t = text.trim().to_ascii_lowercase();
    let (digits, suffix) = if let Some(d) = t.strip_suffix("am") {
        (d.trim(), Some(false))
    } else if let Some(d) = t.strip_suffix("pm") {
        (d.trim(), Some(true))
    } else {
        (t.as_str(), None)
    };
    // "2pm" is allowed; "14" without a suffix is not, since it reads as a bare number.
    let (hours, minutes) = match digits.split_once(':') {
        Some((h, m)) => (h, m),
        None if suffix.is_some() => (digits, "0"),
        None => return None,
    };
    let hours: u32 = hours.parse().ok()?;
    let minutes: u32 = minutes.parse().ok()?;
    let hours = match suffix {
        None => (hours < 24).then_some(hours)?,
        Some(pm) => {
            if !(1..=12).contains(&hours) {
                return None;
            }
            (hours % 12) + if pm { 12 } else { 0 }
        }
    };
    (minutes < 60).then_some(hours * 3600 + minutes * 60)
}

/// Parses "open-close" (either time form) into seconds of day.
#[must_use]
pub fn parse_hours(spec: &str) -> Option<(u32, u32)> {
    let (open, close) = spec.split_once('-')?;
    Some((parse_time(open)?, parse_time(close)?))
}

/// Flood fill of passable tiles from a start, four-connected.
#[must_use]
pub fn flood(world: &World, place: usize, start: Pos) -> BTreeSet<(u16, u16)> {
    let mut seen = BTreeSet::new();
    if !world.passable(place, start.x, start.y) {
        return seen;
    }
    let mut q = VecDeque::from([(start.x, start.y)]);
    seen.insert((start.x, start.y));
    while let Some((x, y)) = q.pop_front() {
        for (nx, ny) in neighbours(x, y) {
            if world.passable(place, nx, ny) && seen.insert((nx, ny)) {
                q.push_back((nx, ny));
            }
        }
    }
    seen
}

/// N, E, S, W in a fixed order, so every search is deterministic.
#[must_use]
pub fn neighbours(x: u16, y: u16) -> [(u16, u16); 4] {
    [
        (x, y.wrapping_sub(1)),
        (x.wrapping_add(1), y),
        (x, y.wrapping_add(1)),
        (x.wrapping_sub(1), y),
    ]
}

fn check_reachability(world: &World, r: &mut Report) {
    // Place graph: every place must be reachable from place 0 (the first zone) via doors.
    let mut reach = BTreeSet::from([0usize]);
    let mut q = VecDeque::from([0usize]);
    while let Some(p) = q.pop_front() {
        for d in &world.doors {
            let next = if d.from_place == p {
                Some(d.to_place)
            } else if d.to_place == p && !d.one_way {
                Some(d.from_place)
            } else {
                None
            };
            if let Some(n) = next
                && reach.insert(n)
            {
                q.push_back(n);
            }
        }
    }
    for (i, p) in world.places.iter().enumerate() {
        if !reach.contains(&i) {
            r.error(
                format!("place {}", p.id),
                "not reachable from the first zone through any door",
            );
        }
    }

    // Within each place: every passable component must contain a door tile, the arrival
    // tile, or (for a zone) be the main component.
    for (pi, p) in world.places.iter().enumerate() {
        let mut unseen: BTreeSet<(u16, u16)> = (0..p.height)
            .flat_map(|y| (0..p.width).map(move |x| (x, y)))
            .filter(|&(x, y)| world.passable(pi, x, y))
            .collect();
        let anchors: BTreeSet<(u16, u16)> = world
            .doors
            .iter()
            .flat_map(|d| {
                let mut v = Vec::new();
                if d.from_place == pi {
                    v.push((d.from.x, d.from.y));
                }
                if d.to_place == pi {
                    v.push((d.to.x, d.to.y));
                }
                v
            })
            .chain(std::iter::once((p.arrival.x, p.arrival.y)))
            .collect();
        let mut component_no = 0;
        while let Some(&(x, y)) = unseen.iter().next() {
            let comp = flood(world, pi, Pos { x, y });
            component_no += 1;
            let anchored = comp.iter().any(|t| anchors.contains(t));
            if !anchored {
                let (ax, ay) = *comp.iter().next().expect("non-empty");
                r.error(
                    format!("place {}", p.id),
                    format!("walkable region {component_no} of {} tiles around ({ax}, {ay}) has no door or arrival; it is sealed off", comp.len()),
                );
            }
            for t in &comp {
                unseen.remove(t);
            }
        }
        // Every door tile in this place must reach the arrival tile.
        let main = flood(world, pi, p.arrival);
        for a in &anchors {
            if !main.contains(a) && world.passable(pi, a.0, a.1) {
                r.error(
                    format!("place {}", p.id),
                    format!(
                        "door tile {a:?} cannot reach the arrival tile {:?}",
                        p.arrival
                    ),
                );
            }
        }
    }
}

fn check_objects(world: &World, r: &mut Report) {
    for o in &world.objects {
        let subj = format!("buildings/{}: object {}", o.building, o.id);
        let p = &world.places[o.place];
        let cell = p.cell(o.at.x, o.at.y);
        // A zone object (no room) stands outdoors; a building object stands on its floor.
        let ok = if o.room.is_empty() {
            matches!(cell, Cell::Ground | Cell::Path)
        } else {
            cell == Cell::Floor
        };
        if !ok {
            r.error(
                &subj,
                format!(
                    "at {:?} is not somewhere an object can stand ({cell:?})",
                    o.at
                ),
            );
            continue;
        }
        if !o.walkable {
            let access = neighbours(o.at.x, o.at.y)
                .into_iter()
                .any(|(x, y)| world.passable(o.place, x, y));
            if !access {
                r.error(&subj, "no walkable tile next to it; nobody can use it");
            }
            let main = flood(world, o.place, p.arrival);
            let reachable = neighbours(o.at.x, o.at.y)
                .into_iter()
                .any(|t| main.contains(&t));
            if access && !reachable {
                r.error(
                    &subj,
                    "its access tiles cannot be reached from the arrival tile",
                );
            }
        }
    }
    let mut owners: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for o in world.objects.iter().filter(|o| o.kind == "bed") {
        if let Some(owner) = &o.owner {
            owners
                .entry(owner.as_str())
                .or_default()
                .push(o.id.as_str());
        }
    }
    for (owner, beds) in owners {
        if beds.len() > 1 {
            r.warn(
                format!("villager {owner}"),
                format!("owns more than one bed: {beds:?}"),
            );
        }
    }
}

fn check_arrivals(world: &World, r: &mut Report) {
    for (pi, p) in world.places.iter().enumerate() {
        if !world.passable(pi, p.arrival.x, p.arrival.y) {
            r.error(
                format!("place {}", p.id),
                format!("arrival {:?} is not passable", p.arrival),
            );
        }
        if p.cell(p.arrival.x, p.arrival.y) == Cell::Door {
            r.error(
                format!("place {}", p.id),
                format!("arrival {:?} is on a door tile", p.arrival),
            );
        }
    }
}

fn check_roofs(content: &Content, world: &World, r: &mut Report) {
    for roof in &world.roofs {
        let covered = world
            .rooms
            .iter()
            .filter(|rm| rm.building == roof.building && rm.place == roof.place)
            .all(|rm| {
                rm.rect.x >= roof.rect.x
                    && rm.rect.y >= roof.rect.y
                    && rm.rect.right() <= roof.rect.right()
                    && rm.rect.bottom() <= roof.rect.bottom()
            });
        if !covered {
            r.error(
                format!("buildings/{}", roof.building),
                "roof does not cover every ground-floor room",
            );
        }
    }
    let _ = content;
}

fn check_villagers(content: &Content, world: &World, r: &mut Report) {
    for v in &content.villagers {
        let subj = format!("villagers/{}", v.id);
        if !content.buildings.iter().any(|b| b.id == v.home) {
            r.error(&subj, format!("home {:?} not found", v.home));
        }
        match world.objects.iter().find(|o| o.id == v.bed) {
            None => r.error(&subj, format!("bed {:?} not found", v.bed)),
            Some(bed) => {
                if bed.kind != "bed" {
                    r.error(&subj, format!("bed {:?} is a {}", v.bed, bed.kind));
                }
                if bed.owner.as_deref() != Some(v.id.as_str()) {
                    r.error(
                        &subj,
                        format!(
                            "bed {:?} is owned by {:?}, not this villager",
                            v.bed, bed.owner
                        ),
                    );
                }
                if bed.building != v.home {
                    r.warn(
                        &subj,
                        format!(
                            "bed {:?} is in {}, not in home {}",
                            v.bed, bed.building, v.home
                        ),
                    );
                }
            }
        }
    }
    for v in &content.villagers {
        let subj = format!("villagers/{}", v.id);
        for (i, e) in v.schedule.iter().enumerate() {
            let es = format!("{subj}: schedule[{i}]");
            if parse_time(&e.at).is_none() {
                r.error(
                    &es,
                    format!("at {:?} is not a time like 7:15am or 19:15", e.at),
                );
            }
            match (&e.target.object, &e.target.place) {
                (Some(o), None) => {
                    if !world.objects.iter().any(|x| &x.id == o) {
                        r.error(&es, format!("object {o:?} not found"));
                    }
                }
                (None, Some(p)) => {
                    if !world.place_index.contains_key(p) {
                        r.error(&es, format!("place {p:?} not found"));
                    }
                }
                _ => r.error(&es, "set exactly one of `object` or `place`"),
            }
        }
    }
    for o in world.objects.iter().filter(|o| o.kind == "bed") {
        if let Some(owner) = &o.owner
            && !content.villagers.iter().any(|v| &v.id == owner)
        {
            {
                r.warn(
                    format!("buildings/{}: object {}", o.building, o.id),
                    format!("owner {owner:?} is not a villager"),
                );
            }
        }
    }
}
