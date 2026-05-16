use super::error::{Error, Result};
use super::event::{CalendarStats, Category, Event, EventFilter, EventStatus};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventStore {
    pub events: Vec<Event>,
    pub categories: Vec<Category>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EventStore {
    pub fn new() -> Self {
        let now = Utc::now();
        EventStore {
            events: Vec::new(),
            categories: Vec::new(),
            version: "0.1.0".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())?;
        let store: EventStore = serde_json::from_str(&content)?;
        Ok(store)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let parent = path.as_ref().parent().ok_or_else(|| {
            Error::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No parent directory",
            ))
        })?;

        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(path.as_ref(), content)?;
        Ok(())
    }

    pub fn add_event(&mut self, event: Event) -> Result<()> {
        self.events.push(event);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_event(&self, id: Uuid) -> Result<&Event> {
        self.events
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| Error::EventNotFound(id.to_string()))
    }

    pub fn get_event_mut(&mut self, id: Uuid) -> Result<&mut Event> {
        self.events
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| Error::EventNotFound(id.to_string()))
    }

    pub fn update_event(&mut self, event: Event) -> Result<()> {
        let index = self
            .events
            .iter()
            .position(|e| e.id == event.id)
            .ok_or_else(|| Error::EventNotFound(event.id.to_string()))?;

        let mut updated_event = event;
        updated_event.updated_at = Utc::now();
        self.events[index] = updated_event;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn delete_event(&mut self, id: Uuid) -> Result<()> {
        let index = self
            .events
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| Error::EventNotFound(id.to_string()))?;

        self.events.remove(index);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn list_events(&self, filter: Option<EventFilter>) -> Vec<&Event> {
        let mut events: Vec<&Event> = self.events.iter().collect();

        if let Some(filter) = filter {
            if let Some(status) = filter.status {
                events.retain(|e| e.status == status);
            }
            if let Some(category_id) = filter.category_id {
                events.retain(|e| e.category_id == Some(category_id));
            }
            if let Some(tags) = filter.tags {
                events.retain(|e| tags.iter().all(|tag| e.tags.contains(tag)));
            }
            if let Some(start_after) = filter.start_after {
                events.retain(|e| e.start_time >= start_after);
            }
            if let Some(start_before) = filter.start_before {
                events.retain(|e| e.start_time <= start_before);
            }
            if filter.overdue_only {
                events.retain(|e| e.is_overdue());
            }
        }

        // Sort by start time (ascending)
        events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        events
    }

    pub fn add_category(&mut self, category: Category) -> Result<()> {
        self.categories.push(category);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_category(&self, id: Uuid) -> Result<&Category> {
        self.categories
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| Error::CategoryNotFound(id.to_string()))
    }

    pub fn get_category_mut(&mut self, id: Uuid) -> Result<&mut Category> {
        self.categories
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or_else(|| Error::CategoryNotFound(id.to_string()))
    }

    pub fn update_category(&mut self, category: Category) -> Result<()> {
        let index = self
            .categories
            .iter()
            .position(|c| c.id == category.id)
            .ok_or_else(|| Error::CategoryNotFound(category.id.to_string()))?;

        self.categories[index] = category;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn delete_category(&mut self, id: Uuid) -> Result<()> {
        let index = self
            .categories
            .iter()
            .position(|c| c.id == id)
            .ok_or_else(|| Error::CategoryNotFound(id.to_string()))?;

        // Remove category
        self.categories.remove(index);

        // Unassign events from this category
        for event in &mut self.events {
            if event.category_id == Some(id) {
                event.category_id = None;
                event.updated_at = Utc::now();
            }
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn list_categories(&self) -> Vec<&Category> {
        self.categories.iter().collect()
    }

    pub fn get_stats(&self) -> CalendarStats {
        let total_events = self.events.len();
        let scheduled = self
            .events
            .iter()
            .filter(|e| e.status == EventStatus::Scheduled)
            .count();
        let in_progress = self
            .events
            .iter()
            .filter(|e| e.status == EventStatus::InProgress)
            .count();
        let completed = self
            .events
            .iter()
            .filter(|e| e.status == EventStatus::Completed)
            .count();
        let cancelled = self
            .events
            .iter()
            .filter(|e| e.status == EventStatus::Cancelled)
            .count();
        let overdue = self.events.iter().filter(|e| e.is_overdue()).count();

        let now = Utc::now();
        let today_end = now + Duration::days(1);
        let week_end = now + Duration::days(7);

        let upcoming_today = self
            .events
            .iter()
            .filter(|e| {
                e.status == EventStatus::Scheduled
                    && e.start_time >= now
                    && e.start_time < today_end
            })
            .count();

        let upcoming_week = self
            .events
            .iter()
            .filter(|e| {
                e.status == EventStatus::Scheduled
                    && e.start_time >= now
                    && e.start_time < week_end
            })
            .count();

        let categories_count = self.categories.len();

        CalendarStats {
            total_events,
            scheduled,
            in_progress,
            completed,
            cancelled,
            overdue,
            upcoming_today,
            upcoming_week,
            categories_count,
        }
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_store_path() -> Result<PathBuf> {
    // Try Firefly Labs production path first
    let firefly_config = PathBuf::from("/firefly/config/firecalendar");
    if firefly_config.exists() {
        return Ok(firefly_config.join("events.json"));
    }

    // Fallback to home directory
    let home = dirs::home_dir().ok_or_else(|| {
        Error::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Home directory not found",
        ))
    })?;

    Ok(home.join(".firecalendar").join("events.json"))
}