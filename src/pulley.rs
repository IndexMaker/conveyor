use alloy::{
    primitives::Address,
    providers::{Provider, WalletProvider},
    rpc::{
        client::PollerStream,
        types::{Filter, Log},
    },
};
use alloy_sol_types::SolEvent;
use futures_util::StreamExt;

use crate::{common::labels::Labels, interfaces::vault_native_orders::IVaultNativeOrders};

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
}

pub struct Pulley<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    _provider: P,
    stream: PollerStream<Vec<Log>>,
}

impl<P> Pulley<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    pub async fn new(provider: P, vault_address: Address) -> eyre::Result<Self> {
        let filter = Filter::new().address(vault_address).events(vec![
            IVaultNativeOrders::BuyOrder::SIGNATURE,
            IVaultNativeOrders::SellOrder::SIGNATURE,
            IVaultNativeOrders::Acquisition::SIGNATURE,
            IVaultNativeOrders::Disposal::SIGNATURE,
        ]);

        let stream = provider.watch_logs(&filter).await?.into_stream();

        Ok(Self {
            _provider: provider,
            stream,
        })
    }

    pub async fn get_message(&mut self) -> eyre::Result<Vec<ChainMessage>> {
        let mut messages = Vec::new();

        while let Some(logs) = self.stream.next().await {
            for log in logs {
                if let Ok(event) = log.log_decode::<IVaultNativeOrders::BuyOrder>() {
                    let event = event.data();
                    messages.push(ChainMessage::BuyOrder {
                        keeper: event.keeper,
                        trader: event.trader,
                        index_id: event.index_id,
                        vendor_id: event.vendor_id,
                        collateral: event.collateral_amount,
                    });
                }

                if let Ok(event) = log.log_decode::<IVaultNativeOrders::SellOrder>() {
                    let event = event.data();
                    messages.push(ChainMessage::SellOrder {
                        keeper: event.keeper,
                        trader: event.trader,
                        index_id: event.index_id,
                        vendor_id: event.vendor_id,
                        itp_amount: event.itp_amount,
                    });
                }

                if let Ok(event) = log.log_decode::<IVaultNativeOrders::Acquisition>() {
                    let event = event.data();
                    messages.push(ChainMessage::Acquisition {
                        controller: event.controller,
                        index_id: event.index_id,
                        vendor_id: event.vendor_id,
                        remain: event.remain,
                        spent: event.spent,
                        minted: event.itp_minted,
                    });
                }

                if let Ok(event) = log.log_decode::<IVaultNativeOrders::Disposal>() {
                    let event = event.data();
                    messages.push(ChainMessage::Disposal {
                        controller: event.controller,
                        index_id: event.index_id,
                        vendor_id: event.vendor_id,
                        remain: event.itp_remain,
                        burned: event.itp_burned,
                        gains: event.gains,
                    });
                }
            }

            eyre::bail!("Event stream closed unexpectedly")
        }

        Ok(messages)
    }
}
