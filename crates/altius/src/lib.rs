//! # Altius: A Custom High-Performance Node Implementation for Reth
//!
//! The Altius module provides a specialized node implementation for the Reth Ethereum client,
//! designed to optimize block execution performance through parallel transaction processing
//! and custom EVM configurations.
//!
//! ## Key Features
//!
//! - **Parallel Block Execution**: Utilizes parallel processing strategies for improved throughput
//! - **Custom EVM Configuration**: Provides flexible EVM setup with specialized execution environments
//! - **Modular Architecture**: Separates concerns between execution strategies, database management, and block assembly
//! - **State Management**: Efficient state tracking and management during block execution
//!
//! ## Main Components
//!
//! - [`AltiusExecutor`]: The core block executor implementing parallel execution strategies
//! - [`AltiusBlockExecutorProvider`]: Provider for creating configured block executors
//! - [`config::AltiusEvmConfig`]: EVM configuration management
//!
//! ## Example Usage
//!
//! ```rust
//! use altius::{AltiusBlockExecutorProvider, config::AltiusEvmConfig};
//! use reth_chainspec::MAINNET;
//!
//! // Create a configuration for mainnet
//! let config = AltiusEvmConfig::mainnet();
//! 
//! // Create an executor provider
//! let provider = AltiusBlockExecutorProvider::new(config);
//! ```

use alloy_evm::FromRecoveredTx;
use reth_evm::{
    execute::{BlockExecutionError, BlockExecutorFactory, Executor},
    ConfigureEvm,
    Database,
    EvmFactory,
    OnStateHook,
};
use reth_primitives_traits::{
    NodePrimitives,
    RecoveredBlock,
};
use revm::{
    database::{State, states::bundle_state::BundleRetention},
    context::TxEnv,
    primitives::hardfork::SpecId
};
use reth_evm::execute::{BlockExecutorProvider, BlockExecutor};
use core::fmt::Debug;
use reth_execution_types::BlockExecutionResult;
use reth_db::mdbx::tx_pool;

/// Altius EVM configuration and setup utilities.
///
/// This module contains the configuration structures and methods needed to set up
/// the Altius EVM with custom parameters, chain specifications, and execution factories.
pub mod config;

/// A high-performance parallel block executor for the Altius implementation.
///
/// The `AltiusExecutor` is the core component responsible for executing blocks
/// within the Altius node. It leverages parallel execution strategies to maximize
/// throughput while maintaining state consistency and transaction ordering.
///
/// # Type Parameters
///
/// * `F` - The strategy factory type that provides block execution strategies
/// * `DB` - The database type implementing the `Database` trait for state storage
///
/// # Key Features
///
/// - **Parallel Execution**: Executes transactions within blocks in parallel when possible
/// - **State Management**: Maintains consistent state through the execution process
/// - **Hook Support**: Allows for custom state monitoring during execution
/// - **Error Handling**: Comprehensive error handling for execution failures
///
/// # Examples
///
/// ```rust
/// use altius::AltiusExecutor;
/// use reth_evm::Database;
/// 
/// // Create an executor with a strategy factory and database
/// let executor = AltiusExecutor::new(strategy_factory, database);
/// ```
pub struct AltiusExecutor<F, DB: Database> {
    /// The strategy factory responsible for creating block execution strategies.
    /// This factory determines how blocks are processed and can implement
    /// various optimization techniques such as parallel execution.
    pub(crate) strategy_factory: F,
    
    /// The database state manager that handles state reads, writes, and caching.
    /// This maintains the current state of the blockchain and manages state
    /// transitions during block execution.
    pub(crate) db: State<DB>,
}

impl<F: Debug, DB: Database> Debug for AltiusExecutor<F, DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AltiusExecutor")
            .field("strategy_factory", &self.strategy_factory)
            .field("db", &"State<DB>")
            .finish()
    }
}

impl<F, DB: Database> AltiusExecutor<F, DB> {
    /// Creates a new `AltiusExecutor` with the specified strategy factory and database.
    ///
    /// This constructor initializes the executor with a state management layer
    /// configured for optimal performance, including bundle updates and state preservation.
    ///
    /// # Parameters
    ///
    /// * `strategy_factory` - The factory that will create execution strategies for blocks
    /// * `db` - The underlying database for state storage and retrieval
    ///
    /// # Returns
    ///
    /// A new `AltiusExecutor` instance ready for block execution
    ///
    /// # State Configuration
    ///
    /// The database is configured with:
    /// - Bundle updates enabled for efficient state batching
    /// - State clearing disabled to preserve intermediate states
    /// - Optimized caching for high-throughput scenarios
    pub fn new(strategy_factory: F, db: DB) -> Self {
        let db = State::builder().with_database(db).with_bundle_update().without_state_clear().build();
        Self { strategy_factory, db }
    }
}

