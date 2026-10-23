//! The committed town must load, validate clean, and render as snapshotted.
use content::{Content, compile, list_doors, list_objects, load_dir, render_place, validate};
use std::path::PathBuf;

fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")
}

fn town() -> Content {
    load_dir(&content_dir()).expect("content loads")
}

#[test]
fn town_validates_with_no_errors_or_warnings() {
    let problems = validate(&town());
    let text: Vec<String> = problems.iter().map(ToString::to_string).collect();
    assert!(problems.is_empty(), "{}", text.join("\n"));
}

#[test]
fn town_renders() {
    let world = compile::compile(&town());
    let mut out = String::new();
    for (i, p) in world.places.iter().enumerate() {
        out.push_str(&render_place(&world, i, &[]));
        out.push_str(&list_objects(&world, i));
        out.push_str("\n\n");
        let _ = p;
    }
    out.push_str(&list_doors(&world));
    out.push('\n');
    insta::assert_snapshot!(out);
}

#[test]
fn a_broken_door_fails_with_a_readable_message() {
    let mut c = town();
    let house = c.buildings.iter_mut().find(|b| b.id == "house").unwrap();
    house.doors[0].to.at = content::Pos { x: 0, y: 0 };
    let problems = validate(&c);
    let text: Vec<String> = problems.iter().map(ToString::to_string).collect();
    assert!(
        text.iter()
            .any(|t| t.contains("door stairs") && t.contains("not walkable")),
        "{}",
        text.join("\n")
    );
}

#[test]
fn a_sealed_room_is_reported() {
    let mut c = town();
    let tavern = c.buildings.iter_mut().find(|b| b.id == "tavern").unwrap();
    tavern.floors[0].openings.clear();
    let problems = validate(&c);
    let text: Vec<String> = problems.iter().map(ToString::to_string).collect();
    assert!(
        text.iter().any(|t| t.contains("sealed off")),
        "{}",
        text.join("\n")
    );
}

#[test]
fn overlapping_footprints_are_reported() {
    let mut c = town();
    let bakery = c.buildings.iter_mut().find(|b| b.id == "bakery").unwrap();
    bakery.at = content::Pos { x: 6, y: 4 };
    let problems = validate(&c);
    let text: Vec<String> = problems.iter().map(ToString::to_string).collect();
    assert!(
        text.iter()
            .any(|t| t.contains("footprint overlaps building")),
        "{}",
        text.join("\n")
    );
}

#[test]
fn times_parse_in_both_forms() {
    use content::validate::parse_time;
    assert_eq!(parse_time("7:15am"), Some(7 * 3600 + 15 * 60));
    assert_eq!(parse_time("7:15 pm"), Some(19 * 3600 + 15 * 60));
    assert_eq!(parse_time("12:00am"), Some(0));
    assert_eq!(parse_time("12pm"), Some(12 * 3600));
    assert_eq!(parse_time("2pm"), Some(14 * 3600));
    assert_eq!(parse_time("19:15"), Some(19 * 3600 + 15 * 60));
    assert_eq!(parse_time("14"), None);
    assert_eq!(parse_time("13pm"), None);
    assert_eq!(parse_time("24:00"), None);
}
