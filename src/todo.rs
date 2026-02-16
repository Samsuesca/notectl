use chrono::{DateTime, Local, NaiveDate};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::Serialize;

use crate::utils::timestamp_to_local;

#[derive(Debug, Serialize)]
pub struct Todo {
    pub id: i64,
    pub task: String,
    pub completed: bool,
    pub priority: String,
    pub due_date: Option<DateTime<Local>>,
    pub created_at: DateTime<Local>,
}

pub fn add(
    conn: &Connection,
    task: &str,
    priority: &str,
    due_date: Option<&str>,
) -> SqlResult<i64> {
    let now = Local::now().timestamp();

    let due_ts: Option<i64> = due_date.and_then(|d| {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .ok()
            .and_then(|nd| {
                nd.and_hms_opt(23, 59, 59)
                    .and_then(|ndt| ndt.and_local_timezone(Local).single())
                    .map(|dt| dt.timestamp())
            })
    });

    conn.execute(
        "INSERT INTO todos (task, priority, due_date, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![task, priority, due_ts, now],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn list_todos(conn: &Connection, pending_only: bool) -> SqlResult<Vec<Todo>> {
    let query = if pending_only {
        "SELECT id, task, completed, priority, due_date, created_at FROM todos WHERE completed = 0 ORDER BY \
         CASE priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END, \
         COALESCE(due_date, 9999999999) ASC"
    } else {
        "SELECT id, task, completed, priority, due_date, created_at FROM todos ORDER BY \
         CASE priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END, \
         COALESCE(due_date, 9999999999) ASC"
    };

    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map([], |row| {
        Ok(Todo {
            id: row.get(0)?,
            task: row.get(1)?,
            completed: row.get(2)?,
            priority: row.get::<_, String>(3)?,
            due_date: row
                .get::<_, Option<i64>>(4)?
                .map(|ts| timestamp_to_local(ts)),
            created_at: timestamp_to_local(row.get(5)?),
        })
    })?;

    rows.collect()
}

pub fn mark_done(conn: &Connection, id: i64) -> SqlResult<bool> {
    let affected = conn.execute(
        "UPDATE todos SET completed = 1 WHERE id = ?1",
        params![id],
    )?;
    Ok(affected > 0)
}

pub fn delete(conn: &Connection, id: i64) -> SqlResult<bool> {
    let affected = conn.execute("DELETE FROM todos WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

pub fn count_stats(conn: &Connection) -> SqlResult<(i64, i64, i64)> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))?;
    let completed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE completed = 1",
        [],
        |row| row.get(0),
    )?;
    let pending = total - completed;
    Ok((total, completed, pending))
}

pub fn count_overdue(conn: &Connection) -> SqlResult<i64> {
    let now = Local::now().timestamp();
    conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE completed = 0 AND due_date IS NOT NULL AND due_date < ?1",
        params![now],
        |row| row.get(0),
    )
}

pub fn count_due_today(conn: &Connection) -> SqlResult<i64> {
    let start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();
    let end = Local::now()
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();

    conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE completed = 0 AND due_date >= ?1 AND due_date <= ?2",
        params![start, end],
        |row| row.get(0),
    )
}
