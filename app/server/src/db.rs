//! Progress storage (SQLite) and scoring.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Progress {
    pub completed: bool,
    pub score: Option<i64>,
    pub failed_submits: i64,
}

pub fn score_for(failed_submits: i64) -> i64 {
    (100 - 10 * failed_submits.max(0)).max(50)
}

/// Applies one submit result. Completed exercises never change; a failed submit
/// (including timeout / compile error) increments `failed_submits`; a passing one
/// completes the exercise with `max(100 - 10 * failed_submits, 50)`.
pub fn apply_submit(prev: Progress, passed: bool) -> Progress {
    if prev.completed {
        return prev;
    }
    if passed {
        Progress {
            completed: true,
            score: Some(score_for(prev.failed_submits)),
            failed_submits: prev.failed_submits,
        }
    } else {
        Progress {
            completed: false,
            score: None,
            failed_submits: prev.failed_submits + 1,
        }
    }
}

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

pub struct SubmitRecord<'a> {
    pub exercise_id: &'a str,
    pub passed: bool,
    pub status: &'a str,
    pub tests_passed: usize,
    pub tests_total: usize,
}

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Db> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Db> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Db> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             CREATE TABLE IF NOT EXISTS progress (
                 exercise_id    TEXT PRIMARY KEY,
                 completed_at   TEXT,
                 score          INTEGER,
                 failed_submits INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS submissions (
                 id           INTEGER PRIMARY KEY AUTOINCREMENT,
                 exercise_id  TEXT NOT NULL,
                 ts           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                 passed       INTEGER NOT NULL,
                 status       TEXT NOT NULL,
                 tests_passed INTEGER NOT NULL,
                 tests_total  INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS submissions_exercise ON submissions(exercise_id);",
        )?;
        Ok(Db {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get(&self, exercise_id: &str) -> rusqlite::Result<Progress> {
        let conn = self.lock();
        Self::get_with(&conn, exercise_id)
    }

    fn get_with(conn: &Connection, exercise_id: &str) -> rusqlite::Result<Progress> {
        Ok(conn
            .query_row(
                "SELECT completed_at IS NOT NULL, score, failed_submits FROM progress WHERE exercise_id = ?1",
                params![exercise_id],
                |r| {
                    Ok(Progress {
                        completed: r.get(0)?,
                        score: r.get(1)?,
                        failed_submits: r.get(2)?,
                    })
                },
            )
            .optional()?
            .unwrap_or_default())
    }

    pub fn all(&self) -> rusqlite::Result<HashMap<String, Progress>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT exercise_id, completed_at IS NOT NULL, score, failed_submits FROM progress",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                Progress {
                    completed: r.get(1)?,
                    score: r.get(2)?,
                    failed_submits: r.get(3)?,
                },
            ))
        })?;
        rows.collect()
    }

    /// Records a submission and updates progress atomically. Returns the new progress.
    pub fn record_submit(&self, rec: &SubmitRecord<'_>) -> rusqlite::Result<Progress> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO submissions (exercise_id, passed, status, tests_passed, tests_total)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                rec.exercise_id,
                rec.passed,
                rec.status,
                rec.tests_passed as i64,
                rec.tests_total as i64
            ],
        )?;
        let prev = Self::get_with(&tx, rec.exercise_id)?;
        let next = apply_submit(prev, rec.passed);
        if next != prev {
            tx.execute(
                "INSERT INTO progress (exercise_id, completed_at, score, failed_submits)
                 VALUES (?1, CASE WHEN ?2 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') END, ?3, ?4)
                 ON CONFLICT(exercise_id) DO UPDATE SET
                     completed_at = excluded.completed_at,
                     score = excluded.score,
                     failed_submits = excluded.failed_submits",
                params![
                    rec.exercise_id,
                    next.completed,
                    next.score,
                    next.failed_submits
                ],
            )?;
        }
        tx.commit()?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_formula() {
        assert_eq!(score_for(0), 100);
        assert_eq!(score_for(1), 90);
        assert_eq!(score_for(5), 50);
        assert_eq!(score_for(9), 50);
    }

    #[test]
    fn submit_transitions() {
        let p = Progress::default();
        let p = apply_submit(p, false);
        assert_eq!(
            p,
            Progress {
                completed: false,
                score: None,
                failed_submits: 1
            }
        );
        let p = apply_submit(p, true);
        assert_eq!(
            p,
            Progress {
                completed: true,
                score: Some(90),
                failed_submits: 1
            }
        );
        // Completed is frozen.
        assert_eq!(apply_submit(p, false), p);
        assert_eq!(apply_submit(p, true), p);
    }

    #[test]
    fn db_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        let id = "02_basic_calculator/04_panics";
        assert_eq!(db.get(id).unwrap(), Progress::default());
        let rec = |passed| SubmitRecord {
            exercise_id: id,
            passed,
            status: if passed { "passed" } else { "failed" },
            tests_passed: 1,
            tests_total: 2,
        };
        db.record_submit(&rec(false)).unwrap();
        db.record_submit(&rec(false)).unwrap();
        let p = db.record_submit(&rec(true)).unwrap();
        assert_eq!(
            p,
            Progress {
                completed: true,
                score: Some(80),
                failed_submits: 2
            }
        );
        let p = db.record_submit(&rec(false)).unwrap();
        assert_eq!(p.score, Some(80));
        assert_eq!(db.get(id).unwrap(), p);
        assert_eq!(db.all().unwrap().len(), 1);
        let n: i64 = db
            .lock()
            .query_row("SELECT COUNT(*) FROM submissions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 4);
    }
}
