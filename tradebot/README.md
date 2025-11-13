# Trade Bot

Simulated orderbook trading bot that replays historical Binance order data on a Polkadot-based orderbook chain.

## Features

- **Multi-Account Trading**: Simulates N configurable trading accounts
- **Historical Data Replay**: Replays recorded Binance WebSocket data
- **Concurrent Order Execution**: Processes multiple orders within a block concurrently
- **Deterministic Account Mapping**: Maps synthetic trader addresses to real dev accounts deterministically

## Data Files

The bot uses synthetic trading data recorded from Binance:

- `ETHUSDC_2025-11-12T22-08-37-339Z_synthetic_blocks.jsonl` - Main data file with blocks of transactions
- `ETHUSDC_2025-11-12T22-08-37-339Z_trades.jsonl.gz` - Raw trade data
- `ETHUSDC_2025-11-12T22-08-37-339Z_depth_updates.jsonl.gz` - Orderbook depth updates
- `ETHUSDC_2025-11-12T22-08-37-339Z_initial_snapshot.jsonl.gz` - Initial orderbook snapshot

## Configuration

Copy `.env.example` to `.env` and configure:

```bash
cp .env.example .env
```

### Environment Variables

- `NODE_WS_URL` - WebSocket endpoint of the Substrate node (default: `ws://127.0.0.1:9944`)
- `NUM_ACCOUNTS` - Number of trading accounts to simulate (default: `6`)
  - Uses dev accounts: Alice, Bob, Charlie, Dave, Eve, Ferdie
  - If you need more than 6, accounts will wrap around
- `WORKER_POOL_SIZE` - Maximum number of concurrent transaction workers (default: `10`)
  - Controls how many transactions can be processed in parallel
  - Higher values = more throughput but more load on the chain
- `SKIP_FUNDING` - Set to `1` to skip account funding (default: `0`)
- `BLOCKS_FILE` - Path to the synthetic blocks file
- `RUST_LOG` - Logging level (trace, debug, info, warn, error)

## Running the Bot

### Prerequisites

1. A running Substrate node with the orderbook pallet
2. Generated metadata file (see below)

### Generate Metadata

Before running the bot, generate the runtime metadata:

```bash
# From the repository root, with your node running:
subxt metadata -f bytes > metadata.scale
```

This creates a `metadata.scale` file used by subxt for type-safe extrinsic construction.

### Run

```bash
cargo run --release
```

## How It Works

1. **Account Generation**: Creates N trading accounts using Substrate dev keypairs (Alice, Bob, etc.)
2. **Account Funding**: Automatically funds each account with 1 trillion ETH (asset 0) and 1 trillion USDC (asset 1)
3. **Data Loading**: Loads synthetic blocks from the JSONL file and flattens all transactions into a single list
4. **Worker Pool Architecture**:
   - Spawns a worker for each transaction up to the pool size limit
   - Workers process transactions concurrently across different accounts
   - Per-account locks ensure proper nonce management (sequential txs per account)
   - Waits for transaction finalization to avoid nonce conflicts
   - Pool saturation prevents overwhelming the chain with too many concurrent requests
5. **Order Mapping**: Deterministically maps synthetic trader addresses to real accounts using a hash function
6. **Price/Quantity Conversion**: Converts floating-point prices/quantities to u128 with 6 decimal places

## Transaction Types

The bot handles two types of transactions from the synthetic data:

### Place Order
```json
{
  "tx_type": "place_order",
  "trader": "165EgRiitA768fkGv1BUH7TcBMpqN5FrNx2S8dLe2xaKkPET",
  "params": {
    "side": "bid",
    "price": 3418.32,
    "quantity": 0.8879149
  }
}
```

### Cancel Order
```json
{
  "tx_type": "cancel_order",
  "trader": "14QLa52sXaKRWxw6PEdnjsbdhapAwScWejWZHTWMT5ysoQAQ",
  "params": {
    "order_id": "118b2e11-46d9-4c8a-bb90-9277b5419ce4"
  }
}
```

**Note**: Cancel operations are currently skipped since synthetic order IDs don't match on-chain order IDs.

## Output

The bot provides detailed logging:

```
🤖 Trade Bot Starting...
✅ Connected to chain: RuntimeVersion { ... }
Generated 6 trading accounts:
  - 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
  - 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
  ...
📂 Loading blocks from: ETHUSDC_2025-11-12T22-08-37-339Z_synthetic_blocks.jsonl
✅ Loaded 1234 blocks
🚀 Starting block replay with 1234 blocks
📦 Processing block 1 (26/1234 transactions)
✅ Order placed: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY bid @ 3418.32 (qty: 0.8879149) [nonce: 0]
...
✅ Completed replay of all 1234 blocks
🎉 Trade bot completed successfully!
```

## Performance Tuning

- **Worker Pool Size**: Adjust `WORKER_POOL_SIZE` to control concurrency (default: 10)
  - Higher values = more throughput but more load on the chain
  - Recommended: Start with pool size = 2-3x number of accounts
- **Account Count**: Increase `NUM_ACCOUNTS` to distribute load across more accounts
  - More accounts = less contention, better parallelism
- **Chain Block Time**: The runtime is configured for 1-second block times (Aura consensus)

## Troubleshooting

### "Failed to connect to node"
- Ensure your Substrate node is running
- Verify `NODE_WS_URL` points to the correct endpoint

### "Failed to parse block"
- Ensure the BLOCKS_FILE path is correct
- Check that the file is a valid JSONL format

### "Order failed in block"
- Check that accounts have sufficient funds
- Verify the orderbook pallet is properly configured
- Check node logs for detailed error messages

## Development

To modify the bot behavior:

- `place_order()` - Handles order submission logic
- `replay_transactions()` - Controls transaction processing flow with worker pool
- `map_trader_to_account()` - Changes how traders map to accounts

### Architecture Overview

The bot uses a worker pool pattern for optimal throughput:

1. Main loop spawns a worker task for each transaction
2. Semaphore limits concurrent workers to `WORKER_POOL_SIZE`
3. Each worker acquires a per-account lock before processing
4. Transactions for the same account are sequential (proper nonce order)
5. Transactions for different accounts run in parallel (maximum throughput)
6. All workers are awaited at the end to ensure completion
