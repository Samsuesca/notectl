# notectl

> Lightning-fast note-taking and task management CLI

![macOS](https://img.shields.io/badge/macOS-Apple_Silicon-blue)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![License](https://img.shields.io/badge/license-MIT-green)

**notectl** is a command-line tool for rapid note capture, TODO management, and knowledge organization without leaving your terminal. Seamlessly integrates with Notion via MCP.

---

## Features

- **Quick Capture**: Add notes instantly with timestamps
- **TODO Management**: Create, complete, and organize tasks
- **Full-Text Search**: Find notes by keyword or tag
- **Tags & Categories**: Organize with flexible tagging
- **Daily Notes**: Automatic daily note creation
- **Templates**: Reusable note templates
- **Sync**: Optional sync with Notion (via MCP)
- **Markdown Support**: Write notes in Markdown
- **Journal Mode**: Daily journaling workflow

---

## Installation

```bash
git clone https://github.com/Samsuesca/notectl.git
cd notectl
cargo build --release
cargo install --path .
```

---

## Usage

### Quick Note Capture

```bash
# Add a quick note
notectl add "Idea for ramctl: add memory leak detection"

# Add with tags
notectl add "Research DiD estimators for panel data" --tags research,stata

# Add to specific category
notectl add "Bug: uniformes-system timezone issue" --category bugs

# Add from stdin
echo "Long note content..." | notectl add --stdin
```

**Output:**
```
✓ Note added (ID: 1234)
  "Idea for ramctl: add memory leak detection"
  Created: 2026-02-15 14:32:15
```

### List Notes

```bash
# Show recent notes (default: last 10)
notectl list

# Show all notes from today
notectl list --today

# Filter by tag
notectl list --tag research

# Filter by category
notectl list --category bugs

# Custom limit
notectl list --limit 20
```

**Output:**
```
Recent Notes (last 10):

┌──────┬─────────────────────┬───────────────────────────────────┬─────────────────┐
│ ID   │ Time                │ Content                           │ Tags            │
├──────┼─────────────────────┼───────────────────────────────────┼─────────────────┤
│ 1234 │ 14:32 (2 mins ago)  │ Idea for ramctl: memory leak...   │                 │
│ 1233 │ 13:15 (1 hour ago)  │ Research DiD estimators for...    │ research, stata │
│ 1232 │ 11:45 (3 hours ago) │ Bug: uniformes-system timezone... │ bugs            │
│ 1231 │ 10:20 (4 hours ago) │ TODO: Update portfolio-pos README │ todo            │
│ 1230 │ 09:05 (5 hours ago) │ Meeting notes: EAFIT thesis...    │ research, eafit │
└──────┴─────────────────────┴───────────────────────────────────┴─────────────────┘
```

### Search Notes

```bash
# Search by keyword
notectl search "ramctl"

# Search with multiple terms (AND)
notectl search "DiD" "panel data"

# Search in tags
notectl search --tag research

# Case-sensitive search
notectl search "PostgreSQL" --case-sensitive

# Search and show full content
notectl search "uniformes" --full
```

**Output:**
```
Search Results: "ramctl"

Found 3 notes:

[1234] 2026-02-15 14:32
  Idea for ramctl: add memory leak detection
  Could use sysinfo crate to track app memory over time...
  Tags: ideas

[1205] 2026-02-12 16:45
  ramctl first release done! 🚀
  Published to GitHub. Next: add Linux support?
  Tags: projects, rust

[1198] 2026-02-11 09:30
  Started ramctl project - CLI for RAM management
  Using Rust, clap, sysinfo...
  Tags: projects, rust
```

### TODO Management

```bash
# Add TODO
notectl todo add "Update README for statsctl"

# List TODOs
notectl todo list

# Show only incomplete
notectl todo list --pending

# Complete a TODO
notectl todo done 1234

# Add TODO with priority
notectl todo add "Fix production bug" --priority high

# Add TODO with due date
notectl todo add "Submit paper revisions" --due "2026-03-01"
```

**Output:**
```
Active TODOs:

┌──────┬──────────────────────────────────┬──────────┬──────────┬────────────┐
│ ID   │ Task                             │ Priority │ Due      │ Status     │
├──────┼──────────────────────────────────┼──────────┼──────────┼────────────┤
│ 1235 │ Fix production bug               │ 🔴 High  │ Today    │ ⚪ Pending │
│ 1234 │ Update README for statsctl       │ 🟡 Med   │ -        │ ⚪ Pending │
│ 1231 │ Update portfolio-pos README      │ 🟢 Low   │ -        │ ⚪ Pending │
│ 1210 │ Submit paper revisions           │ 🔴 High  │ Mar 1    │ ⚪ Pending │
└──────┴──────────────────────────────────┴──────────┴──────────┴────────────┘

Overdue: 0 | Due today: 1 | Due this week: 1
```

### Daily Notes

```bash
# Open today's daily note in $EDITOR
notectl daily

# Show today's daily note
notectl daily --show

# Yesterday's note
notectl daily --date yesterday

# Specific date
notectl daily --date 2026-02-14
```

**Daily Note Template:**
```markdown
# Daily Note - 2026-02-15

## Tasks
- [ ] Update statsctl README
- [ ] Code review for uniformes-system PR
- [x] Deploy wristband API changes

## Notes
- Meeting with Diana re: Corresponsales paper analysis
- DiD estimator showing significant effect (p < 0.01)
- Need to add robustness checks

## Ideas
- notectl integration with Notion via MCP
- Add export feature to ramctl

---
Tags: #daily #research #development
```

### Tags and Categories

```bash
# List all tags
notectl tags

# Show notes by tag
notectl tags --show research

# Rename tag
notectl tags rename "old-tag" "new-tag"

# List categories
notectl categories

# Create category
notectl category create "ideas"
```

**Output:**
```
All Tags:

┌─────────────┬───────┐
│ Tag         │ Count │
├─────────────┼───────┤
│ research    │ 45    │
│ rust        │ 23    │
│ stata       │ 18    │
│ bugs        │ 12    │
│ ideas       │ 34    │
│ eafit       │ 15    │
│ projects    │ 28    │
└─────────────┴───────┘
```

### Templates

```bash
# Create template
notectl template create meeting --editor

# Use template
notectl new --template meeting

# List templates
notectl template list

# Edit template
notectl template edit meeting
```

**Example Template (meeting):**
```markdown
# Meeting: {title}
Date: {date}
Attendees: {attendees}

## Agenda
-

## Notes
-

## Action Items
- [ ]

## Follow-up
-
```

### Sync with Notion (MCP)

```bash
# Sync notes to Notion
notectl sync

# Sync specific tag
notectl sync --tag research

# One-way sync (push only)
notectl sync --push-only

# Configure Notion connection
notectl sync config
```

**Output:**
```
Syncing to Notion...

✓ 3 new notes pushed
✓ 1 note updated
✓ 0 conflicts

Last sync: 2026-02-15 14:35:22
Next auto-sync: in 1 hour
```

### Export Notes

```bash
# Export to Markdown
notectl export --format markdown --output notes_backup.md

# Export as JSON
notectl export --format json --output notes.json

# Export specific date range
notectl export --from 2026-02-01 --to 2026-02-15

# Export by tag
notectl export --tag research --format markdown
```

### Statistics

```bash
# Show note statistics
notectl stats

# Stats for specific period
notectl stats --duration 30d

# Tags frequency
notectl stats --tags
```

**Output:**
```
Note Statistics (All Time):

Total Notes:        347
Total TODOs:        89 (34 completed, 55 pending)
Tags:               12 unique tags
Categories:         5

Activity:
  This week:        23 notes
  This month:       87 notes
  Average/day:      2.8 notes

Top Tags:
  1. research       (45 notes)
  2. ideas          (34 notes)
  3. projects       (28 notes)
  4. rust           (23 notes)
  5. stata          (18 notes)

Busiest days:
  Mon-Fri:          89% of notes
  Weekends:         11% of notes
```

---

## Command Reference

| Command | Description | Options |
|---------|-------------|---------|
| `add` | Add quick note | `--tags`, `--category`, `--stdin` |
| `list` | List notes | `--today`, `--tag`, `--category`, `--limit` |
| `search` | Search notes | `--tag`, `--case-sensitive`, `--full` |
| `todo` | Manage TODOs | `add`, `list`, `done`, `--priority`, `--due` |
| `daily` | Daily notes | `--show`, `--date` |
| `tags` | Manage tags | `--show`, `rename` |
| `categories` | Manage categories | `create`, `list` |
| `template` | Templates | `create`, `list`, `edit` |
| `sync` | Sync with Notion | `--tag`, `--push-only`, `config` |
| `export` | Export notes | `--format`, `--output`, `--from`, `--to` |
| `stats` | Statistics | `--duration`, `--tags` |

---

## Use Cases

### Rapid Idea Capture

```bash
# Working on code, get an idea
notectl add "Add support for custom sensors in tempctl" --tags ideas,tempctl

# Continue working...
```

### Research Workflow

```bash
# Daily research log
notectl daily

# Quick observation
notectl add "Parallel trends assumption violated in spec 2" --tags research,corresponsales

# Search previous notes
notectl search "parallel trends"
```

### Project Management

```bash
# Add tasks for today
notectl todo add "Review uniformes-system PR" --priority high
notectl todo add "Update statsctl README"

# Check TODOs
notectl todo list --pending

# Complete
notectl todo done 1234
```

### Meeting Notes

```bash
# Use template
notectl new --template meeting

# $EDITOR opens with template, fill it out

# Later, search meeting notes
notectl search "meeting" --tag eafit
```

---

## Technical Stack

**Language**: Rust 2021 edition

**Dependencies**:
- `clap` - CLI parsing
- `rusqlite` - Local SQLite database
- `chrono` - Date/time handling
- `serde` / `serde_json` - Serialization
- `colored` - Terminal colors
- `tabled` - Table formatting
- `fuzzy-matcher` - Fuzzy search
- `reqwest` - HTTP (for Notion sync)
- `tokio` - Async runtime

---

## Architecture

```
src/
├── main.rs           # CLI entry point
├── db.rs             # SQLite database
├── note.rs           # Note struct and operations
├── todo.rs           # TODO management
├── search.rs         # Full-text search
├── tags.rs           # Tag management
├── template.rs       # Template engine
├── sync.rs           # Notion sync (MCP)
├── export.rs         # Export functionality
└── display.rs        # Formatted output
```

**Database Schema:**
```sql
CREATE TABLE notes (
  id INTEGER PRIMARY KEY,
  content TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  category TEXT,
  is_daily BOOLEAN DEFAULT 0
);

CREATE TABLE tags (
  note_id INTEGER,
  tag TEXT,
  FOREIGN KEY (note_id) REFERENCES notes(id)
);

CREATE TABLE todos (
  id INTEGER PRIMARY KEY,
  note_id INTEGER,
  task TEXT NOT NULL,
  completed BOOLEAN DEFAULT 0,
  priority TEXT DEFAULT 'medium',
  due_date INTEGER,
  FOREIGN KEY (note_id) REFERENCES notes(id)
);

CREATE TABLE templates (
  name TEXT PRIMARY KEY,
  content TEXT NOT NULL
);

CREATE VIRTUAL TABLE notes_fts USING fts5(content);
```

**Storage Location:**
```
~/.notectl/
├── notes.db          # SQLite database
├── templates/        # Custom templates
└── config.toml       # Configuration
```

---

## Configuration

`~/.notectl/config.toml`:
```toml
[general]
editor = "vim"           # Or $EDITOR
default_category = "general"
auto_tags = true

[sync]
enabled = true
notion_token = "secret_xxx"
notion_database_id = "abc123"
auto_sync_interval = 3600  # seconds

[display]
date_format = "%Y-%m-%d %H:%M"
timezone = "America/Bogota"
color_scheme = "auto"      # auto, always, never
```

---

## Platform Support

| Platform | Support |
|----------|---------|
| macOS (Apple Silicon) | ✅ Full support |
| macOS (Intel) | ✅ Full support |
| Linux | ✅ Full support |
| Windows | ✅ Full support |

---

## Roadmap

- [ ] Encryption for sensitive notes
- [ ] Attachments support (images, files)
- [ ] Web interface (local server)
- [ ] Git integration (version control for notes)
- [ ] Bidirectional Notion sync
- [ ] Vim plugin for inline note capture
- [ ] Mobile companion app (view-only)

---

## License

MIT License

---

## Author

**Angel Samuel Suesca Ríos**
suescapsam@gmail.com

---

## Integration Examples

### With Git Hooks

```bash
# .git/hooks/post-commit
#!/bin/bash
notectl add "Committed: $(git log -1 --pretty=%B)" --tags git,$(basename $(pwd))
```

### With Cron (Daily Summary)

```bash
# Send daily summary email
0 18 * * * notectl list --today | mail -s "Daily Notes" you@example.com
```

### With Notion MCP

Configure in `~/.claude/plugins/`:
```json
{
  "mcpServers": {
    "notion": {
      "command": "npx",
      "args": ["-y", "@notionhq/client"]
    }
  }
}
```

---

**Perfect for**: Developers who live in the terminal, researchers tracking ideas, anyone who wants frictionless note capture.
