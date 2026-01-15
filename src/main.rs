use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{error, info};

mod config;
mod sim;
mod transport;

use config::Config;
use sim::{spawn_simulator, SimState};
use transport::rtu::start_rtu;
use transport::tcp::start_tcp;

#[derive(Parser, Debug)]
#[command(name = "modsim", version, about = "Modbus simulator")]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let config = load_config(&args.config)?;

    let log_value_updates = config
        .logging
        .as_ref()
        .map(|logging| logging.log_value_updates)
        .unwrap_or(false);
    let global_update_ms = config
        .global
        .as_ref()
        .map(|global| global.update_ms)
        .unwrap_or(500);

    let unit_id = config.device.unit_id;
    let state = Arc::new(RwLock::new(SimState::new(
        global_update_ms,
        log_value_updates,
        config.device.coils,
        config.device.discrete_inputs,
        config.device.holding_registers,
        config.device.input_registers,
    )));

    let simulator_state = Arc::clone(&state);
    tokio::spawn(async move { spawn_simulator(simulator_state).await });

    let mut tasks = Vec::new();
    if let Some(tcp) = config.tcp {
        let state = Arc::clone(&state);
        tasks.push(tokio::spawn(async move { start_tcp(&tcp.bind, state).await }));
    }

    if let Some(rtu) = config.rtu {
        let state = Arc::clone(&state);
        tasks.push(tokio::spawn(async move { start_rtu(&rtu, state).await }));
    }

    if tasks.is_empty() {
        error!("no transports configured: enable tcp or rtu");
        return Ok(());
    }

    info!(unit_id, "modsim started");
    tokio::signal::ctrl_c().await?;
    info!("shutdown requested");

    for task in tasks {
        if let Err(err) = task.await? {
            error!(error = %err, "transport task failed");
        }
    }

    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path).with_context(|| format!("failed to read {path}"))?;
    let config: Config = toml::from_str(&content).context("failed to parse TOML")?;
    Ok(config)
}
