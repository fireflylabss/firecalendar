use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Scheduled => "scheduled",
            EventStatus::InProgress => "in_progress",
            EventStatus::Completed => "completed",
            EventStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "scheduled" => Some(EventStatus::Scheduled),
            "in_progress" => Some(EventStatus::InProgress),
            "completed" => Some(EventStatus::Completed),
            "cancelled" => Some(EventStatus::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Recurrence {
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Custom { days: u32 },
}

impl Recurrence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Recurrence::None => "none",
            Recurrence::Daily => "daily",
            Recurrence::Weekly => "weekly",
            Recurrence::Monthly => "monthly",
            Recurrence::Yearly => "yearly",
            Recurrence::Custom { .. } => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Recurrence::None),
            "daily" => Some(Recurrence::Daily),
            "weekly" => Some(Recurrence::Weekly),
            "monthly" => Some(Recurrence::Monthly),
            "yearly" => Some(Recurrence::Yearly),
            "custom" => Some(Recurrence::Custom { days: 1 }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: EventStatus,
    pub category_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub recurrence: Recurrence,
    pub reminder_minutes: Option<u32>,
    pub timezone: Option<String>,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Event {
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Event {
            id: Uuid::new_v4(),
            title,
            description: None,
            start_time: now,
            end_time: None,
            status: EventStatus::Scheduled,
            category_id: None,
            tags: Vec::new(),
            recurrence: Recurrence::None,
            reminder_minutes: None,
            timezone: None,
            location: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_overdue(&self) -> bool {
        if self.status == EventStatus::Completed || self.status == EventStatus::Cancelled {
            return false;
        }
        self.start_time < Utc::now()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.end_time.map(|end| end.signed_duration_since(self.start_time))
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_start_time(mut self, start_time: DateTime<Utc>) -> Self {
        self.start_time = start_time;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = Some(end_time);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_category(mut self, category_id: Uuid) -> Self {
        self.category_id = Some(category_id);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_recurrence(mut self, recurrence: Recurrence) -> Self {
        self.recurrence = recurrence;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_reminder(mut self, minutes: u32) -> Self {
        self.reminder_minutes = Some(minutes);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_timezone(mut self, timezone: String) -> Self {
        self.timezone = Some(timezone);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_status(mut self, status: EventStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Category {
    pub fn new(name: String) -> Self {
        Category {
            id: Uuid::new_v4(),
            name,
            description: None,
            color: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventFilter {
    pub status: Option<EventStatus>,
    pub category_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub start_after: Option<DateTime<Utc>>,
    pub start_before: Option<DateTime<Utc>>,
    pub overdue_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarStats {
    pub total_events: usize,
    pub scheduled: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub cancelled: usize,
    pub overdue: usize,
    pub upcoming_today: usize,
    pub upcoming_week: usize,
    pub categories_count: usize,
}