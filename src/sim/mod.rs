use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use evalexpr::{ContextWithMutableVariables, HashMapContext, Value};
use rand::Rng;
use tracing::info;

use crate::config::{BoolItemConfig, DynamicsSpec, RegisterItemConfig};

#[derive(Debug, Clone)]
pub struct SimState {
    pub coils: BTreeMap<u16, SimBoolItem>,
    pub discrete_inputs: BTreeMap<u16, SimBoolItem>,
    pub holding_registers: BTreeMap<u16, SimRegisterItem>,
    pub input_registers: BTreeMap<u16, SimRegisterItem>,
    pub global_update_ms: u64,
    pub log_value_updates: bool,
    start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct SimBoolItem {
    pub value: bool,
    pub last_value: bool,
    pub dynamics: Option<DynamicsSpec>,
    pub update_ms: u64,
    pub next_due: Instant,
}

#[derive(Debug, Clone)]
pub struct SimRegisterItem {
    pub value: u16,
    pub last_value: u16,
    pub dynamics: Option<DynamicsSpec>,
    pub update_ms: u64,
    pub next_due: Instant,
}

impl SimState {
    pub fn new(
        global_update_ms: u64,
        log_value_updates: bool,
        coils: Vec<BoolItemConfig>,
        discrete_inputs: Vec<BoolItemConfig>,
        holding_registers: Vec<RegisterItemConfig>,
        input_registers: Vec<RegisterItemConfig>,
    ) -> Self {
        let start_time = Instant::now();
        let coils = coils
            .into_iter()
            .map(|item| {
                let update_ms = item.update_ms.unwrap_or(global_update_ms);
                let next_due = start_time + Duration::from_millis(update_ms);
                (
                    item.address,
                    SimBoolItem {
                        value: item.initial,
                        last_value: item.initial,
                        dynamics: item.dynamics,
                        update_ms,
                        next_due,
                    },
                )
            })
            .collect();

        let discrete_inputs = discrete_inputs
            .into_iter()
            .map(|item| {
                let update_ms = item.update_ms.unwrap_or(global_update_ms);
                let next_due = start_time + Duration::from_millis(update_ms);
                (
                    item.address,
                    SimBoolItem {
                        value: item.initial,
                        last_value: item.initial,
                        dynamics: item.dynamics,
                        update_ms,
                        next_due,
                    },
                )
            })
            .collect();

        let holding_registers = holding_registers
            .into_iter()
            .map(|item| {
                let update_ms = item.update_ms.unwrap_or(global_update_ms);
                let next_due = start_time + Duration::from_millis(update_ms);
                (
                    item.address,
                    SimRegisterItem {
                        value: item.initial,
                        last_value: item.initial,
                        dynamics: item.dynamics,
                        update_ms,
                        next_due,
                    },
                )
            })
            .collect();

        let input_registers = input_registers
            .into_iter()
            .map(|item| {
                let update_ms = item.update_ms.unwrap_or(global_update_ms);
                let next_due = start_time + Duration::from_millis(update_ms);
                (
                    item.address,
                    SimRegisterItem {
                        value: item.initial,
                        last_value: item.initial,
                        dynamics: item.dynamics,
                        update_ms,
                        next_due,
                    },
                )
            })
            .collect();

        Self {
            coils,
            discrete_inputs,
            holding_registers,
            input_registers,
            global_update_ms,
            log_value_updates,
            start_time,
        }
    }

    pub fn min_tick_ms(&self) -> u64 {
        let mut min_ms = self.global_update_ms.max(10);
        for item in self.coils.values().chain(self.discrete_inputs.values()) {
            min_ms = min_ms.min(item.update_ms.max(10));
        }
        for item in self
            .holding_registers
            .values()
            .chain(self.input_registers.values())
        {
            min_ms = min_ms.min(item.update_ms.max(10));
        }
        min_ms
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.start_time).as_secs_f64();

        for (address, item) in self.coils.iter_mut() {
            if now < item.next_due {
                continue;
            }
            let value = eval_bool(item.value, &item.dynamics, elapsed);
            let changed = value != item.value;
            item.last_value = item.value;
            item.value = value;
            item.next_due = now + Duration::from_millis(item.update_ms);
            if self.log_value_updates && changed {
                info!(address = *address, value = item.value, "coil updated");
            }
        }

        for (address, item) in self.discrete_inputs.iter_mut() {
            if now < item.next_due {
                continue;
            }
            let value = eval_bool(item.value, &item.dynamics, elapsed);
            let changed = value != item.value;
            item.last_value = item.value;
            item.value = value;
            item.next_due = now + Duration::from_millis(item.update_ms);
            if self.log_value_updates && changed {
                info!(
                    address = *address,
                    value = item.value,
                    "discrete input updated"
                );
            }
        }

        for (address, item) in self.holding_registers.iter_mut() {
            if now < item.next_due {
                continue;
            }
            let value = eval_register(item.value, &item.dynamics, elapsed);
            let changed = value != item.value;
            item.last_value = item.value;
            item.value = value;
            item.next_due = now + Duration::from_millis(item.update_ms);
            if self.log_value_updates && changed {
                info!(
                    address = *address,
                    value = item.value,
                    "holding register updated"
                );
            }
        }

        for (address, item) in self.input_registers.iter_mut() {
            if now < item.next_due {
                continue;
            }
            let value = eval_register(item.value, &item.dynamics, elapsed);
            let changed = value != item.value;
            item.last_value = item.value;
            item.value = value;
            item.next_due = now + Duration::from_millis(item.update_ms);
            if self.log_value_updates && changed {
                info!(
                    address = *address,
                    value = item.value,
                    "input register updated"
                );
            }
        }
    }

    pub fn read_coils(&self, address: u16, count: u16) -> Vec<bool> {
        read_range_bool(&self.coils, address, count)
    }

