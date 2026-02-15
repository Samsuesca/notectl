use rusqlite::{params, Connection, Result as SqlResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TagCount {
    pub tag: String,
    pub count: i64,
}

pub fn list_all(conn: &Connection) -> SqlResult<Vec<TagCount>> {
    let mut stmt = conn.prepare(
        "SELECT tag, COUNT(*) as cnt FROM tags GROUP BY tag ORDER BY cnt DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(TagCount {
            tag: row.get(0)?,
            count: row.get(1)?,
        })
    })?;

    rows.collect()
}

pub fn rename(conn: &Connection, old_name: &str, new_name: &str) -> SqlResult<usize> {
    let affected = conn.execute(
        "UPDATE tags SET tag = ?1 WHERE tag = ?2",
        params![new_name, old_name],
    )?;
    Ok(affected)
}

pub fn add_tag(conn: &Connection, note_id: i64, tag: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO tags (note_id, tag) VALUES (?1, ?2)",
        params![note_id, tag],
    )?;
    Ok(())
}

pub fn remove_tag(conn: &Connection, note_id: i64, tag: &str) -> SqlResult<usize> {
    let affected = conn.execute(
        "DELETE FROM tags WHERE note_id = ?1 AND tag = ?2",
        params![note_id, tag],
    )?;
    Ok(affected)
}
