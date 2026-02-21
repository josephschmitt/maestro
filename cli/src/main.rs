mod commands;
mod ipc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "maestro", about = "Maestro CLI — agent-to-app communication")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Surface an open question for the user
    Question {
        /// The question text
        question: String,
    },
    /// Resolve an open question
    ResolveQuestion {
        /// The question ID to resolve
        #[arg(long)]
        id: String,
        /// Optional resolution text
        #[arg(long)]
        resolution: Option<String>,
    },
    /// Attach an artifact file to the current card
    AddArtifact {
        /// Path to the file to attach
        #[arg(long)]
        file: String,
        /// Display name for the artifact
        #[arg(long)]
        name: Option<String>,
    },
    /// Update the current card's status
    SetStatus {
        /// The new status (e.g. "in-review", "completed")
        status: String,
    },
    /// Record a progress note
    Log {
        /// The log message
        message: String,
    },
    /// Get the current card's details as JSON
    GetCard,
    /// Get all artifacts for the current card as JSON
    GetArtifacts,
    /// Get the parent card's details as JSON
    GetParent,
}

fn env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| {
        format!(
            "{name} not set. Are you running inside a Maestro agent session?"
        )
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    let socket_path = match env_var("MAESTRO_SOCKET") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    let card_id = match env_var("MAESTRO_CARD_ID") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Commands::Question { question } => {
            commands::question::run(&socket_path, &card_id, &question).await
        }
        Commands::ResolveQuestion { id, resolution } => {
            commands::resolve_question::run(&socket_path, &card_id, &id, resolution.as_deref())
                .await
        }
        Commands::AddArtifact { file, name } => {
            commands::artifact::run(&socket_path, &card_id, &file, name.as_deref()).await
        }
        Commands::SetStatus { status } => {
            commands::status::run(&socket_path, &card_id, &status).await
        }
        Commands::Log { message } => {
            commands::log::run(&socket_path, &card_id, &message).await
        }
        Commands::GetCard => commands::get_card::run(&socket_path, &card_id).await,
        Commands::GetArtifacts => commands::get_artifacts::run(&socket_path, &card_id).await,
        Commands::GetParent => commands::get_parent::run(&socket_path, &card_id).await,
    };

    match result {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
