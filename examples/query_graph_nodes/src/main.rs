#!/usr/bin/env rust
//! Query SSA Graph Nodes by Code Hash and Path Hash
//!
//! This tool looks up a specific SSA graph entry from the cache
//! and outputs its graph nodes.
//!
//! Usage:
//!     cargo run --release --example query_graph_nodes -- <code_hash> <path_hash>
//!
//! Arguments:
//!     code_hash - Code hash in hex format (U256)
//!     path_hash - Path hash in hex format (u64)
//!
//! Environment Variables:
//!     SSA_CACHE_PATH - Path to SSA cache file (default: ./ssa_cache.bin)

use std::env;
use altius_revm::ssa::PathKey;
use revm_primitives::U256;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <code_hash> <path_hash>", args[0]);
        eprintln!("\nArguments:");
        eprintln!("  code_hash - Code hash in hex format (U256)");
        eprintln!("  path_hash - Path hash in hex format (u64)");
        eprintln!("\nExample:");
        eprintln!("  {} 0x652b853bbfb85b14c1cfde3a2e36296a7f32dfd18153842a5095184654af2ef 0x347c17d242025249", args[0]);
        std::process::exit(1);
    }

    let code_hash_str = &args[1];
    let path_hash_str = &args[2];

    // Set cache path if not already set
    if env::var("SSA_CACHE_PATH").is_err() {
        env::set_var("SSA_CACHE_PATH", "./ssa_cache");
    }

    println!("=============================================================");
    println!("SSA Graph Nodes Query");
    println!("=============================================================\n");
    println!("Code Hash: {}", code_hash_str);
    println!("Path Hash: {}", path_hash_str);
    println!();

    // Parse code_hash as U256 and path_hash as u64
    let code_hash = parse_u256(code_hash_str)?;
    let path_hash = parse_u64(path_hash_str)?;

    // Construct PathKey
    let path_key = PathKey {
        code_hash,
        path_hash,
    };

    println!("PathKey constructed successfully");
    println!();

    // Load cache
    println!("Loading SSA cache from: {}", env::var("SSA_CACHE_PATH")?);
    match altius_revm::ssa::global_cache::init_graph_cache() {
        Ok(_) => {
            println!("✓ Cache initialized successfully");
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize cache: {}", e);
            return Err(e.into());
        }
    }

    let cache = altius_revm::ssa::global_cache::get_cache();
    let total_entries = cache.len();
    println!("Total cache entries: {}\n", total_entries);

    if total_entries == 0 {
        eprintln!("Cache is empty. Nothing to query.");
        return Ok(());
    }

    // Query the cache directly using the path_key
    println!("Querying cache...");

    let store = cache.store();
    if let Some(entry) = store.get(&path_key) {
        println!("✓ Found entry!\n");

        let artifacts = entry.value();

        match &artifacts.data {
            altius_revm::ssa::SsaData::Graph(graph) => {
                println!("Number of nodes: {}", graph.nodes.len());
                println!("\n=============================================================");
                println!("GRAPH NODES");
                println!("=============================================================\n");

                println!("{:?}", graph.nodes);
            }
            altius_revm::ssa::SsaData::Logs(_) => {
                println!("Graph type: Logs (needs conversion)");
                println!("Converting logs to graph...");

                let artifacts_clone = artifacts.clone();
                match artifacts_clone.ensure_graph(cache.as_ref()) {
                    Ok(converted) => {
                        if let altius_revm::ssa::SsaData::Graph(graph) = &converted.data {
                            println!("✓ Conversion successful");
                            println!("Number of nodes: {}", graph.nodes.len());
                            println!("\n=============================================================");
                            println!("GRAPH NODES");
                            println!("=============================================================\n");

                            println!("{:?}", graph.nodes);
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to convert logs to graph: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }

        println!("\n=============================================================");
        println!("✓ Query complete!");
        println!("=============================================================");
    } else {
        eprintln!("\n✗ No entry found for the given code_hash and path_hash");
        eprintln!("\nSearched for:");
        eprintln!("  Code Hash: {}", code_hash_str);
        eprintln!("  Path Hash: {}", path_hash_str);
        eprintln!("\nTip: Make sure the hashes are in the correct format and exist in the cache");
        std::process::exit(1);
    }

    Ok(())
}

/// Parse hex string to U256
fn parse_u256(s: &str) -> Result<U256, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    U256::from_str_radix(s, 16)
        .map_err(|e| format!("Failed to parse U256 from '{}': {}", s, e))
}

/// Parse hex string to u64
fn parse_u64(s: &str) -> Result<u64, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16)
        .map_err(|e| format!("Failed to parse u64 from '{}': {}", s, e))
}
