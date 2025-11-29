extern crate alloc;

use alloc::{borrow::Cow, sync::Arc};
use alloy_consensus::{BlockHeader, Header};
pub use alloy_evm::EthEvm;
use alloy_evm::{
    eth::EthBlockExecutionCtx, FromRecoveredTx, FromTxWithEncoded, IntoTxEnv,
};
use alloy_primitives::{Bytes, U256};
use core::{convert::Infallible, fmt::Debug};
use reth_chainspec::{ChainSpec, EthChainSpec, MAINNET};
use reth_ethereum_primitives::{Block, EthPrimitives, TransactionSigned};
use reth_evm::{ConfigureEvm, EvmEnv, EvmFactory, NextBlockEnvAttributes, TransactionEnv};
use reth_primitives_traits::{SealedBlock, SealedHeader};
use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    context_interface::block::BlobExcessGasAndPrice,
    primitives::hardfork::SpecId,
};
use alloy_eips::{eip1559::INITIAL_BASE_FEE, eip7840::BlobParams};
use reth_ethereum_forks::EthereumHardfork;
use reth_ethereum::evm::{RethReceiptBuilder, EthBlockAssembler, revm_spec_by_timestamp_and_block_number, revm_spec};
use alloy_altius_evm::block::{AltiusBlockExecutorFactory, AltiusEvmFactory, EnvProvider};
use revm::context_interface::result::HaltReason;

/// Configuration for the Altius Ethereum Virtual Machine (EVM).
/// 
/// This struct encapsulates the necessary components for configuring and running
/// the Altius EVM, including block execution and assembly capabilities. It provides
/// a high-level interface for setting up the EVM with specific chain configurations
/// and custom EVM factories.
/// 
/// # Type Parameters
/// 
/// * `EvmFactory` - The factory type used for creating EVM instances. Defaults to `AltiusEvmFactory`.
/// 
/// # Examples
/// 
/// ```rust
/// use altius::config::AltiusEvmConfig;
/// use reth_chainspec::MAINNET;
/// 
/// // Create a configuration for mainnet
/// let config = AltiusEvmConfig::mainnet();
/// 
/// // Create a configuration with custom chain spec
/// let config = AltiusEvmConfig::new(MAINNET.clone());
/// ```
#[derive(Debug, Clone)]
pub struct AltiusEvmConfig<EvmFactory = AltiusEvmFactory> {
    /// The block executor factory responsible for creating block executors.
    /// This factory handles the creation of executors that can process blocks
    /// using the configured EVM and receipt builder.
    pub executor_factory: AltiusBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, EvmFactory>,
    
    /// The Ethereum block assembler used for constructing new blocks.
    /// This component handles the assembly of transactions into blocks
    /// according to Ethereum protocol rules.
    pub block_assembler: EthBlockAssembler<ChainSpec>,
}

impl AltiusEvmConfig {
    /// Creates a new Altius EVM configuration with the given chain specification.
    /// 
    /// This is a convenience method that creates an Ethereum-compatible configuration
    /// using the default Altius EVM factory.
    /// 
    /// # Parameters
    /// 
    /// * `chain_spec` - The blockchain specification defining the network parameters
    /// 
    /// # Returns
    /// 
    /// A new `AltiusEvmConfig` instance configured for the specified chain
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self::ethereum(chain_spec)
    }

    /// Creates a new Ethereum-compatible EVM configuration.
    /// 
    /// This method sets up the configuration with Ethereum-specific parameters
    /// and the default Altius EVM factory.
    /// 
    /// # Parameters
    /// 
    /// * `chain_spec` - The Ethereum chain specification
    /// 
    /// # Returns
    /// 
    /// A configured `AltiusEvmConfig` instance ready for Ethereum block processing
    pub fn ethereum(chain_spec: Arc<ChainSpec>) -> Self {
        Self::new_with_evm_factory(chain_spec, AltiusEvmFactory::default())
    }

    /// Creates a new Ethereum EVM configuration specifically for the Ethereum mainnet.
    /// 
    /// This is a convenience method that uses the predefined mainnet chain specification.
    /// 
    /// # Returns
    /// 
    /// An `AltiusEvmConfig` instance configured for Ethereum mainnet
    pub fn mainnet() -> Self {
        Self::ethereum(MAINNET.clone())
    }
}

