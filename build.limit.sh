#!/bin/bash

count=0
max_count=1000

while [ $count -lt $max_count ]; do
  # Get current timestamp
  timestamp=$(date "+%Y-%m-%d %H:%M:%S")
  
  # Get current chain height
  height=$(aleth chain height)
  next_height=$((height+1))
  
  echo "[$timestamp] Submitting block height: $next_height"
  
  payload_file="/home/ubuntu/PAYLOADS_19476587_19526585/usr/scratch2/ovunc/reth-exports/PAYLOADS_19476587_19526585/PAYLOAD_B${next_height}.json"
  # payload_file="/home/ubuntu/payloads23517991/PAYLOAD_B${next_height}.json"
  
  # Check if payload file exists
  if [ ! -f "$payload_file" ]; then
    echo "[$timestamp] ERROR: Payload file not found: $payload_file"
    exit 1
  fi
  
  out=$(aleth block submit-block -f "$payload_file")
  echo $out
  if [[ "$out" != *"submitted and"* ]]; then
    exit 1
  fi

  count=$((count+1))
done

echo "[$(date "+%Y-%m-%d %H:%M:%S")] Finished after $count submissions."

