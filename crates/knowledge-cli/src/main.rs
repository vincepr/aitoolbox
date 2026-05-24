use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::generate;
use knowledge_core::capture::{capture_issue, capture_lesson};
use knowledge_core::import::{apply_source_file, apply_source_json};
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

const DB_ENV: &str = "KNOWLEDGE_CLI_DB";
const NOTES_ROOT_ENV: &str = "KNOWLEDGE_CLI_NOTES_ROOT";
const SOURCE_FILE_ENV: &str = "KNOWLEDGE_CLI_SOURCE_FILE";
const DEFAULT_SOURCE_JSON: &str = "{\n  \"entities\": []\n}\n";

#[derive(Parser)]
#[command(
    name = "knowledge-cli",
    about = "Query and capture local engineering knowledge",
    long_about = "Local-first knowledge system CLI backed by SQLite and compact Markdown notes.\nUse exact lookup for known entities and explicit capture commands for lessons and issues.",
    after_help = "Environment fallback order: CLI flag -> env var -> user-level home base.\n  KNOWLEDGE_CLI_DB\n  KNOWLEDGE_CLI_NOTES_ROOT\n  KNOWLEDGE_CLI_SOURCE_FILE\nExamples (normal):\n  knowledge-cli quickstart\n  knowledge-cli init --source-file config/knowledge/sources.example.json\n  knowledge-cli get MyCompanyName.Ebay.Custom.Client\n  knowledge-cli capture-lesson --slug avoid-global-singleton --body 'Global state leaked between tests'\n  knowledge-cli capture-issue --slug stale-mapping-refresh --body 'Need automatic refresh for stale repository paths'\n  knowledge-cli completions bash > ~/.local/share/bash-completion/completions/knowledge-cli\n  knowledge-cli alias bash\nExamples (edge-case overrides):\n  knowledge-cli get MyCompanyName.Ebay.Custom.Client --db /tmp/knowledge.sqlite3 --notes-root /tmp/notes\n  knowledge-cli capture-lesson --slug avoid-global-singleton --body 'text' --db /tmp/knowledge.sqlite3 --notes-root /tmp/notes"
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
        db: Option<Utf8PathBuf>,
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
    #[command(about = "Create default local files and bootstrap the knowledge database")]
    Quickstart {
        #[arg(long, help = "Path to the SQLite knowledge database to initialize")]
        db: Option<Utf8PathBuf>,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Option<Utf8PathBuf>,
        #[arg(long, help = "Path to the source JSON file used by init")]
        source_file: Option<Utf8PathBuf>,
    },
    #[command(about = "Resolve an entity by exact identifier and print its summary")]
    Get {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Option<Utf8PathBuf>,
        #[arg(
            help = "Canonical entity name for exact lookup (for example MyCompanyName.Ebay.Custom.Client)"
        )]
        entity: Option<String>,
        #[arg(
            long,
            help = "Path to JSON input file: {\"entity\":\"<canonical-name>\"}",
            conflicts_with_all = ["input_json", "entity"]
        )]
        input_file: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Escaped JSON input: {\"entity\":\"<canonical-name>\"}",
            conflicts_with_all = ["input_file", "entity"]
        )]
        input_json: Option<String>,
    },
    #[command(about = "Capture a reusable lesson note and register it in the knowledge store")]
    CaptureLesson {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Option<Utf8PathBuf>,
        #[arg(long, help = "Stable lesson slug used as note identifier")]
        slug: Option<String>,
        #[arg(long, help = "Lesson text content to store in the note body")]
        body: Option<String>,
        #[arg(
            long,
            help = "Path to JSON input file: {\"slug\":\"<slug>\",\"body\":\"<text>\"}",
            conflicts_with = "input_json",
            conflicts_with_all = ["slug", "body"]
        )]
        input_file: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Escaped JSON input: {\"slug\":\"<slug>\",\"body\":\"<text>\"}",
            conflicts_with = "input_file",
            conflicts_with_all = ["slug", "body"]
        )]
        input_json: Option<String>,
    },
    #[command(
        about = "Capture a workflow or architecture issue and register it in the knowledge store"
    )]
    CaptureIssue {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Option<Utf8PathBuf>,
        #[arg(long, help = "Stable issue slug used as note identifier")]
        slug: Option<String>,
        #[arg(long, help = "Issue text content to store in the note body")]
        body: Option<String>,
        #[arg(
            long,
            help = "Path to JSON input file: {\"slug\":\"<slug>\",\"body\":\"<text>\"}",
            conflicts_with = "input_json",
            conflicts_with_all = ["slug", "body"]
        )]
        input_file: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Escaped JSON input: {\"slug\":\"<slug>\",\"body\":\"<text>\"}",
            conflicts_with = "input_file",
            conflicts_with_all = ["slug", "body"]
        )]
        input_json: Option<String>,
    },
    #[command(about = "Generate shell completion scripts")]
    Completions {
        #[arg(help = "Shell type to generate completions for")]
        shell: CompletionShell,
    },
    #[command(about = "Print a shell alias for a shorter command name")]
    Alias {
        #[arg(help = "Shell type to print alias syntax for")]
        shell: AliasShell,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    #[value(name = "powershell")]
    PowerShell,
    Zsh,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum AliasShell {
    Bash,
    Fish,
    #[value(name = "powershell")]
    PowerShell,
    Zsh,
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
        anyhow::bail!(
            "exactly one input is required for {context}: pass --input-file <path> or --input-json <escaped-json>\nexample: knowledge-cli {context} --input-json '<json-payload>'"
        )
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
        } => handle_init(resolve_db_path(db)?, source_file, source_json),
        Command::Quickstart {
            db,
            notes_root,
            source_file,
        } => handle_quickstart(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            resolve_source_file(source_file)?,
        ),
        Command::Get {
            db,
            notes_root,
            entity,
            input_file,
            input_json,
        } => handle_get(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            entity,
            input_file,
            input_json,
        ),
        Command::CaptureLesson {
            db,
            notes_root,
            slug,
            body,
            input_file,
            input_json,
        } => handle_capture_lesson(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            slug,
            body,
            input_file,
            input_json,
        ),
        Command::CaptureIssue {
            db,
            notes_root,
            slug,
            body,
            input_file,
            input_json,
        } => handle_capture_issue(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            slug,
            body,
            input_file,
            input_json,
        ),
        Command::Completions { shell } => {
            handle_completions(shell);
            Ok(())
        }
        Command::Alias { shell } => {
            print_alias(shell);
            Ok(())
        }
    }
}

