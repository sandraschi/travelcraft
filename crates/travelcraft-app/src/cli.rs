use clap::{Parser, Subcommand};
use travelcraft_core::TravelCraftEngine;

#[derive(Parser)]
#[command(name = "travelcraft-cli", about = "TravelCraft Sovereign Baedeker 2030 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available sovereign country editions
    Countries,
    /// Inspect country edition details (e.g. JP, AT)
    Inspect {
        country: String,
    },
    /// Plan chrono-optimized route
    Plan {
        country: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let engine = TravelCraftEngine::new_in_memory()?;

    match cli.command {
        Commands::Countries => {
            let editions = engine.list_countries();
            println!("{}", serde_json::to_string_pretty(&editions)?);
        }
        Commands::Inspect { country } => {
            let edition = engine.get_country(&country)?;
            println!("{}", serde_json::to_string_pretty(&edition)?);
        }
        Commands::Plan { country } => {
            let plan = engine.plan_optimized_route(&country)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
    }
    Ok(())
}
