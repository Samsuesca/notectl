use rusqlite::{params, Connection, Result as SqlResult};

use crate::note::Note;
use crate::utils::timestamp_to_local;

pub fn search_notes(
    conn: &Connection,
    terms: &[String],
    tag: Option<&str>,
    case_sensitive: bool,
) -> SqlResult<Vec<Note>> {
    if let Some(t) = tag {
        return search_by_tag(conn, t);
    }

    if terms.is_empty() {
        return Ok(Vec::new());
    }

    // Use FTS5 for full-text search
    let fts_query = terms
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND ");

    let query = format!(
        "SELECT n.id, n.content, n.created_at, n.updated_at, n.category, n.is_daily \
         FROM notes n \
         JOIN notes_fts ON notes_fts.rowid = n.id \
         WHERE notes_fts MATCH ?1 \
         ORDER BY n.created_at DESC"
    );

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(params![fts_query], |row| {
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
    let mut tag_stmt = conn.prepare("SELECT tag FROM tags WHERE note_id = ?1")?;
    for row in rows {
        let (id, content, created_at, updated_at, category, is_daily) = row?;

        // Apply case-sensitive filtering if requested
        if case_sensitive {
            let all_match = terms.iter().all(|term| content.contains(term.as_str()));
            if !all_match {
                continue;
            }
        }

        let tags: Vec<String> = tag_stmt
            .query_map(params![id], |row| row.get(0))?
            .collect::<SqlResult<Vec<String>>>()?;

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

fn search_by_tag(conn: &Connection, tag: &str) -> SqlResult<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.content, n.created_at, n.updated_at, n.category, n.is_daily \
         FROM notes n \
         JOIN tags t ON t.note_id = n.id \
         WHERE t.tag = ?1 \
         ORDER BY n.created_at DESC",
    )?;

    let rows = stmt.query_map(params![tag], |row| {
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
    let mut tag_stmt = conn.prepare("SELECT tag FROM tags WHERE note_id = ?1")?;
    for row in rows {
        let (id, content, created_at, updated_at, category, is_daily) = row?;

        let tags: Vec<String> = tag_stmt
            .query_map(params![id], |row| row.get(0))?
            .collect::<SqlResult<Vec<String>>>()?;

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
