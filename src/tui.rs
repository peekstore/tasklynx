use crate::ai;
use crate::config;
use crate::model::{ProjectData, Repeat, Status, Task};
use crate::store;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io::stdout;

pub fn run(data: &mut ProjectData) -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let mut active_col: usize = 0; // 0: Todo, 1: Doing, 2: Done
    let mut selected_idx: [usize; 3] = [0, 0, 0];
    let mut dirty = false;
    let mut ai_detail: Option<String> = None;
    let undo_base = data.clone();

    let statuses = [Status::Todo, Status::Doing, Status::Done];

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(9),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            // Header
            let branch_str = format!(" branch: {} ", data.current_branch);
            let header_text = format!(
                "TASKLYNX  •  terminal workspace  •  tasks: {}  •  {}",
                data.tasks.len(),
                branch_str
            );
            f.render_widget(
                Paragraph::new(header_text)
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" tasklynx dashboard "),
                    ),
                chunks[0],
            );

            // Columns (Todo, Doing, Done)
            let col_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            for (col_i, st) in statuses.iter().enumerate() {
                let tasks_in_col: Vec<&crate::model::Task> = data
                    .tasks
                    .iter()
                    .filter(|t| &t.status == st && t.branch == data.current_branch)
                    .collect();

                let is_active_col = col_i == active_col;
                let cur_sel = selected_idx[col_i];

                let items: Vec<ListItem> = tasks_in_col
                    .iter()
                    .enumerate()
                    .map(|(item_i, t)| {
                        let is_sel = is_active_col && item_i == cur_sel;
                        let prefix = if is_sel { "▶ " } else { "  " };

                        let pri_color = match t.priority {
                            crate::model::Priority::Urgent => Color::Red,
                            crate::model::Priority::High => Color::Yellow,
                            crate::model::Priority::Normal => Color::White,
                            crate::model::Priority::Low => Color::DarkGray,
                        };

                        let mut spans = vec![
                            Span::raw(prefix),
                            Span::styled(
                                format!("{:.8} ", t.id),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                format!("{} ", t.title),
                                if is_sel {
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default().fg(Color::White)
                                },
                            ),
                            Span::styled(
                                format!("[{}] ", t.priority),
                                Style::default().fg(pri_color),
                            ),
                            Span::styled(
                                format!("<{}>", t.project),
                                Style::default().fg(Color::Cyan),
                            ),
                        ];
                        if t.repeat != Repeat::None {
                            spans.push(Span::styled(
                                format!(" ↻{}", t.repeat),
                                Style::default().fg(Color::Magenta),
                            ));
                        }
                        let line = Line::from(spans);

                        let item_style = if is_sel {
                            Style::default().bg(Color::Rgb(30, 35, 45))
                        } else {
                            Style::default()
                        };

                        ListItem::new(line).style(item_style)
                    })
                    .collect();

                let border_style = if is_active_col {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let col_title = format!(" {} ({}) ", st, tasks_in_col.len());
                let list = List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title(col_title),
                );
                f.render_widget(list, col_chunks[col_i]);
            }

            // Detail / AI panel
            let selected = data
                .tasks
                .iter()
                .filter(|task| task.status == statuses[active_col] && task.branch == data.current_branch)
                .nth(selected_idx[active_col]);
            let mut detail = match selected {
                Some(task) => selected_task_detail(task, data),
                None => "No task selected.".to_string(),
            };
            if let Some(extra) = &ai_detail {
                detail.push_str("\n\n");
                detail.push_str(extra);
            }
            f.render_widget(
                Paragraph::new(detail)
                    .wrap(Wrap { trim: false })
                    .style(Style::default().fg(Color::Gray))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(" details / ai "),
                    ),
                chunks[2],
            );

            // Footer
            let footer_text =
                " [h/l] Column  │  [j/k] Select  │  [Space] Advance  │  [x] Done  │  [a] AI breakdown  │  [t] AI triage  │  [c] Clear  │  [b] Branch  │  [q] Quit ";
            f.render_widget(
                Paragraph::new(footer_text)
                    .style(Style::default().fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::ALL).title(" controls ")),
                chunks[3],
            );
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Press {
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            active_col = active_col.saturating_sub(1);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            if active_col < 2 {
                                active_col += 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let count = data
                                .tasks
                                .iter()
                                .filter(|t| {
                                    t.status == statuses[active_col]
                                        && t.branch == data.current_branch
                                })
                                .count();
                            if count > 0 && selected_idx[active_col] + 1 < count {
                                selected_idx[active_col] += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            selected_idx[active_col] = selected_idx[active_col].saturating_sub(1);
                        }
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            let cur_status = &statuses[active_col];
                            let task_id = {
                                let tasks: Vec<&crate::model::Task> = data
                                    .tasks
                                    .iter()
                                    .filter(|t| {
                                        &t.status == cur_status && t.branch == data.current_branch
                                    })
                                    .collect();
                                tasks.get(selected_idx[active_col]).map(|t| t.id)
                            };

                            if let Some(id) = task_id {
                                let next = if let Ok(task) = store::find_mut(data, &id.to_string())
                                {
                                    let target = match task.status {
                                        Status::Todo => Status::Doing,
                                        Status::Doing => Status::Done,
                                        Status::Done => Status::Todo,
                                    };
                                    let next = task.set_status(target);
                                    dirty = true;
                                    next
                                } else {
                                    None
                                };
                                if let Some(next) = next {
                                    data.tasks.push(next);
                                }
                            }
                        }
                        KeyCode::Char('x') => {
                            let cur_status = &statuses[active_col];
                            let task_id = {
                                let tasks: Vec<&crate::model::Task> = data
                                    .tasks
                                    .iter()
                                    .filter(|t| {
                                        &t.status == cur_status && t.branch == data.current_branch
                                    })
                                    .collect();
                                tasks.get(selected_idx[active_col]).map(|t| t.id)
                            };

                            if let Some(id) = task_id {
                                let next = if let Ok(task) = store::find_mut(data, &id.to_string())
                                {
                                    let target = if task.status == Status::Done {
                                        Status::Todo
                                    } else {
                                        Status::Done
                                    };
                                    let next = task.set_status(target);
                                    dirty = true;
                                    next
                                } else {
                                    None
                                };
                                if let Some(next) = next {
                                    data.tasks.push(next);
                                }
                            }
                        }
                        KeyCode::Char('a') => {
                            if let Some(id) = selected_task_id(
                                data,
                                &statuses[active_col],
                                selected_idx[active_col],
                            ) {
                                ai_detail = Some(match ai_breakdown_preview(data, id) {
                                    Ok(text) => text,
                                    Err(error) => format!("AI breakdown failed: {error}"),
                                });
                            }
                        }
                        KeyCode::Char('t') => {
                            ai_detail = Some(match ai_triage_preview(data) {
                                Ok(text) => text,
                                Err(error) => format!("AI triage failed: {error}"),
                            });
                        }
                        KeyCode::Char('c') => {
                            ai_detail = None;
                        }
                        KeyCode::Char('b') if !data.branches.is_empty() => {
                            // Cycle through branches
                            let cur_idx = data
                                .branches
                                .iter()
                                .position(|b| b == &data.current_branch)
                                .unwrap_or(0);
                            let next_idx = (cur_idx + 1) % data.branches.len();
                            data.current_branch = data.branches[next_idx].clone();
                            selected_idx = [0, 0, 0];
                            dirty = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if dirty {
        store::save_with_undo(data, &undo_base)?;
    }

    Ok(())
}

fn selected_task_id(data: &ProjectData, status: &Status, index: usize) -> Option<uuid::Uuid> {
    data.tasks
        .iter()
        .filter(|task| &task.status == status && task.branch == data.current_branch)
        .nth(index)
        .map(|task| task.id)
}

fn selected_task_detail(task: &Task, data: &ProjectData) -> String {
    let id = task.id.to_string();
    let due = task
        .due
        .map(|due| {
            due.with_timezone(&chrono::Local)
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_else(|| "none".into());
    let blockers = task
        .blocked_by
        .iter()
        .filter(|id| {
            data.tasks
                .iter()
                .any(|candidate| candidate.id == **id && candidate.status != Status::Done)
        })
        .count();
    let mut text = format!(
        "{}  {}\nid: {}\nstatus: {} · priority: {} · project: {}\ndue: {} · branch: {} · blockers: {}\n",
        &id[..8],
        task.title,
        id,
        task.status,
        task.priority,
        task.project,
        due,
        task.branch,
        blockers
    );
    if !task.description.trim().is_empty() {
        let description: String = task.description.chars().take(400).collect();
        text.push_str(&description.replace(['\r', '\n'], " "));
    }
    text
}

fn ai_breakdown_preview(data: &ProjectData, id: uuid::Uuid) -> Result<String> {
    let parent = store::find(data, &id.to_string())?.clone();
    let children: Vec<Task> = data
        .tasks
        .iter()
        .filter(|task| task.parent == Some(parent.id))
        .cloned()
        .collect();
    let config = config::load()?;
    let provider = ai::provider(&config.ai)?;
    let plan = ai::request_breakdown(
        provider.as_ref(),
        &config.ai,
        &parent,
        &children,
        10,
        config.ai.include_descriptions,
    )?;
    let tasks = ai::breakdown_tasks(&parent, &plan)?;
    let mut text = format!(
        "AI breakdown · {}\nparent: {} {}\n",
        provider.name(),
        &parent.id.to_string()[..8],
        parent.title
    );
    for (index, task) in tasks.iter().enumerate() {
        text.push_str(&format!(
            "{}. [{}] {}\n",
            index + 1,
            task.priority,
            task.title
        ));
    }
    text.push_str("preview only; use `tx ai breakdown <id> --apply` to create");
    Ok(text)
}

fn ai_triage_preview(data: &ProjectData) -> Result<String> {
    let config = config::load()?;
    let request = ai::TriageRequest {
        tasks: &data.tasks,
        branch: &data.current_branch,
        project: None,
        limit: 5,
        include_blocked: false,
        include_descriptions: config.ai.include_descriptions,
    };
    let no_candidates =
        ai::triage_candidate_count(&data.tasks, &data.current_branch, None, false) == 0;
    let (provider, summary, picks) = if no_candidates
        || matches!(config.ai.provider_kind()?, ai::ProviderKind::None)
    {
        (
            "offline".to_string(),
            if no_candidates {
                "no eligible tasks to triage".to_string()
            } else {
                "offline triage".to_string()
            },
            ai::deterministic_triage(request),
        )
    } else {
        let provider = ai::provider(&config.ai)?;
        let response = ai::request_triage(provider.as_ref(), &config.ai, request)?;
        let picks = ai::resolve_triage(&response, &data.tasks, &data.current_branch, None, false)?;
        (provider.name().to_string(), response.summary, picks)
    };
    let mut text = format!("AI triage · {provider}\n{summary}\n");
    if picks.is_empty() {
        text.push_str("no eligible tasks");
    }
    for (index, pick) in picks.iter().enumerate() {
        if let Ok(task) = store::find(data, &pick.task_id.to_string()) {
            text.push_str(&format!(
                "{}. {}  [{}] {}  {}\n   {}\n",
                index + 1,
                &task.id.to_string()[..8],
                task.priority,
                task.project,
                task.title,
                pick.reason
            ));
        }
    }
    Ok(text)
}
