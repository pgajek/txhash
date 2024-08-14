use alloy::{
    consensus::ReceiptEnvelope,
    network::{EthereumWallet, TransactionBuilder},
    node_bindings::Anvil,
    primitives::{b256, FixedBytes},
    providers::{ext::DebugApi, Provider, ProviderBuilder},
    rpc::types::trace::geth::{
        GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingOptions,
        GethDefaultTracingOptions,
    },
    rpc::types::TransactionReceipt,
    signers::local::PrivateKeySigner,
    sol,
};

use clap::Parser;
use eyre::Result;
use tx_event::CliInput;

// sol!(
//     #[allow(missing_docs)]
//     #[sol(rpc)]
//     IWETH9,
//     "./abi.json"
// );
#[tokio::main]
pub async fn main() -> Result<()> {
    let args = CliInput::parse();

    let provider = ProviderBuilder::new().on_http(args.rpc_url);

    let tx_hash = b256!("78fc628fdaca0fff3bc4a096f3809706935e415a98f3e6d3470dc8f63c2ff3d5");

    let fixed_bytes_hash: FixedBytes<32> = FixedBytes::from(tx_hash);

    let receipt_option = provider.get_transaction_receipt(fixed_bytes_hash).await?;

    // println!(
    //     "***************RECEIPT****************: {:?}",
    //     receipt_option
    // );

    if let Some(receipt) = receipt_option {
        match receipt.inner {
            ReceiptEnvelope::Legacy(receipt_with_bloom) => {
                let logs = receipt_with_bloom.receipt.logs;

                for log in logs {
                    println!("{:#?}", log);
                }
            }
            _ => println!("The receipt is of an unexpected type."),
        }
    } else {
        println!("No transaction receipt available.");
    }

    Ok(())
}

// Some(TransactionReceipt { inner: Legacy(ReceiptWithBloom { receipt: Receipt { status: Eip658(true), cumulative_gas_used: 49954, logs: [Log { inner: Log { address: 0x5fbdb2315678afecb367f032d93f642f64180aa3, data: LogData { topics: [0x9da6493a92039daf47d1f2d7a782299c5994c6323eb1e972f69c432089ec52bf, 0x0000000000000000000000000000000000000000000000000000000000000002, 0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266], data: 0x000000000000000000000000000000000000000000000000000009184e72a000 } }, block_hash: Some(0x5282469631a3a1866aa397effb44a120ee5ff3be7708bb99d192261299a35aea), block_number: Some(6), block_timestamp: Some(1723637308), transaction_hash: Some(0x78fc628fdaca0fff3bc4a096f3809706935e415a98f3e6d3470dc8f63c2ff3d5), transaction_index: Some(0), log_index: Some(0), removed: false }] }, logs_bloom: 0x04000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000040000000000000000000000000000000000000000000000000000000040000000000000100100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000100000000000000000000000000000000000000000000000040000000200000000000000000000000002000000000000000000000000000000000000000000000000000000000000000008000000000000000000000 }), transaction_hash: 0x78fc628fdaca0fff3bc4a096f3809706935e415a98f3e6d3470dc8f63c2ff3d5, transaction_index: Some(0), block_hash: Some(0x5282469631a3a1866aa397effb44a120ee5ff3be7708bb99d192261299a35aea), block_number: Some(6), gas_used: 49954, effective_gas_price: 1515761873, blob_gas_used: None, blob_gas_price: Some(1), from: 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266, to: Some(0x5fbdb2315678afecb367f032d93f642f64180aa3), contract_address: None, state_root: Some(0xff4f18aa4f9fe3e2ced2e16eec4f40f558690482678f7fbcea5bcb94ab2242cd), authorization_list: None })
