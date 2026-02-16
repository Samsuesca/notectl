mod db;
mod display;
mod export;
mod note;
mod search;
mod tags;
mod template;
mod todo;
pub mod utils;

use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read};
use std::process::Command;

/// Lightning-fast note-taking and task management CLI
#[derive(Parser)]
#[command(name = "notectl", version, about, long_about = None, after_help = "\
Quick start:
  notectl add \"My first note\" --tags work,meeting
  notectl list --today
  notectl search \"meeting notes\"
  notectl todo add \"Review PR\" --priority high --due 2026-03-01
  notectl tags
  notectl daily
  notectl export --format markdown --output notes.md
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new note
    #[command(long_about = "\
Add a new note with optional tags and category.
Content can be provided as a positional argument or piped via stdin.

Examples:
  notectl add \"Quick meeting note\"
  notectl add \"Project idea\" --tags work,ideas --category projects
  echo \"Piped content\" | notectl add --stdin
  notectl add \"Shopping list\" --tags personal
  notectl add \"Bug report: login fails on Safari\" --tags bugs,frontend --category engineering")]
    Add {
        /// Note content (omit to use stdin with --stdin)
        content: Option<String>,

        /// Comma-separated tags
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Category for the note
        #[arg(long)]
        category: Option<String>,

        /// Read content from stdin
        #[arg(long)]
        stdin: bool,
    },

    /// List recent notes
    #[command(long_about = "\
List recent notes with optional filters for date, tag, and category.
By default shows the 10 most recent notes.

Examples:
  notectl list
  notectl list --today
  notectl list --tag work --limit 20
  notectl list --category projects
  notectl list --tag meeting --today")]
    List {
        /// Show only today's notes
        #[arg(long)]
        today: bool,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Maximum number of notes to show
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Search notes by keyword
    #[command(long_about = "\
Search notes using full-text search (FTS5) or filter by tag.
Multiple search terms are combined with AND logic.

Examples:
  notectl search meeting
  notectl search \"project update\" --full
  notectl search --tag work
  notectl search API design --case-sensitive
  notectl search deploy production --full")]
    Search {
        /// Search terms
        terms: Vec<String>,

        /// Search by tag instead
        #[arg(long)]
        tag: Option<String>,

        /// Case-sensitive search
        #[arg(long)]
        case_sensitive: bool,

        /// Show full content
        #[arg(long)]
        full: bool,
    },

    /// Show or edit a specific note
    #[command(long_about = "\
Display the full content of a note by its ID, including metadata.

Examples:
  notectl show 1
  notectl show 42
  notectl show 7")]
    Show {
        /// Note ID
        id: i64,
    },

    /// Edit a note's content
    #[command(long_about = "\
Open a note in your $EDITOR for editing. The note content is loaded
into a temporary file, and saved back after the editor exits.

Examples:
  notectl edit 1
  notectl edit 42
  EDITOR=nano notectl edit 5")]
    Edit {
        /// Note ID
        id: i64,
    },

    /// Delete a note
    #[command(long_about = "\
Permanently delete a note and its associated tags and FTS index entry.

Examples:
  notectl delete 1
  notectl delete 42
  notectl delete 99")]
    Delete {
        /// Note ID
        id: i64,
    },

    /// Manage TODOs
    #[command(long_about = "\
Create, list, complete, and delete TODO items with priorities and due dates.
TODOs are sorted by priority (high > medium > low) then by due date.

Examples:
  notectl todo add \"Review PR\" --priority high --due 2026-03-01
  notectl todo list
  notectl todo list --pending
  notectl todo done 3
  notectl todo delete 5")]
    Todo {
        #[command(subcommand)]
        action: TodoAction,
    },

    /// Open or show daily note
    #[command(long_about = "\
Create or edit a daily note for today (or a specific date) using your $EDITOR.
Daily notes include a structured template with Tasks, Notes, and Ideas sections.

Examples:
  notectl daily
  notectl daily --show
  notectl daily --date yesterday
  notectl daily --date 2026-02-14
  notectl daily --date yesterday --show")]
    Daily {
        /// Show the daily note instead of opening editor
        #[arg(long)]
        show: bool,

        /// Date (YYYY-MM-DD or "yesterday")
        #[arg(long)]
        date: Option<String>,
    },

    /// Manage tags
    #[command(long_about = "\
List all tags with note counts, show notes for a specific tag, or rename tags.
Without arguments, displays all tags sorted by frequency.

Examples:
  notectl tags
  notectl tags --show work
  notectl tags rename old-name new-name
  notectl tags --show daily")]
    Tags {
        /// Show notes for a specific tag
        #[arg(long)]
        show: Option<String>,

        #[command(subcommand)]
        action: Option<TagAction>,
    },

    /// Manage templates
    #[command(long_about = "\
Create, list, edit, and delete reusable note templates.
Templates support variables like {title}, {date}, {time}, and {datetime}.

Examples:
  notectl template create standup --editor
  notectl template create bug --content \"## Bug Report\\n\\n**Steps**: ...\"
  notectl template list
  notectl template edit standup
  notectl template delete old-template")]
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Create a new note from template
    #[command(long_about = "\
Generate a new note from a saved template. Opens your $EDITOR with the
rendered template for further editing before saving.

Examples:
  notectl new --template standup
  notectl new --template meeting --title \"Q1 Planning\"
  notectl new --template bug --title \"Login page crash\"")]
    New {
        /// Template name to use
        #[arg(long)]
        template: String,

        /// Title variable for the template
        #[arg(long)]
        title: Option<String>,
    },

    /// Export notes
    #[command(long_about = "\
