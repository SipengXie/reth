#!/bin/bash

# Script to extract elapsed time and block number from log file
# Usage: ./extract_block_stats.sh <input_log> <output_file>

INPUT_LOG="${1:-1.log}"
OUTPUT_FILE="${2:-block_stats.csv}"

# Check if input file exists
if [ ! -f "$INPUT_LOG" ]; then
    echo "Error: Input file '$INPUT_LOG' not found!"
    exit 1
fi

# Create output file with header
echo "block_number,elapsed_time_ms" > "$OUTPUT_FILE"

# Extract data using grep and sed, then convert to milliseconds with awk
# Remove ANSI color codes, then extract elapsed and number values
grep "Executed block" "$INPUT_LOG" | \
    sed -r 's/\x1B\[[0-9;]*[mK]//g' | \
    sed -n 's/.*elapsed=\([^ ]*\).*number=\([0-9]*\).*/\2,\1/p' | \
    awk -F',' '{
        time = $2
        # Convert to milliseconds
        if (time ~ /s$/ && time !~ /ms$/ && time !~ /us$/ && time !~ /ns$/) {
            # seconds to ms
            gsub(/s$/, "", time)
            time = time * 1000
        } else if (time ~ /ms$/) {
            # already in ms
            gsub(/ms$/, "", time)
        } else if (time ~ /us$/) {
            # microseconds to ms
            gsub(/us$/, "", time)
            time = time / 1000
        } else if (time ~ /ns$/) {
            # nanoseconds to ms
            gsub(/ns$/, "", time)
            time = time / 1000000
        }
        printf "%s,%.6f\n", $1, time
    }' >> "$OUTPUT_FILE"

# Count extracted records
RECORD_COUNT=$(($(wc -l < "$OUTPUT_FILE") - 1))

echo "Extraction complete!"
echo "Total records extracted: $RECORD_COUNT"
echo "Output saved to: $OUTPUT_FILE"
