#[macro_use]
extern crate serde_derive;

use chrono::{DateTime, Utc};
use tokio::runtime::Runtime;

pub mod api;
pub mod humanize;

#[derive(Deserialize, Debug)]
struct Project {
    id: u32,
    name: String,
}
#[derive(Deserialize)]
struct TogglTask {
    description: String,
    start: DateTime<Utc>,
    stop: Option<DateTime<Utc>>,
    project_id: Option<u32>,
}

pub struct Task {
    pub num: u32,
    pub name: String,
    pub project: String,
    pub start: DateTime<Utc>,
}

pub fn get_current_task(api_key: &str) -> Result<Option<Task>, ()> {
    let mut rt = Runtime::new().unwrap();

    let data = rt.block_on(api::fetch_api_future(&api_key, "time_entries"))?;
    let resp: Vec<TogglTask> = serde_json::from_str(data.as_str()).map_err(|_| ())?;

    resp.get(0).map_or_else(
        || Ok(None),
        |t| {
            if t.stop.is_some() {
                return Ok(None);
            }
            let project = fetch_project_from_pid(api_key, t.project_id);
            let task = Task {
                num: 0,
                name: t.description.clone(),
                start: t.start,
                project,
            };
            Ok(Some(task))
        },
    )
}

pub fn get_task_list(api_key: &str) -> Result<Vec<Task>, ()> {
    let mut rt = Runtime::new().unwrap();

    let data = rt.block_on(api::fetch_api_future(&api_key, "time_entries"))?;
    let tasks: Vec<TogglTask> = serde_json::from_str(data.as_str()).map_err(|_| ())?;
    let mut res: Vec<Task> = Vec::new();
    let mut num = 0;
    for t in tasks {
        let project = fetch_project_from_pid(&api_key, t.project_id);
        if !res
            .iter()
            .any(|task| task.name == t.description && task.project == project)
        {
            let task = Task {
                num,
                name: t.description,
                project,
                start: t.start,
            };
            res.push(task);
            num += 1;
            if num > 10 {
                break;
            }
        }
    }
    Ok(res)
}

fn fetch_project_from_pid(api_key: &str, pid: Option<u32>) -> String {
    let mut rt = Runtime::new().unwrap();
    pid.map(|pid| {
        rt.block_on(api::fetch_api_future(&api_key, "projects"))
            .map_err(|_| ())
            .and_then(|data| {
                let resp: Result<Vec<Project>, _> = serde_json::from_str(data.as_str());
                resp.map_err(|_| ())
            })
            .map(|responses| {
                responses
                    .iter()
                    .find(|x| x.id == pid)
                    .map_or("".to_string(), |p| p.name.clone())
            })
    })
    .and_then(|r| r.ok())
    .map_or("".to_string(), |x| x)
}
