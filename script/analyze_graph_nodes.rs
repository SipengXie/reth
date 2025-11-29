#!/usr/bin/env rust-script
//! Analyze SSA Graph Nodes Distribution
//!
//! This script loads the SSA cache and analyzes the distribution of node counts
//! across all cached SSA graphs.
//!
//! Usage:
//!     cargo run --bin analyze_graph_nodes
//!
//! Environment Variables:
//!     SSA_CACHE_PATH - Path to SSA cache file (default: ./ssa_cache.bin)

use std::collections::HashMap;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set cache path if not already set
    if env::var("SSA_CACHE_PATH").is_err() {
        env::set_var("SSA_CACHE_PATH", "./ssa_cache.bin");
    }

    println!("=============================================================");
    println!("SSA Graph Nodes Distribution Analysis");
    println!("=============================================================\n");

    // Load cache
    println!("Loading SSA cache from: {}", env::var("SSA_CACHE_PATH")?);
    let cache = match altius_revm::ssa::global_cache::init_graph_cache() {
        Ok(_) => {
            println!("✓ Cache initialized successfully");
            altius_revm::ssa::global_cache::get_cache()
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize cache: {}", e);
            println!("\nUsing empty cache...");
            altius_revm::ssa::global_cache::get_cache()
        }
    };

    let total_entries = cache.len();
    println!("Total cache entries: {}\n", total_entries);

    if total_entries == 0 {
        println!("Cache is empty. Nothing to analyze.");
        return Ok(());
    }

    // Statistics collectors
    let mut node_counts: Vec<usize> = Vec::new();
    let mut distribution: HashMap<usize, usize> = HashMap::new();
    let mut logs_count = 0;
    let mut graphs_count = 0;
    let mut conversion_failures = 0;

    println!("Analyzing graphs...");

    // Iterate over all cache entries
    for entry in cache.iter() {
        let (path_key, artifacts) = (entry.key(), entry.value());

        match &artifacts.data {
            altius_revm::ssa::SsaData::Graph(graph) => {
                // Already a graph
                graphs_count += 1;
                let node_count = graph.nodes.len();
                node_counts.push(node_count);
                *distribution.entry(node_count).or_insert(0) += 1;
            }
            altius_revm::ssa::SsaData::Logs(_) => {
                // Need to convert logs to graph
                logs_count += 1;

                // Clone artifacts to convert
                let artifacts_clone = artifacts.clone();
                match artifacts_clone.ensure_graph() {
                    Ok(converted) => {
                        if let altius_revm::ssa::SsaData::Graph(graph) = &converted.data {
                            let node_count = graph.nodes.len();
                            node_counts.push(node_count);
                            *distribution.entry(node_count).or_insert(0) += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to convert logs to graph for path {:?}: {}", path_key, e);
                        conversion_failures += 1;
                    }
                }
            }
        }
    }

    // Print summary statistics
    println!("\n=============================================================");
    println!("SUMMARY STATISTICS");
    println!("=============================================================\n");

    println!("Data types:");
    println!("  Graphs (already built):  {}", graphs_count);
    println!("  Logs (converted):        {}", logs_count);
    println!("  Conversion failures:     {}", conversion_failures);
    println!("  Total analyzed:          {}\n", node_counts.len());

    if node_counts.is_empty() {
        println!("No valid graphs to analyze.");
        return Ok(());
    }

    // Calculate statistics
    node_counts.sort_unstable();
    let min_nodes = *node_counts.first().unwrap();
    let max_nodes = *node_counts.last().unwrap();
    let sum: usize = node_counts.iter().sum();
    let avg_nodes = sum as f64 / node_counts.len() as f64;
    let median_nodes = if node_counts.len() % 2 == 0 {
        let mid = node_counts.len() / 2;
        (node_counts[mid - 1] + node_counts[mid]) as f64 / 2.0
    } else {
        node_counts[node_counts.len() / 2] as f64
    };

    // Percentiles
    let p25_idx = node_counts.len() / 4;
    let p75_idx = (node_counts.len() * 3) / 4;
    let p90_idx = (node_counts.len() * 9) / 10;
    let p95_idx = (node_counts.len() * 95) / 100;
    let p99_idx = (node_counts.len() * 99) / 100;

    println!("Node count statistics:");
    println!("  Minimum:      {}", min_nodes);
    println!("  25th percentile: {}", node_counts[p25_idx]);
    println!("  Median (50th):   {:.2}", median_nodes);
    println!("  Average:      {:.2}", avg_nodes);
    println!("  75th percentile: {}", node_counts[p75_idx]);
    println!("  90th percentile: {}", node_counts[p90_idx]);
    println!("  95th percentile: {}", node_counts[p95_idx]);
    println!("  99th percentile: {}", node_counts[p99_idx]);
    println!("  Maximum:      {}\n", max_nodes);

    // Print distribution by ranges
    println!("=============================================================");
    println!("DISTRIBUTION BY NODE COUNT RANGES");
    println!("=============================================================\n");

    let ranges = vec![
        (0, 10, "0-10"),
        (11, 20, "11-20"),
        (21, 50, "21-50"),
        (51, 100, "51-100"),
        (101, 200, "101-200"),
        (201, 500, "201-500"),
        (501, 1000, "501-1K"),
        (1001, 2000, "1K-2K"),
        (2001, 5000, "2K-5K"),
        (5001, 10000, "5K-10K"),
        (10001, usize::MAX, "10K+"),
    ];

    let mut range_counts: Vec<(String, usize, f64)> = Vec::new();

    for (min, max, label) in ranges {
        let count = node_counts
            .iter()
            .filter(|&&n| n >= min && n <= max)
            .count();
        let percentage = (count as f64 / node_counts.len() as f64) * 100.0;
        range_counts.push((label.to_string(), count, percentage));
    }

    println!("{:<15} {:<15} {:<15}", "Range", "Count", "Percentage");
    println!("{}", "-".repeat(50));
    for (label, count, percentage) in range_counts {
        if count > 0 {
            println!("{:<15} {:<15} {:<14.2}%", label, count, percentage);
        }
    }

    // Print top 20 most common node counts
    println!("\n=============================================================");
    println!("TOP 20 MOST COMMON NODE COUNTS");
    println!("=============================================================\n");

    let mut dist_vec: Vec<_> = distribution.iter().collect();
    dist_vec.sort_by(|a, b| b.1.cmp(a.1));

    println!("{:<15} {:<15} {:<15}", "Node Count", "Frequency", "Percentage");
    println!("{}", "-".repeat(50));

    for (i, (node_count, freq)) in dist_vec.iter().take(20).enumerate() {
        let percentage = (**freq as f64 / node_counts.len() as f64) * 100.0;
        println!("{:<2}. {:<12} {:<15} {:<14.2}%", i + 1, node_count, freq, percentage);
    }

    // Find outliers (graphs with unusually high node counts)
    let threshold = p95_idx;
    let outlier_threshold = node_counts[threshold];
    let outliers: Vec<_> = node_counts
        .iter()
        .filter(|&&n| n > outlier_threshold)
        .collect();

    if !outliers.is_empty() {
        println!("\n=============================================================");
        println!("OUTLIERS (Top 5% - Node Count > {})", outlier_threshold);
        println!("=============================================================\n");
        println!("Count: {} graphs ({:.2}%)", outliers.len(), (outliers.len() as f64 / node_counts.len() as f64) * 100.0);
        println!("Node counts: {:?}", &outliers[..outliers.len().min(10)]);
    }

    // Export to JSON
    export_to_json(&node_counts, &distribution)?;

    println!("\n=============================================================");
    println!("✓ Analysis complete!");
    println!("=============================================================");

    Ok(())
}

fn export_to_json(
    node_counts: &[usize],
    distribution: &HashMap<usize, usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;

    let output_file = "graph_nodes_distribution.json";

    let mut dist_vec: Vec<_> = distribution.iter().collect();
    dist_vec.sort_by_key(|a| a.0);

    let json = serde_json::json!({
        "summary": {
            "total_graphs": node_counts.len(),
            "min_nodes": node_counts.first(),
            "max_nodes": node_counts.last(),
            "avg_nodes": node_counts.iter().sum::<usize>() as f64 / node_counts.len() as f64,
        },
        "distribution": dist_vec.iter().map(|(k, v)| {
            serde_json::json!({
                "node_count": k,
                "frequency": v
            })
        }).collect::<Vec<_>>(),
        "raw_data": node_counts,
    });

    let mut file = File::create(output_file)?;
    file.write_all(serde_json::to_string_pretty(&json)?.as_bytes())?;

    println!("\n✓ Results exported to: {}", output_file);

    Ok(())
}
