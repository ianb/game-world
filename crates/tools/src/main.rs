//! `sim-tools`: validators, text views, record and replay. The agent's eyes.
mod paint;
mod png;
use characters::Town;
use clap::{Parser, Subcommand};
use content::{
    Nav, Pos, Spot, compile, list_doors, list_objects, load_dir, nav, render_place, validate,
};
use sim::time::{TICKS_PER_DAY, Tick};
use sim::{Sim, views};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "sim-tools",
    about = "Validators, text views, record and replay"
)]
struct Cli {
    /// Content directory.
    #[arg(long, default_value = "content")]
    content: PathBuf,
    /// Colour the map views even when not writing to a terminal.
    #[arg(long)]
    color: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Load and validate the content directory. Exit 1 on any error.
    Validate,
    /// Render one place (or all) as ASCII, with its objects.
    Render {
        /// Place id, such as `town` or `house/upstairs`. All places if omitted.
        place: Option<String>,
    },
    /// List every door record.
    Doors,
    /// Print the whole content directory as one JSON document (what the wasm facade takes).
    Bundle,
    /// Run one day from a seed and print the emitted events as JSON, plus the hash. The
    /// committed fixture under fixtures/ is made with this and checked by Rust and vitest.
    Events {
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 0)]
        day: u32,
    },
    /// Travel times between every place's arrival tile.
    TravelMatrix,
    /// Write a PNG of every place at a given day and time, villagers drawn in.
    Png {
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 0)]
        day: u32,
        /// Time of day, like "2pm", "7:15am" or "19:15".
        #[arg(long, default_value = "12:00")]
        at: String,
        #[arg(long, default_value = "target/town.png")]
        out: PathBuf,
    },
    /// Money, needs and goods at a given day and time.
    Economy {
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 0)]
        day: u32,
        /// Time of day, like "2pm", "7:15am" or "19:15".
        #[arg(long, default_value = "12:00")]
        at: String,
    },
    /// Where everyone is at a given day and time, with the places drawn.
    Where {
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 0)]
        day: u32,
        /// Time of day, like "2pm", "7:15am" or "19:15".
        #[arg(long, default_value = "12:00")]
        at: String,
    },
    /// Run the town for a day with the baseline characters and print every villager's
    /// itinerary, plus where everyone is at the given hours.
    Day {
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Day index to run (0 is the first day).
        #[arg(long, default_value_t = 0)]
        day: u32,
        /// Hours at which to print the town, e.g. 9 14 20.
        #[arg(long, value_delimiter = ',', default_values_t = vec![9u32, 14, 20])]
        snapshots: Vec<u32>,
    },
    /// Route between two tiles, printed as legs and drawn on the places it crosses.
    Route {
        from_place: String,
        from_x: u16,
        from_y: u16,
        to_place: String,
        to_x: u16,
        to_y: u16,
    },
}

