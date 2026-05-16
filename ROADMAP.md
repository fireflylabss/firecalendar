# FireCalendar Roadmap

## Overview

FireCalendar is an integrated calendar for the Fire Suite, designed for developers and power users who want seamless scheduling and productivity integration.

**Status**: Planning (v0.1.0 planned for Oct 2026)

---

## v0.1.0 - MVP

- [ ] Basic calendar (month/week/day view)
- [ ] Event creation/editing
- [ ] Recurring events
- [ ] Reminders
- [ ] Calendar import/export (iCal)
- [ ] CLI interface

---

## v0.2.0 - Integration

- [ ] Google Calendar sync
- [ ] Outlook Calendar sync
- [ ] Apple Calendar sync
- [ ] firetasks integration (events → tasks)
- [ ] firenotes integration (meeting notes)
- [ ] Video call links (Zoom, Meet, Teams)

---

## v0.3.0 - Collaboration

- [ ] Shared calendars
- [ ] Scheduling assistant (find time)
- [ ] Meeting polls
- [ ] Room booking
- [ ] Attendee management
- [ ] RSVP system

---

## v1.0.0 - Platform

- [ ] Mobile app
- [ ] Desktop app
- [ ] Team features
- [ ] Resource management
- [ ] Analytics

---

## Integration with Fire Suite

- **firekeep**: Credentials for calendar services
- **firetasks**: Convert events to tasks, task reminders in calendar
- **firenotes**: Meeting notes linked to calendar events
- **fireworker**: Calendar widget in dashboard
- **firenotify**: Calendar event notifications

---

## Storage Pattern

```
/firefly/config/firecalendar/
├── events.json          # Calendar events
├── calendars.json       # Calendar sources
└── sync-state.json      # Sync status
```

---

**Last updated**: 2026-05-03