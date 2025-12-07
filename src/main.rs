mod config;
mod log;
mod scheduler;
#[cfg(test)]
mod tests;

use clap::Parser;
use eyre::Result;
use log::*;
use shadow_rs::shadow;

use crate::config::{Cli, Config};
shadow!(build);

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.version {
        println!("{}", build::VERSION);
        return Ok(());
    }
    init_log("info")?;
    let config = Config::new(cli.config)?;
    info!("{:?}", config);

    let cron_job = scheduler::CronJob::from_config(&config);
    cron_job.run().await?;
    wait_for_signal().await?;
    Ok(())
}

pub async fn wait_for_signal() -> eyre::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = sigint.recv() => {}
        _ = sigterm.recv() => {}
    }

    Ok(())
}
