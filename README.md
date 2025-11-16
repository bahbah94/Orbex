# Orbex

A production-ready decentralized exchange (DEX) built on Substrate with batch-matching orderbook, real-time indexing, and high-performance trading infrastructure.

## Overview

Orbex is a complete DEX solution comprising three core components:

**Substrate Chain** — Limit orderbook with price-time priority matching, batch settlement at block finalization, and atomic fund transfers for USDT and ETH.

**Indexer** — Real-time event listener using subxt that maintains an in-memory orderbook state and logs all trades to TimescaleDB with Decimal precision for accuracy.

**Trade Bot** — Synthetic order replay tool for testing and load generation that funds accounts and submits orders to the chain for orderbook simulation.

## Architecture

### Two-Phase Matching

Orders submitted during a block are queued in a temporary cache. At block finalization, all orders match in a single pass: pending orders first match internally, then survivors match against the persistent orderbook. This design eliminates per-order matching overhead and prevents MEV attacks.

**Benefits:** Constant-time order submission, single matching pass per block, better price discovery, race condition prevention.

### Component Integration

- **Substrate Node** processes orders, matches trades, and settles funds atomically
- **Indexer** consumes events in real-time and syncs state to the database
- **TimescaleDB** stores trades with time-bucketed materialized views for 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, and 1m candles
- **Trade Bot** generates synthetic order flow for testing

## Quick Start

### Prerequisites

- Rust 1.70+
- Docker & Docker Compose
- Node.js 18+ (for scripts)

### Setup

1. Copy the env file:

```bash
cp .env.example .env
```

2. Start the DB:

```bash
just db-setup
```

3. Launch the node:

  ```bash
  cargo run --release --bin orbex-node -- --dev --tmp
  ```

4. Start the indexer:

  ```bash
  cargo run --release --bin orbex-indexer
  ```

5. Start the trade bot:

  ```bash
  cargo run --release --bin orbex-trade-bot
  ```

This launches the Substrate node, PostgreSQL with TimescaleDB, indexer, and the trade bot.

6. **Connect to the chain**

Visit the [Polkadot/Substrate Portal](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) and point to `ws://localhost:9944`.

7. **Start the frontend**

```bash
cd frontend
pnpm install
cp .env.example .env
pnpm dev
```

Open `http://localhost:3000/test-indexer` to access the Orbex DEX frontend.

## Components

### Substrate Pallet: Orderbook

Core DEX logic managing limit and market orders with automatic matching.

**Key Features**

- Place/cancel orders with atomic fund locking
- Batch matching at block finalization with price-time priority
- Partial order fills and TTL-based expiry
- Persistent orderbook storage with price-level indexing
- Event emission for all state changes

**Extrinsics**

- `place_order(side, price, quantity, order_type)` — Submit a new order
- `cancel_order(order_id)` — Cancel pending order and unlock funds

**Storage**

- `Orders` — Order metadata and status
- `Trades` — Trade history
- `Bids`/`Asks` — Active orderbook indexed by price level
- `UserOrders` — Per-user order tracking

### Substrate Pallet: Assets

Manages USDT and ETH balances with lock/unlock for active orders.

**Key Features**

- Deposit/withdraw trading assets
- Fund locking for order collateral
- Atomic settlement transfers
- Per-user free and locked balance tracking

### Indexer

Real-time listener that consumes Substrate events and maintains in-memory orderbook state.

**Features**

- Subscribes to chain events
- Maintains live orderbook state (bids/asks at all price levels with accumulated quantities)
- Logs all trades to TimescaleDB with Decimal precision
- Tracks order status changes and fills

**Database Schema**

Trades table with continuous materialized views for OHLCV candles at multiple timeframes. Built-in rollup logic aggregates data from lower to higher timeframes efficiently.

### Trade Bot

Synthetic order generator for load testing and market simulation.

**Features**

