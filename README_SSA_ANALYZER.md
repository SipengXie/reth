# SSA Cache Access Statistics Analyzer

This is a Python script for analyzing SSA cache access statistics, which helps you understand the access patterns and hotspots of execution paths.

## Features

### 1. Basic Statistics Analysis
- Total PathKey count and total access count
- Average, maximum, minimum, median access count
- Clear table format output

### 2. Pareto Analysis (Core Function)
- Analyze the top X% of PathKeys that account for Y% of total accesses
- Support multiple percentage thresholds for analysis
- Help identify critical execution paths

### 3. Access Distribution Analysis
- Analyze the distribution of PathKeys by access count range
- Identify the concentration of access patterns

### 4. Visualization Charts (Optional)
- Access count distribution histogram
- Pareto analysis chart
- Access range distribution pie chart
- Cumulative access distribution curve

### 5. Pure Transfer Filtering (New Feature)
- Automatically identify and filter pure ETH transfer transactions
- Focus on smart contract execution pattern analysis
- Provide comparison statistics before and after filtering

## Installation Requirements

### Basic Function
```bash
# Only Python 3.6+ standard library is required
python3 analyze_ssa_stats.py ssa_cache_static.json
```

### Visualization Function
```bash
# Install matplotlib for generating charts
pip install matplotlib numpy
python3 analyze_ssa_stats.py ssa_cache_static.json --visualize
```

## Usage

### Basic Usage
```bash
# Basic analysis
python analyze_ssa_stats.py ssa_cache_static.json

# Specify Pareto analysis percentage
python analyze_ssa_stats.py ssa_cache_static.json --top-percent 20

# Multiple Pareto thresholds analysis
python analyze_ssa_stats.py ssa_cache_static.json --pareto-thresholds 10,20,50

# Display more popular PathKeys
python analyze_ssa_stats.py ssa_cache_static.json --top-pathkeys 20
```

### Visualization Analysis
```bash
# Generate visualization charts
python analyze_ssa_stats.py ssa_cache_static.json --visualize

# Custom output file prefix
python analyze_ssa_stats.py ssa_cache_static.json --visualize --output-prefix my_analysis

# Complete analysis (recommended)
python analyze_ssa_stats.py ssa_cache_static.json --pareto-thresholds 5,10,20,50 --top-pathkeys 25 --visualize

# Smart contract specific analysis (filter pure transfers)
python analyze_ssa_stats.py ssa_cache_static.json --exclude-pure-transfers --visualize
```

### Command Line Parameters

| Parameter | Type | Default | Description |
|------|------|--------|------|
| `file` | Required | - | SSA statistics JSON file path |
| `--top-percent` | float | 20.0 | Percentage for single Pareto analysis |
| `--pareto-thresholds` | string | "10,20,50" | Multiple Pareto thresholds (comma separated) |
| `--top-pathkeys` | int | 10 | Number of popular PathKeys to display |
| `--no-distribution` | flag | false | Skip access distribution analysis |
| `--visualize` | flag | false | Generate visualization charts |
| `--output-prefix` | string | "ssa_stats" | Output file prefix |
| `--exclude-pure-transfers` | flag | false | Exclude pure ETH transfer transactions |

## Pure Transfer Filtering

### What is a pure transfer?
Pure transfer is a simple ETH transfer transaction that does not involve smart contract code execution. These transactions have a fixed PathKey:
```
c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470_cbf29ce484222325
```

### Why filter pure transfers?
1. **Noise elimination**: Pure transfers usually account for a large number of accesses, but do not reflect the execution patterns of smart contracts
2. **Pattern analysis**: After filtering, it can be more clearly seen that the real execution hotspots of DeFi, NFT, etc. are smart contracts
3. **Performance optimization**: Focus on the cache optimization of complex logic, rather than simple transfers

### Usage scenarios
- **DeFi protocol analysis**: Identify the hotspots of core functions such as transactions and liquidity operations
- **Smart contract optimization**: Focus on the SSA graph cache strategy of contract logic
- **System capacity planning**: Based on actual business logic rather than transfer noise

### Filtering effect example
```bash
python analyze_ssa_stats.py ssa_cache_static.json --exclude-pure-transfers
```

The output will include filtering information:
```
==================================================
DATA FILTERING INFORMATION
==================================================
Pure transfer filtering: ENABLED
Pure transfer PathKey:   c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470_cbf29ce484222325
Pure transfer accesses:  1,234,567 (67.8% of total)
PathKeys removed:        1
Total accesses removed:  1,234,567

BEFORE FILTERING:
  Total PathKeys:        1,234
  Total Accesses:        1,820,000

AFTER FILTERING:
  Total PathKeys:        1,233
  Total Accesses:        585,433

💡 INFO: Pure transfers account for 67.8% of all accesses.
   Filtering provides a cleaner view of smart contract execution patterns.
```

