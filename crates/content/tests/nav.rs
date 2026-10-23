use content::{Content, Nav, Pos, Spot, compile, load_dir, nav, render_place};
use std::path::PathBuf;

fn town() -> Content {
    load_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")).unwrap()
}

fn object_access(world: &content::World, id: &str) -> Spot {
    let o = world.objects.iter().find(|o| o.id == id).unwrap();
    let (x, y) = content::validate::neighbours(o.at.x, o.at.y)
        .into_iter()
        .find(|&(x, y)| world.passable(o.place, x, y))
        .unwrap();
    Spot::new(o.place, x, y)
}

#[test]
fn bed_to_bakery_counter_goes_down_the_stairs_and_across_the_square() {
    let world = compile::compile(&town());
    let nav = Nav::build(&world);
    let from = object_access(&world, "bed_mara");
    let to = object_access(&world, "counter_bakery");
    let route = nav.route(&world, from, to).expect("reachable");
    let places: Vec<&str> = route
        .legs
        .iter()
        .map(|l| world.places[l.place].id.as_str())
        .collect();
    assert_eq!(places, vec!["house/upstairs", "town"]);
    assert_eq!(route.legs[0].door, Some(0), "crosses the stairs");
    assert!(
        route.tiles > 20 && route.tiles < 80,
        "tiles = {}",
        route.tiles
    );
    assert_eq!(route.ticks(), route.tiles * nav::WALK_TICKS_PER_TILE);
    let back = nav.route(&world, to, from).unwrap();
    assert_eq!(back.tiles, route.tiles, "symmetric with two-way doors");
}

#[test]
fn route_is_independent_of_door_declaration_order() {
    let mut c = town();
    // Add a second, longer link so there is something to shuffle: a cellar hatch is not
    // in the content yet, so duplicate the stairs as a one-way slower twin.
    let house = c.buildings.iter_mut().find(|b| b.id == "house").unwrap();
    let mut twin = house.doors[0].clone();
    twin.id = "stairs_twin".into();
    twin.from.at = Pos { x: 1, y: 4 };
    twin.to.at = Pos { x: 7, y: 1 };
    house.doors.push(twin);
    let world_a = compile::compile(&c);
    let house = c.buildings.iter_mut().find(|b| b.id == "house").unwrap();
    house.doors.reverse();
    let world_b = compile::compile(&c);
    let nav_a = Nav::build(&world_a);
    let nav_b = Nav::build(&world_b);
    let from_a = object_access(&world_a, "bed_tom");
    let from_b = object_access(&world_b, "bed_tom");
    let to_a = object_access(&world_a, "bar_tavern");
    let to_b = object_access(&world_b, "bar_tavern");
    let ra = nav_a.route(&world_a, from_a, to_a).unwrap();
    let rb = nav_b.route(&world_b, from_b, to_b).unwrap();
    assert_eq!(ra.tiles, rb.tiles);
    let tiles_a: Vec<(usize, Pos, Pos)> = ra.legs.iter().map(|l| (l.place, l.from, l.to)).collect();
    let tiles_b: Vec<(usize, Pos, Pos)> = rb.legs.iter().map(|l| (l.place, l.from, l.to)).collect();
    assert_eq!(
        tiles_a, tiles_b,
        "same legs regardless of declaration order"
    );
}

#[test]
fn one_way_doors_are_not_crossed_backwards() {
    let mut c = town();
    let house = c.buildings.iter_mut().find(|b| b.id == "house").unwrap();
    house.doors[0].one_way = true;
    let world = compile::compile(&c);
    let nav = Nav::build(&world);
    let up = Spot::new(world.place_index["house/upstairs"], 4, 2);
    let down = Spot::new(world.place_index["town"], 7, 7);
    assert!(nav.route(&world, down, up).is_some());
    assert!(nav.route(&world, up, down).is_none());
}

#[test]
fn door_filter_can_close_the_stairs() {
    let world = compile::compile(&town());
    let nav = Nav::build(&world);
    let up = Spot::new(world.place_index["house/upstairs"], 4, 2);
    let down = Spot::new(world.place_index["town"], 7, 7);
    assert!(nav.route_filtered(&world, up, down, |_| false).is_none());
}

#[test]
fn grid_path_walks_adjacent_passable_tiles() {
    let world = compile::compile(&town());
    let town = world.place_index["town"];
    let path = nav::grid_path(&world, town, Pos { x: 2, y: 2 }, Pos { x: 37, y: 27 }).unwrap();
    assert_eq!(path.first(), Some(&Pos { x: 2, y: 2 }));
    assert_eq!(path.last(), Some(&Pos { x: 37, y: 27 }));
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert_eq!(a.x.abs_diff(b.x) + a.y.abs_diff(b.y), 1);
        assert!(world.passable(town, b.x, b.y));
    }
    let overlay: Vec<(Pos, char)> = path.iter().map(|p| (*p, '*')).collect();
    insta::assert_snapshot!(render_place(&world, town, &overlay));
}

#[test]
fn travel_matrix_snapshot() {
    let world = compile::compile(&town());
    let nav = Nav::build(&world);
    insta::assert_snapshot!(nav::travel_matrix(&world, &nav));
}
