use crate::model::{Priority, ProjectData, Status, Task};
use anyhow::{anyhow, Context, Result};
use std::{fs, io::Write, path::PathBuf};

pub fn data_dir() -> PathBuf {
    std::env::var_os("TASKLYNX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("tasklynx")
}

pub fn path() -> PathBuf {
    data_dir().join("tasks.json")
}

pub fn lock_path() -> PathBuf {
    data_dir().join(".lock")
}

/// Advisory cross-process file lock with timeout and stale-lock recovery.
pub struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    pub fn acquire() -> Result<Self> {
        let dir = data_dir();
        fs::create_dir_all(&dir)?;
        let path = lock_path();
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(3);

        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut f) => {
                    let _ = writeln!(f, "pid:{}", std::process::id());
                    return Ok(Self { path });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Check if stale (older than 10 seconds)
                    if let Ok(meta) = fs::metadata(&path) {
                        if let Ok(mod_time) = meta.modified() {
                            if let Ok(elapsed) = mod_time.elapsed() {
                                if elapsed > std::time::Duration::from_secs(10) {
                                    let _ = fs::remove_file(&path);
                                    continue;
                                }
                            }
                        }
                    }
                    if start.elapsed() > timeout {
                        anyhow::bail!(
                            "could not acquire store lock at {} after {:?}. Another tasklynx process may be active.",
                            path.display(),
                            timeout
                        );
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn load() -> Result<ProjectData> {
    let p = path();
    if !p.exists() {
        return Ok(ProjectData::default());
    }
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}

pub fn save(data: &ProjectData) -> Result<()> {
    fs::create_dir_all(data_dir())?;
    let p = path();
    let tmp = p.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(&serde_json::to_vec_pretty(data)?)?;
        file.sync_all()?;
    }
    fs::rename(tmp, p)?;
    Ok(())
}

pub fn add(data: &mut ProjectData, task: Task) -> Result<()> {
    data.tasks.push(task);
    save(data)
}

pub fn find_mut<'a>(data: &'a mut ProjectData, id: &str) -> Result<&'a mut Task> {
    if let Ok(u) = uuid::Uuid::parse_str(id) {
        if let Some(idx) = data.tasks.iter().position(|t| t.id == u) {
            return Ok(&mut data.tasks[idx]);
        }
        return Err(anyhow!("task not found: {id}"));
    }

    let matches: Vec<usize> = data
        .tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.id.to_string().starts_with(id))
        .map(|(i, _)| i)
        .collect();

    match matches.len() {
        0 => Err(anyhow!("task not found: {id}")),
        1 => Ok(&mut data.tasks[matches[0]]),
        n => Err(anyhow!(
            "ambiguous task id '{id}' matches {n} tasks; please provide more characters"
        )),
    }
}

pub fn find<'a>(data: &'a ProjectData, id: &str) -> Result<&'a Task> {
    if let Ok(u) = uuid::Uuid::parse_str(id) {
        if let Some(t) = data.tasks.iter().find(|t| t.id == u) {
            return Ok(t);
        }
        return Err(anyhow!("task not found: {id}"));
    }

    let matches: Vec<&Task> = data
        .tasks
        .iter()
        .filter(|t| t.id.to_string().starts_with(id))
        .collect();

    match matches.len() {
        0 => Err(anyhow!("task not found: {id}")),
        1 => Ok(matches[0]),
        n => Err(anyhow!(
            "ambiguous task id '{id}' matches {n} tasks; please provide more characters"
        )),
    }
}

pub fn export_json(data: &ProjectData) -> Result<String> {
    Ok(serde_json::to_string_pretty(data)?)
}

pub fn export_md(data: &ProjectData) -> String {
    let mut s = String::from("# Tasklynx\n\n");
    for st in [Status::Todo, Status::Doing, Status::Done] {
        s.push_str(&format!("## {}\n\n", st));
        for t in data.tasks.iter().filter(|t| t.status == st) {
            s.push_str(&format!(
                "- [{}] **{}** `{}` [{}]{}\n",
                if t.status == Status::Done { "x" } else { " " },
                t.title,
                t.priority,
                t.project,
                if t.tags.is_empty() {
                    String::new()
                } else {
                    format!(" #{}", t.tags.join(" #"))
                }
            ));
        }
        s.push('\n');
    }
    s
}

/// Import tasks from JSON data (either full ProjectData or a list of Task objects).
pub fn import_json(data: &mut ProjectData, content: &str, replace: bool) -> Result<(usize, usize)> {
    let imported_tasks: Vec<Task> =
        if let Ok(project) = serde_json::from_str::<ProjectData>(content) {
            for b in project.branches {
                if !data.branches.contains(&b) {
                    data.branches.push(b);
                }
            }
            project.tasks
        } else {
            serde_json::from_str::<Vec<Task>>(content)
                .context("invalid JSON format: expected ProjectData or Task array")?
        };

    if replace {
        let count = imported_tasks.len();
        data.tasks = imported_tasks;
        return Ok((count, 0));
    }

    let mut added = 0;
    let mut updated = 0;
    for task in imported_tasks {
        if let Some(existing) = data.tasks.iter_mut().find(|t| t.id == task.id) {
            *existing = task;
            updated += 1;
        } else {
            data.tasks.push(task);
            added += 1;
        }
    }
    Ok((added, updated))
}