impl<F, DB> Executor<DB> for AltiusExecutor<F, DB>
where
    F: ConfigureEvm,
    <F::BlockExecutorFactory as BlockExecutorFactory>::EvmFactory: EvmFactory<Tx = TxEnv, Spec = SpecId>,
    TxEnv: FromRecoveredTx<<<F as ConfigureEvm>::Primitives as NodePrimitives>::SignedTx>,
    DB: Database,
{
    type Primitives = F::Primitives;
    type Error = BlockExecutionError;
 
    /// Executes a single block and returns the execution result.
    ///
    /// This method orchestrates the complete execution of a block, including:
    /// 1. Setting up the execution environment using the strategy factory
    /// 2. Processing all transactions within the block using parallel execution
    /// 3. Generating receipts and calculating state changes
    ///
    /// # Parameters
    ///
    /// * `block` - The recovered block containing all transaction data
    ///
    /// # Returns
    ///
    /// A `Result` containing the block execution result with receipts and state changes,
    /// or a `BlockExecutionError` if execution fails
    ///
    /// # Execution Flow
    ///
    /// 1. **Strategy Creation**: Creates a block execution strategy tailored to the specific block
    /// 2. **Parallel Execution**: Executes transactions in parallel while maintaining consistency
    /// 3. **Result Aggregation**: Collects and validates all execution results
    fn execute_one(
        &mut self,
        block: &RecoveredBlock<<Self::Primitives as NodePrimitives>::Block>,
    ) -> Result<BlockExecutionResult<<Self::Primitives as NodePrimitives>::Receipt>, Self::Error>
    {
        // Step 1: Create the inner block executor using the strategy factory
        // This sets up the basic execution environment for the block
        let strategy = self.strategy_factory.executor_for_block(&mut self.db, block);

        
        // Step 2: Execute all transactions in the block using parallel execution
        // The execution strategy handles transaction ordering and parallel processing
        let result = strategy.execute_block(block.transactions_recovered());

        // Note: Post-execution changes and finalization are handled within the strategy
        // This includes state root calculation and receipt generation
        let _ = tx_pool::global_tx_manager().reset_tx();

        self.db.merge_transitions(BundleRetention::Reverts);

        result
    }

    /// Executes a single block with a custom state monitoring hook.
    ///
    /// This method provides the same functionality as `execute_one` but allows for
    /// real-time monitoring of state changes during execution through a custom hook.
    /// This is particularly useful for debugging, analytics, or custom validation logic.
    ///
    /// # Parameters
    ///
    /// * `block` - The recovered block to execute
    /// * `state_hook` - A custom hook that will be called during state changes
    ///
    /// # Returns
    ///
    /// A `Result` containing the block execution result with receipts and state changes,
    /// or a `BlockExecutionError` if execution fails
    ///
    /// # State Hook Functionality
    ///
    /// The state hook allows monitoring of:
    /// - Account state changes
    /// - Storage modifications
    /// - Balance updates
    /// - Contract deployments and destructions
    ///
    /// # Type Parameters
    ///
    /// * `H` - The hook type implementing `OnStateHook` for state monitoring
    #[tracing::instrument(skip(self, block, state_hook))]
    fn execute_one_with_state_hook<H>(
        &mut self,
        block: &RecoveredBlock<<Self::Primitives as NodePrimitives>::Block>,
        state_hook: H,
    ) -> Result<BlockExecutionResult<<Self::Primitives as NodePrimitives>::Receipt>, Self::Error>
    where
        H: OnStateHook + 'static,
    {
        // Step 1: Create the inner block executor with state hook attached
        // The state hook will be called during execution to monitor state changes
        let strategy = self
            .strategy_factory
            .executor_for_block(&mut self.db, block)
            .with_state_hook(Some(Box::new(state_hook)));

        // Step 2: Execute all transactions in parallel with state hook monitoring
        // The state hook will be invoked during the parallel execution process
        let result = strategy.execute_block(block.transactions_recovered());

        // Note: The state hook provides real-time visibility into state changes
        // without affecting the execution performance significantly
        let _ = tx_pool::global_tx_manager().reset_tx();

        self.db.merge_transitions(BundleRetention::Reverts);

        result
    }

    /// Consumes the executor and returns the underlying database state.
    ///
    /// This method is useful for extracting the final state after block execution
    /// or for transferring state management to another component.
    ///
    /// # Returns
    ///
    /// The `State<DB>` containing all accumulated state changes
    fn into_state(self) -> State<DB> {
        self.db
    }

    /// Returns a size hint for the current state in the executor.
    ///
    /// This provides an estimate of the memory usage and can be used for
    /// performance monitoring and resource management decisions.
    ///
    /// # Returns
    ///
    /// A size hint in bytes representing the approximate memory usage
    fn size_hint(&self) -> usize {
        self.db.bundle_state.size_hint()
    }
}

