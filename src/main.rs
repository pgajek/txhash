use alloy::{
    consensus::ReceiptEnvelope,
    primitives::{b256, keccak256, Address, FixedBytes, B256, U256},
    providers::{Provider, ProviderBuilder},
};
use clap::Parser;
use eyre::Result;
use tx_event::CliInput;
use url::Url;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = CliInput::parse();

    let tx_hash = b256!("78fc628fdaca0fff3bc4a096f3809706935e415a98f3e6d3470dc8f63c2ff3d5");

    if let Some((id, owner, amount)) = parse_withdraw_event(args.rpc_url, tx_hash).await? {
        println!("Event: Withdraw");
        println!("ID: {:?}", id);
        println!("Owner: {:?}", owner);
        println!("Amount: {:?}", amount);
    } else {
        println!("No matching Withdraw event found.");
    }

    Ok(())
}

async fn parse_withdraw_event(
    rpc_url: Url,
    tx_hash: B256,
) -> Result<Option<(U256, Address, U256)>> {
    let provider = ProviderBuilder::new().on_http(rpc_url);

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

                    if log_topics.len() > 0
                        && log_topics[0].as_slice() == expected_event_signature.as_slice()
                    {
                        let id = U256::from_be_slice(&log_topics[1].as_slice());

                        let owner_bytes: [u8; 20] = log_topics[2].as_slice()[12..32]
                            .try_into()
                            .expect("Incorrect length: expected [u8; 20]");
                        let owner = Address::from_slice(&owner_bytes);

                        let amount = U256::from_be_slice(log_data.as_ref());

                        return Ok(Some((id, owner, amount)));
                    }
                }
            }
            _ => println!("The receipt is of an unexpected type."),
        }
    }

    Ok(None)
}