    pub fn read_discrete_inputs(&self, address: u16, count: u16) -> Vec<bool> {
        read_range_bool(&self.discrete_inputs, address, count)
    }

    pub fn read_holding_registers(&self, address: u16, count: u16) -> Vec<u16> {
        read_range_register(&self.holding_registers, address, count)
    }

    pub fn read_input_registers(&self, address: u16, count: u16) -> Vec<u16> {
        read_range_register(&self.input_registers, address, count)
    }

    pub fn write_single_coil(&mut self, address: u16, value: bool) {
        if let Some(item) = self.coils.get_mut(&address) {
            item.value = value;
        } else {
            self.coils.insert(
                address,
                SimBoolItem {
                    value,
                    last_value: value,
                    dynamics: None,
                    update_ms: self.global_update_ms,
                    next_due: Instant::now() + Duration::from_millis(self.global_update_ms),
                },
            );
        }
    }

    pub fn write_multiple_coils(&mut self, address: u16, values: &[bool]) {
        for (offset, value) in values.iter().copied().enumerate() {
            let addr = address.saturating_add(offset as u16);
            self.write_single_coil(addr, value);
        }
    }

    pub fn write_single_register(&mut self, address: u16, value: u16) {
        if let Some(item) = self.holding_registers.get_mut(&address) {
            item.value = value;
        } else {
            self.holding_registers.insert(
                address,
                SimRegisterItem {
                    value,
                    last_value: value,
                    dynamics: None,
                    update_ms: self.global_update_ms,
                    next_due: Instant::now() + Duration::from_millis(self.global_update_ms),
                },
            );
        }
    }

    pub fn write_multiple_registers(&mut self, address: u16, values: &[u16]) {
        for (offset, value) in values.iter().copied().enumerate() {
            let addr = address.saturating_add(offset as u16);
            self.write_single_register(addr, value);
        }
    }
}

pub async fn spawn_simulator(state: std::sync::Arc<std::sync::RwLock<SimState>>) {
    let tick_ms = state.read().unwrap().min_tick_ms();
    let mut interval = tokio::time::interval(Duration::from_millis(tick_ms));

    loop {
        interval.tick().await;
        let mut guard = state.write().unwrap();
        guard.tick();
    }
}

fn read_range_bool(map: &BTreeMap<u16, SimBoolItem>, address: u16, count: u16) -> Vec<bool> {
    (0..count)
        .map(|offset| {
            map.get(&(address + offset))
                .map(|item| item.value)
                .unwrap_or(false)
        })
        .collect()
}

fn read_range_register(map: &BTreeMap<u16, SimRegisterItem>, address: u16, count: u16) -> Vec<u16> {
    (0..count)
        .map(|offset| {
            map.get(&(address + offset))
                .map(|item| item.value)
                .unwrap_or(0)
        })
        .collect()
}

fn eval_bool(current: bool, dynamics: &Option<DynamicsSpec>, elapsed: f64) -> bool {
    let numeric = eval_numeric(if current { 1.0 } else { 0.0 }, dynamics, elapsed);
    numeric > 0.5
}

fn eval_register(current: u16, dynamics: &Option<DynamicsSpec>, elapsed: f64) -> u16 {
    let numeric = eval_numeric(current as f64, dynamics, elapsed);
    numeric.round().clamp(0.0, u16::MAX as f64) as u16
}

fn eval_numeric(current: f64, dynamics: &Option<DynamicsSpec>, elapsed: f64) -> f64 {
    let mut rng = rand::thread_rng();
    match dynamics {
        None | Some(DynamicsSpec::Static) => current,
        Some(DynamicsSpec::Clamp { min, max }) => current.clamp(*min, *max),
        Some(DynamicsSpec::Sine {
            amplitude,
            offset,
            period_ms,
        }) => {
            let period = (*period_ms as f64) / 1000.0;
            if period <= 0.0 {
                return *offset;
            }
            offset + amplitude * (elapsed * std::f64::consts::TAU / period).sin()
        }
        Some(DynamicsSpec::Ramp {
            min,
            max,
            period_ms,
        }) => {
            let period = (*period_ms as f64) / 1000.0;
            if period <= 0.0 {
                return *min;
            }
            let phase = (elapsed % period) / period;
            min + (max - min) * phase
        }
        Some(DynamicsSpec::Step {
            low,
            high,
            period_ms,
        }) => {
            let period = (*period_ms as f64) / 1000.0;
            if period <= 0.0 {
                return *low;
            }
            let phase = (elapsed % period) / period;
            if phase < 0.5 { *low } else { *high }
        }
        Some(DynamicsSpec::RandomWalk { min, max, step }) => {
            let delta = rng.gen_range(-step..=*step);
            (current + delta).clamp(*min, *max)
        }
        Some(DynamicsSpec::Noise { min, max }) => rng.gen_range(*min..=*max),
        Some(DynamicsSpec::Script { expr, min, max }) => {
            let value = eval_script(expr, elapsed).unwrap_or(current);
            clamp_optional(value, *min, *max)
        }
    }
}

fn clamp_optional(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    match (min, max) {
        (Some(min), Some(max)) => value.clamp(min, max),
        (Some(min), None) => value.max(min),
        (None, Some(max)) => value.min(max),
        _ => value,
    }
}

fn eval_script(expr: &str, elapsed: f64) -> Option<f64> {
    let expr = expr.trim();
    let mut context = HashMapContext::new();
    context
        .set_value("t".to_string(), Value::Float(elapsed))
        .ok()?;
    let value = evalexpr::eval_with_context(expr, &context).ok()?;
    match value {
        Value::Int(value) => Some(value as f64),
        Value::Float(value) => Some(value),
        _ => None,
    }
}
