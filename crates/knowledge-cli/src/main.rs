use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::generate;
use knowledge_core::audit::list_entity_history;
use knowledge_core::capture::{capture_issue, capture_lesson};
use knowledge_core::config::{resolve as resolve_config, EffectiveConfig, ResolveOverrides};
use knowledge_core::embed::{
    cosine_similarity, fingerprint_text, DisabledEmbeddingProvider, EmbeddingProvider,
    OpenAiCompatibleEmbeddingProvider,
};
use knowledge_core::import::{apply_source_file, apply_source_json};
use knowledge_core::ingest::{enqueue_job, queue_status, run_once, DisabledProvider};
use knowledge_core::input_schema::{validate_payload, InputSchemaKind};
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::{bootstrap, schema_version, verify_schema};
use knowledge_core::store::KnowledgeStore;
use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

const DB_ENV: &str = "KNOWLEDGE_CLI_DB";
const NOTES_ROOT_ENV: &str = "KNOWLEDGE_CLI_NOTES_ROOT";
const SOURCE_FILE_ENV: &str = "KNOWLEDGE_CLI_SOURCE_FILE";
const CONFIG_FILE_ENV: &str = "KNOWLEDGE_CLI_CONFIG_FILE";
const RECALL_TOP_K_ENV: &str = "KNOWLEDGE_CLI_RECALL_TOP_K";
const EMBEDDINGS_PROVIDER_ENV: &str = "KNOWLEDGE_CLI_EMBEDDINGS_PROVIDER";
const EMBEDDINGS_MODEL_ENV: &str = "KNOWLEDGE_CLI_EMBEDDINGS_MODEL";
const EMBEDDINGS_BASE_URL_ENV: &str = "KNOWLEDGE_CLI_EMBEDDINGS_BASE_URL";
const EMBEDDINGS_TIMEOUT_MS_ENV: &str = "KNOWLEDGE_CLI_EMBEDDINGS_TIMEOUT_MS";
const EMBEDDINGS_DIMENSIONS_ENV: &str = "KNOWLEDGE_CLI_EMBEDDINGS_DIMENSIONS";
const DEFAULT_SOURCE_JSON: &str =
    "{\n  \"$schema\": \"https://aitoolbox/schemas/entity.v1.json\",\n  \"entities\": []\n}\n";

