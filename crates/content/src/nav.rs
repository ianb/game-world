//! Pathfinding over the compiled world: one search over every tile of every place, where a
//! door is an edge from one end's tile to the other's. Costs come from the cell kind plus a
//! caller-supplied adjustment (the sim passes wear), so routes prefer paths and trodden
//! ground. Tiles are indexed densely, place after place, so a search is arrays, not maps.
//! Derived data, rebuilt from content, never part of a snapshot.
//!
//! Determinism: neighbours are visited N, E, S, W, the heap breaks ties by tile index, and
//! every cost is an integer, so the same inputs always give the same path. A door crossing
//! costs nothing, matching movement, where the crossing happens in the same event as the
//! last step of the leg.
use crate::compile::World;
use crate::model::Pos;
use crate::validate::neighbours;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

/// Ticks a villager spends crossing one tile. Six tiles per game-minute.
pub const WALK_TICKS_PER_TILE: u32 = 10;

/// A tile in a place.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Spot {
    pub place: usize,
    pub pos: Pos,
}

impl Spot {
    #[must_use]
    pub fn new(place: usize, x: u16, y: u16) -> Spot {
        Spot {
            place,
            pos: Pos { x, y },
        }
    }
}

/// One end of a door record, as a graph node.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct DoorEnd {
    door: usize,
    /// false = `from` end, true = `to` end
    far: bool,
}

/// Cost of stepping onto a tile, for routing. Returns `None` for impassable tiles.
pub type CostFn<'a> = &'a dyn Fn(usize, Pos) -> Option<u32>;

/// The plain cost: the cell's base cost, no wear.
pub fn base_cost(world: &World) -> impl Fn(usize, Pos) -> Option<u32> + '_ {
    move |place, pos| {
        world
            .passable(place, pos.x, pos.y)
            .then(|| world.places[place].cell(pos.x, pos.y).base_cost())
    }
}

/// A walk inside one place, from `from` to `to`, then optionally through a door.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Leg {
    pub place: usize,
    pub from: Pos,
    pub to: Pos,
    /// Door record index crossed at the end of this leg, if any.
    pub door: Option<usize>,
    /// True when the crossing goes from the door's `to` end back to its `from` end.
    pub door_reverse: bool,
    /// Tiles walked in this leg (0 when already there).
    pub tiles: u32,
    /// Every tile of the leg, both ends included.
    pub path: Vec<Pos>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Route {
    pub legs: Vec<Leg>,
    /// Tiles stepped, which sets travel time.
    pub tiles: u32,
    /// Summed step cost, which is what the search minimised.
    pub cost: u32,
}

impl Route {
    #[must_use]
    pub fn ticks(&self) -> u32 {
        self.tiles * WALK_TICKS_PER_TILE
    }
}

#[derive(Clone, Debug)]
pub struct Nav {
    /// Door-end spots: (door index, far) -> spot in that end's place.
    ends: Vec<(DoorEnd, Spot)>,
    /// First tile index of each place in the dense numbering.
    offsets: Vec<usize>,
    /// The door ends on each tile, by dense index; most tiles have none.
    ends_at: BTreeMap<usize, Vec<usize>>,
    has_end: Vec<bool>,
}

impl Nav {
    #[must_use]
    pub fn build(world: &World) -> Nav {
        let mut ends = Vec::new();
        for (i, d) in world.doors.iter().enumerate() {
            ends.push((
                DoorEnd {
                    door: i,
                    far: false,
                },
                Spot {
                    place: d.from_place,
                    pos: d.from,
                },
            ));
            ends.push((
                DoorEnd { door: i, far: true },
                Spot {
                    place: d.to_place,
                    pos: d.to,
                },
            ));
        }
        let offsets = offsets(world);
        let total = offsets.last().copied().unwrap_or(0);
        let mut ends_at: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        let mut has_end = vec![false; total];
        for (ei, (_, spot)) in ends.iter().enumerate() {
            let i = index(world, &offsets, *spot);
            ends_at.entry(i).or_default().push(ei);
            has_end[i] = true;
        }
        Nav {
            ends,
            offsets,
            ends_at,
            has_end,
        }
    }

    /// Cheapest route by base cell cost, or `None` if unreachable. Door conditions (hours,
    /// owner) are not applied here; the sim filters doors before asking for a route.
    #[must_use]
    pub fn route(&self, world: &World, from: Spot, to: Spot) -> Option<Route> {
        let cost = base_cost(world);
        self.route_filtered(world, from, to, |_| true, &cost)
    }

