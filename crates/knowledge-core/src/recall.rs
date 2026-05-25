use anyhow::Result;
use rusqlite::{params, Connection};

/// Score components for deterministic hybrid ranking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreParts {
    pub exact: i64,
    pub alias: i64,
    pub fts: i64,
    pub graph: i64,
    pub name_penalty: i64,
}

/// One ranked recall result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallResult {
    pub canonical_name: String,
    pub total_score: i64,
    pub score_parts: ScoreParts,
}

#[derive(Debug, Clone)]
struct Candidate {
    canonical_name: String,
    namespace: Option<String>,
    package_name: Option<String>,
    repo_name: Option<String>,
    aliases: Vec<String>,
    graph_degree: i64,
}

/// Deterministic hybrid recall.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection.
/// * `query` - User recall query.
/// * `top_k` - Maximum results.
///
/// # Returns
///
/// Ranked recall results.
///
/// # Errors
///
/// Returns an error when SQL operations fail.
pub fn recall(conn: &Connection, query: &str, top_k: u32) -> Result<Vec<RecallResult>> {
    let normalized = query.trim().to_lowercase();
    if normalized.is_empty() || top_k == 0 {
        return Ok(Vec::new());
    }

    let candidates = load_candidates(conn, &normalized)?;
    let mut ranked: Vec<RecallResult> = candidates
        .into_iter()
        .map(|candidate| score_candidate(&candidate, &normalized))
        .filter(|result| result.total_score > 0)
        .collect();

    ranked.sort_by(|a, b| {
        b.total_score
            .cmp(&a.total_score)
            .then_with(|| a.canonical_name.cmp(&b.canonical_name))
    });
    ranked.truncate(top_k as usize);

    persist_telemetry(conn, query, &ranked)?;
    Ok(ranked)
}

fn load_candidates(conn: &Connection, normalized_query: &str) -> Result<Vec<Candidate>> {
    let pattern = format!("%{normalized_query}%");
    let mut stmt = conn.prepare(
        "
        SELECT DISTINCT
            e.id,
            e.canonical_name,
            e.namespace,
            e.package_name,
            e.repo_name,
            (
                SELECT COUNT(*)
                FROM relationships r
                WHERE r.from_entity_id = e.id OR r.to_entity_id = e.id
            ) AS degree
        FROM entities e
        LEFT JOIN aliases a ON a.entity_id = e.id
        WHERE
            lower(e.canonical_name) LIKE ?1
            OR lower(COALESCE(e.namespace, '')) LIKE ?1
            OR lower(COALESCE(e.package_name, '')) LIKE ?1
            OR lower(COALESCE(e.repo_name, '')) LIKE ?1
            OR lower(COALESCE(a.alias, '')) LIKE ?1
        ",
    )?;

    let mut rows = stmt.query([pattern])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        out.push(Candidate {
            canonical_name: row.get(1)?,
            namespace: row.get(2)?,
            package_name: row.get(3)?,
            repo_name: row.get(4)?,
            aliases: load_aliases(conn, id)?,
            graph_degree: row.get(5)?,
        });
    }
    Ok(out)
}

fn load_aliases(conn: &Connection, entity_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT alias FROM aliases WHERE entity_id = ?1")?;
    let aliases = stmt
        .query_map([entity_id], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(aliases)
}

fn score_candidate(candidate: &Candidate, normalized_query: &str) -> RecallResult {
    let canonical = candidate.canonical_name.to_lowercase();
    let exact = i64::from(canonical == normalized_query);

    let alias = i64::from(
        candidate
            .aliases
            .iter()
            .any(|a| a.to_lowercase() == normalized_query),
    );

    let fts = field_contains(&canonical, normalized_query)
        + candidate
            .namespace
            .as_deref()
            .map_or(0, |v| field_contains(&v.to_lowercase(), normalized_query))
        + candidate
            .package_name
            .as_deref()
            .map_or(0, |v| field_contains(&v.to_lowercase(), normalized_query))
        + candidate
            .repo_name
            .as_deref()
            .map_or(0, |v| field_contains(&v.to_lowercase(), normalized_query))
        + candidate
            .aliases
            .iter()
            .map(|v| field_contains(&v.to_lowercase(), normalized_query))
            .sum::<i64>();

    let graph = candidate.graph_degree.min(10);
    let name_penalty = (candidate.canonical_name.len() as i64) / 20;

    let parts = ScoreParts {
        exact,
        alias,
        fts,
        graph,
        name_penalty,
    };

    RecallResult {
        canonical_name: candidate.canonical_name.clone(),
        total_score: total_score(&parts),
        score_parts: parts,
    }
}

fn field_contains(value: &str, normalized_query: &str) -> i64 {
    if value.contains(normalized_query) {
        1
    } else {
        0
    }
}

fn total_score(s: &ScoreParts) -> i64 {
    s.exact * 1000 + s.alias * 500 + s.fts * 100 + s.graph * 10 - s.name_penalty
}

fn persist_telemetry(conn: &Connection, query: &str, ranked: &[RecallResult]) -> Result<()> {
    for result in ranked {
        conn.execute(
            "
            INSERT INTO retrieval_telemetry (
                query,
                match_source,
                total_score,
                exact_score,
                alias_score,
                fts_score,
                graph_score,
                selected_entity
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                query,
                "hybrid",
                result.total_score,
                result.score_parts.exact,
                result.score_parts.alias,
                result.score_parts.fts,
                result.score_parts.graph,
                result.canonical_name,
            ],
        )?;
    }
    Ok(())
}
