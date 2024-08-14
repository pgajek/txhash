use alloy::{
    node_bindings::Anvil,
    // primitives::b256,
    primitives::{address, Address, Uint, B256, U256},
    // providers::{ext::DebugApi, ProviderBuilder, WsConnect},
    providers::{Provider, ProviderBuilder, WsConnect},
    // rpc::types::trace::geth::{
    //     GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingOptions,
    //     GethDefaultTracingOptions, GethTrace,
    // },
    rpc::types::{BlockNumberOrTag, Filter},
    sol,
    sol_types::SolEvent,
};

use clap::Parser;
use eyre::Result;
use futures_util::stream::StreamExt;
use std::time::Duration;
use tokio::time::sleep;
use tx_event::CliInput;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc, bytecode = "60808060405234601557610324908161001a8239f35b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c9081632e1a7d4d146102235750806361b8ce8c146102075780638c64ea4a146101b95780639507d39a146101395763b6b55f2514610053575f80fd5b6020366003190112610135576004358015610126575f546100726102ba565b3381526002602082019184835260408101925f8452845f52600160205260405f209160018060a01b039051166bffffffffffffffffffffffff60a01b835416178255516001820155019051151560ff801983541691161790555f549160018301809311610112576020925f55604051908152817feaa18152488ce5959073c9c79c88ca90b3d96c00de1f118cfaad664c3dab06b9843393a3604051908152f35b634e487b7160e01b5f52601160045260245ffd5b635e85ae7360e01b5f5260045ffd5b5f80fd5b34610135576020366003190112610135575f60406101556102ba565b82815282602082015201526004355f526001602052606060405f206101786102ba565b60018060a01b0382541691828252604060ff600260018401549360208601948552015416920191151582526040519283525160208301525115156040820152f35b34610135576020366003190112610135576004355f526001602052606060405f2060018060a01b038154169060ff600260018301549201541690604051928352602083015215156040820152f35b34610135575f3660031901126101355760205f54604051908152f35b3461013557602036600319011261013557600435805f52600160205260405f206002810180549360ff85166102ab5782546001600160a01b03169433860361029c577f9da6493a92039daf47d1f2d7a782299c5994c6323eb1e972f69c432089ec52bf936020936001809360ff191617905501548152a3005b63b39ed90360e01b5f5260045ffd5b63939e783760e01b5f5260045ffd5b604051906060820182811067ffffffffffffffff8211176102da57604052565b634e487b7160e01b5f52604160045260245ffdfea2646970667358221220f79159ae66cea60e633f25b819044323f2ea47717dc1aeccfee8bb8fa16ec03a64736f6c634300081a0033")]
    contract VaultContract {
        struct Vault {
            address owner;
            uint256 amount;
            bool spent;
        }

        uint256 public nextId;
        mapping(uint256 => Vault) public vaults;

        event Withdraw(uint256 indexed id, address indexed owner, uint256 amount);
        event Deposit(uint256 indexed id, address indexed owner, uint256 amount);

        error AmountMustBeGreaterThanZero();
        error CallerMustBeVaultOwner();
        error VaultMustBeUnspent();

        function get(uint256 id) public view returns (Vault memory) {
            return vaults[id];
        }

        function deposit(uint256 amount) public payable returns (uint256) {
            if (amount == 0) {
                revert AmountMustBeGreaterThanZero();
            }

            uint256 id = nextId;
            vaults[id] = Vault({owner: msg.sender, amount: amount, spent: false});
            nextId += 1;

            emit Deposit(id, msg.sender, amount);

            return id;
        }

        function withdraw(uint256 id) public {
            Vault storage vault = vaults[id];

            if (vault.spent) {
                revert VaultMustBeUnspent();
            }

            if (vault.owner != msg.sender) {
                revert CallerMustBeVaultOwner();
            }

            vault.spent = true;

            emit Withdraw(id, vault.owner, vault.amount);
        }
    }
);

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = CliInput::parse();
    let anvil = Anvil::new().block_time(1).try_spawn()?;
    let ws = WsConnect::new(anvil.ws_endpoint());
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    let contract = VaultContract::deploy(provider.clone()).await?;

    // println!("Deployed contract at: {}", contract.address());

    let sender: Address = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        .parse()
        .unwrap();

    let deposit_filter = contract.Deposit_filter().watch().await?;
    let withdraw_filter = contract.Withdraw_filter().watch().await?;

    let deposit_call = contract.deposit(Uint::from(1000u64)).from(sender);

    let deposit_tx = deposit_call.send().await?;
    let deposit_tx_hash = deposit_tx.tx_hash();
    // println!("Deposit transaction sent with hash: {:?}", deposit_tx_hash);

    let deposit_receipt = loop {
        if let Some(receipt) = provider.get_transaction_receipt(*deposit_tx_hash).await? {
            break receipt;
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    };
    // println!(
    //     "Deposit transaction confirmed with receipt: {:?}",
    //     deposit_receipt
    // );

    let vault_id = 0;

    let withdraw_call = contract.withdraw(Uint::from(vault_id)).from(sender);

    let withdraw_tx = withdraw_call.send().await?;
    let withdraw_tx_hash = withdraw_tx.tx_hash();
    // println!(
    //     "Withdraw transaction sent with hash: {:?}",
    //     withdraw_tx_hash
    // );

    let withdraw_receipt = loop {
        if let Some(receipt) = provider.get_transaction_receipt(*withdraw_tx_hash).await? {
            break receipt;
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    };
    // println!(
    //     "Withdraw transaction confirmed with receipt: {:?}",
    //     withdraw_receipt
    // );

    deposit_filter
        .into_stream()
        .take(1)
        .for_each(|log| async {
            match log {
                Ok((_event, log)) => {
                    println!("Received Deposit: {log:?}");
                }
                Err(e) => {
                    println!("Error: {e:?}");
                }
            }
        })
        .await;

    withdraw_filter
        .into_stream()
        .take(1)
        .for_each(|log| async {
            match log {
                Ok((_event, log)) => {
                    println!("Received Withdraw: {log:?}");
                }
                Err(e) => {
                    println!("Error: {e:?}");
                }
            }
        })
        .await;

    // ```````````````````````````````````````````
    // let weth9_token_address = address!("7b16d831F6819aa2C8191392e867C5f86f7f93B9");
    // let filter = Filter::new()
    //     .address(weth9_token_address)
    //     .from_block(BlockNumberOrTag::Latest);

    // let sub = provider.subscribe_logs(&filter).await?;
    // let mut stream = sub.into_stream();

    // while let Some(log) = stream.next().await {
    //     match log.topic0() {
    //         Some(&IWETH9::Withdraw::SIGNATURE_HASH) => {
    //             let IWETH9::Withdraw { id, owner, amount } = log.log_decode()?.inner.data;
    //             println!("Approval from {id} to {owner} of value {amount}");
    //         }
    //         _ => (),
    //     }
    // }
    // {TRACE}
    // let provider = ProviderBuilder::new().on_http(args.rpc_url);

    // let tx_hash = b256!("40ecdeee6e6b3ea914e6f38fb50233d88c76bafee20297368a3cea2675fd18e1");
    // // let tx_hash = B256::from_str(&args.tx_hash)?;

    // let default_options = GethDebugTracingOptions {
    //     tracer: Some(GethDebugTracerType::BuiltInTracer(
    //         GethDebugBuiltInTracerType::CallTracer,
    //     )),
    //     ..Default::default()
    // };
    // let result = provider
    //     .debug_trace_transaction(tx_hash, default_options)
    //     .await?;

    // // println!("---DEFAULT_TRACE---: {result:?}");

    // let call_options = GethDebugTracingOptions {
    //     config: GethDefaultTracingOptions {
    //         disable_storage: Some(true),
    //         enable_memory: Some(true),
    //         enable_return_data: Some(true),
    //         ..Default::default()
    //     },
    //     tracer: Some(GethDebugTracerType::BuiltInTracer(
    //         GethDebugBuiltInTracerType::CallTracer,
    //     )),
    //     ..Default::default()
    // };
    // let result = provider
    //     .debug_trace_transaction(tx_hash, call_options)
    //     .await?;

    // println!("---CALL_TRACE---: {result:?}");

    Ok(())
}

// [{"address":"0x5fbdb2315678afecb367f032d93f642f64180aa3","topics":["0x9da6493a92039daf47d1f2d7a782299c5994c6323eb1e972f69c432089ec52bf","0x0000000000000000000000000000000000000000000000000000000000000001","0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266"],"data":"0x000000000000000000000000000000000000000000000000000009184e72a000","blockHash":"0x9cb14b1fdd3095c544caf0bab91d19ff7ee0cc875df143b13d72d39dd684a641","blockNumber":"0x5","blockTimestamp":"0x66bb2a72","transactionHash":"0x40ecdeee6e6b3ea914e6f38fb50233d88c76bafee20297368a3cea2675fd18e1","transactionIndex":"0x0","logIndex":"0x0","removed":false}]