    /// Like `route`, but only doors for which `allow(door index)` holds may be crossed, and
    /// with the caller's step cost.
    pub fn route_filtered(
        &self,
        world: &World,
        from: Spot,
        to: Spot,
        allow: impl Fn(usize) -> bool,
        cost: CostFn<'_>,
    ) -> Option<Route> {
        if !world.passable(from.place, from.pos.x, from.pos.y)
            || !world.passable(to.place, to.pos.x, to.pos.y)
        {
            return None;
        }
        let goal = index(world, &self.offsets, to);
        let found = search(
            world,
            &self.offsets,
            Some(self),
            from,
            &Target::One(to),
            &|i| i == goal,
            &allow,
            cost,
        )?;
        Some(self.route_from(world, &found))
    }

    /// The cheapest of several targets to reach from `from`, with its cost: the one first
    /// reached, and among those reached at the same cost the least by `T`'s order.
    pub fn nearest<T: Copy + Ord>(
        &self,
        world: &World,
        from: Spot,
        targets: &[(Spot, T)],
        allow: impl Fn(usize) -> bool,
        cost: CostFn<'_>,
    ) -> Option<(u32, T)> {
        if !world.passable(from.place, from.pos.x, from.pos.y) {
            return None;
        }
        let mut at: BTreeMap<usize, Vec<T>> = BTreeMap::new();
        for (spot, t) in targets {
            if world.passable(spot.place, spot.pos.x, spot.pos.y) {
                at.entry(index(world, &self.offsets, *spot))
                    .or_default()
                    .push(*t);
            }
        }
        if at.is_empty() {
            return None;
        }
        let found = search(
            world,
            &self.offsets,
            Some(self),
            from,
            &Target::Any,
            &|i| at.contains_key(&i),
            &allow,
            cost,
        )?;
        // Others reached at the same cost compete on their order.
        let best = found
            .ties
            .iter()
            .chain(std::iter::once(&found.goal))
            .filter_map(|i| at.get(i))
            .flatten()
            .min()
            .copied()?;
        Some((found.cost, best))
    }

    /// Splits a found path into legs at its door crossings.
    fn route_from(&self, world: &World, found: &Found) -> Route {
        let mut legs = Vec::new();
        let mut place = found.steps[0].0.place;
        let mut path = vec![found.steps[0].0.pos];
        for w in found.steps.windows(2) {
            let (prev, _) = w[0];
            let (spot, via) = w[1];
            if let Some(ei) = via {
                let end = self.ends[ei].0;
                legs.push(leg(
                    place,
                    std::mem::take(&mut path),
                    Some(end.door),
                    end.far,
                ));
                place = spot.place;
                path = vec![spot.pos];
            } else {
                debug_assert_eq!(prev.place, spot.place);
                path.push(spot.pos);
            }
        }
        legs.push(leg(place, path, None, false));
        let _ = world;
        let tiles = legs.iter().map(|l| l.tiles).sum();
        Route {
            legs,
            tiles,
            cost: found.cost,
        }
    }

    /// Travel time in ticks between two spots, if reachable.
    #[must_use]
    pub fn travel_ticks(&self, world: &World, from: Spot, to: Spot) -> Option<u32> {
        self.route(world, from, to).map(|r| r.ticks())
    }
}

/// Cheapest-path cost from `start` to every reachable tile in its place.
#[must_use]
pub fn cost_distances(_world: &World, start: Spot, cost: CostFn<'_>) -> BTreeMap<Pos, u32> {
    let mut dist = BTreeMap::new();
    if cost(start.place, start.pos).is_none() {
        return dist;
    }
    let mut heap = std::collections::BinaryHeap::new();
    dist.insert(start.pos, 0u32);
    heap.push(std::cmp::Reverse((0u32, start.pos)));
    while let Some(std::cmp::Reverse((d, p))) = heap.pop() {
        if dist.get(&p).is_some_and(|&best| d > best) {
            continue;
        }
        for (x, y) in neighbours(p.x, p.y) {
            let n = Pos { x, y };
            let Some(c) = cost(start.place, n) else {
                continue;
            };
            let nd = d + c;
            if dist.get(&n).is_none_or(|&best| nd < best) {
                dist.insert(n, nd);
                heap.push(std::cmp::Reverse((nd, n)));
            }
        }
    }
    dist
}

