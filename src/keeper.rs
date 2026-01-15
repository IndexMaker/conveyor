use alloy::{
    primitives::Address,
    providers::{Provider, WalletProvider},
};

use crate::common::labels::Labels;

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

    pub fn get_vault_address(&self) -> Address {
        self.vault_address
    }

    pub async fn setup(&mut self, index_size: usize) -> eyre::Result<()> {
        Ok(())
    }

    pub fn get_assets(&self, index_id: u128) -> &Labels {
        &self.assets
    }

    pub async fn update_quote(&mut self) -> eyre::Result<()> {
        println!("Handle: UpdateQutote");
        Ok(())
    }

    pub async fn buy_order(&mut self) -> eyre::Result<()> {
        println!("Handle: BuyOrder");
        Ok(())
    }

    pub async fn sell_order(&mut self) -> eyre::Result<()> {
        println!("Handle: SellOrder");
        Ok(())
    }
}
