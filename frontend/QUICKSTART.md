# Quick Start: Indexer Integration

## TL;DR

I've integrated your indexer WebSocket and REST APIs with the frontend. Here's how to use it:

## 1. Environment Setup

Your `.env` file has been updated with:
```bash
INDEXER_URL=http://localhost:3000
INDEXER_WS_URL=ws://localhost:3000
```

## 2. Start Everything

```bash
# Terminal 1: Start indexer (if not already running)
cd indexer
cargo run --release

# Terminal 2: Start frontend
cd frontend
npm run dev
```

## 3. Use the New Components

### Option A: Quick Switch (Easiest)

Update `frontend/app/page.tsx`:

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

This gives you:
- ✅ Real-time orderbook from indexer
- ✅ Real-time OHLCV chart from indexer
- ✅ Toggle between TradingView widget and indexer chart
- ✅ Multiple timeframes (1m, 5m, 15m, 30m, 1h, 4h, 1d)

### Option B: Use Individual Components

```tsx
import { IndexerChart } from "@/components/indexer-chart"
import { OrderBook } from "@/components/order-book"

<IndexerChart symbol="ETH/USDC" interval="15m" />
<OrderBook useIndexer={true} symbol="ETH/USDC" />
```

## What's New

### 📊 IndexerChart Component
- Real-time candlestick chart
- WebSocket updates
- Multiple timeframes
- Canvas-based (fast)
- Path: `frontend/components/indexer-chart.tsx`

### 📖 OrderBook with Live Data
- Now pulls from indexer WebSocket
- Real-time price level updates
- Automatic reconnection
- Path: `frontend/components/order-book.tsx`

### 🔌 API Clients

**REST Client**:
```typescript
import { IndexerRestClient } from "@/lib/indexer/rest-client"

const client = new IndexerRestClient(env.INDEXER_URL)
const candles = await client.getCandles({
  symbol: "ETH/USDC",
  start_time: Math.floor(Date.now() / 1000) - 86400,
  end_time: Math.floor(Date.now() / 1000),
  interval: "15m"
})
```

**WebSocket Client**:
```typescript
import { IndexerWebSocketClient } from "@/lib/indexer/websocket-client"

const ws = new IndexerWebSocketClient(env.INDEXER_WS_URL, {
  orderbook: true,
  ohlcv: true,
  symbol: "ETH/USDC",
  timeframes: ["1m", "5m", "15m"]
})

await ws.connect()
ws.onCandle((candle) => console.log(candle))
ws.onOrderbook((book) => console.log(book))
```

## File Structure

```
frontend/
├── lib/
│   ├── env.ts                           # ✅ Updated with indexer URLs
│   └── indexer/
│       ├── index.ts                     # ✨ New: Barrel export
│       ├── types.ts                     # ✨ New: TypeScript types
│       ├── rest-client.ts               # ✨ New: REST API client
│       ├── websocket-client.ts          # ✨ New: WebSocket client
│       └── tradingview-datafeed.ts      # ✨ New: TradingView integration
├── hooks/
│   └── use-indexer-orderbook.ts         # ✨ New: Orderbook hook
├── components/
│   ├── indexer-chart.tsx                # ✨ New: OHLCV chart
│   ├── order-book.tsx                   # ✅ Updated with indexer support
│   └── trading-dashboard-with-indexer.tsx  # ✨ New: Complete dashboard
├── .env                                 # ✅ Updated
├── INDEXER_INTEGRATION.md              # 📚 Full documentation
└── QUICKSTART.md                        # 📚 This file
```

## Testing

1. **Check indexer is running**:
   ```bash
   curl http://localhost:3000/health
   # Should return: OK
   ```

2. **Test REST API**:
   ```bash
   curl "http://localhost:3000/api/candles?symbol=ETH/USDC&start_time=1699000000&end_time=1699086400&interval=15m"
   ```

3. **Test WebSocket** (in browser console):
   ```javascript
   const ws = new WebSocket("ws://localhost:3000/ws/market?symbol=ETH/USDC")
   ws.onmessage = (e) => console.log(JSON.parse(e.data))
   ```

## Features

✅ **Real-time Orderbook**: Live bid/ask updates via WebSocket
✅ **Real-time OHLCV**: Candlestick chart updates as trades happen
✅ **Historical Data**: REST API for past candles
✅ **Multiple Timeframes**: 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, 1M
✅ **Type-safe**: Full TypeScript types matching Rust backend
✅ **Auto-reconnect**: WebSocket automatically reconnects on disconnect
✅ **Fallback**: Components gracefully fall back to mock data if indexer unavailable

## Troubleshooting

**WebSocket not connecting?**
- Check indexer is running: `curl http://localhost:3000/health`
- Check browser console for errors
- Verify `.env` has correct URLs

**No candles showing?**
- Indexer needs to process some trades first
- Check indexer logs for event processing
- Try a different time range

**Chart looks weird?**
- Make sure you have enough data (need at least a few candles)
- Try a different interval
- Check browser console for errors

## What Next?

1. Customize the chart styling in `components/indexer-chart.tsx`
2. Add more trading pairs to the selector
3. Implement TradingView Charting Library (requires license)
4. Add volume bars below the chart
5. Add technical indicators

For detailed documentation, see [INDEXER_INTEGRATION.md](INDEXER_INTEGRATION.md)