#[derive(Parser)]
#[command(
    name = "knowledge-cli",
    version,
    about = "Query and capture local engineering knowledge",
    long_about = "Local-first knowledge system CLI backed by SQLite and compact Markdown notes.\nUse exact lookup for known entities and explicit capture commands for lessons and issues.",
    after_help = "Environment fallback order: CLI flag -> env var -> user-level home base.\n  KNOWLEDGE_CLI_DB\n  KNOWLEDGE_CLI_NOTES_ROOT\n  KNOWLEDGE_CLI_SOURCE_FILE\n  KNOWLEDGE_CLI_EMBEDDINGS_PROVIDER\n  KNOWLEDGE_CLI_EMBEDDINGS_MODEL\n  KNOWLEDGE_CLI_EMBEDDINGS_BASE_URL\n  KNOWLEDGE_CLI_EMBEDDINGS_TIMEOUT_MS\n  KNOWLEDGE_CLI_EMBEDDINGS_DIMENSIONS\nExamples (normal):\n  knowledge-cli quickstart\n  knowledge-cli init --source-file config/knowledge/sources.example.json\n  knowledge-cli get frameworkname-marketplaces-jobs-pricestock\n  knowledge-cli recall marketplaces --embeddings-provider openai-compatible --embeddings-model google/embeddinggemma-300m --embeddings-dimensions 768\n  knowledge-cli capture-lesson --slug avoid-global-singleton --body 'Global state leaked between tests'\n  knowledge-cli capture-issue --slug stale-mapping-refresh --body 'Need automatic refresh for stale repository paths'\n  knowledge-cli completions bash > ~/.local/share/bash-completion/completions/knowledge-cli\n  knowledge-cli alias bash\nExamples (edge-case overrides):\n  knowledge-cli get frameworkname-marketplaces-jobs-pricestock --db /tmp/knowledge.sqlite3 --notes-root /tmp/notes\n  knowledge-cli capture-lesson --slug avoid-global-singleton --body 'text' --db /tmp/knowledge.sqlite3 --notes-root /tmp/notes"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Path to config JSON file used for runtime behavior defaults"
    )]
    config_file: Option<Utf8PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Recall top-k override (highest precedence)"
    )]
    recall_top_k: Option<u32>,
    #[arg(
        long,
        global = true,
        help = "Embeddings provider override: none|openai-compatible"
    )]
    embeddings_provider: Option<String>,
    #[arg(long, global = true, help = "Embeddings model override")]
    embeddings_model: Option<String>,
    #[arg(long, global = true, help = "Embeddings provider base URL override")]
    embeddings_base_url: Option<String>,
    #[arg(
        long,
        global = true,
        help = "Embeddings request timeout override in ms"
    )]
    embeddings_timeout_ms: Option<u64>,
    #[arg(long, global = true, help = "Embeddings vector dimension override")]
    embeddings_dimensions: Option<u32>,
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
            conflicts_with = "source_json"
        )]
        source_file: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Escaped JSON object containing knowledge entities",
            conflicts_with = "source_file"
        )]
        source_json: Option<String>,
        #[arg(long, help = "Print expected JSON schema for init source payload")]
        print_schema: bool,
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
            help = "Canonical entity name for exact lookup (for example frameworkname-marketplaces-jobs-pricestock)"
        )]
        entity: Option<String>,
        #[arg(
            long,
            default_value_t = 3,
            value_parser = clap::value_parser!(u32).range(1..=100),
            help = "Number of ranked matches to print"
        )]
        limit: u32,
        #[arg(
            long,
            default_value_t = 10,
            value_parser = clap::value_parser!(u32).range(1..=100),
            help = "Number of related child entities to print for parent matches"
        )]
        related_limit: u32,
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
    #[command(about = "List entities for discovery when canonical names are unknown")]
    List {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Case-insensitive substring filter across canonical name, namespace, package, repo, and aliases"
        )]
        grep: Option<String>,
        #[arg(long, help = "Filter to one entity kind")]
        kind: Option<ListKind>,
        #[arg(long, default_value_t = 20, help = "Maximum number of rows to print")]
        limit: u32,
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
        #[arg(
            long,
            help = "Print expected JSON schema for capture-lesson JSON payload"
        )]
        print_schema: bool,
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
        #[arg(
            long,
            help = "Print expected JSON schema for capture-issue JSON payload"
        )]
        print_schema: bool,
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
    #[command(about = "Print the knowledge-cli version")]
    Version,
    #[command(about = "Apply pending database migrations or verify schema compatibility")]
    Migrate {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Only verify schema compatibility without applying migrations"
        )]
        verify: bool,
        #[arg(long, help = "Print pending migration status without writing changes")]
        dry_run: bool,
    },
    #[command(about = "Print recent mutation history for an entity")]
    History {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(help = "Canonical entity name for history lookup")]
        entity: String,
        #[arg(long, default_value_t = 20, help = "Maximum number of history rows")]
        limit: u32,
    },
    #[command(about = "Queue one raw payload for background ingestion")]
    PipelineEnqueue {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(long, help = "Stable dedupe key used to avoid duplicate queued jobs")]
        dedupe_key: String,
        #[arg(
            long,
            help = "Escaped JSON input: {\"$schema\":\"...\",\"payload\":\"<raw>\"}"
        )]
        payload: Option<String>,
        #[arg(long, help = "Override max retry attempts for this job")]
        max_attempts: Option<u32>,
        #[arg(long, help = "Print expected JSON schema for pipeline enqueue payload")]
        print_schema: bool,
    },
    #[command(about = "Run one queued ingestion job")]
    PipelineRunOnce {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
    },
    #[command(about = "Show ingestion queue counts")]
    PipelineStatus {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
    },
    #[command(about = "Run semantic recall using embeddings")]
    Recall {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Option<Utf8PathBuf>,
        #[arg(help = "Free-form recall query")]
        query: String,
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..=100), help = "Maximum recall rows")]
        top_k: Option<u32>,
    },
    #[command(about = "Generate or refresh cached embeddings for indexed entities")]
    EmbeddingsIndex {
        #[arg(long, help = "Path to the SQLite knowledge database")]
        db: Option<Utf8PathBuf>,
        #[arg(long, help = "Root directory containing compact knowledge notes")]
        notes_root: Option<Utf8PathBuf>,
        #[arg(
            long,
            help = "Force recomputation even when cached fingerprint matches"
        )]
        reembed: bool,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum EmbedRefreshMode {
    MissingOnly,
    ForceAll,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ListKind {
    Domain,
    System,
    Library,
    Project,
    Lesson,
}

impl ListKind {
    fn as_str(self) -> &'static str {
        match self {
            ListKind::Domain => "domain",
            ListKind::System => "system",
            ListKind::Library => "library",
            ListKind::Project => "project",
            ListKind::Lesson => "lesson",
        }
    }
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

struct CaptureInput {
    slug: Option<String>,
    body: Option<String>,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
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
    run(Cli::parse())
}

fn run(cli: Cli) -> Result<()> {
    // Validate config before command execution so invalid settings fail fast.
    let effective_config = resolve_effective_config(
        &cli.config_file,
        cli.recall_top_k,
        cli.embeddings_provider.clone(),
        cli.embeddings_model.clone(),
        cli.embeddings_base_url.clone(),
        cli.embeddings_timeout_ms,
        cli.embeddings_dimensions,
    )?;

    match cli.command {
        Command::Init {
            db,
            source_file,
            source_json,
            print_schema,
        } => handle_init(
            resolve_db_path(db)?,
            source_file,
            source_json,
            print_schema,
            resolve_notes_root(None)?,
            &effective_config,
        ),
        Command::Quickstart {
            db,
            notes_root,
            source_file,
        } => handle_quickstart(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            resolve_source_file(source_file)?,
            &effective_config,
        ),
        Command::Get {
            db,
            notes_root,
            entity,
            limit,
            related_limit,
            input_file,
            input_json,
        } => handle_get(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            entity,
            limit,
            related_limit,
            input_file,
            input_json,
        ),
        Command::List {
            db,
            grep,
            kind,
            limit,
        } => handle_list(resolve_db_path(db)?, grep, kind, limit),
        Command::CaptureLesson {
            db,
            notes_root,
            slug,
            body,
            input_file,
            input_json,
            print_schema,
        } => handle_capture_lesson(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            CaptureInput {
                slug,
                body,
                input_file,
                input_json,
            },
            print_schema,
            &effective_config,
        ),
        Command::CaptureIssue {
            db,
            notes_root,
            slug,
            body,
            input_file,
            input_json,
            print_schema,
        } => handle_capture_issue(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            CaptureInput {
                slug,
                body,
                input_file,
                input_json,
            },
            print_schema,
            &effective_config,
        ),
        Command::Completions { shell } => {
            handle_completions(shell);
            Ok(())
        }
        Command::Alias { shell } => {
            print_alias(shell);
            Ok(())
        }
        Command::Version => {
            print_version();
            Ok(())
        }
        Command::Migrate {
            db,
            verify,
            dry_run,
        } => handle_migrate(resolve_db_path(db)?, verify, dry_run),
        Command::History { db, entity, limit } => {
            handle_history(resolve_db_path(db)?, &entity, limit)
        }
        Command::PipelineEnqueue {
            db,
            dedupe_key,
            payload,
            max_attempts,
            print_schema,
        } => handle_pipeline_enqueue(
            resolve_db_path(db)?,
            &dedupe_key,
            payload.as_deref(),
            max_attempts.unwrap_or(effective_config.pipeline.max_attempts),
            print_schema,
        ),
        Command::PipelineRunOnce { db } => handle_pipeline_run_once(resolve_db_path(db)?),
        Command::PipelineStatus { db } => handle_pipeline_status(resolve_db_path(db)?),
        Command::Recall {
            db,
            notes_root,
            query,
            top_k,
        } => handle_recall(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            &query,
            top_k.unwrap_or(effective_config.recall.top_k),
            &effective_config,
        ),
        Command::EmbeddingsIndex {
            db,
            notes_root,
            reembed,
        } => handle_embeddings_index(
            resolve_db_path(db)?,
            resolve_notes_root(notes_root)?,
            if reembed {
                EmbedRefreshMode::ForceAll
            } else {
                EmbedRefreshMode::MissingOnly
            },
            &effective_config,
        ),
    }
}

fn resolve_effective_config(
    cli_config_path: &Option<Utf8PathBuf>,
    cli_recall_top_k: Option<u32>,
    cli_embeddings_provider: Option<String>,
    cli_embeddings_model: Option<String>,
    cli_embeddings_base_url: Option<String>,
    cli_embeddings_timeout_ms: Option<u64>,
    cli_embeddings_dimensions: Option<u32>,
) -> Result<EffectiveConfig> {
    let config_path = if cli_config_path.is_some() {
        cli_config_path.clone()
    } else {
        env_path(CONFIG_FILE_ENV)?
    };
    let file_json = match config_path {
        Some(path) => {
            let raw = fs::read_to_string(path.as_std_path())
                .with_context(|| format!("failed to read config file: {path}"))?;
            Some(raw)
        }
        None => None,
    };

    let env_recall_top_k = env::var(RECALL_TOP_K_ENV)
        .ok()
        .map(|raw| raw.parse::<u32>())
        .transpose()
        .context("failed to parse KNOWLEDGE_CLI_RECALL_TOP_K as u32")?;

    let env_embeddings_provider = env::var(EMBEDDINGS_PROVIDER_ENV).ok();
    let env_embeddings_model = env::var(EMBEDDINGS_MODEL_ENV).ok();
    let env_embeddings_base_url = env::var(EMBEDDINGS_BASE_URL_ENV).ok();
    let env_embeddings_timeout_ms = env::var(EMBEDDINGS_TIMEOUT_MS_ENV)
        .ok()
        .map(|raw| raw.parse::<u64>())
        .transpose()
        .context("failed to parse KNOWLEDGE_CLI_EMBEDDINGS_TIMEOUT_MS as u64")?;
    let env_embeddings_dimensions = env::var(EMBEDDINGS_DIMENSIONS_ENV)
        .ok()
        .map(|raw| raw.parse::<u32>())
        .transpose()
        .context("failed to parse KNOWLEDGE_CLI_EMBEDDINGS_DIMENSIONS as u32")?;

    resolve_config(
        file_json.as_deref(),
        ResolveOverrides {
            recall_top_k: env_recall_top_k,
            embeddings_provider: env_embeddings_provider,
            embeddings_model: env_embeddings_model,
            embeddings_base_url: env_embeddings_base_url,
            embeddings_timeout_ms: env_embeddings_timeout_ms,
            embeddings_dimensions: env_embeddings_dimensions,
        },
        ResolveOverrides {
            recall_top_k: cli_recall_top_k,
            embeddings_provider: cli_embeddings_provider,
            embeddings_model: cli_embeddings_model,
            embeddings_base_url: cli_embeddings_base_url,
            embeddings_timeout_ms: cli_embeddings_timeout_ms,
            embeddings_dimensions: cli_embeddings_dimensions,
        },
    )
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
    print_schema: bool,
    notes_root: Utf8PathBuf,
    config: &EffectiveConfig,
) -> Result<()> {
    if print_schema {
        println!("{}", InputSchemaKind::Entity.schema_text());
        return Ok(());
    }
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

    maybe_refresh_embeddings_for_all(&conn, notes_root, config)?;

    Ok(())
}

fn handle_quickstart(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    source_file: Utf8PathBuf,
    config: &EffectiveConfig,
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

    handle_init(
        db.clone(),
        Some(source_file.clone()),
        None,
        false,
        notes_root.clone(),
        config,
    )?;

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
    limit: u32,
    related_limit: u32,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
) -> Result<()> {
    let entity_name = parse_get_entity(entity, input_file, input_json)?;
    let conn = open_bootstrapped_db(&db)?;
    let store = KnowledgeStore::new(&conn);
    let notes = NoteStore::new(notes_root);
    let related = store
        .lookup_exact(&entity_name)?
        .and_then(|lookup| {
            if is_parent_kind(&lookup.entity.kind) {
                Some(store.related_children(
                    lookup.entity.id,
                    &lookup.entity.canonical_name,
                    related_limit,
                ))
            } else {
                None
            }
        })
        .transpose()?;
    let answer = store.query_exact(&entity_name, &notes)?;
    let matches = store.search_best(&entity_name, limit)?;
    print_get_result(&entity_name, answer, related, matches);

    Ok(())
}

fn handle_list(
    db: Utf8PathBuf,
    grep: Option<String>,
    kind: Option<ListKind>,
    limit: u32,
) -> Result<()> {
    let conn = open_bootstrapped_db(&db)?;
    let store = KnowledgeStore::new(&conn);
    let records = store.list(grep.as_deref(), kind.map(ListKind::as_str), limit)?;

    for record in records {
        println!(
            "{}\t{}\t{}",
            record.canonical_name, record.kind, record.repo_name
        );
    }

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
    input: CaptureInput,
    print_schema: bool,
    config: &EffectiveConfig,
) -> Result<()> {
    if print_schema {
        println!("{}", InputSchemaKind::Lesson.schema_text());
        return Ok(());
    }
    let payload = parse_capture_payload(
        input.slug,
        input.body,
        input.input_file,
        input.input_json,
        "capture-lesson",
        InputSchemaKind::Lesson,
    )?;
    let conn = open_bootstrapped_db(&db)?;
    let notes = NoteStore::new(notes_root.clone());
    capture_lesson(&conn, &notes, &payload.slug, &payload.body)?;
    maybe_refresh_embeddings_for_all(&conn, notes_root, config)?;
    Ok(())
}

fn handle_capture_issue(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    input: CaptureInput,
    print_schema: bool,
    config: &EffectiveConfig,
) -> Result<()> {
    if print_schema {
        println!("{}", InputSchemaKind::Issue.schema_text());
        return Ok(());
    }
    let payload = parse_capture_payload(
        input.slug,
        input.body,
        input.input_file,
        input.input_json,
        "capture-issue",
        InputSchemaKind::Issue,
    )?;
    let conn = open_bootstrapped_db(&db)?;
    let notes = NoteStore::new(notes_root.clone());
    capture_issue(&conn, &notes, &payload.slug, &payload.body)?;
    maybe_refresh_embeddings_for_all(&conn, notes_root, config)?;
    Ok(())
}

fn parse_capture_payload(
    slug: Option<String>,
    body: Option<String>,
    input_file: Option<Utf8PathBuf>,
    input_json: Option<String>,
    context: &str,
    schema_kind: InputSchemaKind,
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

    let raw = load_json_input(input_file, input_json, context)?;
    validate_payload(&raw, schema_kind)
        .with_context(|| format!("{context} payload failed schema validation"))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to parse {context} input JSON"))
}

fn print_get_result(
    requested_entity: &str,
    answer: Option<knowledge_core::store::QueryAnswer>,
    related: Option<knowledge_core::store::RelatedEntityPage>,
    matches: Vec<knowledge_core::store::ListEntityRecord>,
) {
    match answer {
        Some(answer) => {
            println!("{}", answer.canonical_name);
            match answer.summary_source {
                knowledge_core::store::SummarySource::Note => println!("{}", answer.summary),
                knowledge_core::store::SummarySource::Entity => {
                    println!("Summary (entity): {}", answer.summary)
                }
                knowledge_core::store::SummarySource::None => {
                    println!("No note summary stored");
                }
            }

            if let Some(location) = answer.location {
                if let Some(local_path) = location.local_path {
                    println!("local: {}", local_path);
                }
                if let Some(git_url) = location.git_url {
                    println!("git:   {}", git_url);
                }
            }
        }
        None => {
            println!("No exact entity match found for {}", requested_entity);
        }
    }

    if let Some(related) = related {
        if !related.rows.is_empty() {
            println!();
            println!("Related ({} of {}):", related.rows.len(), related.total);
            for row in related.rows {
                let note_marker = if row.has_note { "has_note" } else { "no_note" };
                println!(
                    "{}\t{}\t{}\t{}",
                    row.id, row.canonical_name, row.kind, note_marker
                );
            }
        }
    }

    if !matches.is_empty() {
        println!();
        println!("Top matches:");
        for record in matches {
            println!(
                "{}\t{}\t{}",
                record.canonical_name, record.kind, record.repo_name
            );
        }
    }
}

fn is_parent_kind(kind: &str) -> bool {
    matches!(kind, "domain" | "system")
}

fn handle_completions(shell: CompletionShell) {
    let mut command = Cli::command();
    let mut stdout = io::stdout();
    let completion_shell: clap_complete::Shell = shell.into();
    generate(completion_shell, &mut command, "knowledge-cli", &mut stdout);
}

fn handle_migrate(db: Utf8PathBuf, verify: bool, dry_run: bool) -> Result<()> {
    if let Some(parent) = db.parent() {
        fs::create_dir_all(parent.as_std_path())
            .with_context(|| format!("failed to create database directory: {parent}"))?;
    }
    let conn = Connection::open(db.as_std_path())
        .with_context(|| format!("failed to open database: {db}"))?;

    if dry_run {
        let version = schema_version(&conn)?;
        println!("current schema version: {version}");
        return Ok(());
    }

    if verify {
        verify_schema(&conn)?;
        let version = schema_version(&conn)?;
        println!("schema verified at version {version}");
        return Ok(());
    }

    bootstrap(&conn)?;
    let version = schema_version(&conn)?;
    println!("migrations applied; schema version {version}");
    Ok(())
}

fn handle_history(db: Utf8PathBuf, entity: &str, limit: u32) -> Result<()> {
    let conn = open_bootstrapped_db(&db)?;
    verify_schema(&conn)?;
    let store = KnowledgeStore::new(&conn);
    let entity_id = store
        .find_entity_id_by_name(entity)?
        .with_context(|| format!("entity not found: {entity}"))?;

    let rows = list_entity_history(&conn, entity_id, limit)?;
    for row in rows {
        println!("{}\t{}\t{}", row.created_at, row.actor, row.operation);
    }
    Ok(())
}

fn handle_pipeline_enqueue(
    db: Utf8PathBuf,
    dedupe_key: &str,
    payload: Option<&str>,
    max_attempts: u32,
    print_schema: bool,
) -> Result<()> {
    if print_schema {
        println!("{}", InputSchemaKind::PipelinePayload.schema_text());
        return Ok(());
    }
    let conn = open_bootstrapped_db(&db)?;
    let payload = payload.with_context(|| {
        "missing --payload: provide escaped JSON or use --print-schema for schema output"
    })?;
    let value = validate_payload(payload, InputSchemaKind::PipelinePayload)
        .context("pipeline-enqueue payload failed schema validation")?;
    let raw_payload = value
        .get("payload")
        .and_then(|inner| inner.as_str())
        .with_context(|| "pipeline-enqueue payload must include string field 'payload'")?;
    let result = enqueue_job(&conn, raw_payload, dedupe_key, max_attempts)?;
    println!(
        "job_id={};state=queued;deduped={}",
        result.job_id, result.deduped
    );
    Ok(())
}

fn handle_pipeline_run_once(db: Utf8PathBuf) -> Result<()> {
    let conn = open_bootstrapped_db(&db)?;
    let provider = DisabledProvider;
    match run_once(&conn, &provider)? {
        Some(outcome) => {
            let error = outcome.error.unwrap_or_default();
            println!(
                "job_id={};state={};phase={};attempts={};error={}",
                outcome.job_id,
                match outcome.state {
                    knowledge_core::ingest::IngestState::Queued => "queued",
                    knowledge_core::ingest::IngestState::Processing => "processing",
                    knowledge_core::ingest::IngestState::Succeeded => "succeeded",
                    knowledge_core::ingest::IngestState::Failed => "failed",
                },
                match outcome.phase {
                    knowledge_core::ingest::IngestPhase::Parse => "parse",
                    knowledge_core::ingest::IngestPhase::Normalize => "normalize",
                    knowledge_core::ingest::IngestPhase::Classify => "classify",
                    knowledge_core::ingest::IngestPhase::Persist => "persist",
                },
                outcome.attempts,
                error
            );
        }
        None => println!("no_queued_jobs=true"),
    }
    Ok(())
}

fn handle_pipeline_status(db: Utf8PathBuf) -> Result<()> {
    let conn = open_bootstrapped_db(&db)?;
    let status = queue_status(&conn)?;
    println!(
        "queued={};processing={};failed={};unknown_aliases={};unknown_notes={}",
        status.queued,
        status.processing,
        status.failed,
        status.unknown_aliases,
        status.unknown_notes
    );
    Ok(())
}

#[derive(Debug)]
struct ScoredRecallRow {
    score: f32,
    entity_id: i64,
    canonical_name: String,
    kind: String,
}

fn handle_recall(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    query: &str,
    top_k: u32,
    config: &EffectiveConfig,
) -> Result<()> {
    let provider = require_enabled_embedding_provider(config)?;
    let conn = open_bootstrapped_db(&db)?;
    let store = KnowledgeStore::new(&conn);
    let notes = NoteStore::new(notes_root);
    let docs = store.recall_documents(&notes)?;
    if docs.is_empty() {
        println!("no_recall_documents=true");
        return Ok(());
    }

    let cache_model = embedding_cache_model(config);
    let query_embedding = provider.embed(query)?;
    let mut scored = Vec::with_capacity(docs.len());

    for doc in docs {
        let vector = load_or_compute_embedding(
            &conn,
            &*provider,
            doc.entity_id,
            &doc.text,
            &config.embeddings.provider,
            &cache_model,
        )?;
        let Some(score) = cosine_similarity(&query_embedding, &vector) else {
            continue;
        };
        scored.push(ScoredRecallRow {
            score,
            entity_id: doc.entity_id,
            canonical_name: doc.canonical_name,
            kind: doc.kind,
        });
    }

    scored.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then(left.canonical_name.cmp(&right.canonical_name))
            .then(left.entity_id.cmp(&right.entity_id))
    });
    scored.truncate(top_k as usize);

    for row in scored {
        println!(
            "{:.6}\t{}\t{}\t{}",
            row.score, row.entity_id, row.canonical_name, row.kind
        );
    }

    Ok(())
}

