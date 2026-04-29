use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod core;
#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(name = "dotest")]
#[command(version)]
#[command(about = "A fast, minimal, and ergonomic terminal tool for running .NET tests", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Open interactive UI mode
    Ui,
    /// Print total `dotnet test -t` line count under a namespace (same basis as UI subtree totals)
    #[command(alias = "c")]
    Count {
        /// Short segment (Groups, Imports) or full prefix (Tmly.Test.Imports). Short names pick the longest matching prefix in discovery output.
        folder: String,
        #[arg(long)]
        no_build: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Ui => {
            commands::ui::run()?;
        }
        Commands::Count { folder, no_build } => {
            commands::count::run(folder.clone(), *no_build)?;
        }
    }

    Ok(())
}
