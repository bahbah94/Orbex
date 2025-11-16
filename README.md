# Substrate Orderbook DEX

A decentralized exchange (DEX) built on Substrate with a limit orderbook and batch matching engine.

## Overview

This project implements a fully functional orderbook-based DEX on Substrate, featuring:

- **Limit & Market Orders**: Support for both order types with price-time priority matching
- **Batch Matching Engine**: Orders are matched once per block at finalization for optimal gas efficiency
- **Two-Phase Matching**: Pending orders match internally first, then with the persistent orderbook
- **Custom Assets Pallet**: Manages USDT and ETH balances with lock/unlock functionality
- **Atomic Settlement**: All trades are settled atomically with proper fund transfers

## Architecture

### Two-Phase Design

1. **Order Submission** (during block): Orders validated and queued in temporary cache
2. **Batch Matching** (on_finalize): All orders matched once, funds settled, cache cleared

### Benefits

- ✅ Constant-time order submission (no matching during extrinsic)
- ✅ Single matching pass per block (more efficient than per-order matching)
- ✅ Better price discovery (orders within same block match first)
- ✅ Prevents race conditions and MEV attacks

## Pallets

### 1. Assets Pallet (`pallets/assets`)

Manages user balances for trading assets (USDT and ETH).

**Key Features:**

- Deposit/withdraw funds
- Lock funds for active orders
- Unlock funds for cancellations
- Transfer locked funds for trade settlement

**Storage:**

- `FreeBalance`: Available user balances
- `LockedBalance`: Funds locked in active orders

### 2. Orderbook Pallet (`pallets/orderbook`)

Core DEX functionality with orderbook matching engine.

**Key Features:**

- Place limit and market orders
- Cancel pending orders
- Automatic batch matching at block finalization
- Price-time priority (FIFO within price levels)
- Partial order fills
- TTL-based order expiry

**Storage:**

- **Persistent:**
  - `Orders`: All order details
  - `Trades`: Trade history
  - `Bids`/`Asks`: Active orderbook (price → order IDs)
  - `UserOrders`: User's order list
- **Temporary Cache (cleared each block):**
  - `PendingBids`/`PendingAsks`: Orders submitted this block
  - `PendingCancellations`: Cancellation requests

**Extrinsics:**

- `place_order(side, price, quantity, order_type)`: Submit a new order
- `cancel_order(order_id)`: Cancel an existing order

### 3. Matching Engine (`pallets/orderbook/src/engine.rs`)

Pure matching logic separated from pallet for clarity.

**Functions:**

- `match_pending_internal`: Match pending orders amongst themselves
- `match_with_persistent`: Match unmatched orders with persistent orderbook
- `match_buy_order`/`match_sell_order`: Core matching logic with price-time priority
- `execute_trade`: Update order states and create trade records
- `process_cancellations`: Handle order cancellations

## Matching Flow

```
Block N starts
  ↓
Users submit orders via place_order()
  → Orders stored in Orders<T>
  → Added to PendingBids/PendingAsks cache
  → Funds locked
  ↓
Block N ends → on_finalize() triggered
  ↓
1. Process cancellations
   → Remove from orderbook
   → Unlock funds
  ↓
2. Match pending orders internally
   → Pending orders match with each other first
   → Returns trades + unmatched orders
  ↓
3. Match unmatched with persistent orderbook
   → Try to match survivors with existing orders
   → Add remainder to persistent orderbook
  ↓
4. Execute all trades
   → transfer_locked (buyer → seller: USDT)
   → transfer_locked (seller → buyer: ETH)
   → unlock_funds for both parties
   → Store trade records
   → Emit events
  ↓
5. Update storage
   → Save modified orders
   → Save persistent orderbook
   → Clear pending cache
  ↓
Block N+1 starts fresh
```

## Example Trade

```
Initial State:
  Alice: 10,000 USDT (free)
  Bob: 100 ETH (free)

Alice places order:
  place_order(Buy, 100 USDT, 10 ETH, Limit)
  → lock_funds(Alice, USDT, 1000)
  → Add to PendingBids

Bob places order:
  place_order(Sell, 98 USDT, 10 ETH, Limit)
  → lock_funds(Bob, ETH, 10)
  → Add to PendingAsks

on_finalize():
  1. Match pending orders
     → Bob's sell @ 98 matches Alice's buy @ 100
     → Execute at maker price: 100 USDT (Alice's limit)
  2. Settle trade
     → transfer_locked(Alice → Bob, USDT, 1000)
     → transfer_locked(Bob → Alice, ETH, 10)
     → unlock_funds(Bob, USDT, 1000)
     → unlock_funds(Alice, ETH, 10)

Final State:
  Alice: 9,000 USDT, 10 ETH
  Bob: 1,000 USDT, 90 ETH
```

