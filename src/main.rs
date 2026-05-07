mod core;
mod tui;

use clap::{Parser, Subcommand};
use colored::Colorize;
use core::{get_store_path, Category, Event, EventFilter, EventStatus, EventStore, Recurrence};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "firecalendar", about = "Calendar CLI with events and reminders", version)]
struct Cli {
    #[arg(short, long, help = "Path to the events file")]
    store: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Initialize a new event store")]
    Init,

    #[command(about = "Add a new event")]
    Add {
        #[arg(help = "Event title")]
        title: String,

        #[arg(short, long, help = "Event description")]
        description: Option<String>,

        #[arg(short, long, help = "Start time (YYYY-MM-DD HH:MM)")]
        start: Option<String>,

        #[arg(short, long, help = "End time (YYYY-MM-DD HH:MM)")]
        end: Option<String>,

        #[arg(short = 'C', long, help = "Category ID")]
        category: Option<String>,

        #[arg(short = 'T', long, help = "Tags (comma-separated)")]
        tags: Option<String>,

        #[arg(short, long, help = "Recurrence (none, daily, weekly, monthly, yearly)")]
        recurrence: Option<String>,

        #[arg(short = 'R', long, help = "Reminder minutes before event")]
        reminder: Option<u32>,

        #[arg(short, long, help = "Timezone (e.g., America/Sao_Paulo)")]
        timezone: Option<String>,

        #[arg(short = 'L', long, help = "Location")]
        location: Option<String>,
    },

    #[command(about = "List events")]
    List {
        #[arg(short, long, help = "Filter by status")]
        status: Option<String>,

        #[arg(short = 'C', long, help = "Filter by category")]
        category: Option<String>,

        #[arg(short = 'T', long, help = "Filter by tags")]
        tags: Option<String>,

        #[arg(long, help = "Start after date (YYYY-MM-DD HH:MM)")]
        after: Option<String>,

        #[arg(long, help = "Start before date (YYYY-MM-DD HH:MM)")]
        before: Option<String>,

        #[arg(long, help = "Show only overdue events")]
        overdue: bool,
    },

    #[command(about = "Show event details")]
    Show {
        #[arg(help = "Event ID")]
        id: String,
    },

    #[command(about = "Update an event")]
    Update {
        #[arg(help = "Event ID")]
        id: String,

        #[arg(short, long, help = "New title")]
        title: Option<String>,

        #[arg(short, long, help = "New description")]
        description: Option<String>,

        #[arg(short, long, help = "New start time")]
        start: Option<String>,

        #[arg(short, long, help = "New end time")]
        end: Option<String>,

        #[arg(short, long, help = "New status")]
        status: Option<String>,

        #[arg(short = 'C', long, help = "New category")]
        category: Option<String>,

        #[arg(short = 'T', long, help = "New tags (comma-separated)")]
        tags: Option<String>,

        #[arg(short, long, help = "New recurrence")]
        recurrence: Option<String>,

        #[arg(short = 'R', long, help = "New reminder minutes")]
        reminder: Option<u32>,

        #[arg(short = 'L', long, help = "New location")]
        location: Option<String>,
    },

    #[command(about = "Mark event as completed")]
    Complete {
        #[arg(help = "Event ID")]
        id: String,
    },

    #[command(about = "Mark event as in progress")]
    Start {
        #[arg(help = "Event ID")]
        id: String,
    },

    #[command(about = "Cancel an event")]
    Cancel {
        #[arg(help = "Event ID")]
        id: String,
    },

    #[command(about = "Delete an event")]
    Delete {
        #[arg(help = "Event ID")]
        id: String,
    },

    #[command(about = "Add a new category")]
    AddCategory {
        #[arg(help = "Category name")]
        name: String,

        #[arg(short, long, help = "Category description")]
        description: Option<String>,

        #[arg(short, long, help = "Category color (hex)")]
        color: Option<String>,
    },

    #[command(about = "List categories")]
    ListCategories,

    #[command(about = "Show today's events")]
    Today,

    #[command(about = "Show this week's events")]
    Week,