## Output description

### 1. Basic statistics
```
==================================================
BASIC STATISTICS
==================================================
Total PathKeys:        1,234
Total Accesses:        45,678
Average Accesses:      37.02
Median Accesses:       12.00
Max Accesses:          2,345
Min Accesses:          1
```

### 2. Pareto analysis
```
==================================================
PARETO ANALYSIS
==================================================
Top %    PathKeys     Accesses %   Interpretation
----------------------------------------------------------------------
    10%         123        68.5%   Top 10% → 68.5% of accesses
    20%         247        82.1%   Top 20% → 82.1% of accesses
    50%         617        95.3%   Top 50% → 95.3% of accesses
```

### 3. Popular PathKey list
```
==================================================
TOP 10 PATHKEYS BY ACCESS COUNT
==================================================
Rank   Access Count PathKey
----------------------------------------------------------------------
1      2,345        a1b2c3d4_e5f6g7h8
2      1,987        f9e8d7c6_b5a4938
...
```

### 4. Access distribution
```
==================================================
ACCESS COUNT DISTRIBUTION
==================================================
Range        PathKeys   Percentage
-----------------------------------
1-1          456           37.0%
2-5          321           26.0%
6-10         234           19.0%
11-50        156           12.6%
51-100       45            3.6%
101-500      18            1.5%
501+         4             0.3%
```

## Data format

The input JSON file format:
```json
{
  "a1b2c3d4_e5f6g7h8": 2345,
  "f9e8d7c6_b5a49382": 1987,
  "1234abcd_5678efgh": 1654,
  ...
}
```

Where:
- **Key**: PathKey string, format is `{code_hash}_{path_hash}`
- **Value**: The access count of the PathKey (positive integer)

## Visualization output

When using the `--visualize` option, the script will generate a PNG image file, containing four subplots:

1. **Access count distribution histogram**: Display the distribution of access counts
2. **Pareto analysis chart**: Display the access count and cumulative percentage of the top 20 PathKeys
3. **Access range distribution pie chart**: Display the proportion of PathKeys in different access count ranges
4. **Cumulative access distribution curve**: Display the relationship between PathKey percentile and cumulative access percentage

## Actual application scenarios

### Performance optimization
- **Full analysis**: Identify the most frequently visited execution paths (including pure transfers)
- **Smart contract specific**: Use `--exclude-pure-transfers` to focus on contract logic optimization
- **Cache strategy**: Determine the key areas of SSA graph pre-building

### System analysis
- **Load characteristics**: Understand the proportion of pure transfers vs smart contracts in the system
- **Execution mode**: Discover the hot paths of different types of contracts such as DeFi and NFT
- **Parallel optimization**: Evaluate the potential of parallel execution of different types of transactions

### Capacity planning
- **Storage demand**: Estimate the cache size based on actual business logic
- **Network planning**: Distinguish between simple transfers and complex contract resource demands
- **Expansion strategy**: Predict the impact of system expansion in different scenarios

### Business insight
- **User behavior**: Analyze the proportion of pure transfers vs smart contract interactions
- **Protocol analysis**: Identify popular DeFi protocols and operation types
- **Ecosystem development**: Track the evolution trend of smart contract complexity

## Troubleshooting

### Common errors

1. **File not found**
   ```
   Error: File 'ssa_cache_static.json' not found.
   ```
   Check if the file path is correct.

2. **JSON format error**
   ```
   Error: Invalid JSON in file: Expecting ',' delimiter
   ```
   Check if the JSON file format is correct.

3. **Data format error**
   ```
   Error: All values must be non-negative numbers
   ```
   Ensure that all access counts are non-negative integers.

4. **matplotlib not installed**
   ```
   Warning: matplotlib not available. Skipping visualizations.
   ```
   Run `pip install matplotlib numpy` to install the visualization dependencies.

### Performance considerations

- For large datasets (>1 million PathKey), visualization generation may take a long time
- It is recommended to run the basic analysis first to confirm the data is correct before generating visualizations
- For large datasets, it is recommended to use `--no-distribution` to skip the detailed distribution analysis to improve speed

## Example output

Example command for complete analysis:
```bash
python analyze_ssa_stats.py ssa_cache_static.json --pareto-thresholds 5,10,20,50 --top-pathkeys 15 --visualize --output-prefix production_analysis
```

This will generate:
- Complete console statistics report
- `production_analysis_analysis.png` visualization chart file
- Pareto analysis containing 5%, 10%, 20%, 50%
- Detailed information for the top 15 popular PathKeys
