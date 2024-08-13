use clap::Parser;
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct CliInput {
    #[arg(long, short, env = "CONTRACT_ADDRESS")]
    pub contract_address: String,
    #[arg(long, short, env = "TX_HASH")]
    pub tx_hash: String,
    #[arg(long, short, env = "RPC_URL")]
    pub rpc_url: Url,
}
