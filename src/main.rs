use std::env;
use togglrust;

use chrono;

fn help() -> Result<(), ()> {
    println!(
        "usage:
togglrust
    Show the current task
togglrust list
    List recent tasks
togglrust switch <n>
    Switch to another task
togglrust new \"my description\" project
    Create a new task
togglrust stop
    Stop timer"
    );

    Ok(())
}

fn print_current_task(api_key: &str) -> Result<(), ()> {
    togglrust::get_current_task(&api_key)
        .map(|res| match res {
            Some(task) => {
                let now = chrono::Utc::now();
                let duration = now - task.start;
                println!(
                    "{}: {} ({})",
                    task.name,
                    togglrust::humanize::duration(&duration),
                    task.project,
                );
            }
            None => println!("no running task"),
        })
        .map_err(|_| {
            println!("something wrong happened");
        })
}

fn list_tasks(api_key: &str) -> Result<(), ()> {
    togglrust::get_task_list(&api_key)
        .map(|v| {
            for t in v {
                println!("{} {} ({})", t.num, t.name, t.project);
            }
        })
        .map_err(|_| {
            println!("something wrong happened");
        })
}

fn switch_task(api_key: &str, task_num: &str) -> Result<(), ()> {
    let idx: Result<usize, _> = task_num.parse();
    if let Ok(n) = idx {
        togglrust::get_task_list(&api_key)
            .map_err(|_| {
                println!("something wrong happened");
            })
            .and_then(|v| {
                let task = v.get(n).ok_or_else(|| println!("wrong index"));
                task.map(|t| {
                    println!("switching to task: {}", t.name);
                })
            })
    } else {
        println!("task must be a number");
        Err(())
    }
}

fn stop_timer(_api_key: &str) -> Result<(), ()> {
    println!("stop task");
    Ok(())
}

fn new_task(_api_key: &str, _desc: &str, _proj: &str) -> Result<(), ()> {
    Ok(())
}

fn main() {
    if let Ok(api_key) = env::var("TOGGL_KEY") {
        let args: Vec<String> = env::args().collect();
        let ret = match args.len() {
            1 => print_current_task(&api_key),
            2 => {
                let cmd = &args[1];
                match &cmd[..] {
                    "list" => list_tasks(&api_key),
                    "stop" => stop_timer(&api_key),
                    _ => help(),
                }
            }
            3 => {
                let cmd = &args[1];
                let num = &args[2];
                match &cmd[..] {
                    "switch" => switch_task(&api_key, &num),
                    _ => help(),
                }
            }
            4 => {
                let cmd = &args[1];
                let desc = &args[2];
                let proj = &args[3];
                match &cmd[..] {
                    "new" => new_task(&api_key, &desc, &proj),
                    _ => help(),
                }
            }
            _ => help(),
        };
        if ret.is_err() {
            ::std::process::exit(1);
        }
    } else {
        eprintln!("need TOGGL_KEY env");
        ::std::process::exit(1);
    }
}