fn handle_embeddings_index(
    db: Utf8PathBuf,
    notes_root: Utf8PathBuf,
    mode: EmbedRefreshMode,
    config: &EffectiveConfig,
) -> Result<()> {
    let provider = require_enabled_embedding_provider(config)?;
    let conn = open_bootstrapped_db(&db)?;
    let store = KnowledgeStore::new(&conn);
    let notes = NoteStore::new(notes_root);
    let docs = store.recall_documents(&notes)?;
    if docs.is_empty() {
        println!("embedded=0;skipped=0;total=0");
        return Ok(());
    }

    let cache_model = embedding_cache_model(config);
    let mut embedded = 0_u64;
    let mut skipped = 0_u64;
    for doc in docs {
        if mode == EmbedRefreshMode::MissingOnly
            && has_cached_embedding(
                &conn,
                doc.entity_id,
                &config.embeddings.provider,
                &cache_model,
            )?
        {
            skipped += 1;
            continue;
        }
        let _ = load_or_compute_embedding(
            &conn,
            &*provider,
            doc.entity_id,
            &doc.text,
            &config.embeddings.provider,
            &cache_model,
        )?;
        embedded += 1;
    }

    println!(
        "embedded={embedded};skipped={skipped};total={}",
        embedded + skipped
    );
    Ok(())
}

