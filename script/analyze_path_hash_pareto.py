#!/usr/bin/env python3
"""
Analyze path_hash frequency distribution and Pareto principle in SSA cache statistics.
"""

import json
from collections import defaultdict
from pathlib import Path
import sys

def analyze_path_hash_pareto(statistics_file, exclude_path_hashes=None):
    """
    Analyze path_hash frequency distribution and compute Pareto statistics.

    Args:
        statistics_file: Path to statistics.json file
        exclude_path_hashes: Set of path_hashes to exclude from analysis
    """
    if exclude_path_hashes is None:
        exclude_path_hashes = set()

    print(f"Loading data from {statistics_file}...")

    with open(statistics_file, 'r') as f:
        data = json.load(f)

    print(f"Total entrypoints: {len(data)}")

    # Aggregate path_hash frequencies across all entrypoints
    path_hash_freq = defaultdict(int)
    total_entrypoint_path_pairs = 0
    excluded_frequency = 0

    for entrypoint_key, path_hashes in data.items():
        for path_hash, frequency in path_hashes.items():
            if path_hash in exclude_path_hashes:
                excluded_frequency += frequency
                continue
            path_hash_freq[path_hash] += frequency
            total_entrypoint_path_pairs += 1

    if excluded_frequency > 0:
        print(f"\nExcluded path_hashes:")
        for ph in exclude_path_hashes:
            print(f"  - {ph}: excluded (pure transfer / empty)")
        print(f"Total excluded frequency: {excluded_frequency:,}")

    print(f"Total entrypoint-path pairs: {total_entrypoint_path_pairs}")
    print(f"Unique path_hashes: {len(path_hash_freq)}")

    # Sort by frequency (descending)
    sorted_path_hashes = sorted(path_hash_freq.items(), key=lambda x: x[1], reverse=True)

    # Calculate total frequency
    total_frequency = sum(freq for _, freq in sorted_path_hashes)
    print(f"Total frequency (sum of all frequencies): {total_frequency}")
    print(f"Average frequency per path_hash: {total_frequency / len(path_hash_freq):.2f}")

    # Pareto analysis
    print("\n" + "="*80)
    print("PARETO ANALYSIS")
    print("="*80)

    cumulative_freq = 0
    cumulative_count = 0

    # Find thresholds: 50%, 80%, 90%, 95%, 99%
    thresholds = [0.5, 0.8, 0.9, 0.95, 0.99]
    threshold_results = {}

    for path_hash, freq in sorted_path_hashes:
        cumulative_freq += freq
        cumulative_count += 1
        cumulative_pct = cumulative_freq / total_frequency

        for threshold in thresholds:
            if threshold not in threshold_results and cumulative_pct >= threshold:
                threshold_results[threshold] = {
                    'count': cumulative_count,
                    'count_pct': (cumulative_count / len(path_hash_freq)) * 100,
                    'freq': cumulative_freq,
                    'freq_pct': cumulative_pct * 100
                }

    # Print Pareto results
    print("\nPareto Distribution:")
    print(f"{'Threshold':<12} {'Path Count':<15} {'% of Total Paths':<20} {'Cumulative Freq':<18} {'% of Total Freq':<18}")
    print("-" * 85)

    for threshold in thresholds:
        if threshold in threshold_results:
            result = threshold_results[threshold]
            print(f"{threshold*100:>6.0f}%      {result['count']:>8,}        {result['count_pct']:>8.2f}%            "
                  f"{result['freq']:>10,}        {result['freq_pct']:>8.2f}%")

    # Top N analysis
    print("\n" + "="*80)
    print("TOP PATH HASHES BY FREQUENCY")
    print("="*80)

    top_n = min(20, len(sorted_path_hashes))
    print(f"\nTop {top_n} path_hashes:")
    print(f"{'Rank':<6} {'Path Hash':<20} {'Frequency':<12} {'% of Total':<12} {'Cumulative %':<15}")
    print("-" * 80)

    cumulative = 0
    for i, (path_hash, freq) in enumerate(sorted_path_hashes[:top_n], 1):
        cumulative += freq
        pct = (freq / total_frequency) * 100
        cumulative_pct = (cumulative / total_frequency) * 100
        print(f"{i:<6} {path_hash:<20} {freq:>10,}  {pct:>10.2f}%  {cumulative_pct:>12.2f}%")

    # Distribution statistics
    print("\n" + "="*80)
    print("DISTRIBUTION STATISTICS")
    print("="*80)

    frequencies = [freq for _, freq in sorted_path_hashes]

    print(f"\nFrequency Statistics:")
    print(f"  Min frequency: {min(frequencies):,}")
    print(f"  Max frequency: {max(frequencies):,}")
    print(f"  Median frequency: {frequencies[len(frequencies)//2]:,}")
    print(f"  Mean frequency: {sum(frequencies)/len(frequencies):.2f}")

    # Count distribution
    freq_distribution = defaultdict(int)
    for freq in frequencies:
        freq_distribution[freq] += 1

    print(f"\nFrequency Distribution:")
    print(f"{'Frequency':<12} {'Count':<12} {'Percentage':<12}")
    print("-" * 40)

    for freq in sorted(freq_distribution.keys())[:20]:  # Show first 20
        count = freq_distribution[freq]
        pct = (count / len(path_hash_freq)) * 100
        print(f"{freq:<12} {count:<12,} {pct:>10.2f}%")

    if len(freq_distribution) > 20:
        print(f"... ({len(freq_distribution) - 20} more frequency values)")

    # Save detailed results
    output_file = Path(statistics_file).parent / "path_hash_pareto_analysis.json"
    output_data = {
        "summary": {
            "total_entrypoints": len(data),
            "total_entrypoint_path_pairs": total_entrypoint_path_pairs,
            "unique_path_hashes": len(path_hash_freq),
            "total_frequency": total_frequency,
            "average_frequency": total_frequency / len(path_hash_freq)
        },
        "pareto_analysis": threshold_results,
        "top_100_path_hashes": [
            {"path_hash": ph, "frequency": freq}
            for ph, freq in sorted_path_hashes[:100]
        ],
        "frequency_distribution": {
            str(freq): count
            for freq, count in sorted(freq_distribution.items())
        }
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

    analyze_path_hash_pareto(statistics_file, exclude_path_hashes)
