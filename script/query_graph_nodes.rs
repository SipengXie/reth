#!/usr/bin/env rust-script
//! Query SSA Graph Nodes by Code Hash and Path Hash
//!
//! This script looks up a specific SSA graph entry from the cache
//! and outputs its graph nodes.
//!
//! Usage:
//!     cargo run --bin query_graph_nodes -- <code_hash> <path_hash>
//!
//! Arguments:
//!     code_hash - Code hash in hex format (with or without 0x prefix)
//!     path_hash - Path hash in hex format (with or without 0x prefix)
//!
//! Environment Variables:
//!     SSA_CACHE_PATH - Path to SSA cache file (default: ./ssa_cache.bin)

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <code_hash> <path_hash>", args[0]);
        eprintln!("\nArguments:");
        eprintln!("  code_hash - Code hash in hex format (with or without 0x prefix)");
        eprintln!("  path_hash - Path hash in hex format (with or without 0x prefix)");
        eprintln!("\nExample:");
        eprintln!("  {} 0x1234... 0x5678...", args[0]);
        std::process::exit(1);
    }

    let code_hash_str = &args[1];
    let path_hash_str = &args[2];

    // Set cache path if not already set
    if env::var("SSA_CACHE_PATH").is_err() {
        env::set_var("SSA_CACHE_PATH", "./ssa_cache.bin");
    }

    println!("=============================================================");
    println!("SSA Graph Nodes Query");
    println!("=============================================================\n");
    println!("Code Hash: {}", code_hash_str);
    println!("Path Hash: {}", path_hash_str);
    println!();

    // Parse hex strings to bytes
    let code_hash = parse_hex(code_hash_str)?;
    let path_hash = parse_hex(path_hash_str)?;

    println!("Code Hash (bytes): {} bytes", code_hash.len());
    println!("Path Hash (bytes): {} bytes", path_hash.len());
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

    // Search for the entry
    println!("Searching for matching entry...");

    let mut found = false;
    for entry in cache.iter() {
        let (path_key, artifacts) = (entry.key(), entry.value());

        // Check if this is the entry we're looking for
        // path_key should match the path_hash and code_hash
        // The exact matching logic depends on how path_key is structured
        // For now, let's try to match based on the serialized format

        let path_key_str = format!("{:?}", path_key);
        if path_key_str.contains(&format!("{:?}", code_hash)) ||
           path_key_str.contains(&format!("{:?}", path_hash)) {

            println!("Found potential match!");
            println!("Path Key: {:?}", path_key);
            println!();

            match &artifacts.data {
                altius_revm::ssa::SsaData::Graph(graph) => {
                    println!("Graph type: Already built");
                    println!("Number of nodes: {}", graph.nodes.len());
                    println!("\n=============================================================");
                    println!("GRAPH NODES");
                    println!("=============================================================\n");

                    // Output nodes in JSON format for easy parsing
                    let json_output = serde_json::to_string_pretty(&graph.nodes)?;
                    println!("{}", json_output);

                    found = true;
                }
                altius_revm::ssa::SsaData::Logs(_) => {
                    println!("Graph type: Logs (needs conversion)");
                    println!("Converting logs to graph...");

                    let artifacts_clone = artifacts.clone();
                    match artifacts_clone.ensure_graph() {
                        Ok(converted) => {
                            if let altius_revm::ssa::SsaData::Graph(graph) = &converted.data {
                                println!("✓ Conversion successful");
                                println!("Number of nodes: {}", graph.nodes.len());
                                println!("\n=============================================================");
                                println!("GRAPH NODES");
                                println!("=============================================================\n");

                                let json_output = serde_json::to_string_pretty(&graph.nodes)?;
                                println!("{}", json_output);

                                found = true;
                            }
                        }
                        Err(e) => {
                            eprintln!("✗ Failed to convert logs to graph: {}", e);
                            return Err(e.into());
                        }
                    }
                }
            }

            break;
        }
    }

    if !found {
        eprintln!("\n✗ No matching entry found for the given code_hash and path_hash");
        eprintln!("\nTip: Make sure the hashes are in the correct format and exist in the cache");
        std::process::exit(1);
    } else {
        println!("\n=============================================================");
        println!("✓ Query complete!");
        println!("=============================================================");
    }

    Ok(())
}

/// Parse hex string (with or without 0x prefix) to bytes
fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);

    if s.len() % 2 != 0 {
        return Err(format!("Hex string has odd length: {}", s.len()));
    }

    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte_str = &s[i..i + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => bytes.push(byte),
            Err(e) => return Err(format!("Failed to parse hex byte '{}': {}", byte_str, e)),
        }
    }

    Ok(bytes)
}
