use crate::{keeper::Keeper, pulley::ChainMessage, vendor::Vendor};
use alloy::providers::{Provider, WalletProvider};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct App<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    keeper: Keeper<P>,
    vendor: Vendor<P>,
}

impl<P> App<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    pub fn new(keeper: Keeper<P>, vendor: Vendor<P>) -> Self {
        Self { keeper, vendor }
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
                    "⛓️ ChainMessage::BuyOrder"
                );
                if self.keeper.get_index_id() == index_id
                    && self.vendor.get_vendor_id() == vendor_id
                {
                    let assets = self.keeper.get_assets();
                    self.keeper.log_trader_order(trader).await?;
                    self.keeper.log_pending_order().await?;
                    self.vendor.update_market(assets).await?;
                    self.keeper.update_quote().await?;
                    self.keeper.buy_order().await?;
                    self.keeper.log_pending_order().await?;
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
                    "⛓️ ChainMessage::SellOrder"
                );
                if self.keeper.get_index_id() == index_id
                    && self.vendor.get_vendor_id() == vendor_id
                {
                    let assets = self.keeper.get_assets();
                    self.keeper.log_trader_order(trader).await?;
                    self.keeper.log_pending_order().await?;
                    self.vendor.update_market(assets).await?;
                    self.keeper.update_quote().await?;
                    self.keeper.sell_order().await?;
                    self.keeper.log_pending_order().await?;
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
                    "⛓️ ChainMessage::Acquisition"
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
                    "⛓️ ChainMessage::Disposal"
                );
                if self.vendor.get_vendor_id() == vendor_id {
                    self.vendor.update_supply().await?;
                }
            }
            ChainMessage::AcquisitionClaim {
                keeper,
                trader,
                index_id,
                vendor_id,
                remain,
                spent,
            } => {
                info!(
                    %keeper,
                    %trader,
                    %index_id,
                    %vendor_id,
                    %remain,
                    %spent,
                    "⛓️ ChainMessage::AcquisitionClaim"
                );
                if 100 < remain {
                    let assets = self.keeper.get_assets();
                    self.vendor.update_market(assets).await?;
                    self.keeper.update_quote().await?;
                    self.keeper.buy_order().await?;
                    self.keeper.log_pending_order().await?;
                    self.keeper.log_trader_order(trader).await?;
                }
            }
            ChainMessage::DisposalClaim {
                keeper,
                trader,
                index_id,
                vendor_id,
                itp_remain,
                itp_burned,
            } => {
                info!(
                    %keeper,
                    %trader,
                    %index_id,
                    %vendor_id,
                    %itp_remain,
                    %itp_burned,
                    "⛓️ ChainMessage::DisposalClaim"
                );
                if 100 < itp_remain {
                    let assets = self.keeper.get_assets();
                    self.vendor.update_market(assets).await?;
                    self.keeper.update_quote().await?;
                    self.keeper.sell_order().await?;
                    self.keeper.log_pending_order().await?;
                    self.keeper.log_trader_order(trader).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn run(
        &mut self,
        mut recv: UnboundedReceiver<ChainMessage>,
        cancel: CancellationToken,
    ) -> eyre::Result<()> {
        info!("✅ App loop started...");
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    info!("App loop complete.");
                    return Ok(())
                }
                Some(message) = recv.recv() => {
                    self.process_chain_message(message).await?;
                }
            }
        }
    }
}