/// Import tasks from Markdown markdown list.
pub fn import_md(data: &mut ProjectData, content: &str, replace: bool) -> Result<(usize, usize)> {
    let mut tasks = Vec::new();
    let mut current_status = Status::Todo;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("##") {
            let section = line.trim_start_matches('#').trim().to_lowercase();
            if section.contains("doing") {
                current_status = Status::Doing;
            } else if section.contains("done") {
                current_status = Status::Done;
            } else {
                current_status = Status::Todo;
            }
            continue;
        }

        if !line.starts_with("- [") && !line.starts_with("* [") {
            continue;
        }

        let is_done = line.starts_with("- [x]")
            || line.starts_with("- [X]")
            || line.starts_with("* [x]")
            || line.starts_with("* [X]");

        let status = if is_done {
            Status::Done
        } else {
            current_status.clone()
        };

        // Extract content after `]`
        if let Some(bracket_end) = line.find(']') {
            let rest = line[bracket_end + 1..].trim();
            if rest.is_empty() {
                continue;
            }

            let mut title = rest.to_string();
            let mut project = "default".to_string();
            let mut priority = Priority::Normal;
            let mut tags = Vec::new();

            // Extract tags like `#tag`
            let words: Vec<&str> = rest.split_whitespace().collect();
            let mut title_words = Vec::new();

            for word in words {
                if let Some(tag) = word.strip_prefix('#') {
                    if !tag.is_empty() {
                        tags.push(tag.to_string());
                    }
                } else if word.starts_with('[') && word.ends_with(']') && word.len() > 2 {
                    project = word[1..word.len() - 1].to_string();
                } else if word.starts_with('`') && word.ends_with('`') && word.len() > 2 {
                    let pri_str = &word[1..word.len() - 1];
                    priority = match pri_str.to_lowercase().as_str() {
                        "low" => Priority::Low,
                        "high" => Priority::High,
                        "urgent" => Priority::Urgent,
                        _ => Priority::Normal,
                    };
                } else {
                    title_words.push(word);
                }
            }

            if !title_words.is_empty() {
                title = title_words.join(" ").trim_matches('*').trim().to_string();
            }

            let mut t = Task::new(
                title,
                project,
                data.current_branch.clone(),
                priority,
                tags,
                None,
            );
            t.status = status;
            if is_done {
                t.completed_at = Some(chrono::Utc::now());
            }
            tasks.push(t);
        }
    }

    if replace {
        let count = tasks.len();
        data.tasks = tasks;
        return Ok((count, 0));
    }

    let mut added = 0;
    for task in tasks {
        data.tasks.push(task);
        added += 1;
    }
    Ok((added, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Priority, Task};

    fn sample() -> (ProjectData, Task) {
        let mut data = ProjectData::default();
        let task = Task::new(
            "Write tests".into(),
            "api".into(),
            "main".into(),
            Priority::High,
            vec!["backend".into()],
            None,
        );
        data.tasks.push(task.clone());
        (data, task)
    }

    #[test]
    fn finds_by_unique_prefix() {
        let (data, task) = sample();
        let prefix = &task.id.to_string()[..8];
        assert_eq!(find(&data, prefix).unwrap().id, task.id);
    }

    #[test]
    fn finds_by_full_uuid() {
        let (data, task) = sample();
        assert_eq!(find(&data, &task.id.to_string()).unwrap().id, task.id);
    }

    #[test]
    fn missing_task_is_reported() {
        let (mut data, _) = sample();
        assert!(find(&data, "deadbeef").is_err());
        assert!(find_mut(&mut data, "deadbeef").is_err());
    }

    #[test]
    fn rejects_ambiguous_prefix() {
        let mut data = ProjectData::default();
        let mut t1 = Task::new(
            "T1".into(),
            "p".into(),
            "main".into(),
            Priority::Normal,
            vec![],
            None,
        );
        let mut t2 = Task::new(
            "T2".into(),
            "p".into(),
            "main".into(),
            Priority::Normal,
            vec![],
            None,
        );
        t1.id = uuid::Uuid::parse_str("aaaaaaaa-0000-0000-0000-000000000001").unwrap();
        t2.id = uuid::Uuid::parse_str("aaaaaaaa-0000-0000-0000-000000000002").unwrap();
        data.tasks.push(t1);
        data.tasks.push(t2);

        let err = find(&data, "aaaa").unwrap_err();
        assert!(err.to_string().contains("ambiguous"));
        let err_mut = find_mut(&mut data, "aaaa").unwrap_err();
        assert!(err_mut.to_string().contains("ambiguous"));
    }

    #[test]
    fn export_json_includes_task() {
        let (data, task) = sample();
        let json = export_json(&data).unwrap();
        assert!(json.contains(&task.id.to_string()));
        assert!(json.contains("Write tests"));
    }

    #[test]
    fn export_markdown_groups_by_status() {
        let (data, _) = sample();
        let md = export_md(&data);
        assert!(md.contains("## todo"));
        assert!(md.contains("## doing"));
        assert!(md.contains("## done"));
        assert!(md.contains("Write tests"));
    }

    #[test]
    fn import_json_works() {
        let (data, _) = sample();
        let json = export_json(&data).unwrap();
        let mut new_data = ProjectData::default();
        let (added, updated) = import_json(&mut new_data, &json, false).unwrap();
        assert_eq!(added, 1);
        assert_eq!(updated, 0);
        assert_eq!(new_data.tasks.len(), 1);
        assert_eq!(new_data.tasks[0].title, "Write tests");
    }

    #[test]
    fn import_md_works() {
        let md = "# Tasklynx\n\n## todo\n\n- [ ] **Imported item** `high` [ops] #server\n- [x] **Finished item**";
        let mut data = ProjectData::default();
        let (added, _) = import_md(&mut data, md, false).unwrap();
        assert_eq!(added, 2);
        assert_eq!(data.tasks[0].title, "Imported item");
        assert_eq!(data.tasks[0].priority, Priority::High);
        assert_eq!(data.tasks[0].project, "ops");
        assert_eq!(data.tasks[0].tags, vec!["server"]);
        assert_eq!(data.tasks[1].status, Status::Done);
    }
}
