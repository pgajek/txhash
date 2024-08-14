use alloy::{
    consensus::ReceiptEnvelope,
    primitives::{b256, keccak256, Address, FixedBytes, Uint, U256},
    providers::{Provider, ProviderBuilder},
};

use clap::Parser;
use eyre::Result;
use tx_event::CliInput;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = CliInput::parse();

    let provider = ProviderBuilder::new().on_http(args.rpc_url);

    let tx_hash = b256!("78fc628fdaca0fff3bc4a096f3809706935e415a98f3e6d3470dc8f63c2ff3d5");

    let fixed_bytes_hash: FixedBytes<32> = FixedBytes::from(tx_hash);

    let receipt_option = provider.get_transaction_receipt(fixed_bytes_hash).await?;

    if let Some(receipt) = receipt_option {
        match receipt.inner {
            ReceiptEnvelope::Legacy(receipt_with_bloom) => {
                let logs = receipt_with_bloom.receipt.logs;

                let expected_event_signature = keccak256("Withdraw(uint256,address,uint256)");

                for log in logs {
                    let log_topics = &log.inner.data.topics();
                    let log_data = &log.inner.data.data;
                    println!("Data: {:#?}", log_data);
                    println!("Topics: {:#?}", log_topics);

                    if log_topics.len() > 0
                        && log_topics[0].as_slice() == expected_event_signature.as_slice()
                    {
                        // Decode id (topics[1])
                        let id = U256::from_be_slice(&log_topics[1].as_slice());

                        // Decode owner address (topics[2])
                        let owner_bytes: [u8; 20] = log_topics[2].as_slice()[12..32]
                            .try_into()
                            .expect("Incorrect length: expected [u8; 20]");
                        let owner = Address::from_slice(&owner_bytes);

                        let amount = U256::from_be_slice(log_data.as_ref());

                        println!("Event: Withdraw");
                        println!("ID: {:?}", id);
                        println!("Owner: {:?}", owner);
                        println!("Amount: {:?}", amount);
                    }
                }
            }
            _ => println!("The receipt is of an unexpected type."),
        }
    }

    Ok(())
}
