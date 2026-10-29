use crate::model::{ProjectData, Status};
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
    widgets::{Block, Borders, List, ListItem, Paragraph},
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

    let statuses = [Status::Todo, Status::Doing, Status::Done];

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(5),
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

                        let line = Line::from(vec![
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
                        ]);

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

            // Footer
            let footer_text =
                " [h/l] Switch Column  │  [j/k] Select Task  │  [Space] Advance  │  [x] Toggle Done  │  [b] Branch  │  [q] Quit ";
            f.render_widget(
                Paragraph::new(footer_text)
                    .style(Style::default().fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::ALL).title(" controls ")),
                chunks[2],
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
                                if let Ok(task) = store::find_mut(data, &id.to_string()) {
                                    task.status = match task.status {
                                        Status::Todo => Status::Doing,
                                        Status::Doing => {
                                            task.complete();
                                            Status::Done
                                        }
                                        Status::Done => {
                                            task.completed_at = None;
                                            Status::Todo
                                        }
                                    };
                                    task.updated_at = chrono::Utc::now();
                                    dirty = true;
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
                                if let Ok(task) = store::find_mut(data, &id.to_string()) {
                                    if task.status == Status::Done {
                                        task.status = Status::Todo;
                                        task.completed_at = None;
                                    } else {
                                        task.complete();
                                    }
                                    task.updated_at = chrono::Utc::now();
                                    dirty = true;
                                }
                            }
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
        store::save(data)?;
    }

    Ok(())
}
