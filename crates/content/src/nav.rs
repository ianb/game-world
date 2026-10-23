//! Hierarchical pathfinding over the compiled world: a door graph with door-to-door costs
//! inside each place, then grid search inside the current place. Costs come from the cell
//! kind plus a caller-supplied adjustment (the sim passes wear), so routes prefer paths and
//! trodden ground. Derived data, rebuilt from content, never part of a snapshot.
//!
//! Determinism: neighbours are visited N, E, S, W, successor lists are sorted, and every
//! cost is an integer, so the same inputs always give the same path. Door ends are numbered
//! in declaration order; the shuffle test checks that order does not change the route.
//! A door crossing costs nothing, matching movement, where the crossing happens in the same
//! event as the last step of the leg.
use crate::compile::World;
use crate::model::Pos;
use crate::validate::neighbours;
use pathfinding::directed::astar::astar;
use pathfinding::directed::dijkstra::dijkstra;
use std::collections::{BTreeMap, VecDeque};

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

/// Route-search node: the start spot, a door end, or the goal spot.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Node {
    Start,
    End(usize),
    Goal,
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
    /// For each place, the ends located in it, in end order.
    ends_by_place: Vec<Vec<usize>>,
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
        let mut ends_by_place = vec![Vec::new(); world.places.len()];
        for (ei, (_, spot)) in ends.iter().enumerate() {
            ends_by_place[spot.place].push(ei);
        }
        Nav {
            ends,
            ends_by_place,
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
    #[allow(clippy::too_many_lines)]
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
        let from_dist = cost_distances(world, from, cost);
        let to_dist = cost_distances(world, to, cost);
        // Door-to-door costs within each place, computed for the ends we touch.
        let mut end_dist: BTreeMap<usize, BTreeMap<Pos, u32>> = BTreeMap::new();
        let mut within = |ei: usize, ej: usize| -> Option<u32> {
            let spot_i = self.ends[ei].1;
            let spot_j = self.ends[ej].1;
            if spot_i.place != spot_j.place {
                return None;
            }
            let map = end_dist
                .entry(ei)
                .or_insert_with(|| cost_distances(world, spot_i, cost));
            map.get(&spot_j.pos).copied()
        };

        let mut successors = |n: &Node| -> Vec<(Node, u32)> {
            let mut out = Vec::new();
            match *n {
                Node::Start => {
                    if from.place == to.place
                        && let Some(&d) = from_dist.get(&to.pos)
                    {
                        out.push((Node::Goal, d));
                    }
                    for &ei in &self.ends_by_place[from.place] {
                        if let Some(&d) = from_dist.get(&self.ends[ei].1.pos) {
                            out.push((Node::End(ei), d));
                        }
                    }
                }
                Node::End(ei) => {
                    let (end, spot) = self.ends[ei];
                    // Cross the door to its twin end, if allowed in this direction.
                    let door = &world.doors[end.door];
                    let twin = ei ^ 1;
                    let may_cross = allow(end.door) && (!end.far || !door.one_way);
                    if may_cross {
                        out.push((Node::End(twin), 0));
                    }
                    // Walk to another end in the same place.
                    for &ej in &self.ends_by_place[spot.place] {
                        if ej != ei
                            && let Some(d) = within(ei, ej)
                        {
                            out.push((Node::End(ej), d));
                        }
                    }
                    if spot.place == to.place
                        && let Some(&d) = to_dist.get(&spot.pos)
                    {
                        out.push((Node::Goal, d));
                    }
                }
                Node::Goal => {}
            }
            out.sort();
            out
        };

        let (nodes, total_cost) = dijkstra(&Node::Start, &mut successors, |n| *n == Node::Goal)?;

        // Turn the node sequence into legs. A leg ends at a door end that is then crossed.
        let mut legs = Vec::new();
        let mut cur = from;
        let mut i = 1;
        while i < nodes.len() {
            match nodes[i] {
                Node::End(ei) => {
                    let (end, spot) = self.ends[ei];
                    let next_is_twin =
                        matches!(nodes.get(i + 1), Some(Node::End(ej)) if *ej == ei ^ 1);
                    if next_is_twin {
                        let walked = steps_between(world, cur, spot.pos, cost);
                        legs.push(Leg {
                            place: cur.place,
                            from: cur.pos,
                            to: spot.pos,
                            door: Some(end.door),
                            door_reverse: end.far,
                            tiles: walked,
                        });
                        cur = self.ends[ei ^ 1].1;
                        i += 2;
                    } else {
                        // Walking to an end without crossing it is never shortest unless it is
                        // on the way; treat it as an intermediate waypoint of the same leg.
                        i += 1;
                    }
                }
                Node::Goal => {
                    let walked = steps_between(world, cur, to.pos, cost);
                    legs.push(Leg {
                        place: cur.place,
                        from: cur.pos,
                        to: to.pos,
                        door: None,
                        door_reverse: false,
                        tiles: walked,
                    });
                    i += 1;
                }
                Node::Start => i += 1,
            }
        }
        let tiles = legs.iter().map(|l| l.tiles).sum();
        Some(Route {
            legs,
            tiles,
            cost: total_cost,
        })
    }

    /// Travel time in ticks between two spots, if reachable.
    #[must_use]
    pub fn travel_ticks(&self, world: &World, from: Spot, to: Spot) -> Option<u32> {
        self.route(world, from, to).map(|r| r.ticks())
    }
}

/// Tiles stepped along the cheapest path between two tiles of one place.
fn steps_between(world: &World, from: Spot, to: Pos, cost: CostFn<'_>) -> u32 {
    grid_path_cost(world, from.place, from.pos, to, cost).map_or(0, |p| {
        u32::try_from(p.len().saturating_sub(1)).expect("short path")
    })
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

/// Grid path inside one place with the caller's step cost. Deterministic: fixed neighbour
/// order and integer costs. The heuristic is Manhattan distance, admissible because every
/// step costs at least 1.
#[must_use]
pub fn grid_path_cost(
    world: &World,
    place: usize,
    from: Pos,
    to: Pos,
    cost: CostFn<'_>,
) -> Option<Vec<Pos>> {
    let _ = world;
    let heuristic = |p: &Pos| u32::from(p.x.abs_diff(to.x)) + u32::from(p.y.abs_diff(to.y));
    let successors = |p: &Pos| -> Vec<(Pos, u32)> {
        neighbours(p.x, p.y)
            .into_iter()
            .filter_map(|(x, y)| {
                let n = Pos { x, y };
                cost(place, n).map(|c| (n, c.max(1)))
            })
            .collect()
    };
    astar(&from, successors, heuristic, |p| *p == to).map(|(path, _)| path)
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