impl<EvmFactory> AltiusEvmConfig<EvmFactory>
 {
    /// Creates a new Altius EVM configuration with a custom EVM factory.
    /// 
    /// This method allows for maximum flexibility by accepting a custom EVM factory
    /// that can implement specialized behavior for transaction execution.
    /// 
    /// # Parameters
    /// 
    /// * `chain_spec` - The blockchain specification
    /// * `evm_factory` - The custom EVM factory instance
    /// 
    /// # Returns
    /// 
    /// A new `AltiusEvmConfig` configured with the provided factory
    pub fn new_with_evm_factory(chain_spec: Arc<ChainSpec>, evm_factory: EvmFactory) -> Self {
        Self {
            block_assembler: EthBlockAssembler::new(chain_spec.clone()),
            executor_factory: AltiusBlockExecutorFactory::new(
                RethReceiptBuilder::default(),
                chain_spec,
                evm_factory,
            ),
        }
    }

    /// Returns the chain specification associated with this configuration.
    /// 
    /// The chain specification contains all the network-specific parameters
    /// such as hard fork activation blocks, gas limits, and other protocol constants.
    /// 
    /// # Returns
    /// 
    /// A reference to the `ChainSpec` used by this configuration
    pub const fn chain_spec(&self) -> &Arc<ChainSpec> {
        self.executor_factory.spec()
    }

    /// Sets the extra data for block assembly.
    /// 
    /// Extra data is included in block headers and can contain arbitrary information
    /// such as client version, pool identification, or other metadata.
    /// 
    /// # Parameters
    /// 
    /// * `extra_data` - The extra data bytes to include in assembled blocks
    /// 
    /// # Returns
    /// 
    /// A modified configuration with the specified extra data
    pub fn with_extra_data(mut self, extra_data: Bytes) -> Self {
        self.block_assembler.extra_data = extra_data;
        self
    }
}