    #[command(about = "Show statistics")]
    Stats,

    #[command(about = "Open TUI interface")]
    Tui,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let store_path = match cli.store {
        Some(path) => path,
        None => get_store_path()?,
    };

    match cli.command {
        Command::Init => cmd_init(&store_path),
        Command::Add {
            title,
            description,
            start,
            end,
            category,
            tags,
            recurrence,
            reminder,
            timezone,
            location,
        } => cmd_add(&store_path, title, description, start, end, category, tags, recurrence, reminder, timezone, location),
        Command::List {
            status,
            category,
            tags,
            after,
            before,
            overdue,
        } => cmd_list(&store_path, status, category, tags, after, before, overdue),
        Command::Show { id } => cmd_show(&store_path, &id),
        Command::Update {
            id,
            title,
            description,
            start,
            end,
            status,
            category,
            tags,
            recurrence,
            reminder,
            location,
        } => cmd_update(&store_path, &id, title, description, start, end, status, category, tags, recurrence, reminder, location),
        Command::Complete { id } => cmd_complete(&store_path, &id),
        Command::Start { id } => cmd_start(&store_path, &id),
        Command::Cancel { id } => cmd_cancel(&store_path, &id),
        Command::Delete { id } => cmd_delete(&store_path, &id),
        Command::AddCategory {
            name,
            description,
            color,
        } => cmd_add_category(&store_path, name, description, color),
        Command::ListCategories => cmd_list_categories(&store_path),
        Command::Today => cmd_today(&store_path),
        Command::Week => cmd_week(&store_path),
        Command::Stats => cmd_stats(&store_path),
        Command::Tui => cmd_tui(&store_path),
    }
}

fn cmd_init(store_path: &PathBuf) -> anyhow::Result<()> {
    if store_path.exists() {
        println!("{}", "Store already exists!".red());
        return Ok(());
    }

    let store = EventStore::new();
    store.save(store_path)?;
    println!("{} {}", "Store created at:".green(), store_path.display());
    Ok(())
}

fn load_store(store_path: &PathBuf) -> anyhow::Result<EventStore> {
    if !store_path.exists() {
        anyhow::bail!("Store not found. Run 'firecalendar init' first.");
    }
    EventStore::load(store_path).map_err(|e| anyhow::anyhow!("Error loading store: {}", e))
}