Export notes to markdown or JSON format, with optional filters.
Output goes to stdout by default, or to a file with --output.

Examples:
  notectl export
  notectl export --format json --output backup.json
  notectl export --format markdown --output notes.md
  notectl export --tag work --from 2026-01-01 --to 2026-01-31
  notectl export --format json --tag meeting --output meetings.json")]
    Export {
        /// Output format: markdown, json
        #[arg(long, default_value = "markdown")]
        format: String,

        /// Output file path
        #[arg(long)]
        output: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
    },

    /// Show note statistics
    #[command(long_about = "\
Display statistics about your notes, TODOs, and tags.
Shows total counts, today's activity, and optionally top tags.

Examples:
  notectl stats
  notectl stats --tags")]
    Stats {
        /// Show tag frequency
        #[arg(long)]
        tags: bool,
    },
}

#[derive(Subcommand)]
enum TodoAction {
    /// Add a new TODO
    Add {
        /// Task description
        task: String,

        /// Priority: high, medium, low
        #[arg(long, default_value = "medium")]
        priority: String,

        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due: Option<String>,
    },

    /// List TODOs
    List {
        /// Show only pending TODOs
        #[arg(long)]
        pending: bool,
    },

    /// Mark a TODO as done
    Done {
        /// TODO ID
        id: i64,
    },

    /// Delete a TODO
    Delete {
        /// TODO ID
        id: i64,
    },
}

#[derive(Subcommand)]
enum TagAction {
    /// Rename a tag
    Rename {
        /// Old tag name
        old: String,
        /// New tag name
        new: String,
    },
}

#[derive(Subcommand)]
enum TemplateAction {
    /// Create a new template
    Create {
        /// Template name
        name: String,

        /// Open editor to write template content
        #[arg(long)]
        editor: bool,

        /// Template content (if not using --editor)
        #[arg(long)]
        content: Option<String>,
    },

    /// List all templates
    List,

    /// Edit a template
    Edit {
        /// Template name
        name: String,
    },

    /// Delete a template
    Delete {
        /// Template name
        name: String,
    },
}

fn get_editor() -> String {
    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string())
}

fn edit_with_editor(initial_content: &str) -> io::Result<String> {
    let tmp_dir = std::env::temp_dir();
    let tmp_file = tmp_dir.join(format!("notectl_{}.md", std::process::id()));

    fs::write(&tmp_file, initial_content)?;

    let editor = get_editor();
    let status = Command::new(&editor).arg(&tmp_file).status()?;

    if !status.success() {
        fs::remove_file(&tmp_file).ok();
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Editor exited with non-zero status",
        ));
    }

    let content = fs::read_to_string(&tmp_file)?;
    fs::remove_file(&tmp_file).ok();
    Ok(content)
}

