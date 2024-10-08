use clap::Parser;
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct CliInput {
    #[arg(long, short, env = "TX_HASH")]
    pub tx_hash: String,
    #[arg(long, short, env = "RPC_URL")]
    pub rpc_url: Url,
    #[arg(long, short, env = "STARKNET_RPC_URL")]
    pub starknet_rpc_url: String,
}
