use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub logging: Option<LoggingConfig>,
    pub global: Option<GlobalConfig>,
    pub tcp: Option<TcpConfig>,
    pub rtu: Option<RtuConfig>,
    pub device: DeviceConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default)]
    pub log_value_updates: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GlobalConfig {
    #[serde(default = "default_update_ms")]
    pub update_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TcpConfig {
    #[serde(default = "default_tcp_bind")]
    pub bind: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RtuConfig {
    pub mode: RtuMode,
    pub device: Option<String>,
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
    #[serde(default = "default_data_bits")]
    pub data_bits: u8,
    #[serde(default = "default_parity")]
    pub parity: Parity,
    #[serde(default = "default_stop_bits")]
    pub stop_bits: u8,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum RtuMode {
    Serial,
    PseudoPty,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeviceConfig {
    #[serde(default = "default_unit_id")]
    pub unit_id: u8,
    #[serde(default)]
    pub coils: Vec<BoolItemConfig>,
    #[serde(default)]
    pub discrete_inputs: Vec<BoolItemConfig>,
    #[serde(default)]
    pub holding_registers: Vec<RegisterItemConfig>,
    #[serde(default)]
    pub input_registers: Vec<RegisterItemConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BoolItemConfig {
    pub address: u16,
    #[serde(default)]
    pub initial: bool,
    pub update_ms: Option<u64>,
    pub dynamics: Option<DynamicsSpec>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RegisterItemConfig {
    pub address: u16,
    #[serde(default)]
    pub initial: u16,
    pub update_ms: Option<u64>,
    pub dynamics: Option<DynamicsSpec>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum DynamicsSpec {
    Static,
    Clamp { min: f64, max: f64 },
    Sine {
        amplitude: f64,
        offset: f64,
        period_ms: u64,
    },
    Ramp {
        min: f64,
        max: f64,
        period_ms: u64,
    },
    Step {
        low: f64,
        high: f64,
        period_ms: u64,
    },
    RandomWalk {
        min: f64,
        max: f64,
        step: f64,
    },
    Noise {
        min: f64,
        max: f64,
    },
    Script {
        expr: String,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
    },
}

fn default_update_ms() -> u64 {
    500
}

fn default_tcp_bind() -> String {
    "0.0.0.0:5020".to_string()
}

fn default_baud_rate() -> u32 {
    9600
}

fn default_data_bits() -> u8 {
    8
}

fn default_parity() -> Parity {
    Parity::None
}

fn default_stop_bits() -> u8 {
    1
}

fn default_unit_id() -> u8 {
    1
}
