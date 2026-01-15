use alloy::providers::{Provider, WalletProvider};

use crate::common::{labels::Labels, vector::Vector};

pub struct Vendor<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    provider: P,

    assets: Labels,
    margin: Vector,
    market_data: Vector,
    supply: Vector,
    demand: Vector,
}

impl<P> Vendor<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            assets: Labels::new(),
            margin: Vector::new(),
            market_data: Vector::new(),
            supply: Vector::new(),
            demand: Vector::new(),
        }
    }

    pub async fn setup(&mut self, num_assets: usize) -> eyre::Result<()> {
        Ok(())
    }

    pub async fn update_market(&mut self, assets: Labels) -> eyre::Result<()> {
        Ok(())
    }

    pub async fn update_supply(&mut self) -> eyre::Result<()> {
        // let banker = IBanker::new(args.castle_address, &provider);

        // let market_assets = Vector { data: vec![] };
        // let market_assets_bytes = market_assets.to_vec();

        // let tx_builder = banker.submitAssets(args.vendor_id, market_assets_bytes.into());

        // println!("Sending transaction...");
        // let pending_tx = tx_builder.send().await?;

        // println!("Transaction sent! Hash: {}", pending_tx.tx_hash());

        // let receipt = pending_tx.get_receipt().await?;

        // if receipt.status() {
        //     println!("Success: Assets submitted to the VaultWorks Banker.");
        // } else {
        //     println!("Failure: Transaction reverted on-chain.");
        // }

        Ok(())
    }
}
