use clap::{Parser, Subcommand};
use png_parser::commands;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "png-parser")]
#[command(author = "Pranjal Panging")]
#[command(version = "0.1.67")]
#[command(about = "Analyzes, cleans, and hides data within PNG chunks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Inspect {
        #[arg(short, long)]
        file: PathBuf,
    },
    Read {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long)]
        chunk_type: String,
    },
    Strip {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    Hide {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        message: String,
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { file } => {
            commands::inspect(file);
        }
        Commands::Strip { input, output } => {
            commands::strip(input, output);
        }
        Commands::Hide { input, message, output } => {
            commands::hide(input, message, output);
        }
        Commands::Read { file, chunk_type } => {
            commands::read(file, chunk_type);
        }
    }
}