impl<EvmF> ConfigureEvm for AltiusEvmConfig<EvmF>
where
    EvmF: EvmFactory<
            Tx: TransactionEnv
                    + FromRecoveredTx<TransactionSigned> 
                    + FromTxWithEncoded<TransactionSigned>
                    + IntoTxEnv<TxEnv>,
            Spec = SpecId,
            HaltReason = HaltReason,
        > + Clone 
        + Debug
        + Send
        + Sync
        + Unpin
        + EnvProvider
        + 'static,
{
    type Primitives = EthPrimitives;
    type Error = Infallible;
    type NextBlockEnvCtx = NextBlockEnvAttributes;
    type BlockExecutorFactory = AltiusBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, EvmF>;
    type BlockAssembler = EthBlockAssembler<ChainSpec>;

    /// Returns a reference to the block executor factory.
    /// 
    /// The executor factory is responsible for creating block executors that can
    /// process transactions within blocks according to the configured EVM rules.
    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        &self.executor_factory
    }

    /// Returns a reference to the block assembler.
    /// 
    /// The block assembler is used to construct new blocks from pending transactions,
    /// handling all the necessary validation and ordering logic.
    fn block_assembler(&self) -> &Self::BlockAssembler {
        &self.block_assembler
    }

    /// Creates an EVM environment configuration for a given block header.
    /// 
    /// This method configures the EVM execution environment based on the block header,
    /// setting up the proper hard fork specification, gas parameters, and other
    /// block-specific execution context.
    /// 
    /// # Parameters
    /// 
    /// * `header` - The block header to create the environment for
    /// 
    /// # Returns
    /// 
    /// An `EvmEnv` configured for executing transactions in the specified block
    fn evm_env(&self, header: &Header) -> EvmEnv {
        let spec = revm_spec(self.chain_spec(), header);

        // Configure EVM environment based on parent block
        let cfg_env = CfgEnv::new().with_chain_id(self.chain_spec().chain().id()).with_spec(spec);

        // Derive the EIP-4844 blob fees from the header's `excess_blob_gas` and the current
        // blob parameters for dynamic blob pricing
        let blob_excess_gas_and_price = header
            .excess_blob_gas
            .zip(self.chain_spec().blob_params_at_timestamp(header.timestamp))
            .map(|(excess_blob_gas, params)| {
                let blob_gasprice = params.calc_blob_fee(excess_blob_gas);
                BlobExcessGasAndPrice { excess_blob_gas, blob_gasprice }
            });

        let block_env = BlockEnv {
            number: header.number(),
            beneficiary: header.beneficiary(),
            timestamp: header.timestamp(),
            difficulty: if spec >= SpecId::MERGE { U256::ZERO } else { header.difficulty() },
            prevrandao: if spec >= SpecId::MERGE { header.mix_hash() } else { None },
            gas_limit: header.gas_limit(),
            basefee: header.base_fee_per_gas().unwrap_or_default(),
            blob_excess_gas_and_price,
        };

        EvmEnv { cfg_env, block_env }
    }

    /// Creates an EVM environment for the next block based on parent block and attributes.
    /// 
    /// This method calculates the appropriate execution environment for a new block
    /// being assembled, taking into account hard fork transitions, gas limit adjustments,
    /// and fee calculations based on the parent block's state.
    /// 
    /// # Parameters
    /// 
    /// * `parent` - The parent block header
    /// * `attributes` - The attributes for the next block being assembled
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the configured `EvmEnv` for the next block
    fn next_evm_env(
        &self,
        parent: &Header,
        attributes: &NextBlockEnvAttributes,
    ) -> Result<EvmEnv, Self::Error> {
        // Ensure we're not missing any timestamp-based hard forks
        let spec_id = revm_spec_by_timestamp_and_block_number(
            self.chain_spec(),
            attributes.timestamp,
            parent.number() + 1,
        );

        // Configure EVM environment based on parent block
        let cfg = CfgEnv::new().with_chain_id(self.chain_spec().chain().id()).with_spec(spec_id);

        let blob_params = self.chain_spec().blob_params_at_timestamp(attributes.timestamp);
        // If the parent block did not have excess blob gas (i.e., it was pre-Cancun), but it is
        // Cancun now, we need to set the excess blob gas to the default value (0)
        let blob_excess_gas_and_price = parent
            .maybe_next_block_excess_blob_gas(blob_params)
            .or_else(|| (spec_id == SpecId::CANCUN).then_some(0))
            .map(|excess_blob_gas| {
                let blob_gasprice =
                    blob_params.unwrap_or_else(BlobParams::cancun).calc_blob_fee(excess_blob_gas);
                BlobExcessGasAndPrice { excess_blob_gas, blob_gasprice }
            });

        let mut basefee = parent.next_block_base_fee(
            self.chain_spec().base_fee_params_at_timestamp(attributes.timestamp),
        );

        let mut gas_limit = attributes.gas_limit;

        // If we are on the London fork boundary, we need to multiply the parent's gas limit by the
        // elasticity multiplier to get the new gas limit
        if self.chain_spec().fork(EthereumHardfork::London).transitions_at_block(parent.number + 1)
        {
            let elasticity_multiplier = self
                .chain_spec()
                .base_fee_params_at_timestamp(attributes.timestamp)
                .elasticity_multiplier;

            // Multiply the gas limit by the elasticity multiplier
            gas_limit *= elasticity_multiplier as u64;

            // Set the base fee to the initial base fee from the EIP-1559 specification
            basefee = Some(INITIAL_BASE_FEE)
        }

        let block_env = BlockEnv {
            number: parent.number + 1,
            beneficiary: attributes.suggested_fee_recipient,
            timestamp: attributes.timestamp,
            difficulty: U256::ZERO,
            prevrandao: Some(attributes.prev_randao),
            gas_limit,
            // Calculate base fee based on parent block's gas usage
            basefee: basefee.unwrap_or_default(),
            // Calculate excess gas based on parent block's blob gas usage
            blob_excess_gas_and_price,
        };

        Ok((cfg, block_env).into())
    }

    /// Creates an execution context for a specific sealed block.
    /// 
    /// This method extracts the necessary context information from a sealed block
    /// to enable proper execution of its transactions, including parent block hash,
    /// ommers (uncle blocks), and withdrawals.
    /// 
    /// # Parameters
    /// 
    /// * `block` - The sealed block to create context for
    /// 
    /// # Returns
    /// 
    /// An `EthBlockExecutionCtx` containing the execution context for the block
    fn context_for_block<'a>(&self, block: &'a SealedBlock<Block>) -> EthBlockExecutionCtx<'a> {
        EthBlockExecutionCtx {
            parent_hash: block.header().parent_hash,
            parent_beacon_block_root: block.header().parent_beacon_block_root,
            ommers: &block.body().ommers,
            withdrawals: block.body().withdrawals.as_ref().map(Cow::Borrowed),
        }
    }

    /// Creates an execution context for the next block based on parent header and attributes.
    /// 
    /// This method prepares the execution context for a new block being assembled,
    /// setting up the necessary references to parent block information and
    /// proposed block attributes.
    /// 
    /// # Parameters
    /// 
    /// * `parent` - The sealed header of the parent block
    /// * `attributes` - The attributes for the next block
    /// 
    /// # Returns
    /// 
    /// An `EthBlockExecutionCtx` for executing the next block
    fn context_for_next_block(
        &self,
        parent: &SealedHeader,
        attributes: Self::NextBlockEnvCtx,
    ) -> EthBlockExecutionCtx<'_> {
        EthBlockExecutionCtx {
            parent_hash: parent.hash(),
            parent_beacon_block_root: attributes.parent_beacon_block_root,
            ommers: &[],
            withdrawals: attributes.withdrawals.map(Cow::Owned),
        }
    }
}