fn main() {
    let cli = Cli::parse();

    let conn = match db::open_connection() {
        Ok(c) => c,
        Err(e) => {
            display::print_error(&format!("Failed to open database: {}", e));
            std::process::exit(1);
        }
    };

    if let Err(e) = db::initialize(&conn) {
        display::print_error(&format!("Failed to initialize database: {}", e));
        std::process::exit(1);
    }

    match cli.command {
        Commands::Add {
            content,
            tags,
            category,
            stdin,
        } => cmd_add(&conn, content, tags, category, stdin),

        Commands::List {
            today,
            tag,
            category,
            limit,
        } => cmd_list(&conn, today, tag, category, limit),

        Commands::Search {
            terms,
            tag,
            case_sensitive,
            full,
        } => cmd_search(&conn, terms, tag, case_sensitive, full),

        Commands::Show { id } => cmd_show(&conn, id),
        Commands::Edit { id } => cmd_edit(&conn, id),
        Commands::Delete { id } => cmd_delete(&conn, id),

        Commands::Todo { action } => cmd_todo(&conn, action),

        Commands::Daily { show, date } => cmd_daily(&conn, show, date),

        Commands::Tags { show, action } => cmd_tags(&conn, show, action),

        Commands::Template { action } => cmd_template(&conn, action),

        Commands::New { template, title } => cmd_new(&conn, template, title),

        Commands::Export {
            format,
            output,
            tag,
            from,
            to,
        } => cmd_export(&conn, format, output, tag, from, to),

        Commands::Stats { tags } => cmd_stats(&conn, tags),
    }
}

