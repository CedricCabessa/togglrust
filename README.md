# togglrust

Simple rust command line for [toggl](https://toggl.com/)

## Installation and setup

You need to create an account and token on [toggl](https://toggl.com/)

The token must be exported as an environment variable.

```
export TOGGL_KEY="xxxxx"
```

## Usage

Show current task

```
$ togglrust
my awesome task (work): 2 hour(s) and 34 minute(s)
```

List recent tasks (the 5 most recent)

```
$ togglrust list
1. fix bug #42 (work)
2. meeting (work)
3. break (perso)
```

Switch task

```
$ togglrust switch 2
You are now doing:
 meeting (work)
```

Create a new task (and make it active). The project is mandatory and must
already exists.

```
$ togglrust new "prepare the release" "work"
Task "prepare the release" in project "work" created
```

Stop the timer

```
$ togglrust stop
Bye
```
