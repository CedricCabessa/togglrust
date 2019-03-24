use std::env;
use togglrust::Toggl;

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

fn print_current_task(toggl: &mut Toggl) -> Result<(), ()> {
    let res = toggl.current_task()?;
    match res {
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
    };
    Ok(())
}

fn list_tasks(api_key: &str) -> Result<(), ()> {
    let tasks = togglrust::get_task_list(&api_key)?;
    for t in tasks {
        println!("{} {} ({})", t.num, t.name, t.project);
    }
    Ok(())
}

fn switch_task(api_key: &str, task_num: &str) -> Result<(), ()> {
    let idx: Result<usize, _> = task_num.parse();
    if let Ok(n) = idx {
        let tasks = togglrust::get_task_list(&api_key)?;
        let task = tasks.get(n).ok_or_else(|| println!("wrong index"))?;
        println!("switching to task: {}", task.name);
        Ok(())
    } else {
        println!("task must be a number");
        Err(())
    }
}

fn stop_timer(api_key: &str) -> Result<(), ()> {
    togglrust::stop_task(api_key)
}

fn new_task(api_key: &str, desc: &str, proj: &str) -> Result<(), ()> {
    togglrust::create_task(api_key, desc, proj)
}

fn main() {
    if let Ok(api_key) = env::var("TOGGL_KEY") {
        let args: Vec<String> = env::args().collect();
        let mut toggl = Toggl::new(&api_key);
        let ret = match args.len() {
            1 => print_current_task(&mut toggl),
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
            eprintln!("something wrong happened");
            ::std::process::exit(1);
        }
    } else {
        eprintln!("need TOGGL_KEY env");
        ::std::process::exit(1);
    }
}
