# AGENTS.md

This file provides essential guidelines for agentic coding agents working on the FireCalendar event management system.

## Project Overview

- **Name**: FireCalendar
- **Stack**: Rust (core library + CLI binary + TUI)
- **Purpose**: Calendar CLI with events, categories, recurrence, reminders, and TUI interface

## Commands

```bash
# Build everything
cargo build

# Run all tests
cargo test

# Build and run CLI directly
cargo run -- <command>

# Run a single test
cargo test -- test_event_creation
```

## Post-Task Workflow

**ALWAYS run the following sequence after completing any coding task:**

1. **Build**: `cargo build` - verify compilation succeeds
2. **Test**: `cargo test` - run all tests to ensure nothing is broken
3. **Install**: `cargo install --path .` - install/update the binary globally

This ensures the code compiles, tests pass, and the latest version is available system-wide.

## Code Style Guidelines

### Project Structure
```
firecalendar/
├── Cargo.toml              # Project root
├── src/
│   ├── main.rs             # CLI entry point with clap
│   ├── core/               # Core library (pub API for CLI + TUI + future GUIs)
│   │   ├── mod.rs          # Re-exports
│   │   ├── error.rs        # Error types
│   │   ├── event.rs        # Event models and enums
│   │   └── store.rs        # Storage operations
│   └── tui/                # TUI interface
│       ├── mod.rs          # Exports
│       ├── app.rs          # TUI application logic
│       └── ui.rs           # TUI rendering with ratatui
```

### Imports & Dependencies
- Follow Rust convention: std → external crates → internal crates
- Use `anyhow::Result` for CLI code, `crate::Result` for core library
- Re-export public types in `core/mod.rs`
- TUI dependencies: `ratatui` for rendering, `crossterm` for terminal handling
- Notification dependency: `notify-rust` for desktop notifications

### Types & Naming
- **Structs**: PascalCase (`Event`, `Category`, `EventStore`)
- **Functions**: snake_case (`add_event`, `list_events`, `mark_complete`)
- **Constants**: UPPER_SNAKE_CASE (rarely used)
- **Error variants**: PascalCase (`EventNotFound`, `StoreNotFound`)

### Error Handling
- Core library: use `thiserror` for typed errors, return `crate::Result<T>`
- CLI: use `anyhow::Result` with `?` operator
- Never unwrap in core; use `?` or `.map_err()`
- Use `anyhow::bail!` for CLI error messages

### Core Library API Design
- Public API surface in `lib.rs` must be stable (GUI consumers depend on it)
- All pub types must have `#[derive(Debug, Clone, Serialize, Deserialize)]` where possible
- Use `chrono::DateTime<Utc>` for all timestamps
- Use `uuid::Uuid` for all entity IDs
- Storage path defaults to `~/.firecalendar/events.json` via `dirs` crate

### Data Models
- **Event**: id, title, description, start_time, end_time, status, category_id, tags, recurrence, reminder_minutes, timezone, location, timestamps
- **Category**: id, name, description, color, timestamps
- **EventStatus**: Scheduled, InProgress, Completed, Cancelled
- **Recurrence**: None, Daily, Weekly, Monthly, Yearly, Custom
- **EventFilter**: Optional filters for listing events
- **CalendarStats**: Aggregate counts for dashboard

### Storage Layer
- JSON file storage via `serde_json`
- Atomic write operations (read → modify → write)
- Auto-create parent directories
- Version field in store for future migrations
- Timestamps for created_at and updated_at

### CLI Design
- Use `clap` with derive macros
- Subcommands for each operation (init, add, list, show, update, complete, start, cancel, delete, add-category, list-categories, today, week, stats, tui)
- Colorized output using `colored` crate
- Human-friendly error messages
- Support for CSV input (tags, etc.)
- Date/time format: RFC3339 (e.g., "2026-05-10T14:00:00Z")

### TUI Design
- Use `ratatui` for terminal UI rendering
- Use `crossterm` for terminal handling and event processing
- Modes: CategoryList, EventDetail, AddEvent, AddCategory, CalendarView
- Key bindings: vim-style (j/k) + arrow keys + single-letter commands
- Real-time updates to store
- Color-coded status indicators
- Modern UI with sidebar showing statistics and categories
- Auto-dismissing messages (3 seconds)
- Auto-save on every modification