fn maybe_refresh_embeddings_for_all(
    conn: &Connection,
    notes_root: Utf8PathBuf,
    config: &EffectiveConfig,
) -> Result<()> {
    if config.embeddings.provider.eq_ignore_ascii_case("none") {
        return Ok(());
    }
    let provider = build_embedding_provider(config)?;
    let store = KnowledgeStore::new(conn);
    let notes = NoteStore::new(notes_root);
    let docs = store.recall_documents(&notes)?;
    let cache_model = embedding_cache_model(config);
    for doc in docs {
        let _ = load_or_compute_embedding(
            conn,
            &*provider,
            doc.entity_id,
            &doc.text,
            &config.embeddings.provider,
            &cache_model,
        )?;
    }
    Ok(())
}

fn require_enabled_embedding_provider(
    config: &EffectiveConfig,
) -> Result<Box<dyn EmbeddingProvider>> {
    if config.embeddings.provider.eq_ignore_ascii_case("none") {
        anyhow::bail!(
            "embeddings provider is disabled; set --embeddings-provider openai-compatible or KNOWLEDGE_CLI_EMBEDDINGS_PROVIDER=openai-compatible"
        );
    }
    build_embedding_provider(config)
}

fn build_embedding_provider(config: &EffectiveConfig) -> Result<Box<dyn EmbeddingProvider>> {
    if config.embeddings.provider.eq_ignore_ascii_case("none") {
        return Ok(Box::new(DisabledEmbeddingProvider));
    }

    if config
        .embeddings
        .provider
        .eq_ignore_ascii_case("openai-compatible")
    {
        return Ok(Box::new(OpenAiCompatibleEmbeddingProvider::new(
            config.embeddings.base_url.clone(),
            config.embeddings.model.clone(),
            config.embeddings.timeout_ms,
            config.embeddings.dimensions,
        )));
    }

    anyhow::bail!(
        "unsupported embeddings provider: {} (supported: none, openai-compatible)",
        config.embeddings.provider
    )
}

