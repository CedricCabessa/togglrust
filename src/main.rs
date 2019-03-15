use std::env;
use togglrust;

fn main() {
    if let Ok(api_key) = env::var("TOGGL_KEY") {
        let ret = togglrust::get_current_task(&api_key)
            .and_then(|res| {
                match res {
                    Some(task) => println!("{} {}", task.description, task.start),
                    None => println!("no running task"),
                }
                Ok(())
            })
            .or_else(|_| {
                println!("something wrong happened");
                Err(1)
            });
        if let Err(num) = ret {
            ::std::process::exit(num);
        }
    } else {
        eprintln!("need TOGGL_KEY env");
        ::std::process::exit(1);
    }
}
