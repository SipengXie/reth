#![allow(missing_docs)]
#![warn(unused_crate_dependencies)]

#[global_allocator]
static ALLOC: reth_cli_util::allocator::Allocator = reth_cli_util::allocator::new_allocator();

use clap::Parser;
use reth::{
    args::RessArgs,
    cli::Cli,
    ress::install_ress_subprotocol,
    builder::{
        components::{BasicPayloadServiceBuilder, ExecutorBuilder, PayloadBuilderBuilder},
        BuilderContext,
    },
    payload::{EthBuiltPayload, EthPayloadBuilderAttributes},
    rpc::types::engine::PayloadAttributes,
};
use reth_ethereum::{
    chainspec::ChainSpec,
    node::api::{FullNodeTypes, NodeTypes, PayloadTypes},
    pool::{PoolTransaction, TransactionPool},
    EthPrimitives, TransactionSigned,
};
use reth_ethereum_cli::chainspec::EthereumChainSpecParser;
use reth_node_builder::{NodeHandle, Node, PayloadBuilderConfig, components::ComponentsBuilder, NodeComponentsBuilder, NodeAdapter};
use reth_node_ethereum::{node::{EthereumPoolBuilder, EthereumNetworkBuilder, EthereumConsensusBuilder, EthereumAddOns}};
use reth_ethereum_payload_builder::{EthereumPayloadBuilder, EthereumBuilderConfig};
use reth_evm_altius::{config::AltiusEvmConfig, AltiusBlockExecutorProvider};
use reth_trie_db::MerklePatriciaTrie;
use reth_ethereum_engine_primitives::EthEngineTypes;
use reth_provider::EthStorage;
use altius_revm::ssa::global_cache;
use tracing::info;  

use alloy_rpc_types_eth as _;
use reth_ethereum_primitives as _;
use reth_node_api as _;
use tokio as _;

use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::prelude::*;

/// Builds a regular ethereum block executor that uses the custom Altius executor.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct AltiusExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for AltiusExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = AltiusEvmConfig;
    type Executor = AltiusBlockExecutorProvider<Self::EVM>;

    async fn build_evm(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<(Self::EVM, Self::Executor)> {
        let evm_config = AltiusEvmConfig::new(ctx.chain_spec())
            .with_extra_data(ctx.payload_builder_config().extra_data_bytes());
        Ok((evm_config.clone(), AltiusBlockExecutorProvider::new(evm_config)))
    }
}

/// Builds a payload builder that uses the custom Altius EVM.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct AltiusPayloadBuilder;

impl<Types, Node, Pool> PayloadBuilderBuilder<Node, Pool> for AltiusPayloadBuilder
where
    Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>,
    Node: FullNodeTypes<Types = Types>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TransactionSigned>>
        + Unpin
        + 'static,
    Types::Payload: PayloadTypes<
        BuiltPayload = EthBuiltPayload,
        PayloadAttributes = PayloadAttributes,
        PayloadBuilderAttributes = EthPayloadBuilderAttributes,
    >,
{
    type PayloadBuilder = reth_ethereum_payload_builder::EthereumPayloadBuilder<
        Pool,
        Node::Provider,
        AltiusEvmConfig,
    >;

    async fn build_payload_builder(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<Self::PayloadBuilder> {
        let evm_config = AltiusEvmConfig::new(ctx.chain_spec())
            .with_extra_data(ctx.payload_builder_config().extra_data_bytes());
        Ok(EthereumPayloadBuilder::new(ctx.provider().clone(), pool, evm_config, EthereumBuilderConfig::default()))
    }
}

/// Custom Altius node type that uses the Altius executor.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct AltiusNode;

impl NodeTypes for AltiusNode {
    type Primitives = EthPrimitives;
    type ChainSpec = ChainSpec;
    type StateCommitment = MerklePatriciaTrie;
    type Storage = EthStorage;
    type Payload = EthEngineTypes;
}

impl<N> Node<N> for AltiusNode
where
    N: FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        EthereumPoolBuilder,
        BasicPayloadServiceBuilder<AltiusPayloadBuilder>,
        EthereumNetworkBuilder,
        AltiusExecutorBuilder,
        EthereumConsensusBuilder,
    >;

    type AddOns = EthereumAddOns<
        NodeAdapter<N, <Self::ComponentsBuilder as NodeComponentsBuilder<N>>::Components>,
    >;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        ComponentsBuilder::default()
            .node_types::<N>()
            .pool(EthereumPoolBuilder::default())
            .payload(BasicPayloadServiceBuilder::new(AltiusPayloadBuilder::default()))
            .network(EthereumNetworkBuilder::default())
            .executor(AltiusExecutorBuilder::default())
            .consensus(EthereumConsensusBuilder::default())
    }

    fn add_ons(&self) -> Self::AddOns {
        EthereumAddOns::default()
    }
}

fn main() {
    // Configure Chrome tracing，specify the output file  
    // let (chrome_layer, guard) = ChromeLayerBuilder::new()
    //     .file("altius_node_trace.json")  // specify the file name
    //     .build();
    // tracing_subscriber::registry().with(chrome_layer).init();
    
    println!("Chrome tracing enabled - output will be saved to: altius_node_trace.json");
    println!("After program exits, open chrome://tracing in your browser to view the trace");
    
    reth_cli_util::sigsegv_handler::install();
    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    let is_ssa = std::env::var("ENABLE_SSA")
    .unwrap_or_else(|_| "false".to_string())
    .parse::<bool>()
    .unwrap_or(false);
    let is_collector = std::env::var("ENABLE_COLLECTOR")
    .unwrap_or_else(|_| "false".to_string())
    .parse::<bool>()
    .unwrap_or(false);
    let use_cache = is_ssa || is_collector;

    if use_cache {
        let _ = global_cache::init_graph_cache();
    }
    

    if let Err(err) =
        Cli::<EthereumChainSpecParser, RessArgs>::parse().run(async move |builder, ress_args| {
            info!(target: "reth::cli", "Launching Altius node with parallel execution");
            let NodeHandle { node, node_exit_future } =
                builder.node(AltiusNode::default()).launch().await?;

            // Install ress subprotocol if enabled.
            if ress_args.enabled {
                install_ress_subprotocol(
                    ress_args,
                    node.provider,
                    node.block_executor,
                    node.network,
                    node.task_executor,
                    node.add_ons_handle.engine_events.new_listener(),
                )?;
            }

            info!(target: "reth::cli", "Altius node started successfully");
            node_exit_future.await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
    
    // Auto-save SSA cache if enabled
    if use_cache {
        if let Err(_e) = altius_revm::ssa::global_cache::save_cache(){
            println!("Failed to save SSA cache");
        } else {
            println!("Auto-saved SSA cache");
        }
    }        

    println!("Program finished - trace file should be available at: altius_node_trace.json");
} 