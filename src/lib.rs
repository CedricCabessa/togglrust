#[macro_use]
extern crate serde_derive;

use chrono::{DateTime, Utc};
use tokio::runtime::Runtime;

pub mod api;
pub mod humanize;

#[derive(Deserialize)]
struct Response {
    data: Option<TogglTask>,
}
#[derive(Deserialize)]
struct ResponseProj {
    data: Project,
}
#[derive(Deserialize)]
struct Project {
    name: String,
}
#[derive(Deserialize)]
struct TogglTask {
    description: String,
    start: DateTime<Utc>,
    pid: Option<u32>,
}

pub struct Task {
    pub num: u32,
    pub name: String,
    pub project: String,
    pub start: DateTime<Utc>,
}

pub fn get_current_task(api_key: &str) -> Result<Option<Task>, ()> {
    let mut rt = Runtime::new().unwrap();

    rt.block_on(api::fetch_api_future(&api_key, "time_entries/current"))
        .map_err(|_| ())
        .and_then(|data| {
            let resp: Result<Response, _> = serde_json::from_str(data.as_str());
            resp.map_err(|_| ())
        })
        .map(|response| response.data)
        .map(|task| {
            task.map(|task| {
                let project = fetch_project_from_pid(api_key, task.pid);
                Task {
                    num: 0,
                    name: task.description,
                    start: task.start,
                    project,
                }
            })
        })
}

pub fn get_task_list(api_key: &str) -> Result<Vec<Task>, ()> {
    let mut rt = Runtime::new().unwrap();

    rt.block_on(api::fetch_api_future(&api_key, "time_entries"))
        .map_err(|_| ())
        .and_then(|data| {
            let resp: Result<Vec<TogglTask>, _> = serde_json::from_str(data.as_str());
            resp.map_err(|_| ())
        })
        .map(|mut v| {
            let mut res: Vec<Task> = Vec::new();
            let mut num = 0;
            v.reverse();
            for t in v {
                let project = fetch_project_from_pid(&api_key, t.pid);

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
            res
        })
}

fn fetch_project_from_pid(api_key: &str, pid: Option<u32>) -> String {
    let mut rt = Runtime::new().unwrap();
    pid.map(|pid| {
        rt.block_on(api::fetch_api_future(
            &api_key,
            &format!("projects/{}", pid),
        ))
        .map_err(|_| ())
        .and_then(|data| {
            let resp: Result<ResponseProj, _> = serde_json::from_str(data.as_str());
            resp.map_err(|_| ())
        })
        .map(|response| response.data.name)
    })
    .and_then(|r| r.ok())
    .map_or("".to_string(), |x| x)
}
