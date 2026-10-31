use crate::model::{local_end_of_day, Priority, Repeat, Status, Task};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use anyhow::Context;
use anyhow::{bail, Result};
use chrono::{DateTime, Days, Local, Utc};
use std::fmt::Write as _;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::Command;

pub fn upcoming(tasks: &[Task], days: u32, now: DateTime<Utc>) -> Vec<&Task> {
    let today = now.with_timezone(&Local).date_naive();
    let horizon_date = today
        .checked_add_days(Days::new(u64::from(days)))
        .unwrap_or(today);
    let horizon = local_end_of_day(horizon_date);
    let mut due: Vec<&Task> = tasks
        .iter()
        .filter(|task| task.status != Status::Done)
        .filter(|task| task.due.is_some_and(|due| due <= horizon))
        .collect();
    due.sort_by(|a, b| {
        a.due
            .cmp(&b.due)
            .then_with(|| b.priority.cmp(&a.priority))
            .then_with(|| a.title.cmp(&b.title))
    });
    due
}

pub fn reminder_label(due: DateTime<Utc>, now: DateTime<Utc>) -> &'static str {
    let today = now.with_timezone(&Local).date_naive();
    let due_date = due.with_timezone(&Local).date_naive();
    if due_date < today {
        "OVERDUE"
    } else if due_date == today {
        "TODAY"
    } else {
        "UPCOMING"
    }
}

