use crate::{
    keeper::Keeper,
    pulley::{ChainMessage, Pulley},
    vendor::Vendor,
};
use alloy::providers::{Provider, WalletProvider};
use tracing::info;

pub struct App<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    keeper: Keeper<P>,
    vendor: Vendor<P>,
    pulley: Pulley<P>,
}

impl<P> App<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    pub fn new(keeper: Keeper<P>, vendor: Vendor<P>, pulley: Pulley<P>) -> Self {
        Self {
            keeper,
            vendor,
            pulley,
        }
    }

    pub async fn process_chain_message(&mut self, message: ChainMessage) -> eyre::Result<()> {
        match message {
            ChainMessage::BuyOrder {
                keeper,
                trader,
                index_id,
                vendor_id,
                collateral,
            } => {
                info!(
                    %keeper,
                    %trader,
                    %index_id,
                    %vendor_id,
                    %collateral,
                    "ChainMessage::BuyOrder"
                );
                if self.keeper.get_index_id() == index_id
                    && self.vendor.get_vendor_id() == vendor_id
                {
                    let assets = self.keeper.get_assets(index_id);
                    self.vendor.update_market(assets).await?;
                    self.keeper.buy_order().await?;
                }
            }
            ChainMessage::SellOrder {
                keeper,
                trader,
                index_id,
                vendor_id,
                itp_amount,
            } => {
                info!(
                    %keeper,
                    %trader,
                    %index_id,
                    %vendor_id,
                    %itp_amount,
                    "ChainMessage::SellOrder"
                );
                if self.keeper.get_index_id() == index_id
                    && self.vendor.get_vendor_id() == vendor_id
                {
                    let assets = self.keeper.get_assets(index_id);
                    self.vendor.update_market(assets).await?;
                    self.keeper.sell_order().await?;
                }
            }
            ChainMessage::Acquisition {
                controller,
                index_id,
                vendor_id,
                remain,
                spent,
                minted,
            } => {
                info!(
                    %controller,
                    %index_id,
                    %vendor_id,
                    %remain,
                    %spent,
                    %minted,
                    "ChainMessage::Acquisition"
                );
                if self.vendor.get_vendor_id() == vendor_id {
                    self.vendor.update_supply().await?;
                }
            }
            ChainMessage::Disposal {
                controller,
                index_id,
                vendor_id,
                remain,
                burned,
                gains,
            } => {
                info!(
                    %controller,
                    %index_id,
                    %vendor_id,
                    %remain,
                    %burned,
                    %gains,
                    "ChainMessage::Disposal"
                );
                if self.vendor.get_vendor_id() == vendor_id {
                    self.vendor.update_supply().await?;
                }
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> eyre::Result<()> {
        loop {
            tokio::select! {
                Ok(messages) = self.pulley.get_message() => {
                    for message in messages {
                        self.process_chain_message(message).await?;
                    }
                }
            }
        }
    }
}
