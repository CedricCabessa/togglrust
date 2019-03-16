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
                let project = task
                    .pid
                    .map(|pid| {
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
                    .map_or("".to_string(), |x| x);
                Task {
                    name: task.description,
                    start: task.start,
                    project,
                }
            })
        })
}
