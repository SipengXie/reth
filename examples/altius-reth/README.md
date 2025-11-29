# Altius-Reth Install and Integration Guide

This document outlines the complete strategy for integrating the Altius parallel EVM execution engine into the [Reth](https://github.com/paradigmxyz/reth) framework. The core advantage of this solution is its **non-invasive nature**. It leverages the extension interfaces provided by Reth to implement its functionality without modifying Reth's core codebase, ensuring a smooth and straightforward upgrade path for future upstream versions of Reth.

## Quick Start

### Pulling the source
Please use -b option to pull the correct branch as we don't release on main.

`git clone -b altius/1.3.12 https://github.com/Altius-Labs/altius-reth.git`

Note, if using ssh obviously you can also use `git@github.com:Altius-Labs/altius-reth.git`

### Building the Node

```bash
# From the project root
cargo build --release  -p altius-reth

# Or build with optimizations for production
cargo build --release  -p altius-reth --features jemalloc
```

### Running the Node

`altius-reth` supports all the same command-line arguments as the standard Reth node:

```bash
# Run with default settings (mainnet, full sync)
./target/release/altius-reth node

# Run on a testnet (e.g., Sepolia)
./target/release/altius-reth node --chain sepolia

# Run with custom data directory
./target/release/altius-reth node --datadir /path/to/custom/datadir

# Enable RPC endpoints
./target/release/altius-reth node --http --http.api eth,net,web3 --ws --ws.api eth,net,web3

# Run with debug logging
RUST_LOG=debug ./target/release/altius-reth node

# Show all available options
./target/release/altius-reth node --help
```

## Docker build
As of this writing, altius-revm is still a private repo, you will need to inject your github ssh keys into the docker build process.  Run the following command:

`DOCKER_BUILDKIT=1 docker build --ssh default  -t altius-reth:v1.3.12-local .`

Please name as you see fit.  Once you run this, the altius-reth image should be saved into the docker registry on your localhost and you can use it as per your requirements (docker run or docker-compose)

So for example, if one built using the image name "altius-reth:v1.3.12-local", the following would print command line help from reth:

`docker run  --name altius-reth:v1.3.12-local node  --help`

This would be the same as running `./target/release/altius-reth node --help`

At this point, the altius-reth container behaves the same as the reth docker container and can be used as a drop-in replacement.

### Key Features

- **Full Reth Compatibility**: Supports all Reth command-line arguments and configuration options
- **Parallel Execution**: Uses the Altius parallel EVM for improved transaction processing performance
- **Production Ready**: Includes proper error handling, logging, and signal handling
- **Easy Upgrades**: Independent binary that can be updated without modifying Reth source code

## 1. Core Components and Implementation Path

We have designed and implemented a complete set of executor components decoupled from Reth, which work together to bring Altius's parallel execution capabilities into the Reth ecosystem.

### 1.1. Integration on the Reth Side (`AltiusExecutorBuilder`)

This serves as the entry point for the entire solution. The `AltiusExecutorBuilder` encapsulates all necessary configuration and construction logic. Its primary responsibilities include:

-   **Configuration Initialization**: Creates and configures `AltiusEvmConfig`, our custom EVM configuration struct that bridges Reth and Altius.
-   **Executor Provider Construction**: Builds the `AltiusBlockExecutorProvider` based on the `AltiusEvmConfig`.
-   **Executor Instantiation**: Ultimately, the `AltiusBlockExecutorProvider` dynamically creates instances of `AltiusBlockExecutor` as needed by the Reth runtime. The `AltiusBlockExecutor` is the component that performs the actual parallel block processing.

This workflow adheres to Reth's design patterns, separating the specific execution logic (`AltiusBlockExecutor`) from its creation and configuration logic (`Provider` and `Builder`), which ensures clean and modular code.

### 1.2. EVM Layer Abstraction (`alloy-evm` & `ConfigureEvm`)

To enable Reth to understand and utilize the Altius EVM, we worked at the lower-level `alloy-evm` abstraction layer.

-   **Custom EVM Factory (`AltiusEVMFactory`)**: We implemented a custom `EvmFactory`, which serves as the primary entry point for the Altius EVM.
-   **Implementing the `ConfigureEvm` Trait**: We implemented the `ConfigureEvm` trait for our `AltiusEvmConfig`. This is one of the most critical extension points in the Reth framework, as it defines the core logic for configuring the EVM environment (`EvmEnv`), handling block context, and more. By implementing this trait, we successfully injected the `AltiusEVMFactory` into Reth's execution flow.

As noted in the Reth documentation you referenced, `ConfigureEvm` is the bridge connecting a custom EVM implementation to the upper layers of the Reth application, and our work has leveraged this design perfectly.

## 2. Resolving Key Technical Challenges

During the integration process, we addressed a series of technical challenges to ensure the robustness and correctness of the solution.

-   **Type Mismatches**: We precisely managed the versions and import paths for core types like `EvmEnv`, `CfgEnv`, `TxEnv`, and `SpecId`. By carefully adjusting `use` statements and generic constraints, we resolved compile-time type mismatch errors arising from dependency version conflicts.
-   **Thread Safety and Lifecycles**:
    -   To satisfy the `Send + Sync` requirements for parallel execution without modifying Reth's public `BlockExecutorProvider` interface, we introduced a `ThreadSafeDb` wrapper. This wrapper uses `unsafe` to assert the thread safety of the underlying `DB` type to the compiler, cleverly decoupling the implementation's constraints from the external interface.
    -   We addressed the `'static` lifetime requirement, a common challenge in parallel programming, by modifying our implementation to remove the `'static` dependency from the `run_parallel` function, allowing the executor to handle a broader range of database types.

## 3. Standalone Example Node (`altius-reth`)

To validate and demonstrate the effectiveness of the entire solution, we created a standalone binary, `altius-reth`.

This example serves as a complete Reth node that uses the Altius execution engine. Its significance lies in:
-   **Independence**: It is a self-contained binary, completely separate from the Reth repository, which works by importing `reth` and our `altius-reth` library as dependencies.
-   **Demonstration of Non-Invasive Integration**: It provides definitive proof of our integration strategy's success. We can build and run a fully functional Reth node with a custom parallel executor without modifying a single line of Reth's source code.
-   **Ease of Upgrades**: Because it is fully decoupled from Reth's core code, updating to a new version of Reth in the future simply requires updating the version number in `Cargo.toml` and addressing any minor API changes. This significantly reduces long-term maintenance overhead.

## Summary

This project successfully integrates the Altius parallel execution engine as a third-party module into Reth. By deeply understanding and utilizing Reth's `ConfigureEvm` abstraction layer, we have built a set of independent, non-invasive executor components. We have solved critical type system, thread safety, and lifecycle challenges during the process and have ultimately proven the viability and superiority of our approach with a standalone example node. 