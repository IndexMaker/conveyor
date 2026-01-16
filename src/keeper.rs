use alloy::{
    primitives::Address,
    providers::{Provider, WalletProvider},
};
use eyre::{Context, bail};
use itertools::Itertools;
use tracing::{debug, info};

use crate::{
    common::{
        amount::Amount,
        constants::{
            ORDER_BURNED_OFFSET, ORDER_COLLATERAL_OFFSET, ORDER_LOCKED_OFFSET, ORDER_MINTED_OFFSET,
            ORDER_SPENT_OFFSET, ORDER_WITHDRAW_OFFSET,
        },
        labels::Labels,
        rand_pick_assets::rand_pick_assets,
        rand_value::ValueGen,
        vector::Vector,
    },
    interfaces::{
        banker::IBanker, guildmaster::IGuildmaster, steward::ISteward,
        vault_native_orders::IVaultNativeOrders,
    },
};

pub struct Keeper<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    provider: P,
    castle_address: Address,
    custody_address: Address,
    collateral_address: Address,
    vault_address: Address,
    index_id: u128,
    vendor_id: u128,
    assets: Labels,
}

impl<P> Keeper<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    pub fn new(
        provider: P,
        castle_address: Address,
        custody_address: Address,
        collateral_address: Address,
        index_id: u128,
        vendor_id: u128,
    ) -> Self {
        Self {
            provider,
            castle_address,
            custody_address,
            collateral_address,
            index_id,
            vendor_id,
            vault_address: Address::ZERO,
            assets: Labels::new(),
        }
    }

    pub fn get_index_id(&self) -> u128 {
        self.index_id
    }

    pub fn get_vendor_id(&self) -> u128 {
        self.vendor_id
    }

    pub fn get_assets(&self) -> &Labels {
        &self.assets
    }

    pub fn get_custody_address(&self) -> Address {
        self.custody_address
    }

    pub fn get_collateral_address(&self) -> Address {
        self.collateral_address
    }

    pub fn get_vault_address(&self) -> Address {
        self.vault_address
    }

    pub async fn setup(&mut self, market_assets: &Labels, index_size: usize) -> eyre::Result<()> {
        info!("Handle: Keeper Setup");
        let guildmaster = IGuildmaster::new(self.castle_address, &self.provider);
        let keeper = self.provider.default_signer_address();
        let max_order_size = Amount::from_u128_with_scale(100, 0);

        let submit_index_call = guildmaster.submitIndex(
            self.vendor_id,
            self.index_id,
            "name".into(),
            format!("X{}", index_size),
            "Description".into(),
            "Methodology".into(),
            0,
            Address::ZERO,
            "Custody".into(),
            vec![keeper],
            self.custody_address,
            self.collateral_address,
            max_order_size.to_u128_raw(),
        );

        info!("Calling submit Index...");
        let vault_address = submit_index_call
            .call()
            .await
            .context("Failed to call submit index")?;

        info!("Submitting Index...");
        let submit_index = submit_index_call
            .send()
            .await
            .context("Failed to submit index")?;

        let submit_index_receipt = submit_index
            .get_receipt()
            .await
            .context("Failed to submit index")?;

        if !submit_index_receipt.status() {
            bail!("Failed to submit index: {:?}", submit_index_receipt)
        }

        debug!("Submit index receipt: {:?}", submit_index_receipt);

        self.vault_address = vault_address;

        info!("Submitting vote...");
        let vote = guildmaster
            .submitVote(self.index_id, vec![].into())
            .send()
            .await
            .context("Failed to send vote")?;

        let vote_receipt = vote.get_receipt().await.context("Failed to vote")?;

        if !vote_receipt.status() {
            bail!("Failed to vote: {:?}", vote_receipt)
        }

        debug!("Vote receipt: {:?}", vote_receipt);

        let assets = rand_pick_assets(market_assets, index_size);

        let mut weight_gen = ValueGen::new(1_00, 10_00, 2);
        let asset_weights = Vector {
            data: assets.data.iter().map(|_| weight_gen.next()).collect_vec(),
        };

        info!("Submitting asset weights...");
        let submit_asset_weights = guildmaster
            .submitAssetWeights(
                self.index_id,
                assets.to_vec().into(),
                asset_weights.to_vec().into(),
            )
            .send()
            .await
            .context("Failed to submit asset weights")?;

        let submit_asset_weights_receipt = submit_asset_weights
            .get_receipt()
            .await
            .context("Failed to submit asset weights")?;

        if !submit_asset_weights_receipt.status() {
            bail!(
                "Failed to submit asset weights: {:?}",
                submit_asset_weights_receipt
            )
        }

        debug!(
            "Submit asset weights receipt: {:?}",
            submit_asset_weights_receipt
        );

        self.assets = assets;

        self.update_quote().await?;

        Ok(())
    }

    pub async fn update_quote(&mut self) -> eyre::Result<()> {
        info!("🏷️  Handle: UpdateQutote");
        let banker = IBanker::new(self.castle_address, &self.provider);

        info!("Updating quote...");
        let update_quote = banker
            .updateIndexQuote(self.vendor_id, self.index_id)
            .send()
            .await
            .context("Failed to update quote")?;

        let update_quote_receipt = update_quote
            .get_receipt()
            .await
            .context("Failed to confirm update quote")?;

        if !update_quote_receipt.status() {
            bail!("Failed to update quote: {:?}", update_quote_receipt)
        }

        debug!("Update quote receipt: {:?}", update_quote_receipt);

        Ok(())
    }

    pub async fn buy_order(&mut self) -> eyre::Result<()> {
        info!("🚚 Handle: BuyOrder");
        let vault = IVaultNativeOrders::new(self.vault_address, &self.provider);
        let keeper = self.provider.default_signer_address();

        let request = vault
            .processPendingBuyOrder(keeper)
            .send()
            .await
            .context("Failed to process pending Buy order")?;

        let receipt = request
            .get_receipt()
            .await
            .context("Failed to confirm process pending Buy order")?;

        if !receipt.status() {
            bail!("Failed to process pending Buy order: {:?}", receipt)
        }

        debug!("Update pending Buy order receipt: {:?}", receipt);

        Ok(())
    }

    pub async fn sell_order(&mut self) -> eyre::Result<()> {
        info!("🚚 Handle: SellOrder");
        let vault = IVaultNativeOrders::new(self.vault_address, &self.provider);
        let keeper = self.provider.default_signer_address();

        let request = vault
            .processPendingSellOrder(keeper)
            .send()
            .await
            .context("Failed to process pending Sell order")?;

        let receipt = request
            .get_receipt()
            .await
            .context("Failed to confirm process pending Sell order")?;

        if !receipt.status() {
            bail!("Failed to process pending Sell order: {:?}", receipt)
        }

        debug!("Update pending Sell order receipt: {:?}", receipt);

        Ok(())
    }

    pub async fn log_pending_order(&self) -> eyre::Result<()> {
        self.log_trader_order(self.provider.default_signer_address())
            .await?;
        Ok(())
    }

    pub async fn log_trader_order(&self, trader: Address) -> eyre::Result<()> {
        let steward = ISteward::new(self.castle_address, &self.provider);

        let order_bytes = steward
            .getTraderOrder(self.index_id, trader)
            .call()
            .await
            .context("Failed to obtain trader order")?;

        let order = Vector::from_vec(order_bytes);

        info!(
            %trader,
            collateral = %order.data[ORDER_COLLATERAL_OFFSET],
            spent = %order.data[ORDER_SPENT_OFFSET],
            minted = %order.data[ORDER_MINTED_OFFSET],
            locked = %order.data[ORDER_LOCKED_OFFSET],
            burned = %order.data[ORDER_BURNED_OFFSET],
            withdraw = %order.data[ORDER_WITHDRAW_OFFSET],
            "💰 Trading Order"
        );

        Ok(())
    }
}
