use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::{Provider, ProviderBuilder, WalletProvider},
    signers::local::PrivateKeySigner,
};
use clap::Parser;
use conveyor::{
    app::App,
    keeper::{Keeper, KeeperMessage},
    pulley::{ChainMessage, Pulley},
    vendor::Vendor,
};

// --- 2. CLI Arguments ---
#[derive(Parser, Debug)]
#[command(author, version, about = "Conveyor: Off-chain client for VaultWorks")]
struct Args {
    #[arg(
        short,
        long,
        env = "RPC_URL",
        default_value = "http://http://localhost:8547"
    )]
    rpc_url: String,

    #[arg(short, long, env = "PRIVATE_KEY")]
    private_key: String,

    #[arg(short, long, env = "CASTLE_ADDRESS")]
    castle_address: Address,

    #[arg(short, long, env = "CUSTODY_ADDRESS")]
    custody_address: Address,

    #[arg(short, long, default_value = "1")]
    vendor_id: u128,

    #[arg(short, long, default_value = "1001")]
    index_id: u128,

    #[arg(short, long, default_value = "5")]
    market_size: usize,

    #[arg(short, long, default_value = "3")]
    index_size: usize,
}

async fn with_provider(
    rpc_url: String,
    private_key: String,
) -> eyre::Result<impl Provider + WalletProvider + Clone + 'static> {
    let signer: PrivateKeySigner = private_key.parse()?;
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(rpc_url.as_str())
        .await?;

    Ok(provider)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let provider = with_provider(args.rpc_url, args.private_key).await?;

    let mut keeper = Keeper::new(provider.clone());
    let mut vendor = Vendor::new(provider.clone());

    keeper.setup().await?;
    vendor.setup(args.market_size).await?;

    let pulley = Pulley::new(provider, keeper.get_vault_address()).await?;

    let mut app = App::new(keeper, vendor, pulley);

    if let Err(err) = app.run().await {
        eprintln!("Error while running app: {:?}", err);
    }

    Ok(())
}
