use alloy::{
    primitives::b256,
    providers::{ext::DebugApi, ProviderBuilder},
    rpc::types::trace::geth::{
        GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingOptions,
        GethDefaultTracingOptions,
    },
    sol,
};

use clap::Parser;
use eyre::Result;
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

    let provider = ProviderBuilder::new().on_http(args.rpc_url);

    let tx_hash = b256!("ebbd7c26798e0de81b02f916cd97365754e9394d28d24a98e79974254ef8c32b");
    // let tx_hash = B256::from_str(&args.tx_hash)?;

    let default_options = GethDebugTracingOptions {
        tracer: Some(GethDebugTracerType::BuiltInTracer(
            GethDebugBuiltInTracerType::CallTracer,
        )),
        ..Default::default()
    };
    let result = provider
        .debug_trace_transaction(tx_hash, default_options)
        .await?;

    println!("DEFAULT_TRACE: {result:?}");

    let call_options = GethDebugTracingOptions {
        config: GethDefaultTracingOptions {
            disable_storage: Some(true),
            enable_memory: Some(false),
            ..Default::default()
        },
        tracer: Some(GethDebugTracerType::BuiltInTracer(
            GethDebugBuiltInTracerType::CallTracer,
        )),
        ..Default::default()
    };
    let result = provider
        .debug_trace_transaction(tx_hash, call_options)
        .await?;

    println!("CALL_TRACE: {result:?}");

    let js_options = GethDebugTracingOptions {
        tracer: Some(GethDebugTracerType::BuiltInTracer(
            GethDebugBuiltInTracerType::CallTracer,
        )),
        ..Default::default()
    };
    let result = provider
        .debug_trace_transaction(tx_hash, js_options)
        .await?;

    println!("JS_TRACER: {result:?}");
    Ok(())
}
