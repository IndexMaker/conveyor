# Conveyor

Simple implementation of Vendor-Keeper.

This is to provide minimalist off-chain service, which replies to on-chain events.

## Running

Best to set environment vars:

```
export RPC_URL=
export PRIVATE_KEY=
export CASTLE_ADDRESS=
export CUSTODY_ADDRESS=
export COLLATERAL_ADDRESS=
```

Example:
```
RUST_LOG=off,conveyor=debug cargo run -- --market-size 1000 --index-id 1001 --vendor-id 101 --chunk-size 350 --index-size 200
```

After that *Vault Address* of newly deployed *Vault* will be printed, so we can place orders to that *Vault* using another private key (as user).
See [*VaultWorks* README](https://github.com/IndexMaker/vaultworks/blob/main/apps/scenarios/README.md) for details.