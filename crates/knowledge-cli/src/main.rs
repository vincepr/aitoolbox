use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use knowledge_core::capture::{capture_issue, capture_lesson};
use knowledge_core::import::apply_source_file;
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;

#[derive(Parser)]
#[command(name = "knowledge-cli")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init {
        #[arg(long)]
        db: Utf8PathBuf,
        #[arg(long)]
        source: Utf8PathBuf,
    },
    Query {
        query: String,
        #[arg(long)]
        db: Utf8PathBuf,
        #[arg(long)]
        notes_root: Utf8PathBuf,
    },
    Capture {
        kind: String,
        slug: String,
        body: String,
        #[arg(long)]
        db: Utf8PathBuf,
        #[arg(long)]
        notes_root: Utf8PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();
    match Cli::parse().command {
        Command::Init { db, source } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            apply_source_file(&conn, source.as_path())?;
        }
        Command::Query {
            query,
            db,
            notes_root,
        } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            let store = KnowledgeStore::new(&conn);
            let notes = NoteStore::new(notes_root);
            match store.query_exact(&query, &notes)? {
                Some(answer) if answer.summary.is_empty() => {
                    println!("{}\nNo note summary stored", answer.canonical_name);
                }
                Some(answer) => {
                    println!("{}\n{}", answer.canonical_name, answer.summary);
                }
                None => {
                    println!("No exact entity match found for {query}");
                }
            }
        }
        Command::Capture {
            kind,
            slug,
            body,
            db,
            notes_root,
        } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            let notes = NoteStore::new(notes_root);
            match kind.as_str() {
                "lesson" => {
                    capture_lesson(&conn, &notes, &slug, &body)?;
                }
                "issue" => {
                    capture_issue(&conn, &notes, &slug, &body)?;
                }
                other => anyhow::bail!("unsupported capture kind: {other}"),
            }
        }
    }

    Ok(())
}
