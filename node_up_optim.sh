export DATA_DIR="/home/ubuntu/snap19476586/data"
ENABLE_SSA=true \
ENABLE_COLLECTOR=false \
ENABLE_DETER=false \
RUST_LOG="info" \
cargo run -p altius-reth --release -- node \
  --datadir "$DATA_DIR" \
  --http \
  --http.api all \
  --engine.persistence-threshold 0 \
  --engine.memory-block-buffer-target 0 \
  --disable-discovery \
  --trusted-only \
  --block-interval 5 \
  --engine.caching-and-prewarming \
  --prune.senderrecovery.full \
  --prune.transactionlookup.full \
  --prune.receipts.distance=10064 \
  --prune.accounthistory.distance=10064 \
  --prune.storagehistory.distance=10064 \
  --authrpc.jwtsecret="$DATA_DIR/jwt.hex" > 1.log