## Types

### Order

```rust
pub struct Order<T: Config> {
    pub order_id: OrderId,
    pub trader: T::AccountId,
    pub side: OrderSide,           // Buy or Sell
    pub status: OrderStatus,       // Open, PartiallyFilled, Filled, Cancelled, Expired
    pub order_type: OrderType,     // Market or Limit
    pub price: Amount,             // Price per unit
    pub quantity: Amount,          // Total quantity
    pub filled_quantity: Amount,   // Amount filled so far
    pub ttl: Option<u32>,         // Time-to-live (blocks)
}
```

### Trade

```rust
pub struct Trade<T: Config> {
    pub trade_id: TradeId,
    pub buyer: T::AccountId,
    pub seller: T::AccountId,
    pub buy_order_id: OrderId,
    pub sell_order_id: OrderId,
    pub price: Amount,
    pub quantity: Amount,
}
```

## Getting Started

### Build

```sh
cargo build --release
```

### Run Development Chain

```sh
./target/release/orbex-node --dev
```

### Purge Chain State

```sh
./target/release/orbex-node purge-chain --dev
```

### Connect with Polkadot-JS Apps

Visit [Polkadot/Substrate Portal](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) and connect to your local node.

## Usage

### 1. Deposit Funds (Assets Pallet)

```javascript
// Deposit 10,000 USDT
api.tx.assets.deposit(0, 10000000000); // USDT = asset_id 0

// Deposit 100 ETH
api.tx.assets.deposit(1, 100000000000); // ETH = asset_id 1
```

### 2. Place Order (Orderbook Pallet)

```javascript
// Buy 10 ETH at 100 USDT each (limit order)
api.tx.orderbook.placeOrder(
  { Buy },           // side
  100000000000,      // price (100 USDT)
  10000000000,       // quantity (10 ETH)
  { Limit }          // order_type
);

// Sell 10 ETH at market price
api.tx.orderbook.placeOrder(
  { Sell },
  0,                 // price (ignored for market orders)
  10000000000,
  { Market }
);
```

### 3. Cancel Order

```javascript
// Cancel order by ID
api.tx.orderbook.cancelOrder(123);
```

### 4. Query Orders

```javascript
// Get order details
const order = await api.query.orderbook.orders(orderId);

// Get user's orders
const userOrders = await api.query.orderbook.userOrders(accountId);

// Get bids at price level
const bids = await api.query.orderbook.bids(100000000000);

// Get asks at price level
const asks = await api.query.orderbook.asks(100000000000);
```

## Events

- `OrderPlaced`: New order submitted
- `TradeExecuted`: Trade matched and executed
- `OrderFilled`: Order completely filled
- `OrderPartiallyFilled`: Order partially filled
- `OrderCancelled`: Order cancelled by user
- `CancellationRequested`: Cancellation queued for processing
- `MatchingCompleted`: Block finalization complete (summary stats)

## Configuration

Configure constants in your runtime:

```rust
impl pallet_orderbook::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxPendingOrders = ConstU32<1000>;        // Max orders per block
    type MaxCancellationOrders = ConstU32<100>;    // Max cancellations per block
    type MaxOrders = ConstU32<10000>;              // Max orders per price level
    type MaxUserOrders = ConstU32<1000>;           // Max orders per user
}
```

## TODO

- [ ] Order pruning/expiry logic (TTL-based)
- [ ] Mock runtime for testing
- [ ] Comprehensive unit tests
- [ ] Benchmark weights
- [ ] Stop-loss orders
- [ ] Good-til-cancelled (GTC) orders
- [ ] Fill-or-kill (FOK) orders
- [ ] Immediate-or-cancel (IOC) orders

## Key Design Decisions

- **Batch matching**: All orders matched once per block (not per-order)
- **Two-phase matching**: Pending orders match internally before checking persistent orderbook
- **OrderId-based storage**: Orderbook stores IDs, not full Order structs (saves space)
- **BoundedVec for cache**: DoS prevention on temporary storage
- **Vec for persistent**: Orders naturally accumulate, no hard limit
- **Price-time priority**: Standard exchange rules (best price first, FIFO within price level)

## License

Unlicense

## Resources

- [Substrate Documentation](https://docs.substrate.io/)
- [Polkadot-JS Apps](https://polkadot.js.org/apps/)
- [FRAME Development](https://docs.substrate.io/learn/runtime-development/)
