# Indexer Integration Guide

This guide explains how to use the indexer WebSocket and REST API integration in the frontend.

## Overview

The indexer provides real-time and historical market data through:
- **WebSocket**: Real-time orderbook and OHLCV updates
- **REST API**: Historical OHLCV candles and orderbook snapshots

## Environment Configuration

Add these variables to your `.env` file:

```bash
INDEXER_URL=http://localhost:3000
INDEXER_WS_URL=ws://localhost:3000
```

The environment variables are validated using Zod in [lib/env.ts](lib/env.ts).

## Components

### 1. IndexerChart

A custom candlestick chart that displays real-time OHLCV data from the indexer.

```tsx
import { IndexerChart } from "@/components/indexer-chart"

<IndexerChart symbol="ETH/USDC" interval="15m" />
```

**Props:**
- `symbol`: Trading pair (e.g., "ETH/USDC")
- `interval`: Timeframe ("1m", "5m", "15m", "30m", "1h", "4h", "1d", "1w", "1M")

**Features:**
- Fetches historical data on mount
- Subscribes to real-time updates via WebSocket
- Auto-updates chart with new candles
- Canvas-based rendering for performance

### 2. OrderBook with Indexer

The OrderBook component now supports real indexer data.

```tsx
import { OrderBook } from "@/components/order-book"

<OrderBook useIndexer={true} symbol="ETH/USDC" />
```

**Props:**
- `useIndexer`: Enable indexer data (default: true)
- `symbol`: Trading pair (default: "ETH/USDC")

### 3. TradingDashboardWithIndexer

A complete trading dashboard with indexer integration.

```tsx
import { TradingDashboardWithIndexer } from "@/components/trading-dashboard-with-indexer"

<TradingDashboardWithIndexer />
```

**Features:**
- Toggle between custom indexer chart and TradingView widget
- Real-time orderbook updates
- Interval selection (1m, 5m, 15m, 30m, 1h, 4h, 1d)
- Live price updates

## API Clients

### IndexerRestClient

HTTP client for historical data.

```typescript
import { IndexerRestClient } from "@/lib/indexer/rest-client"

const client = new IndexerRestClient("http://localhost:3000")

// Get historical candles
const candles = await client.getCandles({
  symbol: "ETH/USDC",
  start_time: 1699000000,  // Unix timestamp in seconds
  end_time: 1699086400,
  interval: "15m",
})

// Get candles in TradingView format
const bars = await client.getCandlesAsTvBars({
  symbol: "ETH/USDC",
  start_time: 1699000000,
  end_time: 1699086400,
  interval: "1h",
})

// Get current orderbook snapshot
const orderbook = await client.getOrderbook()

// Health check
const isHealthy = await client.healthCheck()
```

### IndexerWebSocketClient

WebSocket client for real-time updates.

```typescript
import { IndexerWebSocketClient } from "@/lib/indexer/websocket-client"

const client = new IndexerWebSocketClient("ws://localhost:3000", {
  orderbook: true,
  ohlcv: true,
  symbol: "ETH/USDC",
  timeframes: ["1m", "5m", "15m"],
})

// Connect
await client.connect()

// Subscribe to all market data
const unsubscribe = client.onMessage((message) => {
  console.log("Market data:", message)
})

// Subscribe to orderbook updates only
client.onOrderbook((orderbook) => {
  console.log("Orderbook update:", orderbook)
})

// Subscribe to candle updates only
client.onCandle((candle) => {
  console.log("New candle:", candle)
})

// Cleanup
client.disconnect()
unsubscribe()
```

## Custom Hooks

### useIndexerOrderbook

React hook for real-time orderbook data.

```typescript
import { useIndexerOrderbook } from "@/hooks/use-indexer-orderbook"

function MyComponent() {
  const orderbook = useIndexerOrderbook("ws://localhost:3000", "ETH/USDC")

  if (!orderbook) {
    return <div>Loading...</div>
  }

  return (
    <div>
      <h3>Asks</h3>
      {orderbook.asks.map((ask, i) => (
        <div key={i}>{ask.price} - {ask.size}</div>
      ))}

      <h3>Spread: {orderbook.spread} ({orderbook.spreadPercent}%)</h3>

      <h3>Bids</h3>
      {orderbook.bids.map((bid, i) => (
        <div key={i}>{bid.price} - {bid.size}</div>
      ))}
    </div>
  )
}
```

## Data Types

### OrderbookUpdate

```typescript
interface OrderbookUpdate {
  symbol: string
  time: number  // timestamp in milliseconds
  levels: [WsPriceLevel[], WsPriceLevel[]]  // [bids, asks]
}

interface WsPriceLevel {
  px: string  // Price
  sz: string  // Size
  n: number   // Number of orders
}
```

### CandleUpdate

```typescript
interface CandleUpdate {
  T: number    // End time in milliseconds
  t: number    // Start time in milliseconds
  o: string    // Open price
  h: string    // High price
  l: string    // Low price
  c: string    // Close price
  v: string    // Volume
  i: string    // Interval (e.g., "1m", "5m", "1h")
  s: string    // Symbol
  n: number    // Number of trades
}
```

