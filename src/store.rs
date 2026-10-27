use anyhow::{anyhow, Result};
use std::{fs, path::PathBuf};
use crate::model::{ProjectData, Task, Status};

pub fn data_dir() -> PathBuf { std::env::var_os("TASKLYNX_HOME").map(PathBuf::from).unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))).join("tasklynx") }
pub fn path() -> PathBuf { data_dir().join("tasks.json") }
pub fn load() -> Result<ProjectData> { let p=path(); if !p.exists() { return Ok(ProjectData::default()) } Ok(serde_json::from_str(&fs::read_to_string(p)?)?) }
pub fn save(data: &ProjectData) -> Result<()> { fs::create_dir_all(data_dir())?; let p=path(); let tmp=p.with_extension("json.tmp"); fs::write(&tmp, serde_json::to_vec_pretty(data)?)?; fs::rename(tmp,p)?; Ok(()) }
pub fn add(data:&mut ProjectData, task:Task) -> Result<()> { data.tasks.push(task); save(data) }
pub fn find_mut<'a>(data:&'a mut ProjectData,id:&str)->Result<&'a mut Task>{ let u=uuid::Uuid::parse_str(id).map_err(|_|anyhow!("invalid task id"))?; data.tasks.iter_mut().find(|t|t.id==u).ok_or_else(||anyhow!("task not found: {id}")) }
pub fn export_json(data:&ProjectData)->Result<String>{Ok(serde_json::to_string_pretty(data)?) }
pub fn export_md(data:&ProjectData)->String { let mut s=String::from("# Tasklynx\n\n"); for st in [Status::Todo,Status::Doing,Status::Done] { s.push_str(&format!("## {}\n\n",st)); for t in data.tasks.iter().filter(|t|t.status==st) { s.push_str(&format!("- [{}] **{}** `{}` [{}]{}\n",if t.status==Status::Done{"x"}else{" "},t.title,t.priority,t.project, if t.tags.is_empty(){String::new()}else{format!(" #{}",t.tags.join(" #"))})); } s.push('\n'); } s }