/// A provider for creating Altius block executors with consistent configuration.
///
/// The `AltiusBlockExecutorProvider` serves as a factory for creating `AltiusExecutor`
/// instances with a consistent strategy factory configuration. This ensures that all
/// executors created by this provider share the same execution characteristics and
/// optimization settings.
///
/// # Type Parameters
///
/// * `F` - The strategy factory type that defines the execution behavior
///
/// # Use Cases
///
/// - **Node Operation**: Creating executors for different database instances
/// - **Testing**: Providing consistent executor configurations across test scenarios  
/// - **Performance Tuning**: Centralized management of execution strategies
/// - **Scaling**: Creating multiple executors for parallel block processing
///
/// # Examples
///
/// ```rust
/// use altius::{AltiusBlockExecutorProvider, config::AltiusEvmConfig};
/// 
/// let config = AltiusEvmConfig::mainnet();
/// let provider = AltiusBlockExecutorProvider::new(config);
/// 
/// // Create executors for different databases
/// let executor1 = provider.executor(database1);
/// let executor2 = provider.executor(database2);
/// ```
#[derive(Debug, Clone)]
pub struct AltiusBlockExecutorProvider<F> {
    /// The strategy factory used to configure all executors created by this provider.
    /// This factory defines the execution behavior, optimization strategies, and
    /// EVM configuration that will be applied to all blocks processed by executors
    /// created from this provider.
    strategy_factory: F,
}

impl<F> AltiusBlockExecutorProvider<F> {
    /// Creates a new `AltiusBlockExecutorProvider` with the specified strategy factory.
    ///
    /// This constructor allows for the creation of a provider with a custom strategy
    /// factory that defines how blocks will be executed across all created executors.
    ///
    /// # Parameters
    ///
    /// * `strategy_factory` - The factory that will be used to configure block execution
    ///
    /// # Returns
    ///
    /// A new provider instance ready to create executors
    ///
    /// # Design Notes
    ///
    /// The provider uses a const constructor to ensure minimal overhead when creating
    /// executor instances, making it suitable for high-frequency executor creation.
    pub const fn new(strategy_factory: F) -> Self {
        Self { strategy_factory }
    }
}

impl<F> BlockExecutorProvider for AltiusBlockExecutorProvider<F>
where
    F: ConfigureEvm + 'static,
    <F::BlockExecutorFactory as BlockExecutorFactory>::EvmFactory: EvmFactory<Tx = TxEnv, Spec = SpecId>,
    TxEnv: FromRecoveredTx<<<F as ConfigureEvm>::Primitives as NodePrimitives>::SignedTx>,
{
    type Primitives = F::Primitives;
    type Executor<DB: Database> = AltiusExecutor<F, DB>;

    /// Creates a new block executor with the specified database.
    ///
    /// This method instantiates a new `AltiusExecutor` configured with the provider's
    /// strategy factory and the supplied database. Each executor maintains its own
    /// state management but shares the execution strategy configuration.
    ///
    /// # Parameters
    ///
    /// * `db` - The database instance to be used for state storage and retrieval
    ///
    /// # Returns
    ///
    /// A new `AltiusExecutor` ready for block execution
    ///
    /// # Performance Considerations
    ///
    /// - Each executor maintains independent state management
    /// - Database access patterns are optimized for high-throughput scenarios
    /// - Memory usage scales with the size of state changes during execution
    ///
    /// # Type Parameters
    ///
    /// * `DB` - The database type implementing the `Database` trait
    fn executor<DB>(&self, db: DB) -> Self::Executor<DB>
    where
        DB: Database,
    {
        AltiusExecutor::new(self.strategy_factory.clone(), db)
    }
} 

