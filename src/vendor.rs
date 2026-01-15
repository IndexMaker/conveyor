use alloy::{
    primitives::Address,
    providers::{Provider, WalletProvider},
};

use crate::common::{labels::Labels, vector::Vector};

pub struct Vendor<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    provider: P,
    castle_address: Address,
    custody_address: Address,
    collateral_address: Address,
    vendor_id: u128,

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
    pub fn new(
        provider: P,
        castle_address: Address,
        custody_address: Address,
        collateral_address: Address,
        vendor_id: u128,
    ) -> Self {
        Self {
            provider,
            castle_address,
            custody_address,
            collateral_address,
            vendor_id,
            assets: Labels::new(),
            margin: Vector::new(),
            market_data: Vector::new(),
            supply: Vector::new(),
            demand: Vector::new(),
        }
    }

    pub fn get_vendor_id(&self) -> u128 {
        self.vendor_id
    }

    pub async fn setup(&mut self, market_size: usize) -> eyre::Result<()> {
        Ok(())
    }

    pub async fn update_market(&mut self, assets: &Labels) -> eyre::Result<()> {
        println!("Handle: UpdateMarket");
        Ok(())
    }

    pub async fn update_supply(&mut self) -> eyre::Result<()> {
        println!("Handle: UpdateSupply");
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
