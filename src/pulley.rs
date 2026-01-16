use alloy::{
    primitives::Address,
    providers::{Provider, WalletProvider},
    rpc::types::Filter,
};
use alloy_sol_types::SolEvent;
use eyre::Context;
use futures_util::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::interfaces::{
    vault_native_claims::IVaultNativeClaims, vault_native_orders::IVaultNativeOrders,
};

#[derive(Debug)]
pub enum ChainMessage {
    BuyOrder {
        keeper: Address,
        trader: Address,
        index_id: u128,
        vendor_id: u128,
        collateral: u128,
    },
    SellOrder {
        keeper: Address,
        trader: Address,
        index_id: u128,
        vendor_id: u128,
        itp_amount: u128,
    },
    Acquisition {
        controller: Address,
        index_id: u128,
        vendor_id: u128,
        remain: u128,
        spent: u128,
        minted: u128,
    },
    Disposal {
        controller: Address,
        index_id: u128,
        vendor_id: u128,
        remain: u128,
        burned: u128,
        gains: u128,
    },
    AcquisitionClaim {
        keeper: Address,
        trader: Address,
        index_id: u128,
        vendor_id: u128,
        remain: u128,
        spent: u128,
    },
    DisposalClaim {
        keeper: Address,
        trader: Address,
        index_id: u128,
        vendor_id: u128,
        itp_remain: u128,
        itp_burned: u128,
    },
}

pub struct Pulley;

impl Pulley {
    pub async fn run<P>(
        provider: P,
        vault_address: Address,
        sender: UnboundedSender<ChainMessage>,
        cancel: CancellationToken,
    ) -> eyre::Result<()>
    where
        P: Provider + WalletProvider + Clone + 'static,
    {
        info!("🏎️  Pulley loop started...");

        let filter = Filter::new().address(vault_address).events(vec![
            IVaultNativeOrders::BuyOrder::SIGNATURE,
            IVaultNativeOrders::SellOrder::SIGNATURE,
            IVaultNativeOrders::Acquisition::SIGNATURE,
            IVaultNativeOrders::Disposal::SIGNATURE,
            IVaultNativeClaims::AcquisitionClaim::SIGNATURE,
            IVaultNativeClaims::DisposalClaim::SIGNATURE,
        ]);

        let mut stream = provider.watch_logs(&filter).await?.into_stream();

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    info!("Pulley loop complete.");
                    return Ok(())
                }
                Some(logs) = stream.next() => {
                    for log in logs {
                        if let Ok(event) = log.log_decode::<IVaultNativeOrders::BuyOrder>() {
                            let event = event.data();
                            sender
                                .send(ChainMessage::BuyOrder {
                                    keeper: event.keeper,
                                    trader: event.trader,
                                    index_id: event.index_id,
                                    vendor_id: event.vendor_id,
                                    collateral: event.collateral_amount,
                                })
                                .context("Failed to send chain event")?;
                        }

                        if let Ok(event) = log.log_decode::<IVaultNativeOrders::SellOrder>() {
                            let event = event.data();
                            sender
                                .send(ChainMessage::SellOrder {
                                    keeper: event.keeper,
                                    trader: event.trader,
                                    index_id: event.index_id,
                                    vendor_id: event.vendor_id,
                                    itp_amount: event.itp_amount,
                                })
                                .context("Failed to send chain event")?;
                        }

                        if let Ok(event) = log.log_decode::<IVaultNativeOrders::Acquisition>() {
                            let event = event.data();
                            sender
                                .send(ChainMessage::Acquisition {
                                    controller: event.controller,
                                    index_id: event.index_id,
                                    vendor_id: event.vendor_id,
                                    remain: event.remain,
                                    spent: event.spent,
                                    minted: event.itp_minted,
                                })
                                .context("Failed to send chain event")?;
                        }

                        if let Ok(event) = log.log_decode::<IVaultNativeOrders::Disposal>() {
                            let event = event.data();
                            sender
                                .send(ChainMessage::Disposal {
                                    controller: event.controller,
                                    index_id: event.index_id,
                                    vendor_id: event.vendor_id,
                                    remain: event.itp_remain,
                                    burned: event.itp_burned,
                                    gains: event.gains,
                                })
                                .context("Failed to send chain event")?;
                        }
                        if let Ok(event) = log.log_decode::<IVaultNativeClaims::AcquisitionClaim>() {
                            let event = event.data();
                            sender
                                .send(ChainMessage::AcquisitionClaim {
                                    keeper: event.keeper,
                                    trader: event.trader,
                                    index_id: event.index_id,
                                    vendor_id: event.vendor_id,
                                    remain: event.remain,
                                    spent: event.spent
                                })
                                .context("Failed to send chain event")?;
                        }
                        if let Ok(event) = log.log_decode::<IVaultNativeClaims::DisposalClaim>() {
                            let event = event.data();
                            sender
                                .send(ChainMessage::DisposalClaim {
                                    keeper: event.keeper,
                                    trader: event.trader,
                                    index_id: event.index_id,
                                    vendor_id: event.vendor_id,
                                    itp_remain: event.itp_remain,
                                    itp_burned: event.itp_burned 
                                })
                                .context("Failed to send chain event")?;

                        }
                    }
                }
            }
        }
    }
}
