# Conveyor

Simple implementation of Vendor-Keeper.

This is to provide minimalist off-chain service, which replies to on-chain events.

## Setting-Up

Best to set environment vars:

```bash
export RPC_URL=             # RPC URL of either Orbit chain or Nitro Dev Node
export PRIVATE_KEY=         # Private Key of the Vendor / Keeper
export CASTLE_ADDRESS=      # Address of the Castle
export CUSTODY_ADDRESS=     # Address of the Custody storing collateral token
export COLLATERAL_ADDRESS=  # Address of Collateral token contract
```

Grant these roles using in your ***Castle Admin*** environment (using `./scripts` in ***VaultWorks*** project and `$DEPLOYER_PRIVATE_KEY`):

```bash
./scripts/roles.sh grant $CASTLE "Castle.ISSUER_ROLE" $VENDOR
./scripts/roles.sh grant $CASTLE "Castle.KEEPER_ROLE" $VENDOR
./scripts/roles.sh grant $CASTLE "Castle.VENDOR_ROLE" $VENDOR
```

Also ensure that `$VENDOR` has enough gas token.

```bash
cast send $VENDOR --value 1.5ether --private-key $GAS_TOKEN_OWNER_KEY --rpc-url $RPC_URL
```

**Note** `$VENDOR` is an address owning `$PRIVATE_KEY` used by *Conveyor*.

The `$GAS_TOKEN_OWNER_KEY` is private key of whoever has gas token to fund *Vendor*.

## Running

Once environment variables are set, roles are granted, and gas token sent, we can run *Conveyor* in following way:

```bash
RUST_LOG=off,conveyor=debug cargo run -- --market-size 1000 --index-id 1001 --vendor-id 101 --chunk-size 350 --index-size 200
```

Once it finishes setting-up following messages will be printed:

```console
conveyor: 🏦 Configured Index / Vault index_size=200 vault_address=👉 0x31fAab8a62b1476b3c66A30CDc9436aB63AadcdA
conveyor: Cranking pulley...
conveyor: 🚦 Starting app...
conveyor::pulley: 🏎️  Pulley loop started...
conveyor::app: ✅ App loop started...
```

The *Vault Address* of newly deployed *Vault* will be printed, so we can place orders to that *Vault* using another private key (as user).
See [*VaultWorks* README](https://github.com/IndexMaker/vaultworks/blob/main/README.md) for details.

In the trader's environment we can now place orders.

First we export *Vault Address* into environment:

```bash
export VAULT=0x31fAab8a62b1476b3c66A30CDc9436aB63AadcdA
```

Let's also export trader's wallet address
```bash
export TRADER=          # Put here address of the trader's wallet
```

Now we can start with *Buy* order (we just deployed new *Index / Vault* so total supply is zero).

First we approve *Vault* to draw cash from trader's (our) wallet, and then we place order:

```bash
./scripts/send.sh $COLLATERAL "approve(address,uint256)" $VAULT 1000000000000000000000
./scripts/send.sh $VAULT "placeBuyOrder(uint128,bool,address,address)(uint128,uint128,uint128)" 1000000000000000000000 true $VENDOR $TRADER
```

Later on, after we acquire some token we can send *Sell* order.

```bash
./scripts/send.sh $VAULT "placeSellOrder(uint128,bool,address,address)(uint128,uint128,uint128)" 10000000000000000 true $VENDOR $TRADER
```

We opted-in for *Instant-Fill* (passing `true` as 2nd parameter), but can check if there is anything remaining on our order:

```bash
./scripts/call.sh $VAULT "getPendingOrder(address,address)(uint128,uint128)" $VENDOR $TRADER
```

This will tell us how much trader (us) can claim in total from this *Vendor*.

The *Conveyor* will automatically observe events generated as a result of our *Buy* order, and will update supply and market data, and then
will push next iteration of the pending order processing forwards.

When we place *Buy* order *Conveyor* will log these messages (among others):
```
conveyor::app: ⛓️ ChainMessage::Acquisition controller=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E index_id=1007 vendor_id=107 remain=900000000556573700862 spent=99999999443426299138 minted=161895332527133
conveyor::vendor: 🚛 Handle: UpdateSupply
conveyor::keeper: 💰 Trading Order trader=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E collateral=0.0 spent=99.999999443426299138 minted=0.000161895332527133 locked=0.0 burned=0.0 withdraw=0.0
conveyor::keeper: 💰 Trading Order trader=0xC0D3C9E530ca6d71469bB678E6592274154D9caD collateral=900.000000556573700862 spent=0.0 minted=0.0 locked=0.0 burned=0.0 withdraw=0.0
2026-01-16T13:08:00.546628Z  INFO conveyor::vendor: 📈 Handle: UpdateMarket
conveyor::keeper: ️🏷  Handle: UpdateQutote
conveyor::keeper: 🚚 Handle: BuyOrder
conveyor::keeper: 💰 Trading Order trader=0xC0D3C9E530ca6d71469bB678E6592274154D9caD collateral=800.000000684390522141 spent=99.999999872183178721 minted=0.000169943179035618 locked=0.0 burned=0.0 withdraw=0.0
conveyor::app: ⛓️ ChainMessage::Acquisition controller=0xC0D3C9E530ca6d71469bB678E6592274154D9caD index_id=1007 vendor_id=107 remain=800000000684390522141 spent=99999999872183178721 minted=169943179035618
conveyor::vendor: 🚛 Handle: UpdateSupply
```

Since *Conveyor* automatically executed next portion of the pending order we can now check how much we can claim in this iteration:

```bash
./scripts/call.sh $VAULT "getClaimableAcquisition(address)(uint128,uint128)" $VENDOR
```

This query will give us total amount claimable across orders from all users, so we need to take minimum of that amount and
the amount on our order that was still pending, and then we can make claim:

```bash
./scripts/send.sh $VAULT "claimAcquisition(uint128,address,address)(uint128)" 99999999872183178721  $VENDOR $TRADER
```

If we are successful (first before someone else makes a claim), then our balance will be updated, and 
we can check that by calling standard ERC-20 method:

```bash
./scripts/call.sh $VAULT "balanceOf(address)" $TRADER | ./scripts/parse_amount.py
```

We can also check the net worth of assets pegging that token amount:

```bash
./scripts/call.sh $VAULT "assetsValue(address)" $TRADER | ./scripts/parse_amount.py
```

**Note** this is worth expressed in *Collateral* token terms.

If we look at *Conveyor* logs, we will see new messages logged (among others):

```
conveyor::app: ⛓️ ChainMessage::AcquisitionClaim keeper=0xC0D3C9E530ca6d71469bB678E6592274154D9caD trader=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E index_id=1007 vendor_id=107 remain=800000000684390522141 spent=99999999872183178721
conveyor::vendor: 📈 Handle: UpdateMarket
conveyor::keeper: 🏷️  Handle: UpdateQutote
conveyor::keeper: 🚚 Handle: BuyOrder
conveyor::keeper: 💰 Trading Order trader=0xC0D3C9E530ca6d71469bB678E6592274154D9caD collateral=700.000000941038607568 spent=99.999999743352503073 minted=0.000159082506013976 locked=0.0 burned=0.0 withdraw=0.0
conveyor::keeper: 💰 Trading Order trader=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E collateral=0.0 spent=199.999999315608889359 minted=0.00033183851156275 locked=0.0 burned=0.0 withdraw=0.0
conveyor::vendor: 🚛 Handle: UpdateSupply
```

By inspecting those messages we can see the flow of the order processing and the amounts on trader's and keeper's orders.

