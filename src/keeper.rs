use alloy::{primitives::Address, providers::{Provider, WalletProvider}};

use crate::common::labels::Labels;

pub enum KeeperMessage {
    AssetsQuoteRequest { assets: Labels },
}

pub struct Keeper<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    provider: P,
    vault_address: Address,
}

impl<P> Keeper<P>
where
    P: Provider + WalletProvider + Clone + 'static,
{
    pub fn new(provider: P) -> Self {
        Self { provider, vault_address: Address::ZERO }
    }

    pub fn get_vault_address(&self) -> Address {
        self.vault_address
    }

    pub async fn setup(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    pub async fn get_message(&mut self) -> eyre::Result<KeeperMessage> {
        Ok(KeeperMessage::AssetsQuoteRequest {
            assets: Labels::new(),
        })
    }

    pub async fn update_quote(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    pub async fn buy_order(&mut self) -> eyre::Result<()> {
        Ok(())
    }
    
    pub async fn sell_order(&mut self) -> eyre::Result<()> {
        Ok(())
    }
}
