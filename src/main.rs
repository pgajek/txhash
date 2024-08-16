use alloy::{
    consensus::ReceiptEnvelope,
    primitives::{b256, keccak256, Address, FixedBytes, B256, U256},
    providers::{Provider, ProviderBuilder},
};
use clap::Parser;
use eyre::Result;
use num_bigint::BigUint;
use tx_event::CliInput;
use url::Url;

use starknet::{
    core::types::{Felt, TransactionReceipt::Invoke},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider as StarknetProvider, Url as StarknetUrl,
    },
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = CliInput::parse();

    let tx_hash = b256!("78fc628fdaca0fff3bc4a096f3809706935e415a98f3e6d3470dc8f63c2ff3d5");

    let starknet_tx_hash = b256!("622154d7353ab3c4696f7cac9cd477bfc90e961cd72143c098fafc66ad35689");

    parse_withdraw_event_starknet(args.starknet_rpc_url).await?;

    if let Some((id, owner, amount)) = parse_withdraw_event_evm(args.rpc_url, tx_hash).await? {
        println!("Event: Withdraw");
        println!("ID: {:?}", id);
        println!("Owner: {:?}", owner);
        println!("Amount: {:?}", amount);
    } else {
        println!("No matching Withdraw event found.");
    }

    Ok(())
}

async fn parse_withdraw_event_starknet(rpc_url: String) -> Result<Option<(U256, Address, U256)>> {
    let starknet_url = StarknetUrl::parse(&rpc_url).unwrap();

    let transport = HttpTransport::new(starknet_url);

    let provider = JsonRpcClient::new(transport);

    let typed_hash = Felt::from_hex_unchecked(
        "0x622154d7353ab3c4696f7cac9cd477bfc90e961cd72143c098fafc66ad35689",
    );
    // let typed_hash = Felt::from_hex_unchecked(&tx_hash);

    let receipt = provider.get_transaction_receipt(typed_hash).await?;

    let events = match receipt.receipt {
        Invoke(invoke_receipt) => invoke_receipt.events,
        _ => vec![],
    };

    for event in events {
        process_withdraw_event(&event);
    }

    Ok(None)
}

fn process_withdraw_event(event: &starknet::core::types::Event) {
    let id = event.keys[1];
    let owner = event.keys[2];

    let amount_low = event.data[0];
    let amount_high = event.data[1];

    let amount_low_biguint: BigUint = amount_low.to_biguint();
    let amount_high_biguint: BigUint = amount_high.to_biguint();

    let amount_low_bytes = amount_low_biguint.to_bytes_be();
    let amount_high_bytes = amount_high_biguint.to_bytes_be();

    let amount_low_u256 = U256::from_be_slice(&amount_low_bytes);
    let amount_high_u256 = U256::from_be_slice(&amount_high_bytes);

    let amount = (amount_high_u256 << 128) | amount_low_u256;

    println!("Withdraw Event:");
    println!("ID: {}", id);
    println!("Owner: {}", owner);
    println!("Amount: {}", amount);
}

async fn parse_withdraw_event_evm(
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
