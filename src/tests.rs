use std::{collections::HashMap, fs::File, io::Read, time::Duration};

use tempfile::tempdir;

use crate::{
    config::Config,
    scheduler::{CronJob, Job},
};

#[tokio::test]
async fn test_cron_job_execution() {
    let dir = tempdir().unwrap();
    let log_path = dir.path().to_str().unwrap().to_string();

    let mut jobs = HashMap::new();
    let mut envs = HashMap::new();
    envs.insert("TEST_VAR".to_string(), "hello".to_string());

    jobs.insert(
        "test_job".to_string(),
        Job {
            command: "echo $TEST_VAR".to_string(),
            working_dir: Some(log_path.clone()),
            schedule: "1/1 * * * * *".to_string(), // Every second
            envs,
        },
    );

    let cron_job = CronJob {
        log_path: Some(log_path.clone()),
        tg_token: None,
        jobs,
        global_envs: None,
    };

    // Run scheduler in a separate task
    // We need to wrap it in an Arc or clone it because run takes &self
    let cron_job = std::sync::Arc::new(cron_job);
    let cron_job_clone = cron_job.clone();

    let handle = tokio::spawn(async move {
        cron_job_clone.run().await.unwrap();
    });

    // Wait for job to run (at least 2 seconds to be safe)
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Check output file
    let stdout_path = dir.path().join("test_job_stdout.log");

    // Retry a few times if file is not created yet
    let mut attempts = 0;
    while !stdout_path.exists() && attempts < 5 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        attempts += 1;
    }

    let mut file = File::open(&stdout_path).expect("stdout file not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains("hello"));

    // Abort the scheduler
    handle.abort();
}

#[test]
fn test_config_loading() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[app]
log_path = "/tmp"
tg_token = "xxx"

[jobs.test]
command = "ls"
working_dir = "/tmp"
schedule = "* * * * * *"
envs = {"FOO"="BAR"}
"#,
    )
    .unwrap();

    // Test loading from file
    let config = Config::new(Some(config_path.clone())).unwrap();
    assert_eq!(config.app.tg_token, Some("xxx".to_string()));
    assert!(config.jobs.contains_key("test"));

    // Test env var override
    unsafe {
        std::env::set_var("CROND__APP__TG_TOKEN", "env_token");
    }
    let config = Config::new(Some(config_path)).unwrap();
    assert_eq!(config.app.tg_token, Some("env_token".to_string()));

    unsafe {
        std::env::remove_var("CROND__APP__TG_TOKEN");
    }
}

#[test]
fn test_global_envs() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[app]
log_path = "/tmp"
tg_token = "xxx"

[app.envs]
GLOBAL_VAR = "global"

[jobs.test]
command = "ls"
working_dir = "/tmp"
schedule = "* * * * * *"
envs = {"FOO"="BAR"}
"#,
    )
    .unwrap();

    let config = Config::new(Some(config_path)).unwrap();
    assert!(config.app.envs.is_some());
    let envs = config.app.envs.unwrap();
    assert_eq!(envs.get("GLOBAL_VAR"), Some(&"global".to_string()));
}
