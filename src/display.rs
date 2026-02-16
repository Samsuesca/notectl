use chrono::{DateTime, Local};
use colored::*;
use tabled::{
    settings::{Style, Modify, object::Columns, Width},
    Table, Tabled,
};

use crate::note::Note;
use crate::tags::TagCount;
use crate::todo::Todo;

#[derive(Tabled)]
struct NoteRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Content")]
    content: String,
    #[tabled(rename = "Tags")]
    tags: String,
}

#[derive(Tabled)]
struct TodoRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Task")]
    task: String,
    #[tabled(rename = "Priority")]
    priority: String,
    #[tabled(rename = "Due")]
    due: String,
    #[tabled(rename = "Status")]
    status: String,
}

#[derive(Tabled)]
struct TagRow {
    #[tabled(rename = "Tag")]
    tag: String,
    #[tabled(rename = "Count")]
    count: i64,
}

fn relative_time(dt: &DateTime<Local>) -> String {
    let now = Local::now();
    let diff = now.signed_duration_since(*dt);

    let minutes = diff.num_minutes();
    let hours = diff.num_hours();
    let days = diff.num_days();

    let time_str = dt.format("%H:%M").to_string();

    if minutes < 1 {
        format!("{} (just now)", time_str)
    } else if minutes < 60 {
        format!("{} ({} min ago)", time_str, minutes)
    } else if hours < 24 {
        format!("{} ({} hour{} ago)", time_str, hours, if hours == 1 { "" } else { "s" })
    } else {
        format!("{} ({} day{} ago)", dt.format("%Y-%m-%d %H:%M"), days, if days == 1 { "" } else { "s" })
    }
}

pub fn print_note_added(id: i64, content: &str) {
    let now = Local::now();
    println!(
        "{} Note added (ID: {})",
        "✓".green().bold(),
        id.to_string().cyan()
    );
    println!("  \"{}\"", truncate(content, 60));
    println!(
        "  Created: {}",
        now.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
    );
}

pub fn print_notes_table(notes: &[Note], title: &str) {
    if notes.is_empty() {
        println!("{}", "No notes found.".dimmed());
        return;
    }

    println!("{}:\n", title.bold());

    let rows: Vec<NoteRow> = notes
        .iter()
        .map(|n| NoteRow {
            id: n.id,
            time: relative_time(&n.created_at),
            content: truncate(&n.content, 40),
            tags: n.tags.join(", "),
        })
        .collect();

    let table = Table::new(rows)
        .with(Style::rounded())
        .with(Modify::new(Columns::single(2)).with(Width::truncate(40).suffix("...")))
        .to_string();

    println!("{}", table);
}

pub fn print_search_results(notes: &[Note], query: &str, full: bool) {
    println!(
        "{}: \"{}\"\n",
        "Search Results".bold(),
        query.yellow()
    );
    println!("Found {} note{}:\n", notes.len(), if notes.len() == 1 { "" } else { "s" });

    for note in notes {
        println!(
            "{} {} {}",
            format!("[{}]", note.id).cyan(),
            note.created_at.format("%Y-%m-%d %H:%M").to_string().dimmed(),
            ""
        );
        if full {
            println!("  {}", note.content);
        } else {
            println!("  {}", truncate(&note.content, 70));
        }
        if !note.tags.is_empty() {
            println!("  Tags: {}", note.tags.join(", ").dimmed());
        }
        println!();
    }
}

pub fn print_todos_table(todos: &[Todo]) {
    if todos.is_empty() {
        println!("{}", "No TODOs found.".dimmed());
        return;
    }

    println!("{}\n", "Active TODOs:".bold());

    let rows: Vec<TodoRow> = todos
        .iter()
        .map(|t| {
            let priority_display = match t.priority.as_str() {
                "high" => "High".red().to_string(),
                "low" => "Low".green().to_string(),
                _ => "Med".yellow().to_string(),
            };

            let due_display = match &t.due_date {
                Some(dt) => {
                    let today = Local::now().date_naive();
                    let due_day = dt.date_naive();
                    if due_day == today {
                        "Today".red().to_string()
                    } else if due_day < today {
                        format!("{} (overdue)", dt.format("%b %-d")).red().to_string()
                    } else {
                        dt.format("%b %-d").to_string()
                    }
                }
                None => "-".dimmed().to_string(),
            };

            let status = if t.completed {
                "Done".green().to_string()
            } else {
                "Pending".dimmed().to_string()
            };

            TodoRow {
                id: t.id,
                task: truncate(&t.task, 35),
                priority: priority_display,
                due: due_display,
                status,
            }
        })
        .collect();

    let table = Table::new(rows)
        .with(Style::rounded())
        .to_string();

    println!("{}", table);
}

pub fn print_todo_summary(overdue: i64, due_today: i64) {
    println!(
        "\nOverdue: {} | Due today: {}",
        if overdue > 0 {
            overdue.to_string().red().to_string()
        } else {
            "0".to_string()
        },
        if due_today > 0 {
            due_today.to_string().yellow().to_string()
        } else {
            "0".to_string()
        },
    );
}

pub fn print_tags_table(tags: &[TagCount]) {
    if tags.is_empty() {
        println!("{}", "No tags found.".dimmed());
        return;
    }

    println!("{}\n", "All Tags:".bold());

    let rows: Vec<TagRow> = tags
        .iter()
        .map(|t| TagRow {
            tag: t.tag.clone(),
            count: t.count,
        })
        .collect();

    let table = Table::new(rows)
        .with(Style::rounded())
        .to_string();

    println!("{}", table);
}

pub fn print_note_deleted(id: i64) {
    println!("{} Note {} deleted", "✓".green().bold(), id.to_string().cyan());
}

pub fn print_todo_added(id: i64, task: &str) {
    println!(
        "{} TODO added (ID: {})",
        "✓".green().bold(),
        id.to_string().cyan()
    );
    println!("  \"{}\"", truncate(task, 60));
}

pub fn print_todo_done(id: i64) {
    println!(
        "{} TODO {} marked as done",
        "✓".green().bold(),
        id.to_string().cyan()
    );
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
}

fn truncate(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}