fn embedding_cache_model(config: &EffectiveConfig) -> String {
    match config.embeddings.dimensions {
        Some(dimensions) => format!("{}#dimensions={dimensions}", config.embeddings.model),
        None => config.embeddings.model.clone(),
    }
}

fn load_or_compute_embedding(
    conn: &Connection,
    provider: &dyn EmbeddingProvider,
    entity_id: i64,
    text: &str,
    provider_name: &str,
    model: &str,
) -> Result<Vec<f32>> {
    let fingerprint = fingerprint_text(text);
    let cached = conn
        .query_row(
            "
            SELECT source_fingerprint, vector_json
            FROM entity_embeddings
            WHERE entity_id = ?1 AND provider = ?2 AND model = ?3
            LIMIT 1
            ",
            (entity_id, provider_name, model),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;

    if let Some((cached_fingerprint, vector_json)) = cached {
        if cached_fingerprint == fingerprint {
            let vector = serde_json::from_str::<Vec<f32>>(&vector_json)
                .context("failed to parse cached embedding vector JSON")?;
            return Ok(vector);
        }
    }

    let vector = provider.embed(text)?;
    let vector_json = serde_json::to_string(&vector).context("failed to serialize embedding")?;
    conn.execute(
        "
        INSERT INTO entity_embeddings (entity_id, provider, model, source_fingerprint, vector_json)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(entity_id, provider, model) DO UPDATE SET
            source_fingerprint = excluded.source_fingerprint,
            vector_json = excluded.vector_json,
            updated_at = CURRENT_TIMESTAMP
        ",
        (entity_id, provider_name, model, fingerprint, vector_json),
    )?;

    Ok(vector)
}

fn has_cached_embedding(
    conn: &Connection,
    entity_id: i64,
    provider_name: &str,
    model: &str,
) -> Result<bool> {
    let exists = conn
        .query_row(
            "
            SELECT 1
            FROM entity_embeddings
            WHERE entity_id = ?1 AND provider = ?2 AND model = ?3
            LIMIT 1
            ",
            (entity_id, provider_name, model),
            |row| row.get::<_, i64>(0),
        )
        .optional()?;
    Ok(exists.is_some())
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

fn print_version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}
