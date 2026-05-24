use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use knowledge_core::capture::{capture_issue, capture_lesson};
use knowledge_core::import::apply_source_file;
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use std::fs;

#[derive(Parser)]
#[command(
    name = "knowledge-cli",
    about = "Query and capture local engineering knowledge",
    long_about = "Local-first knowledge system CLI backed by SQLite and compact Markdown notes.\nUse exact lookup for known entities and explicit capture commands for lessons and issues.",
    after_help = "Examples:\n  knowledge-cli init --db .local/knowledge.db --source config/knowledge/sources.example.json\n  knowledge-cli get MyCompanyName.Ebay.Custom.Client --db .local/knowledge.db --notes-root knowledge/notes\n  knowledge-cli capture-lesson avoid-global-singleton \"Global state leaked between tests\" --db .local/knowledge.db --notes-root knowledge/notes\n  knowledge-cli capture-issue stale-mapping-refresh \"Need automatic refresh for stale repository paths\" --db .local/knowledge.db --notes-root knowledge/notes"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Initialize or refresh the knowledge database from a source JSON file")]
    Init {
        #[arg(long, help = "Path to the SQLite database file to create or update")]
        db: Utf8PathBuf,
        #[arg(long, help = "Path to a JSON source file with knowledge entities")]
        source: Utf8PathBuf,
    },
    #[command(about = "Resolve an entity by exact identifier and print its summary")]
    Get {
        #[arg(help = "Exact entity identifier, such as a namespace or canonical name")]
        entity: String,
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Utf8PathBuf,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Utf8PathBuf,
    },
    #[command(about = "Capture a reusable lesson note and register it in the knowledge store")]
    CaptureLesson {
        #[arg(help = "Stable lesson slug, used in generated entity and note names")]
        slug: String,
        #[arg(help = "Short lesson content that explains the mistake and rule")]
        body: String,
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Utf8PathBuf,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Utf8PathBuf,
    },
    #[command(about = "Capture a workflow or architecture issue and register it in the knowledge store")]
    CaptureIssue {
        #[arg(help = "Stable issue slug, used in generated entity and note names")]
        slug: String,
        #[arg(help = "Short issue content describing the problem and impact")]
        body: String,
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Utf8PathBuf,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Utf8PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();
    match Cli::parse().command {
        Command::Init { db, source } => {
            if let Some(parent) = db.parent() {
                fs::create_dir_all(parent.as_std_path())?;
            }
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            apply_source_file(&conn, source.as_path())?;
        }
        Command::Get {
            entity,
            db,
            notes_root,
        } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            let store = KnowledgeStore::new(&conn);
            let notes = NoteStore::new(notes_root);
            match store.query_exact(&entity, &notes)? {
                Some(answer) if answer.summary.is_empty() => {
                    println!("{}\nNo note summary stored", answer.canonical_name);
                }
                Some(answer) => {
                    println!("{}\n{}", answer.canonical_name, answer.summary);
                }
                None => {
                    println!("No exact entity match found for {entity}");
                }
            }
        }
        Command::CaptureLesson {
            slug,
            body,
            db,
            notes_root,
        } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            let notes = NoteStore::new(notes_root);
            capture_lesson(&conn, &notes, &slug, &body)?;
        }
        Command::CaptureIssue {
            slug,
            body,
            db,
            notes_root,
        } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            let notes = NoteStore::new(notes_root);
            capture_issue(&conn, &notes, &slug, &body)?;
        }
    }

    Ok(())
}