#[allow(clippy::too_many_lines)]
fn main() -> ExitCode {
    let cli = Cli::parse();
    let colour = paint::wanted(cli.color);
    let show = |text: String| if colour { paint::paint(&text) } else { text };
    let content = match load_dir(&cli.content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    if !matches!(cli.command, Command::Validate) {
        let errors: Vec<String> = validate(&content)
            .into_iter()
            .filter(|p| p.severity == content::validate::Severity::Error)
            .map(|p| p.to_string())
            .collect();
        if !errors.is_empty() {
            eprintln!("content has errors; run `validate`:\n{}", errors.join("\n"));
            return ExitCode::from(2);
        }
    }
    match cli.command {
        Command::Validate => {
            let problems = validate(&content);
            for p in &problems {
                println!("{p}");
            }
            let errors = problems
                .iter()
                .filter(|p| p.severity == content::validate::Severity::Error)
                .count();
            println!(
                "{} zone(s), {} building(s), {} villager(s): {} error(s), {} warning(s)",
                content.zones.len(),
                content.buildings.len(),
                content.villagers.len(),
                errors,
                problems.len() - errors
            );
            if errors > 0 {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Command::Render { place } => {
            let world = compile::compile(&content);
            for (i, p) in world.places.iter().enumerate() {
                if place.as_ref().is_some_and(|want| want != &p.id) {
                    continue;
                }
                print!("{}", show(render_place(&world, i, &[])));
                println!("{}\n", list_objects(&world, i));
            }
            ExitCode::SUCCESS
        }
        Command::Doors => {
            let world = compile::compile(&content);
            println!("{}", list_doors(&world));
            ExitCode::SUCCESS
        }
        Command::Day {
            seed,
            day,
            snapshots,
        } => {
            let mut sim: Sim<Town> = Sim::new(seed, content);
            let start = Tick(day * TICKS_PER_DAY);
            let mut events = sim.step_until(start).unwrap();
            events.retain(|e| e.tick() >= start);
            let mut hours: Vec<u32> = snapshots;
            hours.sort_unstable();
            for h in hours {
                events.extend(sim.step_until(start.at_time(h.min(23), 0)).unwrap());
                print!("{}", views::where_is_everyone(&sim));
                for i in 0..sim.world().places.len() {
                    print!("{}", show(views::render_place_now(&sim, i)));
                }
                println!();
            }
            events.extend(sim.step_until(start.plus(TICKS_PER_DAY)).unwrap());
            for i in 0..sim.state().villagers.len() {
                let who = sim::ids::VillagerId(u16::try_from(i).unwrap());
                print!("{}", views::itinerary(&sim, who, &events));
                print!("{}", views::place_changes(&sim, who, &events));
            }
            println!("hash {}", sim.hash().hex());
            ExitCode::SUCCESS
        }
        Command::Bundle => {
            println!(
                "{}",
                serde_json::to_string(&content).expect("content serializes")
            );
            ExitCode::SUCCESS
        }
        Command::Events { seed, day } => {
            let mut sim: Sim<Town> = Sim::new(seed, content);
            let start = Tick(day * TICKS_PER_DAY);
            let mut events = sim.step_until(start).unwrap();
            events.retain(|e| e.tick() >= start);
            events.extend(sim.step_until(start.plus(TICKS_PER_DAY)).unwrap());
            let fixture = serde_json::json!({
                "seed": seed,
                "day": day,
                "protocol_version": sim::protocol::PROTOCOL_VERSION,
                "hash": sim.hash().hex(),
                "events": events,
            });
            println!("{}", serde_json::to_string_pretty(&fixture).unwrap());
            ExitCode::SUCCESS
        }
        Command::Png { seed, day, at, out } => {
            let Some(secs) = content::validate::parse_time(&at) else {
                eprintln!("--at must be a time of day, like 2pm, 7:15am or 19:15");
                return ExitCode::from(1);
            };
            let mut sim: Sim<Town> = Sim::new(seed, content);
            sim.step_until(Tick(day * TICKS_PER_DAY).plus(secs))
                .unwrap();
            let img = png::render_town(&sim);
            if let Err(e) = img.save(&out) {
                eprintln!("could not write {}: {e}", out.display());
                return ExitCode::from(1);
            }
            println!("wrote {} ({}x{})", out.display(), img.width(), img.height());
            print!("{}", views::where_is_everyone(&sim));
            ExitCode::SUCCESS
        }
        Command::Economy { seed, day, at } => {
            let Some(secs) = content::validate::parse_time(&at) else {
                eprintln!("--at must be a time of day, like 2pm, 7:15am or 19:15");
                return ExitCode::from(1);
            };
            let mut sim: Sim<Town> = Sim::new(seed, content);
            sim.step_until(Tick(day * TICKS_PER_DAY).plus(secs))
                .unwrap();
            print!("{}", views::economy(&sim));
            ExitCode::SUCCESS
        }
        Command::Where { seed, day, at } => {
            let Some(secs) = content::validate::parse_time(&at) else {
                eprintln!("--at must be a time of day, like 2pm, 7:15am or 19:15");
                return ExitCode::from(1);
            };
            let mut sim: Sim<Town> = Sim::new(seed, content);
            sim.step_until(Tick(day * TICKS_PER_DAY).plus(secs))
                .unwrap();
            print!("{}", views::where_is_everyone(&sim));
            for i in 0..sim.world().places.len() {
                print!("{}", show(views::render_place_now(&sim, i)));
            }
            ExitCode::SUCCESS
        }
        Command::TravelMatrix => {
            let world = compile::compile(&content);
            let nav = Nav::build(&world);
            print!("{}", nav::travel_matrix(&world, &nav));
            ExitCode::SUCCESS
        }
        Command::Route {
            from_place,
            from_x,
            from_y,
            to_place,
            to_x,
            to_y,
        } => {
            let world = compile::compile(&content);
            let nav = Nav::build(&world);
            let (Some(&fp), Some(&tp)) = (
                world.place_index.get(&from_place),
                world.place_index.get(&to_place),
            ) else {
                eprintln!("unknown place");
                return ExitCode::from(1);
            };
            let from = Spot::new(fp, from_x, from_y);
            let to = Spot::new(tp, to_x, to_y);
            let Some(route) = nav.route(&world, from, to) else {
                println!("unreachable");
                return ExitCode::from(1);
            };
            println!(
                "{} tiles, {} ticks ({}m{:02})",
                route.tiles,
                route.ticks(),
                route.ticks() / 60,
                route.ticks() % 60
            );
            for leg in &route.legs {
                let door = leg
                    .door
                    .map(|d| format!(" then door {}", world.doors[d].id))
                    .unwrap_or_default();
                println!(
                    "  {}: ({}, {}) -> ({}, {}) {} tiles{door}",
                    world.places[leg.place].id,
                    leg.from.x,
                    leg.from.y,
                    leg.to.x,
                    leg.to.y,
                    leg.tiles
                );
                let overlay: Vec<(Pos, char)> = nav::grid_path(&world, leg.place, leg.from, leg.to)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| (p, '*'))
                    .collect();
                print!("{}", show(render_place(&world, leg.place, &overlay)));
            }
            ExitCode::SUCCESS
        }
    }
}