fn resolve_db_path(cli_value: Option<Utf8PathBuf>) -> Result<Utf8PathBuf> {
    if let Some(path) = cli_value {
        return Ok(path);
    }
    if let Some(path) = env_path(DB_ENV)? {
        return Ok(path);
    }
    join_utf8(data_home_base()?, &["knowledge-cli", "knowledge.sqlite3"])
}

fn resolve_notes_root(cli_value: Option<Utf8PathBuf>) -> Result<Utf8PathBuf> {
    if let Some(path) = cli_value {
        return Ok(path);
    }
    if let Some(path) = env_path(NOTES_ROOT_ENV)? {
        return Ok(path);
    }
    join_utf8(data_home_base()?, &["knowledge-cli", "notes"])
}

fn resolve_source_file(cli_value: Option<Utf8PathBuf>) -> Result<Utf8PathBuf> {
    if let Some(path) = cli_value {
        return Ok(path);
    }
    if let Some(path) = env_path(SOURCE_FILE_ENV)? {
        return Ok(path);
    }
    join_utf8(
        config_home_base()?,
        &["knowledge-cli", "sources.example.json"],
    )
}

fn env_path(name: &str) -> Result<Option<Utf8PathBuf>> {
    match env::var_os(name) {
        Some(value) => utf8_from_path(PathBuf::from(value), name).map(Some),
        None => Ok(None),
    }
}

fn data_home_base() -> Result<Utf8PathBuf> {
    let data_dir =
        dirs::data_local_dir().or_else(|| dirs::home_dir().map(|home| home.join(".local/share")));
    let base = data_dir.unwrap_or_else(|| PathBuf::from("."));
    utf8_from_path(base, "data_home")
}

fn config_home_base() -> Result<Utf8PathBuf> {
    let config_dir =
        dirs::config_dir().or_else(|| dirs::home_dir().map(|home| home.join(".config")));
    let base = config_dir.unwrap_or_else(|| PathBuf::from("."));
    utf8_from_path(base, "config_home")
}

fn join_utf8(base: Utf8PathBuf, segments: &[&str]) -> Result<Utf8PathBuf> {
    let mut path = base.into_std_path_buf();
    for segment in segments {
        path.push(segment);
    }
    utf8_from_path(path, "default path")
}

fn utf8_from_path(path: PathBuf, context: &str) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path).map_err(|invalid| {
        anyhow::anyhow!(
            "path for {context} is not valid UTF-8: {}",
            invalid.display()
        )
    })
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
    if let Some(parent) = db.parent() {
        fs::create_dir_all(parent.as_std_path())
            .with_context(|| format!("failed to create database directory: {parent}"))?;
    }

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
    let conn = open_bootstrapped_db(&db)?;

    if let Some(source) = source_file {
        apply_source_file(&conn, source.as_path())
            .with_context(|| format!("failed to apply source file: {source}"))?;
    } else if let Some(source_json) = source_json {
        apply_source_json(&conn, &source_json, "--source-json")
            .context("failed to apply source JSON from --source-json")?;
    } else {
        let source_hint = resolve_source_file(None)?;
        anyhow::bail!(
            "exactly one input is required: pass --source-file <path> or --source-json <escaped-json>\nexample: knowledge-cli init --source-file {source_hint}"
        )
    }

    Ok(())
}