/// Tile distances from `start` to every passable tile in its place.
#[must_use]
pub fn bfs_distances(world: &World, start: Spot) -> BTreeMap<Pos, u32> {
    let mut dist = BTreeMap::new();
    if !world.passable(start.place, start.pos.x, start.pos.y) {
        return dist;
    }
    dist.insert(start.pos, 0);
    let mut q = VecDeque::from([start.pos]);
    while let Some(p) = q.pop_front() {
        let d = dist[&p];
        for (x, y) in neighbours(p.x, p.y) {
            let n = Pos { x, y };
            if world.passable(start.place, x, y) && !dist.contains_key(&n) {
                dist.insert(n, d + 1);
                q.push_back(n);
            }
        }
    }
    dist
}

/// Grid path inside one place by base cell cost, tile by tile, including both ends.
#[must_use]
pub fn grid_path(world: &World, place: usize, from: Pos, to: Pos) -> Option<Vec<Pos>> {
    let cost = base_cost(world);
    grid_path_cost(world, place, from, to, &cost)
}

/// Grid path inside one place with the caller's step cost, doors not crossed.
#[must_use]
pub fn grid_path_cost(
    world: &World,
    place: usize,
    from: Pos,
    to: Pos,
    cost: CostFn<'_>,
) -> Option<Vec<Pos>> {
    if !world.passable(place, from.x, from.y) || !world.passable(place, to.x, to.y) {
        return None;
    }
    let offsets = offsets(world);
    let goal_spot = Spot { place, pos: to };
    let goal = index(world, &offsets, goal_spot);
    let found = search(
        world,
        &offsets,
        None,
        Spot { place, pos: from },
        &Target::One(goal_spot),
        &|i| i == goal,
        &|_| false,
        cost,
    )?;
    Some(found.steps.into_iter().map(|(s, _)| s.pos).collect())
}

fn leg(place: usize, path: Vec<Pos>, door: Option<usize>, door_reverse: bool) -> Leg {
    Leg {
        place,
        from: path[0],
        to: path[path.len() - 1],
        door,
        door_reverse,
        tiles: u32::try_from(path.len() - 1).expect("short path"),
        path,
    }
}

/// First dense index of each place, and the total at the end.
fn offsets(world: &World) -> Vec<usize> {
    let mut out = Vec::with_capacity(world.places.len() + 1);
    let mut n = 0;
    for p in &world.places {
        out.push(n);
        n += usize::from(p.width) * usize::from(p.height);
    }
    out.push(n);
    out
}

fn index(world: &World, offsets: &[usize], s: Spot) -> usize {
    offsets[s.place]
        + usize::from(s.pos.y) * usize::from(world.places[s.place].width)
        + usize::from(s.pos.x)
}

fn spot_of(world: &World, offsets: &[usize], i: usize) -> Spot {
    // Few places: the last one starting at or before `i`.
    let place = offsets.partition_point(|&o| o <= i) - 1;
    let w = usize::from(world.places[place].width);
    let r = i - offsets[place];
    Spot {
        place,
        pos: Pos {
            x: u16::try_from(r % w).expect("in place"),
            y: u16::try_from(r / w).expect("in place"),
        },
    }
}

/// What a search is looking for: one spot, which allows a distance estimate within its
/// place, or whichever target comes first.
enum Target {
    One(Spot),
    Any,
}

/// A finished search: the tiles from start to goal, each with the door end it was reached
/// through if it was a crossing, the cost, and other targets reached at the same cost.
struct Found {
    steps: Vec<(Spot, Option<usize>)>,
    cost: u32,
    goal: usize,
    ties: Vec<usize>,
}

/// The tiles from `start` to `goal` by following parents back, each with the door end it
/// was reached through if that step was a crossing.
fn trace(
    world: &World,
    offsets: &[usize],
    start: usize,
    goal: usize,
    parent: &[u32],
    via: &[u32],
) -> Vec<(Spot, Option<usize>)> {
    let mut steps = Vec::new();
    let mut i = goal;
    loop {
        let door = (via[i] != NO_DOOR).then(|| via[i] as usize);
        steps.push((spot_of(world, offsets, i), door));
        if i == start {
            break;
        }
        i = parent[i] as usize;
    }
    steps.reverse();
    steps
}

const UNSEEN: u32 = u32::MAX;
const NO_DOOR: u32 = u32::MAX;

