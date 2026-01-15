use alloy::{
    primitives::Address,
    providers::{Provider, WalletProvider},
};
use eyre::{Context, bail};
use itertools::Itertools;
use tracing::{debug, info};

use crate::{
    common::{labels::Labels, vector::Vector},
    constants::{DEMAND_LONG_OFFSET, DEMAND_SHORT_OFFSET},
    interfaces::{banker::IBanker, steward::ISteward},
    rand_value::ValueGen,
};

pub struct Vendor<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    provider: P,
    castle_address: Address,
    custody_address: Address,
    collateral_address: Address,
    vendor_id: u128,
    market_assets: Labels,
    chunk_size: usize,
}

impl<P> Vendor<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    pub fn new(
        provider: P,
        castle_address: Address,
        custody_address: Address,
        collateral_address: Address,
        vendor_id: u128,
        chunk_size: usize,
    ) -> Self {
        Self {
            provider,
            castle_address,
            custody_address,
            collateral_address,
            vendor_id,
            chunk_size,
            market_assets: Labels::new(),
        }
    }

    pub fn get_vendor_id(&self) -> u128 {
        self.vendor_id
    }

    pub fn get_assets(&self) -> &Labels {
        &self.market_assets
    }

    pub async fn setup(&mut self, market_size: usize) -> eyre::Result<()> {
        println!("Handle: Setup");

        let assets = Labels {
            data: (1..market_size + 1)
                .into_iter()
                .map(|i| i as u128)
                .collect_vec(),
        };

        for chunk in assets.data.chunks(self.chunk_size) {
            self._submit_assets(chunk).await?;
        }

        self.update_market(&assets).await?;

        self.update_supply().await?;

        self.market_assets = assets;

        Ok(())
    }

    async fn _submit_assets(&mut self, assets: &[u128]) -> eyre::Result<()> {
        let banker = IBanker::new(self.castle_address, &self.provider);
        let mut margin_gen = ValueGen::new(1_00, 10_00, 2);
        let margin = Vector {
            data: assets.iter().map(|_| margin_gen.next()).collect_vec(),
        };

        let asset_names = Labels::from_vec_u128(assets.to_vec());

        info!("Submitting assets...");
        let submit_assets = banker
            .submitAssets(self.vendor_id, asset_names.to_vec().into())
            .send()
            .await
            .context("Failed to submit assets")?;

        info!("Submitting margin...");
        let submit_margin = banker
            .submitMargin(
                self.vendor_id,
                asset_names.to_vec().into(),
                margin.to_vec().into(),
            )
            .send()
            .await
            .context("Failed to submit margin")?;

        let submit_assets_receipt = submit_assets
            .get_receipt()
            .await
            .context("Failed to confirm submit assets")?;

        if !submit_assets_receipt.status() {
            bail!("Failed to submit assets: {:?}", submit_assets_receipt)
        }

        debug!("Submit assets receipt: {:?}", submit_assets_receipt);

        let submit_margin_receipt = submit_margin
            .get_receipt()
            .await
            .context("Failed to confirm submit margin")?;

        if !submit_margin_receipt.status() {
            bail!("Failed to submit margin: {:?}", submit_margin_receipt)
        }

        debug!("Submit margin receipt: {:?}", submit_margin_receipt);

        Ok(())
    }

    pub async fn update_market(&mut self, assets: &Labels) -> eyre::Result<()> {
        println!("Handle: UpdateMarket");
        for chunk in assets.data.chunks(self.chunk_size) {
            self._update_market(chunk).await?;
        }
        Ok(())
    }

    async fn _update_market(&mut self, assets: &[u128]) -> eyre::Result<()> {
        let banker = IBanker::new(self.castle_address, &self.provider);

        let mut price_gen = ValueGen::new(100_00, 1000_00, 2);
        let prices = Vector {
            data: assets.iter().map(|_| price_gen.next()).collect_vec(),
        };

        let mut slope_gen = ValueGen::new(0_01, 0_10, 2);
        let slopes = Vector {
            data: assets.iter().map(|_| slope_gen.next()).collect_vec(),
        };

        let mut liquidity_gen = ValueGen::new(0_10, 10_00, 2);
        let liquidity = Vector {
            data: assets.iter().map(|_| liquidity_gen.next()).collect_vec(),
        };

        let asset_names = Labels::from_vec_u128(assets.to_vec());

        info!("Submitting market data...");
        let submit_market_data = banker
            .submitMarketData(
                self.vendor_id,
                asset_names.to_vec().into(),
                liquidity.to_vec().into(),
                prices.to_vec().into(),
                slopes.to_vec().into(),
            )
            .send()
            .await
            .context("")?;

        let submit_market_data_receipt = submit_market_data
            .get_receipt()
            .await
            .context("Failed to confirm submit market data")?;

        if !submit_market_data_receipt.status() {
            bail!(
                "Failed to submit market data: {:?}",
                submit_market_data_receipt
            )
        }

        debug!("Submit market receipt: {:?}", submit_market_data_receipt);

        Ok(())
    }

    pub async fn update_supply(&mut self) -> eyre::Result<()> {
        println!("Handle: UpdateSupply");
        let steward = ISteward::new(self.castle_address, &self.provider);

        let demand_bytes = steward
            .getVendorDemand(self.vendor_id)
            .call()
            .await
            .context("Faile to obtain demand")?;

        let demand_long = Vector::from_vec(demand_bytes[DEMAND_LONG_OFFSET].to_vec());
        let demand_short = Vector::from_vec(demand_bytes[DEMAND_SHORT_OFFSET].to_vec());

        let zipped = self
            .market_assets
            .data
            .iter()
            .zip(demand_long.data.iter())
            .zip(demand_short.data.iter())
            .map(|((a, b), c)| (*a, *b, *c))
            .collect_vec();

        for chunk in zipped.chunks(self.chunk_size) {
            let assets_chunk = chunk.iter().map(|(a, b, c)| *a).collect_vec();
            let demand_long_chunk = chunk.iter().map(|(a, b, c)| *b).collect_vec();
            let demand_short_chunk = chunk.iter().map(|(a, b, c)| *c).collect_vec();

            self._submit_supply(
                Labels { data: assets_chunk },
                Vector {
                    data: demand_long_chunk,
                },
                Vector {
                    data: demand_short_chunk,
                },
            )
            .await?;
        }

        Ok(())
    }

    async fn _submit_supply(
        &mut self,
        assets: Labels,
        supply_long: Vector,
        supply_short: Vector,
    ) -> eyre::Result<()> {
        let banker = IBanker::new(self.castle_address, &self.provider);

        info!("Submitting supply...");
        let submit_supply = banker
            .submitSupply(
                self.vendor_id,
                assets.to_vec().into(),
                supply_short.to_vec().into(),
                supply_long.to_vec().into(),
            )
            .send()
            .await
            .context("Failed to submit supply")?;

        let submit_supply_receipt = submit_supply
            .get_receipt()
            .await
            .context("Failed to confirm submit supply")?;

        if !submit_supply_receipt.status() {
            bail!("Failed to submit supply: {:?}", submit_supply_receipt)
        }

        debug!("Submit supply receipt: {:?}", submit_supply_receipt);

        Ok(())
    }
}
