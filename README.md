# Conveyor

Simple implementation of Vendor-Keeper.

This is to provide minimalist off-chain service, which replies to on-chain events.

## Running

Example:
```
RUST_LOG=off,conveyor=debug cargo run -- --market-size 1000 --index-id 1001 --vendor-id 101 --chunk-size 350 --index-size 200
```