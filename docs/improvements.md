# Improvements

This document tracks suggested improvements and hardening steps for the Modbus simulator.

## Runtime & Concurrency
- Replace `std::sync::RwLock` with an async-aware lock (or use a read-optimized concurrent map) to avoid blocking the Tokio runtime in Modbus request handlers.

## Protocol Semantics
- Return Modbus exceptions for illegal address ranges instead of default `0`/`false` values.
- Enforce read-only semantics for discrete inputs and input registers (reject writes to read-only tables).

## Simulation Engine
- Cache compiled script expressions to avoid parsing on every tick.
- Validate configuration on startup (duplicate addresses, invalid ranges, `update_ms` too small).



## Documentation
- Align README example with full config fields (include `data_bits` and `stop_bits`).
