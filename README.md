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

## Useful commands (development & CI) ✅

- Run the simulator (uses `config.toml` by default):

```bash
cargo run
# or run with a specific config file
cargo run -- --config config.example.toml
```

- Run tests:

```bash
cargo test -- --nocapture
```

- Formatting & lint checks (same as CI):

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
# optional: security scan
cargo audit || true
```

- Run the full CI locally (single command):

```bash
cargo fmt -- --check && \
  cargo clippy --all-targets --all-features -- -D warnings && \
  cargo test --workspace --verbose
```

- Build release binary:

```bash
cargo build --release
```

- Quick troubleshooting:
  - If RTU is failing to start, ensure `rtu.device` is set in `config.toml` (RTU is serial-only).
  - To run TCP-only, comment out the `[rtu]` section in `config.toml`.

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
