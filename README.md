# crond

A simple scheduler job runner

## Example config

```toml
[app]
log_path = "/tmp"
tg_token = "xxx"

[jobs.test]
command = "ls"
working_dir = "/tmp"
schedule = "* * * * * *"
envs = {"FOO"="BAR","FOO1"="BAR1"}

```

## Usage

```sh
vim ~/.config/crond/config.toml
crond
```