fn cmd_add(
    conn: &rusqlite::Connection,
    content: Option<String>,
    tags: Option<Vec<String>>,
    category: Option<String>,
    stdin: bool,
) {
    let text = if stdin {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            display::print_error(&format!("Failed to read stdin: {}", e));
            std::process::exit(1);
        }
        buf.trim().to_string()
    } else if let Some(c) = content {
        c
    } else {
        display::print_error("Please provide note content or use --stdin");
        std::process::exit(1);
    };

    if text.is_empty() {
        display::print_error("Note content cannot be empty");
        std::process::exit(1);
    }

    let tag_list = tags.unwrap_or_default();

    match note::add(conn, &text, &tag_list, category.as_deref(), false) {
        Ok(id) => display::print_note_added(id, &text),
        Err(e) => {
            display::print_error(&format!("Failed to add note: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_list(
    conn: &rusqlite::Connection,
    today: bool,
    tag: Option<String>,
    category: Option<String>,
    limit: usize,
) {
    let title = if today {
        "Today's Notes".to_string()
    } else {
        format!("Recent Notes (last {})", limit)
    };

    match note::list(conn, limit, tag.as_deref(), category.as_deref(), today) {
        Ok(notes) => display::print_notes_table(&notes, &title),
        Err(e) => {
            display::print_error(&format!("Failed to list notes: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_search(
    conn: &rusqlite::Connection,
    terms: Vec<String>,
    tag: Option<String>,
    case_sensitive: bool,
    full: bool,
) {
    let query_display = if let Some(ref t) = tag {
        format!("tag:{}", t)
    } else {
        terms.join(" ")
    };

    match search::search_notes(conn, &terms, tag.as_deref(), case_sensitive) {
        Ok(notes) => display::print_search_results(&notes, &query_display, full),
        Err(e) => {
            display::print_error(&format!("Search failed: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_show(conn: &rusqlite::Connection, id: i64) {
    match note::get_by_id(conn, id) {
        Ok(Some(n)) => {
            use colored::Colorize;
            println!("{} Note #{}\n", "---".dimmed(), id.to_string().cyan());
            println!("{}", n.content);
            println!(
                "\n{} {}",
                "Created:".dimmed(),
                n.created_at.format("%Y-%m-%d %H:%M:%S")
            );
            if let Some(ref cat) = n.category {
                println!("{} {}", "Category:".dimmed(), cat);
            }
            if !n.tags.is_empty() {
                println!("{} {}", "Tags:".dimmed(), n.tags.join(", "));
            }
        }
        Ok(None) => {
            display::print_error(&format!("Note {} not found", id));
            std::process::exit(1);
        }
        Err(e) => {
            display::print_error(&format!("Failed to get note: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_edit(conn: &rusqlite::Connection, id: i64) {
    let existing = match note::get_by_id(conn, id) {
        Ok(Some(n)) => n,
        Ok(None) => {
            display::print_error(&format!("Note {} not found", id));
            std::process::exit(1);
        }
        Err(e) => {
            display::print_error(&format!("Failed to get note: {}", e));
            std::process::exit(1);
        }
    };

    match edit_with_editor(&existing.content) {
        Ok(new_content) => {
            let trimmed = new_content.trim();
            if trimmed.is_empty() {
                display::print_error("Note content cannot be empty");
                std::process::exit(1);
            }
            match note::update(conn, id, trimmed) {
                Ok(true) => {
                    use colored::Colorize;
                    println!("{} Note {} updated", "✓".green().bold(), id.to_string().cyan());
                }
                Ok(false) => {
                    display::print_error(&format!("Note {} not found", id));
                    std::process::exit(1);
                }
                Err(e) => {
                    display::print_error(&format!("Failed to update note: {}", e));
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            display::print_error(&format!("Editor error: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_delete(conn: &rusqlite::Connection, id: i64) {
    match note::delete(conn, id) {
        Ok(true) => display::print_note_deleted(id),
        Ok(false) => {
            display::print_error(&format!("Note {} not found", id));
            std::process::exit(1);
        }
        Err(e) => {
            display::print_error(&format!("Failed to delete note: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_todo(conn: &rusqlite::Connection, action: TodoAction) {
    match action {
        TodoAction::Add {
            task,
            priority,
            due,
        } => {
            let prio = match priority.to_lowercase().as_str() {
                "high" | "h" => "high",
                "low" | "l" => "low",
                _ => "medium",
            };

            match todo::add(conn, &task, prio, due.as_deref()) {
                Ok(id) => display::print_todo_added(id, &task),
                Err(e) => {
                    display::print_error(&format!("Failed to add TODO: {}", e));
                    std::process::exit(1);
                }
            }
        }

        TodoAction::List { pending } => {
            match todo::list_todos(conn, pending) {
                Ok(todos) => {
                    display::print_todos_table(&todos);
                    if let (Ok(overdue), Ok(due_today)) =
                        (todo::count_overdue(conn), todo::count_due_today(conn))
                    {
                        display::print_todo_summary(overdue, due_today);
                    }
                }
                Err(e) => {
                    display::print_error(&format!("Failed to list TODOs: {}", e));
                    std::process::exit(1);
                }
            }
        }

        TodoAction::Done { id } => match todo::mark_done(conn, id) {
            Ok(true) => display::print_todo_done(id),
            Ok(false) => {
                display::print_error(&format!("TODO {} not found", id));
                std::process::exit(1);
            }
            Err(e) => {
                display::print_error(&format!("Failed to complete TODO: {}", e));
                std::process::exit(1);
            }
        },

        TodoAction::Delete { id } => match todo::delete(conn, id) {
            Ok(true) => {
                use colored::Colorize;
                println!("{} TODO {} deleted", "✓".green().bold(), id.to_string().cyan());
            }
            Ok(false) => {
                display::print_error(&format!("TODO {} not found", id));
                std::process::exit(1);
            }
            Err(e) => {
                display::print_error(&format!("Failed to delete TODO: {}", e));
                std::process::exit(1);
            }
        },
    }
}

fn cmd_daily(conn: &rusqlite::Connection, show: bool, date: Option<String>) {
    use chrono::{Duration, Local, NaiveDate};

    let target_date = match date.as_deref() {
        Some("yesterday") => Local::now().date_naive() - Duration::days(1),
        Some(d) => match NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            Ok(nd) => nd,
            Err(_) => {
                display::print_error("Invalid date format. Use YYYY-MM-DD or 'yesterday'");
                std::process::exit(1);
            }
        },
        None => Local::now().date_naive(),
    };

    let daily_title = format!("# Daily Note - {}\n", target_date);

    // Check if daily note already exists for this date
    let start_ts = target_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();
    let end_ts = target_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();

    let existing: Option<(i64, String)> = conn
        .prepare("SELECT id, content FROM notes WHERE is_daily = 1 AND created_at >= ?1 AND created_at <= ?2 LIMIT 1")
        .and_then(|mut stmt| {
            let mut rows = stmt.query(rusqlite::params![start_ts, end_ts])?;
            if let Some(row) = rows.next()? {
                Ok(Some((row.get(0)?, row.get(1)?)))
            } else {
                Ok(None)
            }
        })
        .unwrap_or(None);

    if show {
        match existing {
            Some((id, content)) => {
                use colored::Colorize;
                println!("{} Daily Note #{} ({})\n", "---".dimmed(), id, target_date);
                println!("{}", content);
            }
            None => {
                display::print_error(&format!("No daily note found for {}", target_date));
                std::process::exit(1);
            }
        }
        return;
    }

    // Open in editor
    let initial = match existing {
        Some((_, ref content)) => content.clone(),
        None => {
            format!(
                "{}\n## Tasks\n- [ ] \n\n## Notes\n- \n\n## Ideas\n- \n\n---\nTags: #daily\n",
                daily_title
            )
        }
    };

    match edit_with_editor(&initial) {
        Ok(new_content) => {
            let trimmed = new_content.trim().to_string();
            if trimmed.is_empty() {
                display::print_error("Daily note cannot be empty");
                std::process::exit(1);
            }

            match existing {
                Some((id, _)) => {
                    // Update existing
                    match note::update(conn, id, &trimmed) {
                        Ok(_) => {
                            use colored::Colorize;
                            println!("{} Daily note updated ({})", "✓".green().bold(), target_date);
                        }
                        Err(e) => {
                            display::print_error(&format!("Failed to update daily note: {}", e));
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Create new
                    let daily_tags = vec!["daily".to_string()];
                    match note::add(conn, &trimmed, &daily_tags, None, true) {
                        Ok(id) => {
                            use colored::Colorize;
                            println!(
                                "{} Daily note created (ID: {}, {})",
                                "✓".green().bold(),
                                id.to_string().cyan(),
                                target_date
                            );
                        }
                        Err(e) => {
                            display::print_error(&format!("Failed to create daily note: {}", e));
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Err(e) => {
            display::print_error(&format!("Editor error: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_tags(
    conn: &rusqlite::Connection,
    show: Option<String>,
    action: Option<TagAction>,
) {
    if let Some(tag_name) = show {
        // Show notes for this tag
        match note::list(conn, 100, Some(&tag_name), None, false) {
            Ok(notes) => {
                display::print_notes_table(&notes, &format!("Notes tagged '{}'", tag_name))
            }
            Err(e) => {
                display::print_error(&format!("Failed to list notes by tag: {}", e));
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(act) = action {
        match act {
            TagAction::Rename { old, new } => match tags::rename(conn, &old, &new) {
                Ok(count) => {
                    use colored::Colorize;
                    println!(
                        "{} Renamed tag '{}' -> '{}' ({} note{})",
                        "✓".green().bold(),
                        old,
                        new,
                        count,
                        if count == 1 { "" } else { "s" }
                    );
                }
                Err(e) => {
                    display::print_error(&format!("Failed to rename tag: {}", e));
                    std::process::exit(1);
                }
            },
        }
        return;
    }

    // Default: list all tags
    match tags::list_all(conn) {
        Ok(tag_list) => display::print_tags_table(&tag_list),
        Err(e) => {
            display::print_error(&format!("Failed to list tags: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_template(conn: &rusqlite::Connection, action: TemplateAction) {
    match action {
        TemplateAction::Create {
            name,
            editor,
            content,
        } => {
            let tmpl_content = if editor {
                match edit_with_editor("") {
                    Ok(c) => c,
                    Err(e) => {
                        display::print_error(&format!("Editor error: {}", e));
                        std::process::exit(1);
                    }
                }
            } else if let Some(c) = content {
                c
            } else {
                display::print_error("Provide --content or use --editor");
                std::process::exit(1);
            };

            if tmpl_content.trim().is_empty() {
                display::print_error("Template content cannot be empty");
                std::process::exit(1);
            }

            match template::create(conn, &name, tmpl_content.trim()) {
                Ok(_) => {
                    use colored::Colorize;
                    println!(
                        "{} Template '{}' created",
                        "✓".green().bold(),
                        name.cyan()
                    );
                }
                Err(e) => {
                    display::print_error(&format!("Failed to create template: {}", e));
                    std::process::exit(1);
                }
            }
        }

        TemplateAction::List => match template::list_all(conn) {
            Ok(templates) => {
                if templates.is_empty() {
                    use colored::Colorize;
                    println!("{}", "No templates found.".dimmed());
                    return;
                }
                use colored::Colorize;
                println!("{}\n", "Templates:".bold());
                for t in &templates {
                    let preview = t.content.lines().next().unwrap_or("(empty)");
                    println!("  {} - {}", t.name.cyan(), preview.dimmed());
                }
            }
            Err(e) => {
                display::print_error(&format!("Failed to list templates: {}", e));
                std::process::exit(1);
            }
        },

        TemplateAction::Edit { name } => {
            let existing = match template::get(conn, &name) {
                Ok(Some(t)) => t,
                Ok(None) => {
                    display::print_error(&format!("Template '{}' not found", name));
                    std::process::exit(1);
                }
                Err(e) => {
                    display::print_error(&format!("Failed to get template: {}", e));
                    std::process::exit(1);
                }
            };

            match edit_with_editor(&existing.content) {
                Ok(new_content) => {
                    if new_content.trim().is_empty() {
                        display::print_error("Template content cannot be empty");
                        std::process::exit(1);
                    }
                    match template::create(conn, &name, new_content.trim()) {
                        Ok(_) => {
                            use colored::Colorize;
                            println!(
                                "{} Template '{}' updated",
                                "✓".green().bold(),
                                name.cyan()
                            );
                        }
                        Err(e) => {
                            display::print_error(&format!("Failed to update template: {}", e));
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    display::print_error(&format!("Editor error: {}", e));
                    std::process::exit(1);
                }
            }
        }

        TemplateAction::Delete { name } => match template::delete(conn, &name) {
            Ok(true) => {
                use colored::Colorize;
                println!(
                    "{} Template '{}' deleted",
                    "✓".green().bold(),
                    name.cyan()
                );
            }
            Ok(false) => {
                display::print_error(&format!("Template '{}' not found", name));
                std::process::exit(1);
            }
            Err(e) => {
                display::print_error(&format!("Failed to delete template: {}", e));
                std::process::exit(1);
            }
        },
    }
}

fn cmd_new(conn: &rusqlite::Connection, template_name: String, title: Option<String>) {
    let tmpl = match template::get(conn, &template_name) {
        Ok(Some(t)) => t,
        Ok(None) => {
            display::print_error(&format!("Template '{}' not found", template_name));
            std::process::exit(1);
        }
        Err(e) => {
            display::print_error(&format!("Failed to get template: {}", e));
            std::process::exit(1);
        }
    };

    let mut vars: Vec<(&str, &str)> = Vec::new();
    let title_val = title.unwrap_or_default();
    if !title_val.is_empty() {
        vars.push(("title", &title_val));
    }

    let rendered = template::render(&tmpl.content, &vars);

    // Open in editor for further editing
    match edit_with_editor(&rendered) {
        Ok(final_content) => {
            let trimmed = final_content.trim();
            if trimmed.is_empty() {
                display::print_error("Note content cannot be empty");
                std::process::exit(1);
            }
            match note::add(conn, trimmed, &[], None, false) {
                Ok(id) => display::print_note_added(id, trimmed),
                Err(e) => {
                    display::print_error(&format!("Failed to add note: {}", e));
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            display::print_error(&format!("Editor error: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_export(
    conn: &rusqlite::Connection,
    format: String,
    output: Option<String>,
    tag: Option<String>,
    from: Option<String>,
    to: Option<String>,
) {
    match export::export_notes(conn, &format, tag.as_deref(), from.as_deref(), to.as_deref()) {
        Ok(content) => {
            if let Some(path) = output {
                match fs::write(&path, &content) {
                    Ok(_) => {
                        use colored::Colorize;
                        println!(
                            "{} Exported to {}",
                            "✓".green().bold(),
                            path.cyan()
                        );
                    }
                    Err(e) => {
                        display::print_error(&format!("Failed to write file: {}", e));
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", content);
            }
        }
        Err(e) => {
            display::print_error(&format!("Export failed: {}", e));
            std::process::exit(1);
        }
    }
}

fn cmd_stats(conn: &rusqlite::Connection, show_tags: bool) {
    use colored::Colorize;

    let note_count = note::count_all(conn).unwrap_or(0);
    let (todo_total, todo_completed, todo_pending) = todo::count_stats(conn).unwrap_or((0, 0, 0));

    let tag_list = tags::list_all(conn).unwrap_or_default();
    let unique_tags = tag_list.len();

    println!("{}\n", "Note Statistics:".bold());
    println!("  Total Notes:        {}", note_count.to_string().cyan());
    println!(
        "  Total TODOs:        {} ({} completed, {} pending)",
        todo_total.to_string().cyan(),
        todo_completed.to_string().green(),
        todo_pending.to_string().yellow()
    );
    println!("  Tags:               {} unique tags", unique_tags.to_string().cyan());

    // Notes today
    let today_notes = note::list(conn, 1000, None, None, true)
        .map(|n| n.len())
        .unwrap_or(0);
    println!("\n{}:", "Activity".bold());
    println!("  Today:              {} notes", today_notes.to_string().cyan());

    if show_tags && !tag_list.is_empty() {
        println!("\n{}:", "Top Tags".bold());
        for (i, t) in tag_list.iter().take(10).enumerate() {
            println!(
                "  {}. {} ({} notes)",
                i + 1,
                t.tag.cyan(),
                t.count
            );
        }
    }
}
