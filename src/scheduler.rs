use std::collections::HashMap;

use eyre::Result;
use serde_derive::Deserialize;
use tokio_cron_scheduler::{Job as SchedJob, JobScheduler};

use crate::{config::Config, log::*};

const DEFAULT_WORKING_DIR: &str = "/tmp";
const DEFAULT_LOG_DIR: &str = "/tmp";

#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub command: String,
    pub working_dir: Option<String>,
    pub schedule: String,
    pub envs: HashMap<String, String>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CronJob {
    pub log_path: Option<String>,
    pub tg_token: Option<String>,
    pub jobs: HashMap<String, Job>,
    pub global_envs: Option<HashMap<String, String>>,
}

impl CronJob {
    pub fn from_config(config: &Config) -> CronJob {
        CronJob {
            log_path: config.app.log_path.clone(),
            tg_token: config.app.tg_token.clone(),
            jobs: config.jobs.clone(),
            global_envs: config.app.envs.clone(),
        }
    }
    pub async fn run(&self) -> Result<()> {
        let mut sched = JobScheduler::new().await?;
        for (name, job) in self.jobs.iter() {
            let name = name.clone();
            let job = job.clone();
            let log_path = self.log_path.clone().unwrap_or(DEFAULT_LOG_DIR.to_string());
            let global_envs = self.global_envs.clone();

            info!("Loaded job {:?}", name);
            sched
                .add(SchedJob::new_async(
                    job.schedule.clone(),
                    move |_uuid, _l| {
                        let name = name.clone();
                        let job = job.clone();
                        let log_path = log_path.clone();
                        let global_envs = global_envs.clone();

                        Box::pin(async move {
                            info!("Running job {:?}", name);
                            let stdout_file_path = format!("{log_path}/{name}_stdout.log");
                            let stderr_file_path = format!("{log_path}/{name}_stderr.log");

                            let stdout_file = match std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&stdout_file_path)
                            {
                                Ok(f) => f,
                                Err(e) => {
                                    error!("Failed to open stdout file for job {:?}: {}", name, e);
                                    return;
                                }
                            };

                            let stderr_file = match std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&stderr_file_path)
                            {
                                Ok(f) => f,
                                Err(e) => {
                                    error!("Failed to open stderr file for job {:?}: {}", name, e);
                                    return;
                                }
                            };

                            let working_dir = job
                                .working_dir
                                .clone()
                                .unwrap_or(DEFAULT_WORKING_DIR.to_string());

                            let mut command = tokio::process::Command::new("sh");
                            command.arg("-c").arg(&job.command);

                            if let Some(envs) = global_envs {
                                command.envs(envs);
                            }
                            command.envs(&job.envs);
                            command.current_dir(&working_dir);
                            command.stdout(std::process::Stdio::from(stdout_file));
                            command.stderr(std::process::Stdio::from(stderr_file));

                            match command.spawn() {
                                Ok(mut child) => match child.wait().await {
                                    Ok(status) => {
                                        if status.success() {
                                            info!("Job {:?} finished successfully", name);
                                        } else {
                                            error!("Job {:?} failed with status: {}", name, status);
                                        }
                                    }
                                    Err(e) => error!("Job {:?} failed to wait: {}", name, e),
                                },
                                Err(e) => error!("Job {:?} failed to start: {}", name, e),
                            }
                        })
                    },
                )?)
                .await?;
        }
        sched.shutdown_on_ctrl_c();
        sched.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("Scheduler shutdown");
            })
        }));
        sched.start().await?;
        Ok(())
    }
}
