use crate::core::{Category, Event, EventFilter, EventStatus, EventStore, Recurrence};
use anyhow::Result;
use chrono::{Datelike, Timelike, TimeZone, Utc};
use uuid::Uuid;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    CategoryList,
    EventDetail,
    AddEvent,
    AddCategory,
    CalendarView,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub store: EventStore,
    pub store_path: std::path::PathBuf,
    pub mode: Mode,
    pub input_mode: InputMode,
    pub selected_event_index: usize,
    pub selected_category_index: usize,
    pub selected_day_event_index: usize,
    pub events: Vec<Event>,
    pub categories: Vec<Category>,
    pub filter_status: Option<EventStatus>,
    pub input: String,
    pub message: String,
    pub message_timestamp: Option<Instant>,
    pub calendar_year: i32,
    pub calendar_month: u32,
    pub selected_day: Option<u32>,
    pub calendar_cursor_col: usize, // 0-6 (day of week)
    pub calendar_cursor_row: usize, // week number in month
}

impl App {
    pub fn new(store: EventStore, store_path: std::path::PathBuf) -> Self {
        let events = store.list_events(None).into_iter().cloned().collect();
        let categories = store.list_categories().into_iter().cloned().collect();

        let now = chrono::Utc::now();

        App {
            store,
            store_path,
            mode: Mode::CalendarView,
            input_mode: InputMode::Normal,
            selected_event_index: 0,
            selected_category_index: 0,
            selected_day_event_index: 0,
            events,
            categories,
            filter_status: None,
            input: String::new(),
            message: String::new(),
            message_timestamp: None,
            calendar_year: now.year(),
            calendar_month: now.month(),
            selected_day: Some(now.day()),
            calendar_cursor_col: now.weekday().num_days_from_sunday() as usize,
            calendar_cursor_row: 0,
        }
    }

    pub fn refresh_events(&mut self) {
        let filter = EventFilter {
            status: self.filter_status.clone(),
            category_id: None,
            tags: None,
            start_after: None,
            start_before: None,
            overdue_only: false,
        };
        self.events = self.store.list_events(Some(filter)).into_iter().cloned().collect();
        self.selected_event_index = 0;
    }

    pub fn refresh_categories(&mut self) {
        self.categories = self.store.list_categories().into_iter().cloned().collect();
        self.selected_category_index = 0;
    }

    pub fn next_event(&mut self) {
        if !self.events.is_empty() {
            self.selected_event_index = (self.selected_event_index + 1) % self.events.len();
        }
    }

    pub fn previous_event(&mut self) {
        if !self.events.is_empty() {
            self.selected_event_index = if self.selected_event_index == 0 {
                self.events.len() - 1
            } else {
                self.selected_event_index - 1
            };
        }
    }

    pub fn next_category(&mut self) {
        if !self.categories.is_empty() {
            self.selected_category_index = (self.selected_category_index + 1) % self.categories.len();
        }
    }

    pub fn previous_category(&mut self) {
        if !self.categories.is_empty() {
            self.selected_category_index = if self.selected_category_index == 0 {
                self.categories.len() - 1
            } else {
                self.selected_category_index - 1
            };
        }
    }

    pub fn toggle_event_complete(&mut self) -> Result<()> {
        if let Some(event) = self.events.get(self.selected_event_index) {
            let mut event_clone = event.clone();
            match event_clone.status {
                EventStatus::Completed => event_clone.status = EventStatus::Scheduled,
                _ => event_clone.status = EventStatus::Completed,
            }
            event_clone.updated_at = chrono::Utc::now();
            self.store.update_event(event_clone)?;
            self.refresh_events();
            self.save()?;
            self.set_message("Event updated!".to_string());
        }
        Ok(())
    }

    pub fn delete_event(&mut self) -> Result<()> {
        if let Some(event) = self.events.get(self.selected_event_index) {
            self.store.delete_event(event.id)?;
            self.refresh_events();
            self.save()?;
            self.set_message("Event deleted!".to_string());
        }
        Ok(())
    }

    pub fn add_event(&mut self, title: String) -> Result<()> {
        let now = Utc::now();
        let start_time = if let Some(day) = self.selected_day {
            chrono::Utc.with_ymd_and_hms(
                self.calendar_year,
                self.calendar_month,
                day,
                now.hour(),
                now.minute(),
                now.second()
            ).unwrap()
        } else {
            now
        };

        let event = Event {
            id: Uuid::new_v4(),
            title,
            description: None,
            start_time,
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
        };

        self.store.add_event(event)?;
        self.refresh_events();
        self.save()?;
        self.set_message("Event added!".to_string());
        self.input.clear();
        Ok(())
    }