### TradingViewBar

```typescript
interface TradingViewBar {
  time: number      // Unix timestamp in seconds
  open: number
  high: number
  low: number
  close: number
  volume?: number
}
```

## TradingView Datafeed (Advanced)

For full TradingView Charting Library integration:

```typescript
import { IndexerTradingViewDatafeed } from "@/lib/indexer/tradingview-datafeed"

const datafeed = new IndexerTradingViewDatafeed(
  "http://localhost:3000",
  "ws://localhost:3000"
)

// Use with TradingView Charting Library
const widget = new TradingView.widget({
  datafeed: datafeed,
  symbol: "ETH/USDC",
  interval: "15",
  // ... other options
})
```

**Note**: This requires the TradingView Charting Library license and installation.

## Indexer API Endpoints

### WebSocket

**Endpoint**: `ws://localhost:3000/ws/market`

**Query Parameters**:
- `orderbook`: Subscribe to orderbook (default: true)
- `ohlcv`: Subscribe to OHLCV (default: true)
- `symbol`: Symbol filter (default: "ETH/USDC")
- `timeframes`: Comma-separated timeframes (e.g., "1m,5m")

**Example**:
```
ws://localhost:3000/ws/market?orderbook=true&ohlcv=true&symbol=ETH/USDC&timeframes=1m,5m,15m
```

### REST API

**Get Candles**: `GET /api/candles`

Query parameters:
- `symbol`: Trading pair (required)
- `start_time`: Start timestamp in seconds (required)
- `end_time`: End timestamp in seconds (required)
- `interval`: Timeframe (required)

**Example**:
```
GET /api/candles?symbol=ETH/USDC&start_time=1699000000&end_time=1699086400&interval=15m
```

**Get Orderbook**: `GET /api/orderbook`

**Health Check**: `GET /health`

## Usage Example

To use the indexer integration in your app:

1. **Update your main page** ([app/page.tsx](app/page.tsx)):

```tsx
import { TradingDashboardWithIndexer } from "@/components/trading-dashboard-with-indexer"

export default function Home() {
  return (
    <main className="min-h-screen bg-background">
      <TradingDashboardWithIndexer />
    </main>
  )
}
```

2. **Or use individual components**:

```tsx
import { IndexerChart } from "@/components/indexer-chart"
import { OrderBook } from "@/components/order-book"

export default function TradingPage() {
  return (
    <div className="flex gap-4">
      <div className="flex-1">
        <IndexerChart symbol="ETH/USDC" interval="15m" />
      </div>
      <div className="w-96">
        <OrderBook useIndexer={true} symbol="ETH/USDC" />
      </div>
    </div>
  )
}
```

## Troubleshooting

### WebSocket Connection Issues

If WebSocket connections fail:

1. Check that the indexer is running: `http://localhost:3000/health`
2. Verify environment variables are set correctly
3. Check browser console for connection errors
4. Ensure firewall allows WebSocket connections

### No Data Showing

If components show "No data available":

1. Verify the indexer has processed blockchain events
2. Check that the symbol matches exactly (e.g., "ETH/USDC")
3. Look at browser network tab for failed API requests
4. Check indexer logs for errors

### Performance Issues

For better performance:

1. Limit the number of timeframes in WebSocket subscriptions
2. Use appropriate intervals (longer intervals = less data)
3. Keep historical data queries under 24-48 hours
4. Disconnect unused WebSocket clients

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend                             │
│                                                             │
│  ┌──────────────┐      ┌──────────────┐                   │
│  │ IndexerChart │      │  OrderBook   │                   │
│  └──────┬───────┘      └──────┬───────┘                   │
│         │                     │                            │
│         └─────────┬───────────┘                            │
│                   │                                         │
│         ┌─────────▼─────────┐                              │
│         │  REST Client /    │                              │
│         │  WS Client        │                              │
│         └─────────┬─────────┘                              │
└───────────────────┼─────────────────────────────────────────┘
                    │
                    │ HTTP / WebSocket
                    │
┌───────────────────▼─────────────────────────────────────────┐
│                      Indexer (Rust)                         │
│                                                             │
│  ┌───────────┐    ┌────────────┐    ┌──────────────┐      │
│  │ WS Server │    │ REST API   │    │ Event        │      │
│  │ (Axum)    │    │ (Axum)     │    │ Processor    │      │
│  └───────────┘    └────────────┘    └──────────────┘      │
│                                                             │
│         ┌──────────────────────────────────┐               │
│         │   TimescaleDB / PostgreSQL       │               │
│         │   (OHLCV Aggregates + Orders)    │               │
│         └──────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────┘
```

## Next Steps

1. Start the indexer: `cargo run --release --bin orderbook-indexer`
2. Start the frontend: `npm run dev`
3. Open http://localhost:3000
4. Toggle between TradingView and Indexer chart
5. Watch real-time updates!