- Generates accounts with development keypairs
- Funds accounts with native tokens and trading assets
- Replays orders from JSON files with proper sequencing
- Per-account locking prevents nonce conflicts
- Graceful error handling and logging

## Configuration

### Environment Variables

```
NODE_WS_URL=ws://127.0.0.1:9944          # Substrate node endpoint
NUM_ACCOUNTS=20                             # Number of trading accounts
ORDER_DATA_FILE=ethusdt.jsonl               # Order file path
POSTGRES_USER=postgres
POSTGRES_PASSWORD=password
POSTGRES_DB=orbex
ENV=dev
RUST_LOG=info
INDEXER_PORT=8080
```

### Runtime Config

Edit runtime configuration in the Substrate node to adjust:

- `MaxPendingOrders` — Max orders queued per block
- `MaxCancellationOrders` — Max cancellations per block
- `MaxOrders` — Max orders per price level
- `MaxUserOrders` — Max orders per user

## Development

### Build

```bash
cargo build --release
```

### Run Local Node

```bash
./target/release/orbex-node --dev
```

### Purge Chain State

```bash
./target/release/orbex-node purge-chain --dev
```

### Database Queries

Access TimescaleDB directly:

```sql
-- Query 1m candles
SELECT * FROM one_minute_candles
WHERE symbol = 'ETH/USDT'
AND bucket >= NOW() - INTERVAL '1 hour'
ORDER BY bucket DESC;

-- Query aggregated data
SELECT bucket, high, low, volume FROM one_day_candles
WHERE symbol = 'ETH/USDT'
ORDER BY bucket DESC LIMIT 30;
```

## Example Flow

Alice deposits 10,000 USDT and places a buy order for 10 ETH at 100 USDT. Bob deposits 100 ETH and places a sell order for 10 ETH at 98 USDT. At block finalization, their orders match at Alice's limit price (100 USDT). Funds are transferred atomically: Alice receives 10 ETH, Bob receives 1,000 USDT. The trade is logged to TimescaleDB in real-time.

## API Reference

### Query Orders

```javascript
// Get order by ID
const order = await api.query.orderbook.orders(orderId);

// Get user's orders
const userOrders = await api.query.orderbook.userOrders(accountId);

// Get bids at price level
const bids = await api.query.orderbook.bids(priceLevel);

// Get asks at price level
const asks = await api.query.orderbook.asks(priceLevel);
```

### Listen to Events

```javascript
// Subscribe to trade events
api.query.system.events((events) => {
  events.forEach(({ event }) => {
    if (event.section === 'orderbook' && event.method === 'TradeExecuted') {
      console.log('Trade:', event.data);
    }
  });
});
```

## Events

- `OrderPlaced` — Order submitted to chain
- `TradeExecuted` — Trade matched and settled
- `OrderFilled` — Order completely filled
- `OrderPartiallyFilled` — Order partially filled
- `OrderCancelled` — Cancellation executed
- `CancellationRequested` — Cancellation queued
- `MatchingCompleted` — Block finalization summary

## Roadmap

- Order pruning with TTL-based expiry
- Stop-loss and take-profit orders
- Good-til-cancelled (GTC) orders
- Fill-or-kill (FOK) orders
- Immediate-or-cancel (IOC) orders
- Advanced indexer query API
- REST gateway for orderbook data

## Stack

- **Blockchain:** Substrate with custom FRAME pallets
- **Indexer:** Rust + subxt + tokio
- **Database:** PostgreSQL + TimescaleDB with continuous aggregates
- **Testing:** Synthetic order bot with load generation
- **Deployment:** Docker & Docker Compose

## License

Unlicense

## Resources

- [Substrate Documentation](https://docs.substrate.io/)
- [Polkadot-JS Apps](https://polkadot.js.org/apps/)
- [TimescaleDB Docs](https://docs.timescale.com/)
- [subxt Documentation](https://docs.rs/subxt/latest/subxt/)
