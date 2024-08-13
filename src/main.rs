// use alloy::{
//     primitives::H256,
//     providers::{Provider, ProviderBuilder},
//     sol,
//     types::{Trace, TransactionTrace},
// };
use alloy::primitives::{Address, B256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::eth::*;
use alloy::rpc::types::trace;
use alloy::sol;
use clap::Parser;
use eyre::Result;
use std::str::FromStr;
use tx_event::CliInput;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IWETH9,
    "./abi.json"
);
#[tokio::main]
pub async fn main() -> Result<()> {
    let args = CliInput::parse();

    // let provider = ProviderBuilder::new().on_http(args.rpc_url);
    // let latest_block = provider.get_block_number().await?;

    let provider = ProviderBuilder::new()
        .on_http(args.rpc_url.clone())
        .build()?;
    let tx_hash = H256::from_str(&args.tx_hash)?;
    let trace: Option<TransactionTrace> = provider.trace_transaction(tx_hash).await?;

    if let Some(tx_trace) = trace {
        for step in tx_trace.steps {
            if let Trace::Log(log) = step {
                println!("Found log: {:?}", log);
            }
        }
    } else {
        println!("No trace found for transaction hash: {}", args.tx_hash);
    }
    // println!("Latest block number: {latest_block}");
    Ok(())
}
