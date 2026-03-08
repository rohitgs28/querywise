mod app;
mod config;
mod db;
mod ui;
mod ai;

use anyhow::Result;
use clap::Parser;
use app::App;

#[derive(Parser)]
#[command(name = "querywise", version, about = "AI-powered universal database TUI client")]
struct Cli {
    /// Database connection URL (e.g., postgres://user:pass@localhost/db)
    #[arg(short, long)]
    url: Option<String>,

    /// Path to SQLite database file
    #[arg(short, long)]
    file: Option<String>,

    /// Execute a single query and exit
    #[arg(short, long)]
    execute: Option<String>,

    /// AI provider: anthropic, openai, ollama (default: anthropic)
    #[arg(long, default_value = "anthropic")]
    ai_provider: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let conn_url = if let Some(file) = &cli.file {
        format!("sqlite:{}", file)
    } else if let Some(url) = &cli.url {
        url.clone()
    } else {
        return Err(anyhow::anyhow!(
            "Provide a connection URL (-u) or SQLite file (-f)\n\n\
             Examples:\n  \
             querywise -u postgres://user:pass@localhost/mydb\n  \
             querywise -u mysql://user:pass@localhost/mydb\n  \
             querywise -f ./data.db"
        ));
    };

    let mut app = App::new(conn_url, cli.ai_provider).await?;

    if let Some(query) = &cli.execute {
        app.execute_and_print(query).await?;
    } else {
        app.run().await?;
    }

    Ok(())
}
