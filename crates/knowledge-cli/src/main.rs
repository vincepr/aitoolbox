use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use knowledge_core::capture::{capture_issue, capture_lesson};
use knowledge_core::import::{apply_source_file, apply_source_json};
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use serde::Deserialize;
use std::fs;

#[derive(Parser)]
#[command(
    name = "knowledge-cli",
    about = "Query and capture local engineering knowledge",
    long_about = "Local-first knowledge system CLI backed by SQLite and compact Markdown notes.\nUse exact lookup for known entities and explicit capture commands for lessons and issues.",
    after_help = "Examples:\n  knowledge-cli init --db .local/knowledge.sqlite3 --source-file config/knowledge/sources.example.json\n  knowledge-cli init --db .local/knowledge.sqlite3 --source-json '{\"entities\":[{\"canonical_name\":\"MyCompanyName.Ebay.Custom.Client\",\"kind\":\"library\",\"namespace\":\"MyCompanyName.Ebay.Custom.Client\"}]}'\n  knowledge-cli get --db .local/knowledge.sqlite3 --notes-root knowledge/notes --input-json '{\"entity\":\"MyCompanyName.Ebay.Custom.Client\"}'\n  knowledge-cli capture-lesson --db .local/knowledge.sqlite3 --notes-root knowledge/notes --input-json '{\"slug\":\"avoid-global-singleton\",\"body\":\"Global state leaked between tests\"}'\n  knowledge-cli capture-issue --db .local/knowledge.sqlite3 --notes-root knowledge/notes --input-json '{\"slug\":\"stale-mapping-refresh\",\"body\":\"Need automatic refresh for stale repository paths\"}'"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Initialize or refresh the knowledge database from a source JSON file")]
    Init {
        #[arg(
            long,
            default_value = ".local/knowledge.sqlite3",
            help = "Path to the SQLite database file to create or update"
        )]
        db: Utf8PathBuf,
        #[arg(
            long,
            help = "Path to a JSON source file with knowledge entities",
            conflicts_with = "source_json",
            required_unless_present = "source_json"
        )]
        source_file: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Escaped JSON object containing knowledge entities",
            conflicts_with = "source_file",
            required_unless_present = "source_file"
        )]
        source_json: Option<String>,
    },
    #[command(about = "Resolve an entity by exact identifier and print its summary")]
    Get {
        #[arg(
            long,
            default_value = ".local/knowledge.sqlite3",
            help = "Path to the SQLite knowledge database"
        )]
        db: Utf8PathBuf,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Utf8PathBuf,
        #[arg(
            long,
            help = "Path to JSON input file: {\"entity\":\"<canonical-name>\"}",
            conflicts_with = "input_json",
            required_unless_present = "input_json"
        )]
        input_file: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Escaped JSON input: {\"entity\":\"<canonical-name>\"}",
            conflicts_with = "input_file",
            required_unless_present = "input_file"
        )]
        input_json: Option<String>,
    },
    #[command(about = "Capture a reusable lesson note and register it in the knowledge store")]
    CaptureLesson {
        #[arg(
            long,
            default_value = ".local/knowledge.sqlite3",
            help = "Path to the SQLite knowledge database"
        )]
        db: Utf8PathBuf,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Utf8PathBuf,
        #[arg(
            long,
            help = "Path to JSON input file: {\"slug\":\"<slug>\",\"body\":\"<text>\"}",
            conflicts_with = "input_json",
            required_unless_present = "input_json"
        )]
        input_file: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Escaped JSON input: {\"slug\":\"<slug>\",\"body\":\"<text>\"}",
            conflicts_with = "input_file",
            required_unless_present = "input_file"
        )]
        input_json: Option<String>,
    },
    #[command(
        about = "Capture a workflow or architecture issue and register it in the knowledge store"
    )]
    CaptureIssue {
        #[arg(
            long,
            default_value = ".local/knowledge.sqlite3",
            help = "Path to the SQLite knowledge database"
        )]
        db: Utf8PathBuf,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Utf8PathBuf,
        #[arg(
            long,
            help = "Path to JSON input file: {\"slug\":\"<slug>\",\"body\":\"<text>\"}",
            conflicts_with = "input_json",
            required_unless_present = "input_json"
        )]
        input_file: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Escaped JSON input: {\"slug\":\"<slug>\",\"body\":\"<text>\"}",
            conflicts_with = "input_file",
            required_unless_present = "input_file"
        )]
        input_json: Option<String>,
    },
}