fn handle_quickstart(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    source_file: Utf8PathBuf,
) -> Result<()> {
    fs::create_dir_all(notes_root.as_std_path())
        .with_context(|| format!("failed to create notes directory: {notes_root}"))?;

    if let Some(parent) = source_file.parent() {
        fs::create_dir_all(parent.as_std_path())
            .with_context(|| format!("failed to create source file directory: {parent}"))?;
    }

    if !source_file.exists() {
        fs::write(source_file.as_std_path(), DEFAULT_SOURCE_JSON)
            .with_context(|| format!("failed to write default source file: {source_file}"))?;
    }

    handle_init(db.clone(), Some(source_file.clone()), None)?;

    println!("Database ready: {db}");
    println!("Notes root ready: {notes_root}");
    println!("Source file ready: {source_file}");
    println!("Next: knowledge-cli get <entity>");
    Ok(())
}

fn handle_get(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    entity: Option<String>,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
) -> Result<()> {
    let entity_name = parse_get_entity(entity, input_file, input_json)?;
    let conn = open_bootstrapped_db(&db)?;
    let store = KnowledgeStore::new(&conn);
    let notes = NoteStore::new(notes_root);
    let answer = store.query_exact(&entity_name, &notes)?;
    print_get_result(&entity_name, answer);

    Ok(())
}

fn parse_get_entity(
    entity: Option<String>,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
) -> Result<String> {
    if let Some(entity_name) = entity {
        return Ok(entity_name);
    }

    if input_file.is_none() && input_json.is_none() {
        anyhow::bail!(
            "missing lookup input: pass <ENTITY> or one of --input-file/--input-json\nexample: knowledge-cli get marketplaces"
        );
    }

    let payload = parse_payload::<GetPayload>(input_file, input_json, "get")?;
    Ok(payload.entity)
}

fn handle_capture_lesson(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    slug: Option<String>,
    body: Option<String>,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
) -> Result<()> {
    let payload = parse_capture_payload(slug, body, input_file, input_json, "capture-lesson")?;
    let conn = open_bootstrapped_db(&db)?;
    let notes = NoteStore::new(notes_root);
    capture_lesson(&conn, &notes, &payload.slug, &payload.body)?;
    Ok(())
}

fn handle_capture_issue(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    slug: Option<String>,
    body: Option<String>,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
) -> Result<()> {
    let payload = parse_capture_payload(slug, body, input_file, input_json, "capture-issue")?;
    let conn = open_bootstrapped_db(&db)?;
    let notes = NoteStore::new(notes_root);
    capture_issue(&conn, &notes, &payload.slug, &payload.body)?;
    Ok(())
}

fn parse_capture_payload(
    slug: Option<String>,
    body: Option<String>,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
    context: &str,
) -> Result<CapturePayload> {
    match (slug, body) {
        (Some(slug), Some(body)) => return Ok(CapturePayload { slug, body }),
        (Some(_), None) | (None, Some(_)) => {
            anyhow::bail!(
                "both --slug and --body are required together for {context} when not using JSON input\nexample: knowledge-cli {context} --slug sample-slug --body 'note text'"
            );
        }
        (None, None) => {}
    }

    parse_payload::<CapturePayload>(input_file, input_json, context)
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

fn handle_completions(shell: CompletionShell) {
    let mut command = Cli::command();
    let mut stdout = io::stdout();
    let completion_shell: clap_complete::Shell = shell.into();
    generate(completion_shell, &mut command, "knowledge-cli", &mut stdout);
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            CompletionShell::Bash => clap_complete::Shell::Bash,
            CompletionShell::Elvish => clap_complete::Shell::Elvish,
            CompletionShell::Fish => clap_complete::Shell::Fish,
            CompletionShell::PowerShell => clap_complete::Shell::PowerShell,
            CompletionShell::Zsh => clap_complete::Shell::Zsh,
        }
    }
}

fn print_alias(shell: AliasShell) {
    let alias_line = match shell {
        AliasShell::Bash | AliasShell::Zsh => "alias kno='knowledge-cli'",
        AliasShell::Fish => "alias kno 'knowledge-cli'",
        AliasShell::PowerShell => "Set-Alias -Name kno -Value knowledge-cli",
    };
    println!("{alias_line}");
}