fn cmd_add(
    store_path: &PathBuf,
    title: String,
    description: Option<String>,
    start: Option<String>,
    end: Option<String>,
    category: Option<String>,
    tags: Option<String>,
    recurrence: Option<String>,
    reminder: Option<u32>,
    timezone: Option<String>,
    location: Option<String>,
) -> anyhow::Result<()> {
    let mut store = load_store(store_path)?;

    let mut event = Event::new(title);

    if let Some(desc) = description {
        event = event.with_description(desc);
    }

    if let Some(start_str) = start {
        let start_time = chrono::DateTime::parse_from_rfc3339(&start_str)
            .map_err(|e| anyhow::anyhow!("Invalid start date (use YYYY-MM-DD HH:MM): {}", e))?
            .with_timezone(&chrono::Utc);
        event = event.with_start_time(start_time);
    }

    if let Some(end_str) = end {
        let end_time = chrono::DateTime::parse_from_rfc3339(&end_str)
            .map_err(|e| anyhow::anyhow!("Invalid end date (use YYYY-MM-DD HH:MM): {}", e))?
            .with_timezone(&chrono::Utc);
        event = event.with_end_time(end_time);
    }

    if let Some(category_str) = category {
        let category_id = Uuid::parse_str(&category_str)
            .map_err(|e| anyhow::anyhow!("Invalid category ID: {}", e))?;
        event = event.with_category(category_id);
    }

    if let Some(tags_str) = tags {
        let tag_list: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
        event = event.with_tags(tag_list);
    }

    if let Some(recurrence_str) = recurrence {
        let recurrence = Recurrence::from_str(&recurrence_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid recurrence: {}", recurrence_str))?;
        event = event.with_recurrence(recurrence);
    }

    if let Some(reminder_minutes) = reminder {
        event = event.with_reminder(reminder_minutes);
    }

    if let Some(tz) = timezone {
        event = event.with_timezone(tz);
    }

    if let Some(loc) = location {
        event = event.with_location(loc);
    }

    store.add_event(event.clone())?;
    store.save(store_path)?;

    println!("{} {}", "Event created:".green(), event.id);
    println!("  {}", event.title);
    Ok(())
}

fn cmd_list(
    store_path: &PathBuf,
    status: Option<String>,
    category: Option<String>,
    tags: Option<String>,
    after: Option<String>,
    before: Option<String>,
    overdue: bool,
) -> anyhow::Result<()> {
    let store = load_store(store_path)?;

    let mut filter = EventFilter {
        status: None,
        category_id: None,
        tags: None,
        start_after: None,
        start_before: None,
        overdue_only: false,
    };

    if let Some(status_str) = status {
        filter.status = Some(
            EventStatus::from_str(&status_str)
                .ok_or_else(|| anyhow::anyhow!("Invalid status: {}", status_str))?,
        );
    }

    if let Some(category_str) = category {
        filter.category_id = Some(
            Uuid::parse_str(&category_str)
                .map_err(|e| anyhow::anyhow!("Invalid category ID: {}", e))?,
        );
    }

    if let Some(tags_str) = tags {
        filter.tags = Some(tags_str.split(',').map(|s| s.trim().to_string()).collect());
    }

    if let Some(after_str) = after {
        filter.start_after = Some(
            chrono::DateTime::parse_from_rfc3339(&after_str)
                .map_err(|e| anyhow::anyhow!("Invalid date: {}", e))?
                .with_timezone(&chrono::Utc),
        );
    }

    if let Some(before_str) = before {
        filter.start_before = Some(
            chrono::DateTime::parse_from_rfc3339(&before_str)
                .map_err(|e| anyhow::anyhow!("Invalid date: {}", e))?
                .with_timezone(&chrono::Utc),
        );
    }

    filter.overdue_only = overdue;

    let events = store.list_events(Some(filter));

    if events.is_empty() {
        println!("{}", "No events found.".yellow());
        return Ok(());
    }

    println!("{}", "Events:".cyan());
    for event in events {
        print_event(event);
    }

    Ok(())
}

fn print_event(event: &Event) {
    let status_str = match event.status {
        EventStatus::Scheduled => "SCHEDULED".blue(),
        EventStatus::InProgress => "IN PROGRESS".yellow(),
        EventStatus::Completed => "COMPLETED".green(),
        EventStatus::Cancelled => "CANCELLED".red(),
    };

    let overdue_indicator = if event.is_overdue() {
        " [OVERDUE]".red()
    } else {
        "".normal()
    };

    println!(
        "  {} {} {}{}",
        event.id,
        status_str,
        event.title,
        overdue_indicator
    );

    println!("    Start: {}", event.start_time.format("%Y-%m-%d %H:%M"));

    if let Some(end_time) = &event.end_time {
        println!("    End: {}", end_time.format("%Y-%m-%d %H:%M"));
    }

    if let Some(desc) = &event.description {
        println!("    Description: {}", desc.dimmed());
    }

    if let Some(location) = &event.location {
        println!("    Location: {}", location);
    }

    if !event.tags.is_empty() {
        println!("    Tags: {}", event.tags.join(", "));
    }
}

fn cmd_show(store_path: &PathBuf, id: &str) -> anyhow::Result<()> {
    let store = load_store(store_path)?;
    let uuid = Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid ID: {}", e))?;
    let event = store.get_event(uuid)?;

    println!("{}", "Event:".cyan());
    println!("  ID: {}", event.id);
    println!("  Title: {}", event.title);
    println!("  Status: {:?}", event.status);
    println!("  Start: {}", event.start_time.format("%Y-%m-%d %H:%M"));
    println!("  Created: {}", event.created_at.format("%Y-%m-%d %H:%M"));
    println!("  Updated: {}", event.updated_at.format("%Y-%m-%d %H:%M"));

    if let Some(end_time) = &event.end_time {
        println!("  End: {}", end_time.format("%Y-%m-%d %H:%M"));
    }

    if let Some(desc) = &event.description {
        println!("  Description: {}", desc);
    }

    if let Some(category_id) = &event.category_id {
        println!("  Category: {}", category_id);
    }

    if !event.tags.is_empty() {
        println!("  Tags: {}", event.tags.join(", "));
    }

    if let Some(location) = &event.location {
        println!("  Location: {}", location);
    }

    if let Some(reminder) = &event.reminder_minutes {
        println!("  Reminder: {} minutes before", reminder);
    }

    if let Some(timezone) = &event.timezone {
        println!("  Timezone: {}", timezone);
    }

    Ok(())
}

fn cmd_update(
    store_path: &PathBuf,
    id: &str,
    title: Option<String>,
    description: Option<String>,
    start: Option<String>,
    end: Option<String>,
    status: Option<String>,
    category: Option<String>,
    tags: Option<String>,
    recurrence: Option<String>,
    reminder: Option<u32>,
    location: Option<String>,
) -> anyhow::Result<()> {
    let mut store = load_store(store_path)?;
    let uuid = Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid ID: {}", e))?;
    let mut event = store.get_event(uuid)?.clone();

    if let Some(new_title) = title {
        event.title = new_title;
    }

    if let Some(new_desc) = description {
        event.description = Some(new_desc);
    }

    if let Some(start_str) = start {
        event.start_time = chrono::DateTime::parse_from_rfc3339(&start_str)
            .map_err(|e| anyhow::anyhow!("Invalid date: {}", e))?
            .with_timezone(&chrono::Utc);
    }

    if let Some(end_str) = end {
        event.end_time = Some(
            chrono::DateTime::parse_from_rfc3339(&end_str)
                .map_err(|e| anyhow::anyhow!("Invalid date: {}", e))?
                .with_timezone(&chrono::Utc),
        );
    }

    if let Some(status_str) = status {
        event.status = EventStatus::from_str(&status_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid status: {}", status_str))?;
    }

    if let Some(category_str) = category {
        if category_str == "none" {
            event.category_id = None;
        } else {
            event.category_id = Some(
                Uuid::parse_str(&category_str)
                    .map_err(|e| anyhow::anyhow!("Invalid category ID: {}", e))?,
            );
        }
    }

    if let Some(tags_str) = tags {
        event.tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Some(recurrence_str) = recurrence {
        event.recurrence = Recurrence::from_str(&recurrence_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid recurrence: {}", recurrence_str))?;
    }

    if let Some(reminder_minutes) = reminder {
        event.reminder_minutes = Some(reminder_minutes);
    }

    if let Some(loc) = location {
        event.location = Some(loc);
    }

    store.update_event(event)?;
    store.save(store_path)?;

    println!("{}", "Event updated!".green());
    Ok(())
}

fn cmd_complete(store_path: &PathBuf, id: &str) -> anyhow::Result<()> {
    let mut store = load_store(store_path)?;
    let uuid = Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid ID: {}", e))?;
    let event = store.get_event_mut(uuid)?;
    event.status = EventStatus::Completed;
    event.updated_at = chrono::Utc::now();
    store.save(store_path)?;

    println!("{}", "Event marked as completed!".green());
    Ok(())
}

fn cmd_start(store_path: &PathBuf, id: &str) -> anyhow::Result<()> {
    let mut store = load_store(store_path)?;
    let uuid = Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid ID: {}", e))?;
    let event = store.get_event_mut(uuid)?;
    event.status = EventStatus::InProgress;
    event.updated_at = chrono::Utc::now();
    store.save(store_path)?;

    println!("{}", "Event marked as in progress!".green());
    Ok(())
}

fn cmd_cancel(store_path: &PathBuf, id: &str) -> anyhow::Result<()> {
    let mut store = load_store(store_path)?;
    let uuid = Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid ID: {}", e))?;
    let event = store.get_event_mut(uuid)?;
    event.status = EventStatus::Cancelled;
    event.updated_at = chrono::Utc::now();
    store.save(store_path)?;

    println!("{}", "Event cancelled!".green());
    Ok(())
}

fn cmd_delete(store_path: &PathBuf, id: &str) -> anyhow::Result<()> {
    let mut store = load_store(store_path)?;
    let uuid = Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid ID: {}", e))?;
    store.delete_event(uuid)?;
    store.save(store_path)?;

    println!("{}", "Event deleted!".green());
    Ok(())
}

fn cmd_add_category(
    store_path: &PathBuf,
    name: String,
    description: Option<String>,
    color: Option<String>,
) -> anyhow::Result<()> {
    let mut store = load_store(store_path)?;

    let mut category = Category::new(name);

    if let Some(desc) = description {
        category = category.with_description(desc);
    }

    if let Some(col) = color {
        category = category.with_color(col);
    }

    store.add_category(category.clone())?;
    store.save(store_path)?;

    println!("{} {}", "Category created:".green(), category.id);
    println!("  {}", category.name);
    Ok(())
}

fn cmd_list_categories(store_path: &PathBuf) -> anyhow::Result<()> {
    let store = load_store(store_path)?;
    let categories = store.list_categories();

    if categories.is_empty() {
        println!("{}", "No categories found.".yellow());
        return Ok(());
    }

    println!("{}", "Categories:".cyan());
    for category in categories {
        println!("  {} {}", category.id, category.name);
        if let Some(desc) = &category.description {
            println!("    {}", desc.dimmed());
        }
    }

    Ok(())
}

fn cmd_today(store_path: &PathBuf) -> anyhow::Result<()> {
    let store = load_store(store_path)?;
    let now = chrono::Utc::now();
    let today_end = now + chrono::Duration::days(1);

    let filter = EventFilter {
        status: Some(EventStatus::Scheduled),
        category_id: None,
        tags: None,
        start_after: Some(now),
        start_before: Some(today_end),
        overdue_only: false,
    };

    let events = store.list_events(Some(filter));

    if events.is_empty() {
        println!("{}", "No events scheduled for today.".yellow());
        return Ok(());
    }

    println!("{}", "Today's events:".cyan());
    for event in events {
        print_event(event);
    }

    Ok(())
}

fn cmd_week(store_path: &PathBuf) -> anyhow::Result<()> {
    let store = load_store(store_path)?;
    let now = chrono::Utc::now();
    let week_end = now + chrono::Duration::days(7);

    let filter = EventFilter {
        status: Some(EventStatus::Scheduled),
        category_id: None,
        tags: None,
        start_after: Some(now),
        start_before: Some(week_end),
        overdue_only: false,
    };

    let events = store.list_events(Some(filter));

    if events.is_empty() {
        println!("{}", "No events scheduled for this week.".yellow());
        return Ok(());
    }

    println!("{}", "This week's events:".cyan());
    for event in events {
        print_event(event);
    }

    Ok(())
}

fn cmd_stats(store_path: &PathBuf) -> anyhow::Result<()> {
    let store = load_store(store_path)?;
    let stats = store.get_stats();

    println!("{}", "Statistics:".cyan());
    println!("  Total events: {}", stats.total_events);
    println!("  {} Scheduled: {}", "SCHEDULED".blue(), stats.scheduled);
    println!("  {} In Progress: {}", "IN PROGRESS".yellow(), stats.in_progress);
    println!("  {} Completed: {}", "COMPLETED".green(), stats.completed);
    println!("  {} Cancelled: {}", "CANCELLED".red(), stats.cancelled);
    if stats.overdue > 0 {
        println!("  {} Overdue: {}", "OVERDUE".red(), stats.overdue);
    }
    println!("  Upcoming today: {}", stats.upcoming_today);
    println!("  Upcoming this week: {}", stats.upcoming_week);
    println!("  Categories: {}", stats.categories_count);

    Ok(())
}

fn cmd_tui(store_path: &PathBuf) -> anyhow::Result<()> {
    let store = load_store(store_path)?;
    tui::run(store, store_path.clone())
}