//! `sim-tools`: validators, text views, record and replay. The agent's eyes.
use clap::{Parser, Subcommand};
use content::{
    Nav, Pos, Spot, compile, list_doors, list_objects, load_dir, nav, render_place, validate,
};
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
    /// Travel times between every place's arrival tile.
    TravelMatrix,
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
    let content = match load_dir(&cli.content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
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
                print!("{}", render_place(&world, i, &[]));
                println!("{}\n", list_objects(&world, i));
            }
            ExitCode::SUCCESS
        }
        Command::Doors => {
            let world = compile::compile(&content);
            println!("{}", list_doors(&world));
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
                print!("{}", render_place(&world, leg.place, &overlay));
            }
            ExitCode::SUCCESS
        }
    }
}
