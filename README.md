# modsim

Rust-based Modbus simulator supporting Modbus TCP and RTU (OS serial). Configuration is TOML and defines all coils/registers and their dynamics.

## Quick Start

1. Copy the sample config:

```
cp config.example.toml config.toml
```

2. Run the simulator:

```
cargo run
```

## Configuration (TOML)

```toml
[logging]
log_value_updates = false

[global]
update_ms = 500

[tcp]
bind = "0.0.0.0:5020"

[rtu]
# device = "/dev/tty.usbserial-1420" # required for serial mode
baud_rate = 9600
parity = "none"     # none|even|odd
stop_bits = 1

[device]
unit_id = 1

[[device.coils]]
address = 0
initial = false
update_ms = 1000
[device.coils.dynamics]
kind = "step"
low = 0.0
high = 1.0
period_ms = 2000

[[device.discrete_inputs]]
address = 0
initial = true
[device.discrete_inputs.dynamics]
kind = "noise"
min = 0.0
max = 1.0

[[device.holding_registers]]
address = 0
initial = 100
update_ms = 250
[device.holding_registers.dynamics]
kind = "sine"
amplitude = 50.0
offset = 100.0
period_ms = 4000

[[device.input_registers]]
address = 0
initial = 10
[device.input_registers.dynamics]
kind = "script"
expr = "100 + 20*sin(t)"
min = 0
max = 200
```

## Dynamics

- `static`
- `clamp`
- `sine`
- `ramp`
- `step`
- `random-walk`
- `noise`
- `script` (math + time only; use `t` for seconds)



## Notes

- One device per server configuration.
- Per-item `update_ms` overrides the global default.
- Value update logging is controlled by `logging.log_value_updates`.
