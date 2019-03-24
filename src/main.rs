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

fn list_tasks(toggl: &mut Toggl) -> Result<(), ()> {
    let tasks = toggl.list_tasks()?;
    for t in tasks {
        println!("{} {} ({})", t.num, t.name, t.project);
    }
    Ok(())
}

fn switch_task(toggl: &mut Toggl, task_num: &str) -> Result<(), ()> {
    let idx: Result<usize, _> = task_num.parse();
    if let Ok(n) = idx {
        toggl.switch_task(n)
    } else {
        println!("task must be a number");
        Err(())
    }
}

fn stop_timer(toggl: &mut Toggl) -> Result<(), ()> {
    toggl.stop_task()
}

fn new_task(toggl: &mut Toggl, desc: &str, proj: &str) -> Result<(), ()> {
    toggl.create_task(desc, proj)
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
                    "list" => list_tasks(&mut toggl),
                    "stop" => stop_timer(&mut toggl),
                    _ => help(),
                }
            }
            3 => {
                let cmd = &args[1];
                let num = &args[2];
                match &cmd[..] {
                    "switch" => switch_task(&mut toggl, &num),
                    _ => help(),
                }
            }
            4 => {
                let cmd = &args[1];
                let desc = &args[2];
                let proj = &args[3];
                match &cmd[..] {
                    "new" => new_task(&mut toggl, &desc, &proj),
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
