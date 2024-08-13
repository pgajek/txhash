use alloy::{
    node_bindings::Anvil,
    // primitives::b256,
    primitives::{address, B256, U256},
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

use futures_util::stream::StreamExt;

use clap::Parser;
use eyre::Result;
use tx_event::CliInput;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc, bytecode = "6080604052348015600f57600080fd5b5061039c8061001f6000396000f3fe60806040526004361061004a5760003560e01c80632e1a7d4d1461004f57806361b8ce8c146100715780638c64ea4a1461009a5780639507d39a1461010b578063b6b55f25146101ae575b600080fd5b34801561005b57600080fd5b5061006f61006a366004610326565b6101c1565b005b34801561007d57600080fd5b5061008760005481565b6040519081526020015b60405180910390f35b3480156100a657600080fd5b506100e46100b5366004610326565b60016020819052600091825260409091208054918101546002909101546001600160a01b039092169160ff1683565b604080516001600160a01b0390941684526020840192909252151590820152606001610091565b34801561011757600080fd5b50610181610126366004610326565b6040805160608082018352600080835260208084018290529284018190529384526001808352938390208351918201845280546001600160a01b03168252938401549181019190915260029092015460ff1615159082015290565b6040805182516001600160a01b031681526020808401519082015291810151151590820152606001610091565b6100876101bc366004610326565b610282565b6000818152600160205260409020600281015460ff16156101f55760405163939e783760e01b815260040160405180910390fd5b80546001600160a01b0316331461021f5760405163b39ed90360e01b815260040160405180910390fd5b60028101805460ff191660019081179091558154908201546040516001600160a01b039092169184917f9da6493a92039daf47d1f2d7a782299c5994c6323eb1e972f69c432089ec52bf9161027691815260200190565b60405180910390a35050565b6000816000036102a557604051635e85ae7360e01b815260040160405180910390fd5b60008054604080516060810182523381526020808201878152828401868152858752600192839052938620925183546001600160a01b0319166001600160a01b03909116178355518282015591516002909101805460ff1916911515919091179055825491929091819061031a90849061033f565b90915550909392505050565b60006020828403121561033857600080fd5b5035919050565b8082018082111561036057634e487b7160e01b600052601160045260246000fd5b9291505056fea26469706673582212204c96afc9f418175c4c5d0a373113b4ecda4bac65f7474c25c2ed8ab71db8255c64736f6c634300081a0033")]
    contract VaultContract {
        struct Vault {
            address owner;
            uint256 amount;
            bool spent;
        }

        uint256 public nextId;
        mapping(uint256 => Vault) public vaults;

        event Withdraw(uint256 indexed id, address indexed owner, uint256 amount);

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

    println!("Deployed contract at: {}", contract.address());

    let withdraw_filter = contract.Withdraw_filter().watch().await?;

    let amount = U256::from(10);

    let deposit_call = contract.deposit(amount);
    let deposit_call1 = contract.deposit(amount);
    let deposit_call2 = contract.deposit(amount);
    let deposit_call3 = contract.deposit(amount);

    let amount = U256::from(1);

    let withdraw_call = contract.withdraw(amount);

    let dep = deposit_call.send().await?;
    println!("Receipt {:?}", dep);
    let dep1 = deposit_call1.send().await?;
    println!("Receipt {:?}", dep1);
    let dep2 = deposit_call2.send().await?;
    println!("Receipt {:?}", dep2);
    let dep3 = deposit_call3.send().await?;
    println!("Receipt {:?}", dep3);
    println!("Deposit");
    let with = withdraw_call.send().await?;
    println!("Receipt {:?}", with);
    println!("withdraw");

    withdraw_filter
        .into_stream()
        .take(1)
        .for_each(|log| async {
            match log {
                Ok((_event, log)) => {
                    println!("Received withdraw: {log:?}");
                }
                Err(e) => {
                    println!("Error: {e:?}");
                }
            }
        })
        .await;
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
