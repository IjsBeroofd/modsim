use std::sync::Arc;

use anyhow::{Context, Result};
use tokio_modbus::server::rtu::Server;
use tokio_serial::{DataBits, Parity, SerialPort, SerialPortBuilderExt, StopBits};
use tracing::info;

use crate::config::{Parity as ConfigParity, RtuConfig, RtuMode};
use crate::sim::SimState;
use crate::transport::tcp::ModbusService;

pub async fn start_rtu(
    config: &RtuConfig,
    state: Arc<std::sync::RwLock<SimState>>,
) -> Result<()> {
    let service = ModbusService::new(state);
    match config.mode {
        RtuMode::Serial => {
            let device = config
                .device
                .as_ref()
                .context("rtu.device is required for serial mode")?;
            info!(device = %device, "modbus rtu serial listening");
            let serial = build_serial(device, config)?;
            Server::new(serial).serve_forever(service).await?;
        }
        RtuMode::PseudoPty => {
            let (master, slave_path) = create_pty_pair()?;
            info!(slave = %slave_path, "modbus rtu pty listening");
            Server::new(master).serve_forever(service).await?;
        }
    }
    Ok(())
}

fn build_serial(device: &str, config: &RtuConfig) -> Result<tokio_serial::SerialStream> {
    let mut builder = tokio_serial::new(device, config.baud_rate);
    builder = builder.data_bits(match config.data_bits {
        5 => DataBits::Five,
        6 => DataBits::Six,
        7 => DataBits::Seven,
        _ => DataBits::Eight,
    });
    builder = builder.parity(match config.parity {
        ConfigParity::None => Parity::None,
        ConfigParity::Even => Parity::Even,
        ConfigParity::Odd => Parity::Odd,
    });
    builder = builder.stop_bits(match config.stop_bits {
        2 => StopBits::Two,
        _ => StopBits::One,
    });
    builder
        .open_native_async()
        .context("failed to open serial device")
}

fn create_pty_pair() -> Result<(tokio_serial::SerialStream, String)> {
    let (master, slave) = tokio_serial::SerialStream::pair()
        .context("failed to create pseudo-pty pair")?;
    let slave_name = slave
        .name()
        .unwrap_or_else(|| "unknown".to_string());
    Ok((master, slave_name))
}
