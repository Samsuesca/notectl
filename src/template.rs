use chrono::Local;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Template {
    pub name: String,
    pub content: String,
}

pub fn create(conn: &Connection, name: &str, content: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO templates (name, content) VALUES (?1, ?2)",
        params![name, content],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, name: &str) -> SqlResult<Option<Template>> {
    let mut stmt = conn.prepare("SELECT name, content FROM templates WHERE name = ?1")?;
    let mut rows = stmt.query(params![name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Template {
            name: row.get(0)?,
            content: row.get(1)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_all(conn: &Connection) -> SqlResult<Vec<Template>> {
    let mut stmt = conn.prepare("SELECT name, content FROM templates ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Template {
            name: row.get(0)?,
            content: row.get(1)?,
        })
    })?;
    rows.collect()
}

pub fn delete(conn: &Connection, name: &str) -> SqlResult<bool> {
    let affected = conn.execute("DELETE FROM templates WHERE name = ?1", params![name])?;
    Ok(affected > 0)
}

pub fn render(template_content: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template_content.to_string();

    // Replace built-in variables
    let now = Local::now();
    result = result.replace("{date}", &now.format("%Y-%m-%d").to_string());
    result = result.replace("{time}", &now.format("%H:%M").to_string());
    result = result.replace("{datetime}", &now.format("%Y-%m-%d %H:%M").to_string());

    // Replace custom variables
    for (key, value) in vars {
        result = result.replace(&format!("{{{}}}", key), value);
    }

    result
}