    pub fn add_category(&mut self, name: String) -> Result<()> {
        let category = Category::new(name);
        self.store.add_category(category)?;
        self.refresh_categories();
        self.save()?;
        self.set_message("Category added!".to_string());
        self.input.clear();
        Ok(())
    }

    pub fn cycle_filter_status(&mut self) {
        self.filter_status = match &self.filter_status {
            None => Some(EventStatus::Scheduled),
            Some(EventStatus::Scheduled) => Some(EventStatus::InProgress),
            Some(EventStatus::InProgress) => Some(EventStatus::Completed),
            Some(EventStatus::Completed) => Some(EventStatus::Cancelled),
            Some(EventStatus::Cancelled) => None,
        };
        self.refresh_events();
    }

    pub fn clear_filters(&mut self) {
        self.filter_status = None;
        self.refresh_events();
    }

    pub fn next_month(&mut self) {
        if self.calendar_month == 12 {
            self.calendar_month = 1;
            self.calendar_year += 1;
        } else {
            self.calendar_month += 1;
        }
        self.selected_day = Some(1);
        self.calendar_cursor_row = 0;
        self.selected_day_event_index = 0;
    }

    pub fn previous_month(&mut self) {
        if self.calendar_month == 1 {
            self.calendar_month = 12;
            self.calendar_year -= 1;
        } else {
            self.calendar_month -= 1;
        }
        self.selected_day = Some(1);
        self.calendar_cursor_row = 0;
        self.selected_day_event_index = 0;
    }

    pub fn next_day(&mut self) {
        let days_in_month = self.days_in_month();
        if let Some(day) = self.selected_day {
            if day < days_in_month {
                self.selected_day = Some(day + 1);
                // Move cursor right
                self.calendar_cursor_col = (self.calendar_cursor_col + 1) % 7;
                if self.calendar_cursor_col == 0 {
                    self.calendar_cursor_row += 1;
                }
            } else {
                self.next_month();
            }
            self.selected_day_event_index = 0;
        } else {
            self.selected_day = Some(1);
            self.selected_day_event_index = 0;
        }
    }

    pub fn previous_day(&mut self) {
        if let Some(day) = self.selected_day {
            if day > 1 {
                self.selected_day = Some(day - 1);
                // Move cursor left
                if self.calendar_cursor_col == 0 {
                    self.calendar_cursor_col = 6;
                    if self.calendar_cursor_row > 0 {
                        self.calendar_cursor_row -= 1;
                    }
                } else {
                    self.calendar_cursor_col -= 1;
                }
            } else {
                self.previous_month();
                let days_in_month = self.days_in_month();
                self.selected_day = Some(days_in_month);
            }
            self.selected_day_event_index = 0;
        } else {
            self.selected_day = Some(1);
            self.selected_day_event_index = 0;
        }
    }

    pub fn go_to_today(&mut self) {
        let now = chrono::Utc::now();
        self.calendar_year = now.year();
        self.calendar_month = now.month();
        self.selected_day = Some(now.day());
        self.calendar_cursor_col = now.weekday().num_days_from_sunday() as usize;
        self.calendar_cursor_row = 0;
        self.selected_day_event_index = 0;
    }

    pub fn move_cursor_up(&mut self) {
        if self.calendar_cursor_row > 0 {
            self.calendar_cursor_row -= 1;
            // Calculate the day based on cursor position
            self.update_selected_day_from_cursor();
        } else {
            // Move to previous month
            self.previous_month();
            // Set cursor to last row
            self.calendar_cursor_row = 5; // Max weeks in a month
            self.update_selected_day_from_cursor();
        }
    }

