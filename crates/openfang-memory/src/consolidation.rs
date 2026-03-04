//! Memory consolidation and decay logic.
//!
//! Reduces confidence of old, unaccessed memories and merges
//! duplicate/similar memories.

use chrono::Utc;
use openfang_types::error::{OpenFangError, OpenFangResult};
use openfang_types::memory::ConsolidationReport;
use rusqlite::Connection;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Cosine similarity threshold above which two memories are considered duplicates.
const MERGE_THRESHOLD: f32 = 0.95;

/// Memory consolidation engine.
#[derive(Clone)]
pub struct ConsolidationEngine {
    conn: Arc<Mutex<Connection>>,
    /// Decay rate: how much to reduce confidence per consolidation cycle.
    decay_rate: f32,
}

impl ConsolidationEngine {
    /// Create a new consolidation engine.
    pub fn new(conn: Arc<Mutex<Connection>>, decay_rate: f32) -> Self {
        Self { conn, decay_rate }
    }

    /// Run a consolidation cycle: decay old memories and merge near-duplicates.
    pub fn consolidate(&self) -> OpenFangResult<ConsolidationReport> {
        let start = std::time::Instant::now();
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenFangError::Internal(e.to_string()))?;

        // ── Step 1: Decay confidence of memories not accessed in the last 7 days ──
        let cutoff = (Utc::now() - chrono::Duration::days(7)).to_rfc3339();
        let decay_factor = 1.0 - self.decay_rate as f64;

        let decayed = conn
            .execute(
                "UPDATE memories SET confidence = MAX(0.1, confidence * ?1)
                 WHERE deleted = 0 AND accessed_at < ?2 AND confidence > 0.1",
                rusqlite::params![decay_factor, cutoff],
            )
            .map_err(|e| OpenFangError::Memory(e.to_string()))?;

        // ── Step 2: Merge near-duplicate memories using cosine similarity ──
        // Load all non-deleted memories that have embeddings, oldest first so that
        // when a duplicate pair is found we always keep the earlier record.
        let rows: Vec<(String, Vec<u8>)> = {
            let mut stmt = conn
                .prepare(
                    "SELECT id, embedding FROM memories
                     WHERE deleted = 0 AND embedding IS NOT NULL
                     ORDER BY created_at ASC",
                )
                .map_err(|e| OpenFangError::Memory(e.to_string()))?;

            // Assign to `x` before the block closes so stmt's borrow is released
            // before stmt itself drops (avoids temporary-borrow-at-end-of-block issue).
            let x: Vec<(String, Vec<u8>)> = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
                })
                .map_err(|e| OpenFangError::Memory(e.to_string()))?
                .filter_map(|r| r.ok())
                .collect();
            x
        };

        // Decode BLOB → Vec<f32> (4-byte little-endian per value)
        let embeddings: Vec<(String, Vec<f32>)> = rows
            .into_iter()
            .map(|(id, bytes)| {
                let vec = bytes
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect::<Vec<f32>>();
                (id, vec)
            })
            .collect();

        // O(n²) pairwise similarity scan — fine for typical in-process memory sizes.
        // Keep the oldest record; soft-delete any later near-duplicate (sim ≥ threshold).
        let mut to_delete: HashSet<String> = HashSet::new();
        let n = embeddings.len();
        for i in 0..n {
            if to_delete.contains(&embeddings[i].0) {
                continue;
            }
            for j in (i + 1)..n {
                if to_delete.contains(&embeddings[j].0) {
                    continue;
                }
                if cosine_sim(&embeddings[i].1, &embeddings[j].1) >= MERGE_THRESHOLD {
                    to_delete.insert(embeddings[j].0.clone());
                }
            }
        }

        for id in &to_delete {
            let _ = conn.execute(
                "UPDATE memories SET deleted = 1 WHERE id = ?1",
                rusqlite::params![id],
            );
        }

        let memories_merged = to_delete.len() as u64;
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ConsolidationReport {
            memories_merged,
            memories_decayed: decayed as u64,
            duration_ms,
        })
    }
}

/// Cosine similarity between two f32 slices. Returns 0.0 for mismatched or empty inputs.
fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < f32::EPSILON {
        0.0
    } else {
        dot / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::run_migrations;

    fn setup() -> ConsolidationEngine {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        ConsolidationEngine::new(Arc::new(Mutex::new(conn)), 0.1)
    }

    #[test]
    fn test_consolidation_empty() {
        let engine = setup();
        let report = engine.consolidate().unwrap();
        assert_eq!(report.memories_decayed, 0);
    }

    #[test]
    fn test_consolidation_decays_old_memories() {
        let engine = setup();
        let conn = engine.conn.lock().unwrap();
        // Insert an old memory
        let old_date = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
        conn.execute(
            "INSERT INTO memories (id, agent_id, content, source, scope, confidence, metadata, created_at, accessed_at, access_count, deleted)
             VALUES ('test-id', 'agent-1', 'old memory', '\"conversation\"', 'episodic', 0.9, '{}', ?1, ?1, 0, 0)",
            rusqlite::params![old_date],
        ).unwrap();
        drop(conn);

        let report = engine.consolidate().unwrap();
        assert_eq!(report.memories_decayed, 1);

        // Verify confidence was reduced
        let conn = engine.conn.lock().unwrap();
        let confidence: f64 = conn
            .query_row(
                "SELECT confidence FROM memories WHERE id = 'test-id'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(confidence < 0.9);
    }

    #[test]
    fn test_merge_near_duplicates() {
        let engine = setup();
        let now = Utc::now().to_rfc3339();
        let later = (Utc::now() + chrono::Duration::seconds(10)).to_rfc3339();

        // Helper: pack f32 slice into little-endian bytes
        fn pack(vals: &[f32]) -> Vec<u8> {
            vals.iter().flat_map(|v| v.to_le_bytes()).collect()
        }

        // emb_a and emb_b are nearly identical (cosine ≈ 1.0); emb_c is orthogonal
        let emb_a = pack(&[0.6, 0.8]);
        let emb_b = pack(&[0.601, 0.799]);
        let emb_c = pack(&[0.0, 1.0]);

        let conn = engine.conn.lock().unwrap();
        let insert = "INSERT INTO memories (id, agent_id, content, source, scope, confidence, \
                      metadata, created_at, accessed_at, access_count, deleted, embedding) \
                      VALUES (?1, 'agent-1', ?2, '\"conversation\"', 'semantic', 1.0, '{}', ?3, ?3, 0, 0, ?4)";
        conn.execute(insert, rusqlite::params!["id-a", "fact A",           &now,   emb_a]).unwrap();
        conn.execute(insert, rusqlite::params!["id-b", "fact B (dup)",     &later, emb_b]).unwrap();
        conn.execute(insert, rusqlite::params!["id-c", "fact C (unique)",  &now,   emb_c]).unwrap();
        drop(conn);

        let report = engine.consolidate().unwrap();
        assert_eq!(report.memories_merged, 1, "exactly one duplicate merged");

        let conn = engine.conn.lock().unwrap();
        let alive: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT id FROM memories WHERE deleted = 0 ORDER BY id")
                .unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        assert!(alive.contains(&"id-a".to_string()), "older record kept");
        assert!(alive.contains(&"id-c".to_string()), "unique record kept");
        assert!(!alive.contains(&"id-b".to_string()), "duplicate deleted");
    }
}