/// A* over every tile, with door crossings as free edges when `nav` is given. The estimate
/// is the Manhattan distance on the target's own place and zero elsewhere; every step costs
/// at least 1, so within a place it never overestimates.
#[allow(clippy::too_many_arguments)]
fn search(
    world: &World,
    offsets: &[usize],
    nav: Option<&Nav>,
    from: Spot,
    target: &Target,
    is_goal: &dyn Fn(usize) -> bool,
    allow: &dyn Fn(usize) -> bool,
    cost: CostFn<'_>,
) -> Option<Found> {
    let total = offsets.last().copied().unwrap_or(0);
    let estimate = |s: Spot| -> u32 {
        match target {
            Target::One(t) if t.place == s.place => {
                u32::from(s.pos.x.abs_diff(t.pos.x)) + u32::from(s.pos.y.abs_diff(t.pos.y))
            }
            _ => 0,
        }
    };
    let mut best = vec![UNSEEN; total];
    let mut parent = vec![u32::MAX; total];
    let mut via = vec![NO_DOOR; total];
    let mut done = vec![false; total];
    let mut heap = BinaryHeap::new();
    let start = index(world, offsets, from);
    best[start] = 0;
    heap.push(Reverse((estimate(from), start)));
    let mut goal: Option<(usize, u32)> = None;
    let mut ties = Vec::new();
    while let Some(Reverse((f, i))) = heap.pop() {
        if done[i] {
            continue;
        }
        let g = best[i];
        if let Some((_, gc)) = goal {
            // Keep popping only for other targets at the same cost.
            if f > gc {
                break;
            }
            if is_goal(i) && g == gc {
                ties.push(i);
            }
            done[i] = true;
            continue;
        }
        done[i] = true;
        if is_goal(i) {
            goal = Some((i, g));
            if matches!(target, Target::One(_)) {
                break;
            }
            continue;
        }
        let here = spot_of(world, offsets, i);
        let mut relax = |j: usize, nd: u32, door: u32, s: Spot, heap: &mut BinaryHeap<_>| {
            if !done[j] && nd < best[j] {
                best[j] = nd;
                parent[j] = u32::try_from(i).expect("few tiles");
                via[j] = door;
                heap.push(Reverse((nd + estimate(s), j)));
            }
        };
        let place = &world.places[here.place];
        for (x, y) in neighbours(here.pos.x, here.pos.y) {
            if !place.in_bounds(x, y) {
                continue;
            }
            let n = Pos { x, y };
            let Some(c) = cost(here.place, n) else {
                continue;
            };
            let s = Spot {
                place: here.place,
                pos: n,
            };
            relax(
                index(world, offsets, s),
                g + c.max(1),
                NO_DOOR,
                s,
                &mut heap,
            );
        }
        if let Some(nav) = nav
            && nav.has_end[i]
        {
            for &ei in &nav.ends_at[&i] {
                let (end, _) = nav.ends[ei];
                let door = &world.doors[end.door];
                if !allow(end.door) || (end.far && door.one_way) {
                    continue;
                }
                let twin = nav.ends[ei ^ 1].1;
                let j = index(world, offsets, twin);
                relax(j, g, u32::try_from(ei).expect("few doors"), twin, &mut heap);
            }
        }
    }
    let (goal, cost) = goal?;
    let steps = trace(world, offsets, start, goal, &parent, &via);
    Some(Found {
        steps,
        cost,
        goal,
        ties,
    })
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.y, self.x).cmp(&(other.y, other.x))
    }
}

impl std::hash::Hash for Pos {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

/// Travel-time matrix between every place's arrival tile, as text.
#[must_use]
pub fn travel_matrix(world: &World, nav: &Nav) -> String {
    use std::fmt::Write as _;
    let spots: Vec<(String, Spot)> = world
        .places
        .iter()
        .enumerate()
        .map(|(i, p)| {
            (
                p.id.clone(),
                Spot {
                    place: i,
                    pos: p.arrival,
                },
            )
        })
        .collect();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "travel time in game-minutes between arrival tiles ({WALK_TICKS_PER_TILE} ticks per tile)"
    );
    let width = spots
        .iter()
        .map(|(id, _)| id.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let _ = write!(out, "{:width$}", "");
    for (id, _) in &spots {
        let _ = write!(out, " {id:>width$}");
    }
    out.push('\n');
    for (id, a) in &spots {
        let _ = write!(out, "{id:width$}");
        for (_, b) in &spots {
            match nav.travel_ticks(world, *a, *b) {
                Some(t) => {
                    let _ = write!(out, " {:>width$}", format!("{}m{:02}", t / 60, t % 60));
                }
                None => {
                    let _ = write!(out, " {:>width$}", "-");
                }
            }
        }
        out.push('\n');
    }
    out
}
