use anyhow::{Context, Result};
use axum::serve;
use camino::Utf8PathBuf;
use clap::Parser;
use knowledge_core::schema::verify_schema;
use rusqlite::Connection;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod http;

#[derive(Parser, Debug)]
#[command(name = "knowledge-daemon", version, about = "Axum daemon mode for knowledge system")]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:3401", help = "HTTP listen address")]
    listen: String,
    #[arg(long, help = "Path to the SQLite knowledge database")]
    db: Utf8PathBuf,
    #[arg(long, help = "Root directory containing compact knowledge notes")]
    notes_root: Utf8PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();

    let cli = Cli::parse();
    verify_startup_schema(&cli.db)?;

    let addr: SocketAddr = cli
        .listen
        .parse()
        .with_context(|| format!("invalid listen address: {}", cli.listen))?;

    let state = http::AppState {
        db_path: cli.db,
        notes_root: cli.notes_root,
    };

    let app = http::router(state);
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind daemon listener on {addr}"))?;

    serve(listener, app)
        .await
        .context("knowledge-daemon server error")?;

    Ok(())
}

fn verify_startup_schema(db_path: &Utf8PathBuf) -> Result<()> {
    let conn = Connection::open(db_path.as_std_path())
        .with_context(|| format!("failed to open database: {db_path}"))?;
    verify_schema(&conn).context("schema verification failed at daemon startup")
}
