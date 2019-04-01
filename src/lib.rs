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
    stop: Option<DateTime<Utc>>,
}

pub struct Toggl {
    toggl_tasks: Option<Vec<TogglTask>>,
    projects: Option<Vec<Project>>,
    api_key: String,
}

impl Toggl {
    pub fn new(api_key: &str) -> Toggl {
        Toggl {
            toggl_tasks: None,
            projects: None,
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
            let t = Task {
                num: 0,
                id: task.id,
                name: task.description.clone(),
                start: task.start,
                stop: task.stop,
                project: self.project_name(task),
            };
            Ok(Some(t))
        })
    }

    pub fn list_tasks(&mut self) -> Result<Vec<Task>, ()> {
        if self.toggl_tasks.is_none() {
            self.toggl_tasks = Some(self.get_tasks()?);
        }
        if self.projects.is_none() {
            // we will need projects at some point, just fetch them
            self.projects = Some(self.get_projects()?);
        }
        self.toggl_tasks.as_ref().map_or(Ok(Vec::new()), |tasks| {
            let mut res: Vec<Task> = Vec::new();
            let mut num = 0;
            for task in tasks {
                let project = self.project_name(&task);
                if !res
                    .iter()
                    .any(|t| t.name == task.description && t.project == project)
                {
                    let t = Task {
                        num,
                        id: task.id,
                        name: task.description.clone(),
                        start: task.start,
                        stop: task.stop,
                        project,
                    };
                    num += 1;
                    res.push(t);
                    if num > 10 {
                        break;
                    }
                }
            }
            Ok(res)
        })
    }

    pub fn stop_task(&mut self) -> Result<(), ()> {
        if self.toggl_tasks.is_none() {
            self.toggl_tasks = Some(self.get_tasks()?);
        }

        let toggltask = self.toggl_tasks.as_ref().and_then(|tasks| tasks.get(0));
        if let Some(task) = toggltask {
            if task.stop.is_none() {
                let duration = Utc::now().timestamp() - task.start.timestamp();
                self.stop_task_by_id(task.id, duration)?;
            }
        }
        Ok(())
    }

    pub fn create_task(&mut self, description: &str, project: &str) -> Result<(), ()> {
        let mut rt = Runtime::new().unwrap();

        if self.projects.is_none() {
            self.projects = Some(self.get_projects()?);
        }

        let wid = self.get_workspace_id()?;

        let pid = self
            .projects
            .as_ref()
            .and_then(|projects| projects.iter().find(|x| x.name == project))
            .map(|project| project.id.to_string())
            .unwrap_or("null".to_string());

        let now = Utc::now();
        let payload = format!(
            r#"{{"start":"{}", "wid": {}, "pid": {}, "description":"{}", "duration": {}, "created_with": "togglrust"}}"#,
            now.format("%Y-%m-%dT%H:%M:%S.000Z"), wid, pid, description, -1 * now.timestamp()
        );
        rt.block_on(api::post_api_future(
            &self.api_key,
            &"time_entries".to_string(),
            payload
        ))
    }

    pub fn switch_task(&mut self, idx: usize) -> Result<(), ()> {
        let tasks = self.list_tasks()?;
        let current = tasks.get(0);
        let task = tasks.get(idx);
        if let Some(task) = current {
            if task.stop.is_none() {
                let duration = Utc::now().timestamp() - task.start.timestamp();
                self.stop_task_by_id(task.id, duration)?;
            }
        }
        if let Some(task) = task {
            self.create_task(&task.name, &task.project)?;
            println!("{} ({})", task.name, task.project);
        } else {
            println!("wrong index");
            return Err(());
        }
        Ok(())
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

    fn project_name(&self, task: &TogglTask) -> String {
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
        project
    }

    fn get_workspace_id(&self) -> Result<u32, ()> {
        let mut rt = Runtime::new().unwrap();

        let data = rt.block_on(api::fetch_api_future(&self.api_key, "workspaces"))?;
        let resp: Vec<Workspace> = serde_json::from_str(data.as_str()).map_err(|_| ())?;
        resp.get(0).map_or_else(|| Err(()), |w| Ok(w.id))
    }

    fn stop_task_by_id(&self, id: u32, duration: i64) -> Result<(), ()> {
        let mut rt = Runtime::new().unwrap();

        let payload = format!(
            r#"{{"stop":"{}", "duration":{}}}"#,
            Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z"),
            duration
        );
        rt.block_on(api::put_api_future(
            &self.api_key,
            &format!("time_entries/{}", id),
            payload,
        ))
    }
}
