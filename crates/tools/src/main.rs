//! `sim-tools`: validators, text views, record and replay. The agent's eyes.
use clap::{Parser, Subcommand};
use content::{compile, list_doors, list_objects, load_dir, render_place, validate};
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
}

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
    }
}
