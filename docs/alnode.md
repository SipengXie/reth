# Altius Node Setup and Testing Guide

This document provides a comprehensive guide for compiling, configuring, running, and testing the Altius node. Altius is a custom Ethereum execution client. We will use the `aleth` command-line tool to interact with the node and submit block payloads for testing.

Please follow the steps below.

## Step 1: Preparation - Compiling the Altius Node

This initial step involves cloning the Altius node source code from GitHub, switching to the specified working branch, and then using Cargo (Rust's package manager and build tool) to compile the executable binary.

```bash
# Clone the source code repository
git clone https://github.com/Altius-Parallel-EVM/reth

# Navigate into the project directory
cd reth

# Check out the specified branch
git checkout <branch_name>

# Compile the altius-reth package in release mode
cargo build --release -p altius-reth
```

Upon successful compilation, you will find the `altius-reth` executable in the `./target/release/` directory.

## Step 2: Generate and Configure the JWT Secret

To ensure secure communication on the Engine API between the node and local clients like `aleth-cli`, we need to create a JWT secret. This secret must be identical for both the node and any client that connects to its authenticated port.

1.  **Generate the JWT Secret and Save to a File**

    Use `openssl` to generate a 32-byte hexadecimal string and save it to a file named `jwt.hex`. This file will be referenced when starting the node.

    ```bash
    # Generate the secret and redirect the output to the jwt.hex file
    # Ensure you replace /home/ubuntu/jwt.hex with your desired absolute path
    openssl rand -hex 32 > /home/ubuntu/jwt.hex
    ```

2.  **Configure the `aleth` Client**

    Next, we need to configure the `aleth` tool with the same secret to allow it to authenticate with the node.

    ```bash
    # Display the content of the secret file so you can copy it
    cat /home/ubuntu/jwt.hex

    # Run the aleth config command, which will open a text editor
    aleth config -e
    ```

    In the configuration file that opens, locate the `[eth_server]` section. Paste the hexadecimal string you copied from `jwt.hex` into the `authrpc_secret` field. **It is critical that the secret is exactly the same in both places.**

    Example configuration (replace the `authrpc_secret` value with your own):

    ```toml
    # Configuration settings for the Ethereum server (i.e. the node)
    [eth_server]
    host = 'localhost'
    rpc_port = 8545
    authrpc_port = 8551
    # Paste your generated hex string here
    authrpc_secret = 'your_generated_32_byte_hex_string_here'

    # Public RPC endpoints providing live chain data.
    [rpc_providers]
    ankr = 'https://rpc.ankr.com/eth'
    llama = 'https://eth.llamarpc.com'
    cf = 'https://cloudflare-eth.com'
    ```

    Save the changes and close the editor.

## Step 3: Starting the Altius Node

This step launches the compiled `altius-reth` executable. We configure its behavior using a combination of environment variables and command-line flags. It's recommended to run this in a separate terminal session or a background process manager like `screen` or `tmux`.

```bash
rm -rf /home/ubuntu/datadir  # Clean up the data directory if exists
export DATA_DIR=/home/ubuntu/datadir
export RUST_LOG=INFO
export ENABLE_PARALLEL=false
export ENABLE_SSA=false
export JWT_SECRET=/home/ubuntu/jwt.hex

# Define an alias for caching-related arguments for convenience
cache=--engine.caching-and-prewarming

# Start the node
# Note: This command should be executed from the root of the 'reth' project directory
./target/release/altius-reth node $cache \
    --datadir $DATA_DIR \
    --http --http.api all \
    --disable-discovery --trusted-only \
    --authrpc.jwtsecret=$JWT_SECRET \
    --chain altius \
    --engine.persistence-threshold 0 \
    --engine.memory-block-buffer-target 0 \
    --block-interval 5 \
    --prune.senderrecovery.full \
    --prune.transactionlookup.full \
    --prune.receipts.distance=10064 \
    --prune.accounthistory.distance=10064 \
    --prune.storagehistory.distance=10064
```

Once the node starts successfully, it will begin to output log messages to your terminal.

## Step 4: Testing the Node with aleth-cli

Important: The following commands must be executed in a new, separate terminal window. The terminal used in Step 3 is now occupied by the running Altius node and must be left open.

In this final step, we will use the `aleth` tool to submit pre-configured block payloads to the running node to verify that it is functioning correctly.

1.  **Unzip the Test Data**

    The test payload is located in a zip file within the project repository. You must first extract it.

    ```bash
    # Execute this from the root of the 'reth' project directory
    unzip examples/altius-reth/data/alitus-payload.zip -d examples/altius-reth/data/
    ```

    This will create a new directory named `payload` inside `examples/altius-reth/data/`.

2.  **Submit Blocks**

    Use the `aleth block submit-blocks` command to send a range of blocks (from block 1 to 4) from the extracted data directory to your running node.

    ```bash
    # The -d flag must point to the 'payload' directory you just unzipped
    aleth block submit-blocks -d examples/altius-reth/data/payload -f 1 -t 4
    ```

    A successful run will produce an output similar to the following:

    ```bash
    2025-09-05 09:07:19.001 | INFO     | Submitting 4 blocks... [from_block=1; to_block=4; payloads_dir=examples/altius-reth/data/payload]
    2025-09-05 09:07:19.063 | INFO     | Block #1 submitted. [hash=0x25fa2bd5899f51ab3955159cf6fe7093e6c118d463f3aa0057950bbfd3218205; sroot=0xe7454538b0d2a336119504f754cefdc9566a60c14d5fb855507e9cdf8be0dcf0; #txns=1000; t_blk_submit=     56ms; t_blk_commit=     2ms; t_total=     61ms; progress=1/4  25.00%]
    2025-09-05 09:07:19.068 | INFO     | Block #2 submitted. [hash=0x6895398a6019c543f0a0d463f27248efd5364cda00c883b732c07fe4ae25bff5; sroot=0xa497f2f0a0b53bd9a15d5b6579cc8addd9d6743b36a59d470beb0dd4b926d652; #txns=   1; t_blk_submit=      2ms; t_blk_commit=     1ms; t_total=      4ms; progress=2/4  50.00%]
    2025-09-05 09:07:19.072 | INFO     | Block #3 submitted. [hash=0xe4ae44a74debd5ecafa1a65978332c87fc6ab13ae0cd8f566f23772fdc959f1b; sroot=0xd3c65da67824a051b883a30717f9a687de2433ef6155f4d2e9380e5422a62337; #txns=   1; t_blk_submit=      2ms; t_blk_commit=     1ms; t_total=      3ms; progress=3/4  75.00%]
    2025-09-05 09:07:21.194 | INFO     | Block #4 submitted. [hash=0x9c42b5a73a688a72dd5d2f7b30d07415c3b368c70beb9f38aa36f85b515acdb1; sroot=0x2f9657e01f90f1636bac1e8739998f61c8eb44c567c8dd5a4e78d7c58e7706dd; #txns= 854; t_blk_submit= 2116ms; t_blk_commit=     2ms; t_total= 2118ms; progress=4/4 100.00%]
    2025-09-05 09:07:21.194 | SUCCESS  | Completed submission of 4 blocks. [head=(4, '0x9c42b5a73a688a72dd5d2f7b30d07415c3b368c70beb9f38aa36f85b515acdb1'); sroot=0x2f9657e01f90f1636bac1e8739998f61c8eb44c567c8dd5a4e78d7c58e7706dd; #blocks=4; #txns=1856; t_blk_submits=2.18s; t_blk_commits=0.01s; t_total=2.19s]
    ```

If the command completes without errors and you can see log messages on the node's terminal indicating that new blocks have been received and processed, your Altius node has been successfully deployed and is working correctly.

## Enabling Parallel Execution

The default instructions in this guide run the node in **serial execution mode**. This provides a baseline for performance. Altius node also supports two different parallel execution modes, which can be enabled via environment variables.

To test the parallel execution capabilities, stop the node (Ctrl+C in its terminal), change the `export` commands as described below, and then re-run the node command from Step 3 and the test command from Step 4.

The execution mode is controlled by the `ENABLE_PARALLEL` and `ENABLE_SSA` variables:

  * **Serial Execution (Default)**

      * This is the baseline mode.
      * `export ENABLE_PARALLEL=false`
      * `export ENABLE_SSA=false`

  * **Parallel Execution (OCCDA)**

      * This mode enables the parallel engine using Optimistic Concurrency Control with Deterministic Abort (OCCDA).
      * `export ENABLE_PARALLEL=true`
      * `export ENABLE_SSA=false`

  * **Parallel Execution (OCCDA + SSA)**

      * This mode adds Static Single Assignment (SSA) optimizations on top of the OCCDA parallel engine for potentially improved performance.
      * `export ENABLE_PARALLEL=true`
      * `export ENABLE_SSA=true`

## Understanding the Test Execution (Blocks 1-4)

The test script submits four pre-defined blocks (`PAYLOAD_B1.json` to `PAYLOAD_B4.json`) to the running node. Each block serves a specific purpose in this test scenario. The primary focus of the performance test is Block #4, which executes a high volume of transactions against the deployed smart contracts.

Here is a breakdown of what happens in each block:

  * **Block #1: Gas Funding**

      * **Transactions:** 1000
      * **Purpose:** This block contains a large number of simple value-transfer transactions. Its main function is to distribute gas (ETH) to various accounts that will be used to initiate transactions in the subsequent blocks. This is a preparatory step to ensure the necessary accounts are funded for the main test.

  * **Block #2: Deploy `MockKZGPrecompile` Contract**

      * **Transactions:** 1
      * **Purpose:** This block deploys the `MockKZGPrecompile` contract. The `BlobKZGVerifier` contract, which will be deployed next, depends on a KZG point verification precompile. For this test environment, instead of using the actual precompile address(`0x0a`) , we deploy a mock version. This mock contract simulates the precompile's behavior, returning `1` for a correctly formatted 192-byte input, which allows us to test the verifier logic in isolation.

  * **Block #3: Deploy `BlobKZGVerifier` Contract**

      * **Transactions:** 1
      * **Purpose:** This block deploys the main contract for this test: `BlobKZGVerifier`. During its deployment, it is linked to the address of the `MockKZGPrecompile` contract deployed in Block #2. This contract is designed for batch verification of KZG proofs and includes a function, `verifyBatchAndStress`, which is built to be computationally intensive for stress-testing purposes.

  * **Block #4: Execute Stress Test**

      * **Transactions:** 854
      * **Purpose:** This is the core of the test. This block is filled with transactions that call the `verifyBatchAndStress` function on the `BlobKZGVerifier` contract. Each transaction submits a batch of proofs, triggering the KZG verification logic (via the mock precompile) and subsequent CPU-intensive computations. As seen in the log output, this block takes significantly longer to process (`t_blk_submit= 2116ms`) compared to the others, demonstrating the computational load it places on the node. This effectively stress-tests the node's ability to handle complex, high-CPU smart contract executions.