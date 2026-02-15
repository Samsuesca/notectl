use chrono::{Local, TimeZone};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::Serialize;

use crate::note::Note;

#[derive(Serialize)]
struct ExportNote {
    id: i64,
    content: String,
    created_at: String,
    updated_at: String,
    category: Option<String>,
    tags: Vec<String>,
}

fn timestamp_to_local(ts: i64) -> chrono::DateTime<Local> {
    Local.timestamp_opt(ts, 0).single().unwrap_or_else(Local::now)
}

pub fn export_notes(
    conn: &Connection,
    format: &str,
    tag: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> SqlResult<String> {
    let notes = fetch_export_notes(conn, tag, from, to)?;

    match format {
        "json" => Ok(export_json(&notes)),
        "markdown" | "md" => Ok(export_markdown(&notes)),
        _ => Ok(export_markdown(&notes)),
    }
}

fn fetch_export_notes(
    conn: &Connection,
    tag: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> SqlResult<Vec<Note>> {
    let mut conditions = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(t) = tag {
        conditions.push(format!(
            "n.id IN (SELECT note_id FROM tags WHERE tag = ?{})",
            param_values.len() + 1
        ));
        param_values.push(Box::new(t.to_string()));
    }

    if let Some(f) = from {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(f, "%Y-%m-%d") {
            let ts = date
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp();
            conditions.push(format!("n.created_at >= ?{}", param_values.len() + 1));
            param_values.push(Box::new(ts));
        }
    }

    if let Some(t) = to {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d") {
            let ts = date
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp();
            conditions.push(format!("n.created_at <= ?{}", param_values.len() + 1));
            param_values.push(Box::new(ts));
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT n.id, n.content, n.created_at, n.updated_at, n.category, n.is_daily \
         FROM notes n {} ORDER BY n.created_at DESC",
        where_clause
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(params_ref.as_slice(), |row| {
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
    for row in rows {
        let (id, content, created_at, updated_at, category, is_daily) = row?;

        let mut tag_stmt = conn.prepare("SELECT tag FROM tags WHERE note_id = ?1")?;
        let tags: Vec<String> = tag_stmt
            .query_map(params![id], |r| r.get(0))?
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

fn export_json(notes: &[Note]) -> String {
    let export_notes: Vec<ExportNote> = notes
        .iter()
        .map(|n| ExportNote {
            id: n.id,
            content: n.content.clone(),
            created_at: n.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: n.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            category: n.category.clone(),
            tags: n.tags.clone(),
        })
        .collect();

    serde_json::to_string_pretty(&export_notes).unwrap_or_else(|_| "[]".to_string())
}

fn export_markdown(notes: &[Note]) -> String {
    let mut md = String::from("# Notes Export\n\n");
    md.push_str(&format!(
        "Exported: {}\n\n---\n\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    for note in notes {
        md.push_str(&format!("## Note #{}\n\n", note.id));
        md.push_str(&format!(
            "**Date**: {}\n\n",
            note.created_at.format("%Y-%m-%d %H:%M")
        ));
        if let Some(ref cat) = note.category {
            md.push_str(&format!("**Category**: {}\n\n", cat));
        }
        if !note.tags.is_empty() {
            md.push_str(&format!("**Tags**: {}\n\n", note.tags.join(", ")));
        }
        md.push_str(&note.content);
        md.push_str("\n\n---\n\n");
    }

    md
}
