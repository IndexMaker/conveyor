use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::{Provider, ProviderBuilder, WalletProvider},
    signers::local::PrivateKeySigner,
};
use clap::Parser;
use conveyor::{app::App, keeper::Keeper, pulley::Pulley, vendor::Vendor};
use eyre::bail;
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::mpsc::unbounded_channel,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

// --- 2. CLI Arguments ---
#[derive(Parser, Debug)]
#[command(author, version, about = "Conveyor: Off-chain client for VaultWorks")]
struct Args {
    #[arg(long, env = "RPC_URL", default_value = "http://http://localhost:8547")]
    rpc_url: String,

    #[arg(long, env = "PRIVATE_KEY")]
    private_key: String,

    #[arg(long, env = "CASTLE_ADDRESS")]
    castle_address: Address,

    #[arg(long, env = "CUSTODY_ADDRESS")]
    custody_address: Address,

    #[arg(long, env = "COLLATERAL_ADDRESS")]
    collateral_address: Address,

    #[arg(long, default_value = "1")]
    vendor_id: u128,

    #[arg(long, default_value = "1001")]
    index_id: u128,

    #[arg(long, default_value = "5")]
    market_size: usize,

    #[arg(long, default_value = "3")]
    index_size: usize,

    #[arg(long, default_value = "500")]
    chunk_size: usize,
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

fn init_tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_tracing();

    let args = Args::try_parse()?;

    let provider = with_provider(args.rpc_url, args.private_key).await?;

    info!(
        wallet= %provider.default_signer_address(),
        "Provider connected"
    );

    let mut keeper = Keeper::new(
        provider.clone(),
        args.castle_address,
        args.custody_address,
        args.collateral_address,
        args.index_id,
        args.vendor_id,
    );

    let mut vendor = Vendor::new(
        provider.clone(),
        args.castle_address,
        args.custody_address,
        args.collateral_address,
        args.vendor_id,
        args.chunk_size,
    );

    info!(
        castle_address = %args.castle_address,
        custody_address = %args.custody_address,
        collateral_address = %args.collateral_address,
        index_id = %args.index_id,
        vendor_id = %args.vendor_id,
        "Configured Keeper & Vendor"
    );

    vendor.setup(args.market_size).await?;

    info!(
        market_size = %args.market_size,
        "Configured Market"
    );

    keeper.setup(vendor.get_market_assets(), args.index_size).await?;

    let vault_address = keeper.get_vault_address();
    if vault_address.is_zero() {
        bail!("Vault address is zero")
    }

    info!(
        index_size = %args.index_size,
        %vault_address,
        "Configured Index / Vault"
    );

    info!("Cranking pulley...");

    let (tx, rx) = unbounded_channel();

    let cancel_token = CancellationToken::new();
    let pulley_task = tokio::spawn(Pulley::run(
        provider,
        vault_address,
        tx,
        cancel_token.clone(),
    ));

    info!("Starting app...");

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigquit = signal(SignalKind::quit())?;

    let cancel_token_clone = cancel_token.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = sigint.recv() => cancel_token_clone.cancel(),
            _ = sigterm.recv() => cancel_token_clone.cancel(),
            _ = sigquit.recv() => cancel_token_clone.cancel(),
        }
    });

    let mut app = App::new(keeper, vendor);

    if let Err(err) = app.run(rx, cancel_token.clone()).await {
        error!("Error while running app: {:?}", err);
    }

    info!("Terminating app...");
    if let Err(err) = pulley_task.await {
        error!("Error while terminating: {:?}", err);
    }

    Ok(())
}