pub fn notification_text(tasks: &[&Task]) -> (String, String) {
    let title = format!("Tasklynx: {} task(s) due", tasks.len());
    let mut body = tasks
        .iter()
        .take(3)
        .map(|task| task.title.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    if tasks.len() > 3 {
        let _ = write!(body, ", +{} more", tasks.len() - 3);
    }
    (title, body)
}

pub fn export_ics(tasks: &[Task], now: DateTime<Utc>) -> String {
    let stamp = now.format("%Y%m%dT%H%M%SZ");
    let mut out = String::new();
    push_line(&mut out, "BEGIN:VCALENDAR");
    push_line(&mut out, "VERSION:2.0");
    push_line(&mut out, "PRODID:-//Tasklynx//Tasklynx//EN");
    push_line(&mut out, "CALSCALE:GREGORIAN");
    push_line(&mut out, "METHOD:PUBLISH");
    push_line(&mut out, "X-WR-CALNAME:Tasklynx");
    for task in tasks {
        let Some(due) = task.due else {
            continue;
        };
        let date = due.with_timezone(&Local).date_naive();
        let end = date.checked_add_days(Days::new(1)).unwrap_or(date);
        push_line(&mut out, "BEGIN:VEVENT");
        push_line(&mut out, &format!("UID:{}@tasklynx", task.id));
        push_line(&mut out, &format!("DTSTAMP:{stamp}"));
        push_line(
            &mut out,
            &format!("DTSTART;VALUE=DATE:{}", date.format("%Y%m%d")),
        );
        push_line(
            &mut out,
            &format!("DTEND;VALUE=DATE:{}", end.format("%Y%m%d")),
        );
        push_line(&mut out, &format!("SUMMARY:{}", escape_text(&task.title)));
        let description = description_text(task);
        if !description.is_empty() {
            push_line(
                &mut out,
                &format!("DESCRIPTION:{}", escape_text(&description)),
            );
        }
        push_line(
            &mut out,
            &format!(
                "STATUS:{}",
                if task.status == Status::Done {
                    "COMPLETED"
                } else {
                    "CONFIRMED"
                }
            ),
        );
        push_line(&mut out, "TRANSP:TRANSPARENT");
        push_line(
            &mut out,
            &format!("PRIORITY:{}", ics_priority(&task.priority)),
        );
        if task.status != Status::Done {
            if let Some(rule) = recurrence_rule(task) {
                push_line(&mut out, &format!("RRULE:{rule}"));
            }
        }
        push_line(&mut out, "END:VEVENT");
    }
    push_line(&mut out, "END:VCALENDAR");
    out
}

fn description_text(task: &Task) -> String {
    let mut parts = Vec::new();
    if !task.description.trim().is_empty() {
        parts.push(task.description.trim().to_string());
    }
    parts.push(format!("Project: {}", task.project));
    parts.push(format!("Priority: {}", task.priority));
    if !task.tags.is_empty() {
        parts.push(format!("Tags: {}", task.tags.join(", ")));
    }
    if task.branch != "main" {
        parts.push(format!("Branch: {}", task.branch));
    }
    if let Some(estimate) = task.estimate_minutes {
        parts.push(format!("Estimate: {estimate} minutes"));
    }
    parts.push(format!("Tasklynx ID: {}", task.id));
    parts.join("\n")
}

fn recurrence_rule(task: &Task) -> Option<String> {
    let frequency = match task.repeat {
        Repeat::None => return None,
        Repeat::Daily => "DAILY",
        Repeat::Weekly => "WEEKLY",
        Repeat::Monthly => "MONTHLY",
        Repeat::Yearly => "YEARLY",
    };
    let mut rule = format!("FREQ={frequency}");
    if let Some(until) = task.repeat_until {
        let date = until.with_timezone(&Local).date_naive();
        let _ = write!(rule, ";UNTIL={}", date.format("%Y%m%d"));
    }
    Some(rule)
}

fn ics_priority(priority: &Priority) -> u8 {
    match priority {
        Priority::Urgent => 1,
        Priority::High => 3,
        Priority::Normal => 5,
        Priority::Low => 9,
    }
}

fn escape_text(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

fn push_line(out: &mut String, line: &str) {
    let mut first = true;
    let mut chunk = String::new();
    let mut bytes = 0;
    for ch in line.chars() {
        let length = ch.len_utf8();
        let limit = if first { 74 } else { 73 };
        if bytes + length > limit && !chunk.is_empty() {
            out.push_str(&chunk);
            out.push_str("\r\n ");
            chunk.clear();
            bytes = 0;
            first = false;
        }
        chunk.push(ch);
        bytes += length;
    }
    out.push_str(&chunk);
    out.push_str("\r\n");
}

#[cfg(target_os = "linux")]
pub fn notify(summary: &str, body: &str) -> Result<()> {
    let status = Command::new("notify-send")
        .args([
            "--app-name=Tasklynx",
            "--urgency=normal",
            "--",
            summary,
            body,
        ])
        .status()
        .context("failed to run notify-send; install libnotify (or omit --notify)")?;
    if !status.success() {
        bail!("notify-send exited with {status}");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn notify(summary: &str, body: &str) -> Result<()> {
    let script = format!(
        "display notification \"{}\" with title \"Tasklynx\" subtitle \"{}\"",
        applescript_escape(body),
        applescript_escape(summary)
    );
    let status = Command::new("osascript")
        .args(["-e", &script])
        .status()
        .context("failed to run osascript for desktop notification")?;
    if !status.success() {
        bail!("osascript exited with {status}");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn notify(_summary: &str, _body: &str) -> Result<()> {
    bail!("--notify is not supported on Windows; use `tx remind` without --notify")
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn notify(_summary: &str, _body: &str) -> Result<()> {
    bail!("--notify is not supported on this platform")
}

#[cfg(target_os = "macos")]
fn applescript_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn task(title: &str, due: Option<DateTime<Utc>>) -> Task {
        Task::new(
            title.into(),
            "default".into(),
            "main".into(),
            Priority::Normal,
            vec![],
            due,
        )
    }

    fn due_in(days: i64) -> DateTime<Utc> {
        local_end_of_day(Local::now().date_naive() + Days::new(days.unsigned_abs()))
    }

    #[test]
    fn upcoming_includes_overdue_today_and_window() {
        let now = Utc::now();
        let today = now.with_timezone(&Local).date_naive();
        let yesterday = local_end_of_day(today - Days::new(1));
        let today_due = local_end_of_day(today);
        let soon = local_end_of_day(today + Days::new(2));
        let far = local_end_of_day(today + Days::new(10));
        let mut done = task("Done", Some(today_due));
        done.status = Status::Done;
        let tasks = vec![
            task("Yesterday", Some(yesterday)),
            task("Today", Some(today_due)),
            task("Soon", Some(soon)),
            task("Far", Some(far)),
            task("No due", None),
            done,
        ];
        let upcoming = upcoming(&tasks, 3, now);
        let titles: Vec<&str> = upcoming.iter().map(|task| task.title.as_str()).collect();
        assert_eq!(titles, vec!["Yesterday", "Today", "Soon"]);
        assert_eq!(reminder_label(yesterday, now), "OVERDUE");
        assert_eq!(reminder_label(today_due, now), "TODAY");
        assert_eq!(reminder_label(soon, now), "UPCOMING");
    }

    #[test]
    fn notification_text_limits_titles() {
        let tasks: Vec<Task> = (1..=5)
            .map(|index| task(&format!("Task {index}"), Some(due_in(1))))
            .collect();
        let refs: Vec<&Task> = tasks.iter().collect();
        let (title, body) = notification_text(&refs);
        assert_eq!(title, "Tasklynx: 5 task(s) due");
        assert_eq!(body, "Task 1, Task 2, Task 3, +2 more");
    }

    #[test]
    fn exports_ics_with_escaping_and_recurrence() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let due = local_end_of_day(chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap());
        let mut task = task("Fix, build; \"now\"", Some(due));
        task.description = "Line one\nLine two".into();
        task.repeat = Repeat::Weekly;
        task.repeat_until = Some(local_end_of_day(
            chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        ));
        let ics = export_ics(&[task], now);
        assert!(ics.contains("BEGIN:VCALENDAR\r\n"), "{ics}");
        assert!(ics.contains("UID:"), "{ics}");
        assert!(ics.contains("DTSTAMP:20260101T120000Z"), "{ics}");
        assert!(ics.contains("DTSTART;VALUE=DATE:20260102"), "{ics}");
        assert!(ics.contains("DTEND;VALUE=DATE:20260103"), "{ics}");
        assert!(ics.contains("SUMMARY:Fix\\, build\\; \"now\""), "{ics}");
        assert!(ics.contains("DESCRIPTION:Line one\\nLine two"), "{ics}");
        assert!(ics.contains("RRULE:FREQ=WEEKLY;UNTIL=20260301"), "{ics}");
        assert!(ics.contains("PRIORITY:5"), "{ics}");
    }

    #[test]
    fn ics_omits_tasks_without_due_and_marks_done() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let mut done = task(
            "Finished",
            Some(local_end_of_day(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            )),
        );
        done.status = Status::Done;
        done.repeat = Repeat::Weekly;
        let ics = export_ics(&[task("No due", None), done], now);
        assert!(!ics.contains("No due"), "{ics}");
        assert!(ics.contains("SUMMARY:Finished"), "{ics}");
        assert!(ics.contains("STATUS:COMPLETED"), "{ics}");
        assert!(!ics.contains("RRULE:"), "{ics}");
    }

    #[test]
    fn ics_folds_long_lines() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let long = "A".repeat(300);
        let due = local_end_of_day(chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap());
        let ics = export_ics(&[task(&long, Some(due))], now);
        assert!(ics.contains("\r\n "), "{ics}");
        assert!(ics.lines().all(|line| line.len() <= 75), "{ics}");
    }
}
