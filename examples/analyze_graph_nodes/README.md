# SSA Graph Nodes Distribution Analyzer

统计 SSA Cache 中所有 SSA Graph 的 nodes 数量分布。

## 功能

- ✅ 自动加载 SSA cache (从 `SSA_CACHE_PATH` 或默认路径)
- ✅ 统计所有 Graph 的 nodes 数量
- ✅ 自动将 Logs 转换为 Graph 进行分析
- ✅ 计算详细的统计数据 (min, max, avg, median, percentiles)
- ✅ 按范围展示分布
- ✅ 展示 Top 20 最常见的 node 数量
- ✅ 识别异常值 (outliers)
- ✅ 导出 JSON 格式结果

## 编译

```bash
cd /home/ubuntu/sipeng/reth

# 编译
cargo build --release -p analyze_graph_nodes

# 或直接运行
cargo run --release -p analyze_graph_nodes
```

## 使用

### 基本用法

```bash
# 使用默认路径 (./ssa_cache.bin)
./target/release/analyze_graph_nodes

# 或指定自定义路径
SSA_CACHE_PATH=/path/to/ssa_cache.bin ./target/release/analyze_graph_nodes
```

### 示例输出

```
=============================================================
SSA Graph Nodes Distribution Analysis
=============================================================

Loading SSA cache from: ./ssa_cache.bin
✓ Cache initialized successfully

Total cache entries: 134601

Analyzing 134601 graphs...
  Progress: 0/134601 (0.0%)
  Progress: 13460/134601 (10.0%)
  Progress: 26920/134601 (20.0%)
  ...
  Progress: 134601/134601 (100.0%)

=============================================================
SUMMARY STATISTICS
=============================================================

Data types:
  Graphs (already built):  50000
  Logs (converted):        84601
  Conversion failures:     0
  Total analyzed:          134601

Node count statistics:
  Minimum:         5
  25th percentile: 45
  Median (50th):   120.00
  Average:         185.50
  75th percentile: 250
  90th percentile: 400
  95th percentile: 600
  99th percentile: 1200
  Maximum:         5000

=============================================================
DISTRIBUTION BY NODE COUNT RANGES
=============================================================

Range           Count           Percentage
--------------------------------------------------
0-10            1234            0.92% █
11-20           5678            4.22% ████
21-50           12345           9.17% █████████
51-100          45678           33.94% ██████████████████████████████████
101-200         32145           23.88% ███████████████████████
201-500         25678           19.08% ███████████████████
501-1K          8765            6.51% ██████
1K-2K           2345            1.74% █
2K-5K           678             0.50%
5K-10K          45              0.03%
10K+            10              0.01%

=============================================================
TOP 20 MOST COMMON NODE COUNTS
=============================================================

#     Node Count      Frequency       Percentage
-------------------------------------------------------
1     120             2345            1.74%
2     85              1987            1.48%
3     150             1876            1.39%
4     95              1654            1.23%
5     200             1543            1.15%
...

=============================================================
OUTLIERS (Top 5% - Node Count > 600)
=============================================================

Count: 6730 graphs (5.00%)

Sample outlier node counts (showing up to 10):
  1. 5000 nodes
  2. 4567 nodes
  3. 3890 nodes
  4. 3456 nodes
  5. 2987 nodes
  6. 2765 nodes
  7. 2543 nodes
  8. 2234 nodes
  9. 2156 nodes
  10. 2098 nodes
  ... and 6720 more

✓ Detailed results exported to: graph_nodes_distribution.json

=============================================================
✓ Analysis complete!
=============================================================
```

## 输出文件

### graph_nodes_distribution.json

```json
{
  "summary": {
    "total_graphs": 134601,
    "min_nodes": 5,
    "max_nodes": 5000,
    "avg_nodes": 185.5,
    "median_nodes": 120,
    "p25_nodes": 45,
    "p75_nodes": 250,
    "p90_nodes": 400,
    "p95_nodes": 600,
    "p99_nodes": 1200
  },
  "range_distribution": [
    {
      "range": "0-10",
      "count": 1234,
      "percentage": 0.92
    },
    ...
  ],
  "exact_distribution": [
    {
      "node_count": 120,
      "frequency": 2345
    },
    ...
  ]
}
```

## 环境变量

| 变量 | 默认值 | 说明 |
|-----|--------|------|
| `SSA_CACHE_PATH` | `./ssa_cache.bin` | SSA cache 文件路径 |

## 注意事项

1. **Cache 文件不存在**: 如果 cache 文件不存在，程序会创建一个空的 cache，不会报错
2. **Logs 转换**: 如果 cache 中存储的是 Logs 而非 Graph，程序会自动转换（可能需要一些时间）
3. **大文件**: 对于大型 cache 文件，分析可能需要几分钟时间
4. **内存占用**: 程序会加载整个 cache 到内存，请确保有足够的可用内存

## 故障排除

### 问题: "Failed to initialize cache"

**原因**: cache 文件格式错误或损坏

**解决方案**:
```bash
# 备份旧文件
mv ssa_cache.bin ssa_cache.bin.backup

# 重新运行分析（会创建空 cache）
./target/release/analyze_graph_nodes
```

### 问题: 编译错误 "cannot find type `SsaData` in module `ssa`"

**原因**: `altius-revm` 版本不匹配或 API 未导出

**解决方案**:
检查 `altius-revm` 依赖版本，确保使用的是支持 `ssa` 模块的版本。

## 与其他工具配合使用

### 与 Python 脚本结合

```bash
# 1. 运行 Rust 分析器生成 JSON
./target/release/analyze_graph_nodes

# 2. 使用 Python 可视化
python3 script/visualize_graph_nodes.py graph_nodes_distribution.json
```

### 定期监控

```bash
# 添加到 cron
0 */6 * * * cd /path/to/reth && ./target/release/analyze_graph_nodes
```

## 性能优化建议

- ✅ 使用 `--release` 编译以获得最佳性能
- ✅ 对于大型 cache，考虑使用 SSD 存储
- ✅ 可以通过修改代码添加多线程支持（使用 `rayon`）
