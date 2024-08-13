use alloy::providers::{Provider, ProviderBuilder};
use clap::Parser;
use eyre::Result;
use tx_event::CliInput;
#[tokio::main]
pub async fn main() -> Result<()> {
    let args = CliInput::parse();

    let provider = ProviderBuilder::new().on_http(args.rpc_url);
    let latest_block = provider.get_block_number().await?;
    println!("Latest block number: {latest_block}");
    Ok(())
}
