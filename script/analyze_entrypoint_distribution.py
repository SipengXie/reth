#!/usr/bin/env python3
"""
Analyze entrypoint frequency distribution and show top path_hashes for each entrypoint.
"""

import json
from collections import defaultdict
from pathlib import Path
import sys

def analyze_entrypoint_distribution(statistics_file, exclude_path_hashes=None, exclude_entrypoints=None):
    """
    Analyze entrypoint frequency distribution.

    Args:
        statistics_file: Path to statistics.json file
        exclude_path_hashes: Set of path_hashes to exclude from analysis
        exclude_entrypoints: Set of entrypoint keys to exclude from analysis
    """
    if exclude_path_hashes is None:
        exclude_path_hashes = set()
    if exclude_entrypoints is None:
        exclude_entrypoints = set()

    print(f"Loading data from {statistics_file}...")

    with open(statistics_file, 'r') as f:
        data = json.load(f)

    print(f"Total entrypoints (before filtering): {len(data)}")

    # Calculate total frequency for each entrypoint
    entrypoint_stats = {}
    excluded_entrypoints_count = 0
    excluded_path_hashes_count = 0

    for entrypoint_key, path_hashes in data.items():
        # Check if this entrypoint should be excluded
        if entrypoint_key in exclude_entrypoints:
            excluded_entrypoints_count += 1
            continue

        # Calculate total frequency for this entrypoint (excluding filtered path_hashes)
        total_freq = 0
        filtered_path_hashes = {}

        for path_hash, frequency in path_hashes.items():
            if path_hash in exclude_path_hashes:
                excluded_path_hashes_count += frequency
                continue
            total_freq += frequency
            filtered_path_hashes[path_hash] = frequency

        # Skip entrypoints with no valid path_hashes after filtering
        if total_freq == 0:
            continue

        entrypoint_stats[entrypoint_key] = {
            'total_frequency': total_freq,
            'path_hashes': filtered_path_hashes,
            'unique_paths': len(filtered_path_hashes)
        }

    print(f"Total entrypoints (after filtering): {len(entrypoint_stats)}")
    print(f"Excluded entrypoints: {excluded_entrypoints_count}")
    print(f"Excluded path_hash occurrences: {excluded_path_hashes_count:,}")

    # Sort entrypoints by total frequency
    sorted_entrypoints = sorted(
        entrypoint_stats.items(),
        key=lambda x: x[1]['total_frequency'],
        reverse=True
    )

    # Calculate total frequency across all entrypoints
    total_frequency = sum(stats['total_frequency'] for _, stats in sorted_entrypoints)
    print(f"\nTotal frequency (sum across all entrypoints): {total_frequency:,}")

    # Pareto analysis for entrypoints
    print("\n" + "="*80)
    print("ENTRYPOINT PARETO ANALYSIS")
    print("="*80)

    cumulative_freq = 0
    cumulative_count = 0
    thresholds = [0.5, 0.8, 0.9, 0.95, 0.99]
    threshold_results = {}

    for entrypoint_key, stats in sorted_entrypoints:
        cumulative_freq += stats['total_frequency']
        cumulative_count += 1
        cumulative_pct = cumulative_freq / total_frequency

        for threshold in thresholds:
            if threshold not in threshold_results and cumulative_pct >= threshold:
                threshold_results[threshold] = {
                    'count': cumulative_count,
                    'count_pct': (cumulative_count / len(sorted_entrypoints)) * 100,
                    'freq': cumulative_freq,
                    'freq_pct': cumulative_pct * 100
                }

    print("\nPareto Distribution (by Entrypoint):")
    print(f"{'Threshold':<12} {'Entrypoint Count':<18} {'% of Total':<15} {'Cumulative Freq':<18} {'% of Total Freq':<18}")
    print("-" * 85)

    for threshold in thresholds:
        if threshold in threshold_results:
            result = threshold_results[threshold]
            print(f"{threshold*100:>6.0f}%      {result['count']:>10,}        {result['count_pct']:>8.2f}%      "
                  f"{result['freq']:>10,}        {result['freq_pct']:>8.2f}%")

    # Top 20 entrypoints with their top 5 path_hashes
    print("\n" + "="*80)
    print("TOP 20 ENTRYPOINTS WITH THEIR TOP PATH HASHES")
    print("="*80)

    top_n = min(20, len(sorted_entrypoints))
    cumulative = 0

    for rank, (entrypoint_key, stats) in enumerate(sorted_entrypoints[:top_n], 1):
        cumulative += stats['total_frequency']
        pct = (stats['total_frequency'] / total_frequency) * 100
        cumulative_pct = (cumulative / total_frequency) * 100

        # Parse entrypoint key
        parts = entrypoint_key.rsplit('_', 2)
        if len(parts) == 3:
            contract_addr, selector, entry_type = parts
            display_name = f"{contract_addr[:8]}...{contract_addr[-4:]}_{selector}_{entry_type}"
        else:
            display_name = entrypoint_key[:60]

        print(f"\n{'='*80}")
        print(f"Rank {rank}: {display_name}")
        print(f"{'='*80}")
        print(f"Total Frequency: {stats['total_frequency']:>10,}  ({pct:>6.2f}% of total, cumulative: {cumulative_pct:>6.2f}%)")
        print(f"Unique Paths:    {stats['unique_paths']:>10,}")

        # Sort path_hashes for this entrypoint
        sorted_paths = sorted(
            stats['path_hashes'].items(),
            key=lambda x: x[1],
            reverse=True
        )

        # Show top 5 path_hashes
        print(f"\nTop 5 Path Hashes:")
        print(f"  {'Rank':<6} {'Path Hash':<20} {'Frequency':<12} {'% of Entrypoint':<18}")
        print(f"  {'-'*60}")

        top_paths = min(5, len(sorted_paths))
        for i, (path_hash, freq) in enumerate(sorted_paths[:top_paths], 1):
            path_pct = (freq / stats['total_frequency']) * 100
            print(f"  {i:<6} {path_hash:<20} {freq:>10,}  {path_pct:>14.2f}%")

        if len(sorted_paths) > 5:
            remaining_freq = sum(freq for _, freq in sorted_paths[5:])
            remaining_pct = (remaining_freq / stats['total_frequency']) * 100
            print(f"  {'...':<6} {'(other paths)':<20} {remaining_freq:>10,}  {remaining_pct:>14.2f}%")

    # Summary statistics
    print("\n" + "="*80)
    print("SUMMARY STATISTICS")
    print("="*80)

    frequencies = [stats['total_frequency'] for _, stats in sorted_entrypoints]
    unique_paths = [stats['unique_paths'] for _, stats in sorted_entrypoints]

    print(f"\nEntrypoint Frequency Statistics:")
    print(f"  Min frequency: {min(frequencies):,}")
    print(f"  Max frequency: {max(frequencies):,}")
    print(f"  Median frequency: {frequencies[len(frequencies)//2]:,}")
    print(f"  Mean frequency: {sum(frequencies)/len(frequencies):.2f}")

    print(f"\nUnique Paths per Entrypoint:")
    print(f"  Min paths: {min(unique_paths):,}")
    print(f"  Max paths: {max(unique_paths):,}")
    print(f"  Median paths: {unique_paths[len(unique_paths)//2]:,}")
    print(f"  Mean paths: {sum(unique_paths)/len(unique_paths):.2f}")

    # Save detailed results
    output_file = Path(statistics_file).parent / "entrypoint_distribution_analysis.json"
    output_data = {
        "summary": {
            "total_entrypoints": len(entrypoint_stats),
            "excluded_entrypoints": excluded_entrypoints_count,
            "excluded_path_hashes_occurrences": excluded_path_hashes_count,
            "total_frequency": total_frequency,
            "average_frequency_per_entrypoint": total_frequency / len(entrypoint_stats)
        },
        "pareto_analysis": threshold_results,
        "top_100_entrypoints": [
            {
                "entrypoint": key,
                "total_frequency": stats['total_frequency'],
                "unique_paths": stats['unique_paths'],
                "top_5_paths": [
                    {"path_hash": ph, "frequency": freq}
                    for ph, freq in sorted(
                        stats['path_hashes'].items(),
                        key=lambda x: x[1],
                        reverse=True
                    )[:5]
                ]
            }
            for key, stats in sorted_entrypoints[:100]
        ]
    }

    with open(output_file, 'w') as f:
        json.dump(output_data, f, indent=2)

    print(f"\n" + "="*80)
    print(f"Detailed analysis saved to: {output_file}")
    print("="*80)

if __name__ == "__main__":
    statistics_file = sys.argv[1] if len(sys.argv) > 1 else "ssa_cache/statistics.json"

    # Exclude special path_hashes (pure transfer / empty values)
    exclude_path_hashes = {
        "cbf29ce484222325",  # Pure transfer (keccak256_empty equivalent)
    }

    # Exclude special entrypoints (pure transfer contracts)
    exclude_entrypoints = {
        "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470_NONE_Transfer",  # Pure transfer entrypoint
    }

    analyze_entrypoint_distribution(statistics_file, exclude_path_hashes, exclude_entrypoints)
