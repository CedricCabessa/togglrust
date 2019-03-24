#[macro_use]
extern crate serde_derive;

use chrono::{DateTime, Utc};
use tokio::runtime::Runtime;

pub mod api;
pub mod humanize;

#[derive(Deserialize)]
struct Project {
    id: u32,
    name: String,
}
#[derive(Deserialize)]
struct TogglTask {
    id: u32,
    description: String,
    start: DateTime<Utc>,
    stop: Option<DateTime<Utc>>,
    project_id: Option<u32>,
}
#[derive(Deserialize)]
struct Workspace {
    id: u32,
}

pub struct Task {
    id: u32,
    pub num: u32,
    pub name: String,
    pub project: String,
    pub start: DateTime<Utc>,
}

pub struct Toggl {
    toggl_tasks: Option<Vec<TogglTask>>,
    projects: Option<Vec<Project>>,
    wid: u32,
    api_key: String,
}

impl Toggl {
    pub fn new(api_key: &str) -> Toggl {
        Toggl {
            toggl_tasks: None,
            projects: None,
            wid: 0,
            api_key: api_key.to_string(),
        }
    }

    pub fn current_task(&mut self) -> Result<Option<Task>, ()> {
        if self.toggl_tasks.is_none() {
            self.toggl_tasks = Some(self.get_tasks()?);
        }

        let toggltask = self.toggl_tasks.as_ref().and_then(|tasks| tasks.get(0));
        if let Some(task) = toggltask {
            if task.stop.is_some() {
                return Ok(None);
            }
        }

        if self.projects.is_none() {
            let projects =
                toggltask.and_then(|task| task.project_id.and_then(|_| Some(self.get_projects())));
            if let Some(r) = projects {
                self.projects = Some(r?);
            }
        }

        toggltask.map_or(Ok(None), |task| {
            let mut project = "".to_string();
            if let Some(pid) = task.project_id {
                project = self.projects.as_ref().map_or("".to_string(), |projects| {
                    for proj in projects {
                        if task.project_id.is_some() && proj.id == pid {
                            return proj.name.clone();
                        }
                    }
                    "".to_string()
                })
            }
            let t = Task {
                num: 0,
                id: task.id,
                name: task.description.clone(),
                start: task.start,
                project,
            };
            Ok(Some(t))
        })
    }

    fn get_tasks(&self) -> Result<Vec<TogglTask>, ()> {
        let mut rt = Runtime::new().unwrap();

        let data = rt.block_on(api::fetch_api_future(&self.api_key, "time_entries"))?;
        let resp: Vec<TogglTask> = serde_json::from_str(data.as_str()).map_err(|_| ())?;

        Ok(resp)
    }

    fn get_projects(&self) -> Result<Vec<Project>, ()> {
        let mut rt = Runtime::new().unwrap();

        let data = rt.block_on(api::fetch_api_future(&self.api_key, "projects"))?;
        let resp: Vec<Project> = serde_json::from_str(data.as_str()).map_err(|_| ())?;

        Ok(resp)
    }
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
                id: 0,
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

pub fn stop_task(_api_key: &str) -> Result<(), ()> {
    /*
    let mut rt = Runtime::new().unwrap();
    let task = get_current_task(api_key)?;
    task.map_or_else(
        || Ok(()),
        |task| {
            let payload = format!(
                r#"{{"stop":"{}"}}"#,
                Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z")
            );
            rt.block_on(api::put_api_future(
                &api_key,
                &format!("time_entries/{}", task.id),
                payload,
            ))
        },
    )
     */
    Err(())
}

pub fn create_task(api_key: &str, description: &str, project: &str) -> Result<(), ()> {
    let mut rt = Runtime::new().unwrap();
    let wid = get_workspace_id(api_key)?;
    let pid = fetch_pid_from_project(api_key, project);

    pid.map_or_else(
        || {
            println!("unknown project");
            Err(())
        },
        |pid| {
            let payload = format!(
                r#"{{"start":"{}", "wid": {}, "pid": {}, "description":"{}", "created_with": "togglrust"}}"#,
                Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z"),
                wid,
                pid,
                description
            );
            rt.block_on(api::post_api_future(
                &api_key,
                &format!("time_entries"),
                payload
            ))
        }
    )
}

fn fetch_projects(api_key: &str) -> Result<Vec<Project>, ()> {
    let mut rt = Runtime::new().unwrap();
    rt.block_on(api::fetch_api_future(&api_key, "projects"))
        .map_err(|_| ())
        .and_then(|data| {
            let resp: Result<Vec<Project>, _> = serde_json::from_str(data.as_str());
            resp.map_err(|_| ())
        })
}

fn fetch_project_from_pid(api_key: &str, pid: Option<u32>) -> String {
    pid.map(|pid| {
        fetch_projects(api_key).map(|responses| {
            responses
                .iter()
                .find(|x| x.id == pid)
                .map_or("".to_string(), |p| p.name.clone())
        })
    })
    .and_then(|r| r.ok())
    .map_or("".to_string(), |x| x)
}

fn fetch_pid_from_project(api_key: &str, project: &str) -> Option<u32> {
    fetch_projects(api_key)
        .and_then(|projects| {
            for p in projects {
                if p.name == project {
                    return Ok(p.id);
                }
            }
            Err(())
        })
        .ok()
}

fn get_workspace_id(api_key: &str) -> Result<u32, ()> {
    let mut rt = Runtime::new().unwrap();

    let data = rt.block_on(api::fetch_api_future(&api_key, "workspaces"))?;
    let resp: Vec<Workspace> = serde_json::from_str(data.as_str()).map_err(|_| ())?;
    resp.get(0).map_or_else(|| Err(()), |w| Ok(w.id))
}