### Testing
- Core library must have test coverage for CRUD operations
- Tests use temp directories or in-memory stores
- Test file naming: `tests.rs` in core
- CLI integration tests via shell script (not Rust tests)

## DO NOT:
- Change the public API of firecalendar core without updating README
- Use `unwrap()` in core library code
- Hardcode paths (use `dirs` crate)
- Break backward compatibility of store format
- Test TUI in non-TTY environments

## DO:
- Follow existing patterns in the codebase
- Run `cargo test` before committing
- Create parent directories when needed (`create_dir_all`)
- Use `anyhow::bail!` for CLI error messages
- Keep store format backward compatible
- Add version field to store for migrations
- **ALWAYS run the following sequence after completing tasks**:
  1. `cargo build` - verify compilation
  2. `cargo test` - run all tests
  3. `cargo install --path .` - install/update the binary globally

## TUI Status (2026-05-03)
The TUI is fully implemented and functional with the following features:
- **Modes**: CategoryList, EventDetail, AddEvent, AddCategory, CalendarView
- **Navigation**: Vim-style (j/k) + arrow keys
- **Actions**: Add events/categories, mark complete, delete, view details
- **Filters**: Filter by status
- **Real-time updates**: Changes immediately reflected in the UI
- **Color-coded indicators**: Status (blue/yellow/green/red)
- **Auto-dismissing messages**: Status messages auto-hide after 3 seconds

### Modern UI Design
- Sidebar with real-time statistics and category overview
- Table-based layout for events and categories with proper headers
- Unicode box-drawing characters for polished borders (╭─╮, │, ├─┤, ╰─╯)
- Emoji indicators for visual clarity (📅, 🔵, 🟡, 🟢, ✗, etc.)
- Status icons: ○ (scheduled), ◐ (in progress), ● (completed), ⊘ (cancelled)
- Color-coded overdue warnings with ✗ indicator

### Improved Layout
- Header with mode-specific icons and filter indicators
- Sidebar showing event statistics and category list
- Footer with compact keyboard shortcuts and status messages
- Modern table design with proper column spacing

### Better User Experience
- Empty state messages for events/categories lists
- Inline help in add event/category forms
- Consistent color scheme (red accent matching other fire apps)
- Selected row highlighting with red accent background
- Message Auto-Dismiss: Status messages automatically clear after 3 seconds
- Auto-Save: Every modification in TUI automatically saves to storage
- Firefly Labs Context Integration:
  - Storage path follows Firefly Labs pattern: `/firefly/config/firecalendar/events.json` (production) with fallback to `~/.firecalendar/events.json`
  - Prioritizes production config over home directory
  - Documented in CONTEXT.md alongside other Firefly Labs projects

## CLI Commands Reference

### Basic Commands
```bash
firecalendar init                          # Initialize event store
firecalendar add "Meeting" --start "2026-05-10T14:00:00Z" --end "2026-05-10T15:00:00Z"
firecalendar list                          # List all events
firecalendar show <uuid>                   # Show event details
firecalendar today                         # Show today's events
firecalendar week                          # Show this week's events
firecalendar stats                         # Show statistics
firecalendar tui                           # Open TUI interface
```

### Event Management
```bash
firecalendar update <uuid> --title "New Title"
firecalendar complete <uuid>               # Mark as completed
firecalendar start <uuid>                  # Mark as in progress
firecalendar cancel <uuid>                 # Cancel event
firecalendar delete <uuid>                 # Delete event
```

### Categories
```bash
firecalendar add-category "Work" --description "Work events"
firecalendar list-categories               # List all categories
```

### Advanced Options
```bash
firecalendar add "Meeting" \
  --description "Team sync" \
  --start "2026-05-10T14:00:00Z" \
  --end "2026-05-10T15:00:00Z" \
  --category <uuid> \
  --tags "important,work" \
  --recurrence weekly \
  --reminder 15 \
  --timezone "America/Sao_Paulo" \
  --location "Conference Room A"
```