use std::future::{Ready, ready};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio_modbus::prelude::{Request, Response};
use tokio_modbus::server::Service;
use tokio_modbus::server::tcp::{Server, accept_tcp_connection};
use tracing::info;

use crate::sim::SimState;

#[derive(Clone)]
pub struct ModbusService {
    state: Arc<std::sync::RwLock<SimState>>,
}

impl ModbusService {
    pub fn new(state: Arc<std::sync::RwLock<SimState>>) -> Self {
        Self { state }
    }
}

impl Service for ModbusService {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = Ready<Result<Response, io::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let mut state = self.state.write().unwrap();
        let response = match req {
            Request::ReadCoils(addr, cnt) => Response::ReadCoils(state.read_coils(addr, cnt)),
            Request::ReadDiscreteInputs(addr, cnt) => {
                Response::ReadDiscreteInputs(state.read_discrete_inputs(addr, cnt))
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                Response::ReadHoldingRegisters(state.read_holding_registers(addr, cnt))
            }
            Request::ReadInputRegisters(addr, cnt) => {
                Response::ReadInputRegisters(state.read_input_registers(addr, cnt))
            }
            Request::WriteSingleCoil(addr, value) => {
                state.write_single_coil(addr, value);
                Response::WriteSingleCoil(addr, value)
            }
            Request::WriteSingleRegister(addr, value) => {
                state.write_single_register(addr, value);
                Response::WriteSingleRegister(addr, value)
            }
            Request::WriteMultipleCoils(addr, values) => {
                state.write_multiple_coils(addr, &values);
                Response::WriteMultipleCoils(addr, values.len() as u16)
            }
            Request::WriteMultipleRegisters(addr, values) => {
                state.write_multiple_registers(addr, &values);
                Response::WriteMultipleRegisters(addr, values.len() as u16)
            }
            _ => {
                return ready(Err(io::Error::other("unsupported request")));
            }
        };

        ready(Ok(response))
    }
}

pub async fn start_tcp(bind: &str, state: Arc<std::sync::RwLock<SimState>>) -> Result<()> {
    let addr: SocketAddr = bind.parse()?;
    info!(addr = %addr, "modbus tcp listening");
    let listener = TcpListener::bind(addr).await?;
    let server = Server::new(listener);
    let service = ModbusService::new(state);
    let on_connected = move |stream, socket_addr| {
        let service = service.clone();
        async move { accept_tcp_connection(stream, socket_addr, move |_| Ok(Some(service.clone()))) }
    };
    let on_error = |err| {
        tracing::error!(error = %err, "modbus tcp connection error");
    };
    // Start the server in the background so tests can connect to it when needed.
    tokio::spawn(async move { let _ = server.serve(&on_connected, on_error).await; });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RegisterItemConfig;
    use crate::sim::SimState;
    use std::sync::{Arc, RwLock};
    use tokio_modbus::client::tcp as client_tcp;
    use tokio_modbus::prelude::Reader;

    #[tokio::test]
    async fn tcp_end_to_end_read_holding_registers() {
        // reserve a free port
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let bind = format!("127.0.0.1:{}", port);

        // create a state with a known holding register
        let reg_cfg = RegisterItemConfig {
            address: 0,
            initial: 123u16,
            update_ms: None,
            dynamics: None,
        };
        let state = Arc::new(RwLock::new(SimState::new(
            500,
            false,
            vec![],
            vec![],
            vec![reg_cfg],
            vec![],
        )));

        // start the TCP server (spawned inside start_tcp)
        start_tcp(&bind, Arc::clone(&state)).await.unwrap();

        // give the server a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // connect as a client and read the register
        let socket_addr = format!("127.0.0.1:{}", port).parse().unwrap();
        let mut ctx = client_tcp::connect(socket_addr).await.unwrap();
        let regs = ctx.read_holding_registers(0u16, 1u16).await.unwrap();
        assert_eq!(regs[0], 123u16);
    }
}
