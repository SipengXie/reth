#!/usr/bin/env python3
"""
Compare state_cache_sequential.json and state_cache_parallel.json
to find account differences.
"""

import json
import sys
from typing import Dict, Any, Set

def load_json(filepath: str) -> Dict[str, Any]:
    """Load JSON file."""
    print(f"Loading {filepath}...")
    with open(filepath, 'r') as f:
        data = json.load(f)
    print(f"  Loaded {len(data)} entries")
    return data

def filter_loaded_status(data: Dict[str, Any]) -> Dict[str, Any]:
    """Filter out accounts with status 'Loaded'."""
    if 'accounts' in data:
        original_count = len(data['accounts'])
        # Filter out accounts with status 'Loaded'
        data['accounts'] = {
            addr: account_data
            for addr, account_data in data['accounts'].items()
            if not (isinstance(account_data, dict) and account_data.get('status') == 'Loaded')
        }
        filtered_count = original_count - len(data['accounts'])
        if filtered_count > 0:
            print(f"  Filtered out {filtered_count} accounts with status 'Loaded'")
    return data

def compare_values(addr: str, val1: Any, val2: Any) -> list:
    """Compare two values and return list of differences."""
    differences = []

    if type(val1) != type(val2):
        differences.append(f"  Type mismatch: {type(val1).__name__} vs {type(val2).__name__}")
        return differences

    if isinstance(val1, dict):
        keys1 = set(val1.keys())
        keys2 = set(val2.keys())

        only_in_1 = keys1 - keys2
        only_in_2 = keys2 - keys1
        common_keys = keys1 & keys2

        if only_in_1:
            differences.append(f"  Keys only in sequential ({len(only_in_1)}):")
            for key in sorted(only_in_1):
                differences.append(f"    - {key}")
                # Show the data for this key
                data_str = str(val1[key])
                if len(data_str) < 2000:
                    differences.append(f"      Data: {val1[key]}")
                else:
                    differences.append(f"      Data: (too large, {len(data_str)} chars)")
                    # Show summary for large data
                    if isinstance(val1[key], dict):
                        differences.append(f"      Keys: {list(val1[key].keys())}")
                        for k, v in val1[key].items():
                            if isinstance(v, (str, bytes, bytearray)):
                                differences.append(f"      {k}: {len(v)} chars/bytes")
                            else:
                                differences.append(f"      {k}: {type(v).__name__}")

        if only_in_2:
            differences.append(f"  Keys only in parallel ({len(only_in_2)}):")
            for key in sorted(only_in_2):
                differences.append(f"    - {key}")
                # Show the data for this key
                data_str = str(val2[key])
                if len(data_str) < 2000:
                    differences.append(f"      Data: {val2[key]}")
                else:
                    differences.append(f"      Data: (too large, {len(data_str)} chars)")

        for key in common_keys:
            if val1[key] != val2[key]:
                differences.append(f"  Field '{key}' differs:")
                differences.append(f"    Sequential: {val1[key]}")
                differences.append(f"    Parallel:   {val2[key]}")
    else:
        if val1 != val2:
            differences.append(f"  Sequential: {val1}")
            differences.append(f"  Parallel:   {val2}")

    return differences

def main():
    seq_file = "state_cache_sequential.json"
    par_file = "state_cache_parallel.json"

    # Load both files
    sequential = load_json(seq_file)
    parallel = load_json(par_file)

    # Filter out accounts with status 'Loaded'
    print("\nFiltering accounts with status 'Loaded'...")
    sequential = filter_loaded_status(sequential)
    parallel = filter_loaded_status(parallel)

    print("\n" + "="*80)
    print("COMPARISON RESULTS")
    print("="*80)

    # Get account sets
    seq_accounts = set(sequential.keys())
    par_accounts = set(parallel.keys())

    only_sequential = seq_accounts - par_accounts
    only_parallel = par_accounts - seq_accounts
    common_accounts = seq_accounts & par_accounts

    # Print summary statistics
    print(f"\nSummary Statistics:")
    print(f"  Total accounts in sequential: {len(seq_accounts)}")
    print(f"  Total accounts in parallel:   {len(par_accounts)}")
    print(f"  Common accounts:              {len(common_accounts)}")
    print(f"  Only in sequential:           {len(only_sequential)}")
    print(f"  Only in parallel:             {len(only_parallel)}")

    # Accounts only in sequential
    if only_sequential:
        print(f"\n{'='*80}")
        print(f"ACCOUNTS ONLY IN SEQUENTIAL ({len(only_sequential)}):")
        print('='*80)
        for addr in sorted(only_sequential):
            print(f"\n{addr}:")
            print(f"  Data: {sequential[addr]}")

    # Accounts only in parallel
    if only_parallel:
        print(f"\n{'='*80}")
        print(f"ACCOUNTS ONLY IN PARALLEL ({len(only_parallel)}):")
        print('='*80)
        for addr in sorted(only_parallel):
            print(f"\n{addr}:")
            print(f"  Data: {parallel[addr]}")

    # Compare common accounts for value differences
    different_values = []
    for addr in sorted(common_accounts):
        if sequential[addr] != parallel[addr]:
            different_values.append(addr)

    if different_values:
        print(f"\n{'='*80}")
        print(f"ACCOUNTS WITH DIFFERENT VALUES ({len(different_values)}):")
        print('='*80)
        for addr in different_values:
            print(f"\n{addr}:")
            diffs = compare_values(addr, sequential[addr], parallel[addr])
            for diff in diffs:
                print(diff)

    # Final summary
    print(f"\n{'='*80}")
    print("FINAL SUMMARY:")
    print('='*80)
    total_differences = len(only_sequential) + len(only_parallel) + len(different_values)
    print(f"Total accounts with differences: {total_differences}")
    print(f"  - Only in sequential:  {len(only_sequential)}")
    print(f"  - Only in parallel:    {len(only_parallel)}")
    print(f"  - Different values:    {len(different_values)}")

    if total_differences == 0:
        print("\nThe two files are IDENTICAL!")
    else:
        print(f"\nThe two files have DIFFERENCES in {total_differences} accounts.")

if __name__ == "__main__":
    main()
