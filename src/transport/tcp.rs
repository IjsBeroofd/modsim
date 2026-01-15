use std::future::{ready, Ready};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio_modbus::prelude::{Request, Response};
use tokio_modbus::server::tcp::{accept_tcp_connection, Server};
use tokio_modbus::server::Service;
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
                return ready(Err(io::Error::new(
                    io::ErrorKind::Other,
                    "unsupported request",
                )))
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
        async move {
            accept_tcp_connection(stream, socket_addr, move |_| Ok(Some(service.clone())))
        }
    };
    let on_error = |err| {
        tracing::error!(error = %err, "modbus tcp connection error");
    };
    server.serve(&on_connected, on_error).await?;
    Ok(())
}
