use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashMap;

/// Score components for deterministic hybrid ranking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreParts {
    pub exact: i64,
    pub alias: i64,
    pub fts: i64,
    pub graph: i64,
    pub recency: i64,
    pub name_penalty: i64,
}

/// One ranked recall result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallResult {
    pub canonical_name: String,
    pub total_score: i64,
    pub score_parts: ScoreParts,
}

/// Runtime knobs for hybrid recall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallOptions {
    pub top_k: u32,
    pub recency_weight: i64,
    pub namespace_diversity_cap: Option<usize>,
}

impl RecallOptions {
    /// Creates default deterministic options.
    pub fn defaults(top_k: u32) -> Self {
        Self {
            top_k,
            recency_weight: 0,
            namespace_diversity_cap: None,
        }
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    canonical_name: String,
    namespace: Option<String>,
    package_name: Option<String>,
    repo_name: Option<String>,
    aliases: Vec<String>,
    graph_degree: i64,
    updated_at: String,
}

/// Deterministic hybrid recall with default options.
pub fn recall(conn: &Connection, query: &str, top_k: u32) -> Result<Vec<RecallResult>> {
    recall_with_options(conn, query, &RecallOptions::defaults(top_k))
}

/// Deterministic hybrid recall with explicit options.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection.
/// * `query` - User recall query.
/// * `options` - Ranking and filtering options.
///
/// # Returns
///
/// Ranked recall results.
///
/// # Errors
///
/// Returns an error when SQL operations fail.
pub fn recall_with_options(
    conn: &Connection,
    query: &str,
    options: &RecallOptions,
) -> Result<Vec<RecallResult>> {
    let normalized = query.trim().to_lowercase();
    if normalized.is_empty() || options.top_k == 0 {
        return Ok(Vec::new());
    }

    let candidates = load_candidates(conn, &normalized)?;
    let recency_by_name = build_recency_scores(&candidates, options.recency_weight);

    let mut ranked: Vec<(RecallResult, Option<String>)> = candidates
        .into_iter()
        .map(|candidate| {
            let recency = *recency_by_name.get(&candidate.canonical_name).unwrap_or(&0);
            (
                score_candidate(&candidate, &normalized, recency),
                candidate.namespace,
            )
        })
        .filter(|(result, _)| result.total_score > 0)
        .collect();

    ranked.sort_by(|(a, _), (b, _)| {
        b.total_score
            .cmp(&a.total_score)
            .then_with(|| a.canonical_name.cmp(&b.canonical_name))
    });

    let selected =
        apply_namespace_diversity(ranked, options.namespace_diversity_cap, options.top_k);
    persist_telemetry(conn, query, &selected)?;
    Ok(selected)
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
            ) AS degree,
            e.updated_at
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
            updated_at: row.get(6)?,
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

fn build_recency_scores(candidates: &[Candidate], recency_weight: i64) -> HashMap<String, i64> {
    if recency_weight <= 0 {
        return HashMap::new();
    }

    let mut ordered: Vec<&Candidate> = candidates.iter().collect();
    ordered.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| a.canonical_name.cmp(&b.canonical_name))
    });

    let mut map: HashMap<String, i64> = HashMap::with_capacity(ordered.len());
    let max = ordered.len() as i64;
    for (idx, candidate) in ordered.into_iter().enumerate() {
        let rank_bonus = (max - (idx as i64)).max(1);
        map.insert(candidate.canonical_name.clone(), rank_bonus * recency_weight);
    }
    map
}

fn score_candidate(candidate: &Candidate, normalized_query: &str, recency: i64) -> RecallResult {
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
        recency,
        name_penalty,
    };

    RecallResult {
        canonical_name: candidate.canonical_name.clone(),
        total_score: total_score(&parts),
        score_parts: parts,
    }
}

fn apply_namespace_diversity(
    ranked: Vec<(RecallResult, Option<String>)>,
    cap: Option<usize>,
    top_k: u32,
) -> Vec<RecallResult> {
    if cap.is_none() {
        return ranked
            .into_iter()
            .map(|(result, _)| result)
            .take(top_k as usize)
            .collect();
    }

    let max_per_ns = cap.unwrap_or(usize::MAX);
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::with_capacity(top_k as usize);

    for (result, namespace) in ranked {
        let key = namespace.unwrap_or_else(|| "__none__".to_string());
        let count = counts.entry(key).or_insert(0);
        if *count >= max_per_ns {
            continue;
        }
        *count += 1;
        out.push(result);
        if out.len() >= top_k as usize {
            break;
        }
    }

    out
}

fn field_contains(value: &str, normalized_query: &str) -> i64 {
    if value.contains(normalized_query) {
        1
    } else {
        0
    }
}

fn total_score(s: &ScoreParts) -> i64 {
    s.exact * 1000 + s.alias * 500 + s.fts * 100 + s.graph * 10 + s.recency - s.name_penalty
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
                recency_score,
                selected_entity
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            params![
                query,
                "hybrid",
                result.total_score,
                result.score_parts.exact,
                result.score_parts.alias,
                result.score_parts.fts,
                result.score_parts.graph,
                result.score_parts.recency,
                result.canonical_name,
            ],
        )?;
    }
    Ok(())
}