#[derive(Deserialize)]
struct GetPayload {
    entity: String,
}

#[derive(Deserialize)]
struct CapturePayload {
    slug: String,
    body: String,
}

fn load_json_input(
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
    context: &str,
) -> Result<String> {
    if let Some(path) = input_file {
        fs::read_to_string(path.as_std_path())
            .with_context(|| format!("failed to read {context} input file: {path}"))
    } else if let Some(json) = input_json {
        Ok(json)
    } else {
        anyhow::bail!("exactly one input is required for {context}: pass --input-file <path> or --input-json <escaped-json>")
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();
    run(Cli::parse().command)
}

fn run(command: Command) -> Result<()> {
    match command {
        Command::Init {
            db,
            source_file,
            source_json,
        } => handle_init(db, source_file, source_json),
        Command::Get {
            db,
            notes_root,
            input_file,
            input_json,
        } => handle_get(db, notes_root, input_file, input_json),
        Command::CaptureLesson {
            db,
            notes_root,
            input_file,
            input_json,
        } => handle_capture_lesson(db, notes_root, input_file, input_json),
        Command::CaptureIssue {
            db,
            notes_root,
            input_file,
            input_json,
        } => handle_capture_issue(db, notes_root, input_file, input_json),
    }
}

fn parse_payload<T: for<'de> Deserialize<'de>>(
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
    context: &str,
) -> Result<T> {
    let raw = load_json_input(input_file, input_json, context)?;
    serde_json::from_str(&raw).with_context(|| format!("failed to parse {context} input JSON"))
}

fn open_bootstrapped_db(db: &Utf8PathBuf) -> Result<Connection> {
    let conn = Connection::open(db.as_std_path())
        .with_context(|| format!("failed to open database: {db}"))?;
    bootstrap(&conn).context("failed to bootstrap knowledge database schema")?;
    Ok(conn)
}

fn handle_init(
    db: Utf8PathBuf,
    source_file: Option<Utf8PathBuf>,
    source_json: Option<String>,
) -> Result<()> {
    if let Some(parent) = db.parent() {
        fs::create_dir_all(parent.as_std_path())
            .with_context(|| format!("failed to create database directory: {parent}"))?;
    }
    let conn = open_bootstrapped_db(&db)?;

    if let Some(source) = source_file {
        apply_source_file(&conn, source.as_path())
            .with_context(|| format!("failed to apply source file: {source}"))?;
    } else if let Some(source_json) = source_json {
        apply_source_json(&conn, &source_json, "--source-json")
            .context("failed to apply source JSON from --source-json")?;
    } else {
        anyhow::bail!(
            "exactly one input is required: pass --source-file <path> or --source-json <escaped-json>"
        )
    }

    Ok(())
}

fn handle_get(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
) -> Result<()> {
    let conn = open_bootstrapped_db(&db)?;
    let store = KnowledgeStore::new(&conn);
    let notes = NoteStore::new(notes_root);
    let payload = parse_payload::<GetPayload>(input_file, input_json, "get")?;
    let answer = store.query_exact(&payload.entity, &notes)?;
    print_get_result(&payload.entity, answer);

    Ok(())
}

fn handle_capture_lesson(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
) -> Result<()> {
    let conn = open_bootstrapped_db(&db)?;
    let notes = NoteStore::new(notes_root);
    let payload = parse_payload::<CapturePayload>(input_file, input_json, "capture-lesson")?;
    capture_lesson(&conn, &notes, &payload.slug, &payload.body)?;
    Ok(())
}

fn handle_capture_issue(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
) -> Result<()> {
    let conn = open_bootstrapped_db(&db)?;
    let notes = NoteStore::new(notes_root);
    let payload = parse_payload::<CapturePayload>(input_file, input_json, "capture-issue")?;
    capture_issue(&conn, &notes, &payload.slug, &payload.body)?;
    Ok(())
}

fn print_get_result(requested_entity: &str, answer: Option<knowledge_core::store::QueryAnswer>) {
    match answer {
        Some(answer) if answer.summary.is_empty() => {
            println!("{}\nNo note summary stored", answer.canonical_name);
        }
        Some(answer) => {
            println!("{}\n{}", answer.canonical_name, answer.summary);
        }
        None => {
            println!("No exact entity match found for {}", requested_entity);
        }
    }
}
