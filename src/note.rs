use chrono::{DateTime, Local, TimeZone};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Note {
    pub id: i64,
    pub content: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub category: Option<String>,
    pub is_daily: bool,
    pub tags: Vec<String>,
}

fn timestamp_to_local(ts: i64) -> DateTime<Local> {
    Local.timestamp_opt(ts, 0).single().unwrap_or_else(Local::now)
}

pub fn add(
    conn: &Connection,
    content: &str,
    tags: &[String],
    category: Option<&str>,
    is_daily: bool,
) -> SqlResult<i64> {
    let now = Local::now().timestamp();

    conn.execute(
        "INSERT INTO notes (content, created_at, updated_at, category, is_daily) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![content, now, now, category, is_daily],
    )?;

    let note_id = conn.last_insert_rowid();

    // Insert into FTS index
    conn.execute(
        "INSERT INTO notes_fts (rowid, content) VALUES (?1, ?2)",
        params![note_id, content],
    )?;

    // Insert tags
    for tag in tags {
        conn.execute(
            "INSERT INTO tags (note_id, tag) VALUES (?1, ?2)",
            params![note_id, tag.trim()],
        )?;
    }

    Ok(note_id)
}

pub fn list(
    conn: &Connection,
    limit: usize,
    tag: Option<&str>,
    category: Option<&str>,
    today_only: bool,
) -> SqlResult<Vec<Note>> {
    let mut conditions = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if today_only {
        let start_of_day = Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp();
        conditions.push(format!("n.created_at >= ?{}", param_values.len() + 1));
        param_values.push(Box::new(start_of_day));
    }

    if let Some(cat) = category {
        conditions.push(format!("n.category = ?{}", param_values.len() + 1));
        param_values.push(Box::new(cat.to_string()));
    }

    if let Some(t) = tag {
        conditions.push(format!(
            "n.id IN (SELECT note_id FROM tags WHERE tag = ?{})",
            param_values.len() + 1
        ));
        param_values.push(Box::new(t.to_string()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT n.id, n.content, n.created_at, n.updated_at, n.category, n.is_daily \
         FROM notes n {} ORDER BY n.created_at DESC LIMIT ?{}",
        where_clause,
        param_values.len() + 1
    );

    param_values.push(Box::new(limit as i64));

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&query)?;
    let note_rows = stmt.query_map(params_ref.as_slice(), |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, bool>(5)?,
        ))
    })?;

    let mut notes = Vec::new();
    for row in note_rows {
        let (id, content, created_at, updated_at, category, is_daily) = row?;

        let tags = get_tags_for_note(conn, id)?;

        notes.push(Note {
            id,
            content,
            created_at: timestamp_to_local(created_at),
            updated_at: timestamp_to_local(updated_at),
            category,
            is_daily,
            tags,
        });
    }

    Ok(notes)
}

pub fn get_by_id(conn: &Connection, id: i64) -> SqlResult<Option<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, created_at, updated_at, category, is_daily FROM notes WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        let note_id: i64 = row.get(0)?;
        let tags = get_tags_for_note(conn, note_id)?;
        Ok(Some(Note {
            id: note_id,
            content: row.get(1)?,
            created_at: timestamp_to_local(row.get(2)?),
            updated_at: timestamp_to_local(row.get(3)?),
            category: row.get(4)?,
            is_daily: row.get(5)?,
            tags,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete(conn: &Connection, id: i64) -> SqlResult<bool> {
    conn.execute("DELETE FROM notes_fts WHERE rowid = ?1", params![id])?;
    conn.execute("DELETE FROM tags WHERE note_id = ?1", params![id])?;
    let affected = conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

pub fn update(conn: &Connection, id: i64, content: &str) -> SqlResult<bool> {
    let now = Local::now().timestamp();
    let affected = conn.execute(
        "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
        params![content, now, id],
    )?;
    if affected > 0 {
        // Update FTS
        conn.execute("DELETE FROM notes_fts WHERE rowid = ?1", params![id])?;
        conn.execute(
            "INSERT INTO notes_fts (rowid, content) VALUES (?1, ?2)",
            params![id, content],
        )?;
    }
    Ok(affected > 0)
}

fn get_tags_for_note(conn: &Connection, note_id: i64) -> SqlResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT tag FROM tags WHERE note_id = ?1")?;
    let tags = stmt
        .query_map(params![note_id], |row| row.get(0))?
        .collect::<SqlResult<Vec<String>>>()?;
    Ok(tags)
}

pub fn count_all(conn: &Connection) -> SqlResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
}