    pub fn move_cursor_down(&mut self) {
        let max_weeks = (self.days_in_month() + 6) / 7 + 1; // Approximate max weeks
        if self.calendar_cursor_row < (max_weeks as usize) - 1 {
            self.calendar_cursor_row += 1;
            self.update_selected_day_from_cursor();
        } else {
            // Move to next month
            self.next_month();
            self.calendar_cursor_row = 0;
            self.update_selected_day_from_cursor();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.calendar_cursor_col > 0 {
            self.calendar_cursor_col -= 1;
            self.update_selected_day_from_cursor();
        } else {
            self.calendar_cursor_col = 6;
            self.move_cursor_up();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.calendar_cursor_col < 6 {
            self.calendar_cursor_col += 1;
            self.update_selected_day_from_cursor();
        } else {
            self.calendar_cursor_col = 0;
            self.move_cursor_down();
        }
    }

    fn update_selected_day_from_cursor(&mut self) {
        // Calculate day from cursor position
        let day_offset = self.calendar_cursor_row * 7 + self.calendar_cursor_col;
        let first_day_weekday = chrono::NaiveDate::from_ymd_opt(self.calendar_year, self.calendar_month, 1)
            .map(|d| d.weekday().num_days_from_sunday() as usize)
            .unwrap_or(0);
        
        let day = (day_offset as i32 - first_day_weekday as i32 + 1) as u32;
        
        if day >= 1 && day <= self.days_in_month() {
            self.selected_day = Some(day);
            self.selected_day_event_index = 0;
        }
    }

    pub fn get_selected_day_events(&self) -> Vec<&Event> {
        if let Some(day) = self.selected_day {
            self.get_events_for_day(day)
        } else {
            Vec::new()
        }
    }

    pub fn next_day_event(&mut self) {
        let events = self.get_selected_day_events();
        if !events.is_empty() {
            self.selected_day_event_index = (self.selected_day_event_index + 1) % events.len();
        }
    }

    pub fn previous_day_event(&mut self) {
        let events = self.get_selected_day_events();
        if !events.is_empty() {
            self.selected_day_event_index = if self.selected_day_event_index == 0 {
                events.len() - 1
            } else {
                self.selected_day_event_index - 1
            };
        }
    }

    pub fn toggle_selected_day_event_complete(&mut self) -> Result<()> {
        let event_id = self.get_selected_day_events()
            .get(self.selected_day_event_index)
            .map(|e| e.id);

        if let Some(id) = event_id {
            if let Some(mut event) = self.events.iter().find(|e| e.id == id).cloned() {
                match event.status {
                    EventStatus::Completed => event.status = EventStatus::Scheduled,
                    _ => event.status = EventStatus::Completed,
                }
                event.updated_at = chrono::Utc::now();
                self.store.update_event(event)?;
                self.refresh_events();
                self.save()?;
                self.set_message("Event updated!".to_string());
            }
        }
        Ok(())
    }

    pub fn delete_selected_day_event(&mut self) -> Result<()> {
        let event_id = self.get_selected_day_events()
            .get(self.selected_day_event_index)
            .map(|e| e.id);
        let events_len = self.get_selected_day_events().len();

        if let Some(id) = event_id {
            self.store.delete_event(id)?;
            self.refresh_events();
            self.save()?;
            self.set_message("Event deleted!".to_string());
            if self.selected_day_event_index > 0 && self.selected_day_event_index >= events_len.saturating_sub(1) {
                self.selected_day_event_index = self.selected_day_event_index.saturating_sub(1);
            }
        }
        Ok(())
    }

    pub fn view_selected_day_event_details(&mut self) {
        let events = self.get_selected_day_events();
        if events.get(self.selected_day_event_index).is_some() {
            // Store the selected event index for EventDetail mode
            self.selected_event_index = self.selected_day_event_index;
            self.mode = Mode::EventDetail;
        }
    }

    pub fn days_in_month(&self) -> u32 {
        // Handle month overflow (December + 1 = January next year)
        let (year, month) = if self.calendar_month == 12 {
            (self.calendar_year + 1, 1)
        } else {
            (self.calendar_year, self.calendar_month + 1)
        };

        // Get first day of next month
        let first_of_next_month = match chrono::NaiveDate::from_ymd_opt(year, month, 1) {
            Some(date) => date,
            None => {
                // Fallback to current date if invalid
                let now = chrono::Utc::now();
                match chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()) {
                    Some(date) => date,
                    None => {
                        // Ultimate fallback: use a safe default date
                        chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
                    }
                }
            }
        };

        // Get last day of current month
        match first_of_next_month.pred_opt() {
            Some(date) => date.day(),
            None => {
                // Fallback to a reasonable default
                31
            }
        }
    }

    pub fn get_events_for_day(&self, day: u32) -> Vec<&Event> {
        let start_of_day = chrono::Utc.with_ymd_and_hms(
            self.calendar_year,
            self.calendar_month,
            day,
            0, 0, 0
        ).single().unwrap_or(chrono::Utc::now());
        let end_of_day = start_of_day + chrono::Duration::days(1);

        self.events.iter()
            .filter(|e| {
                e.start_time >= start_of_day && e.start_time < end_of_day
            })
            .collect()
    }

    pub fn set_message(&mut self, msg: String) {
        self.message = msg;
        self.message_timestamp = Some(Instant::now());
    }

    pub fn clear_old_message(&mut self) {
        if let Some(timestamp) = self.message_timestamp {
            if timestamp.elapsed() >= std::time::Duration::from_secs(3) {
                self.message.clear();
                self.message_timestamp = None;
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        self.store.save(&self.store_path).map_err(|e| anyhow::anyhow!("Failed to save: {}", e))
    }
}