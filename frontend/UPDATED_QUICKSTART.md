# Quick Start: Indexer Integration (UPDATED)

## TL;DR

Environment variables are now properly configured with `NEXT_PUBLIC_` prefix.

## Ports

- **Indexer**: http://localhost:3000 (WebSocket: ws://localhost:3000)
- **Frontend**: http://localhost:4000

## 1. Environment Setup

Your `.env` file is configured with:
```bash
NEXT_PUBLIC_INDEXER_URL=http://localhost:3000
NEXT_PUBLIC_INDEXER_WS_URL=ws://localhost:3000
NEXT_PUBLIC_SERVER_URL=http://localhost:4000
NEXT_PUBLIC_APP_ENV=dev
```

## 2. Start Everything

```bash
# Terminal 1: Start indexer on port 3000
cargo run --release --bin orderbook-indexer

# Terminal 2: Start frontend on port 4000
cd frontend
npm run dev
```

## 3. Use the Integration

### Quick Test
Visit: http://localhost:4000/test-indexer

### Use the New Dashboard
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

Then visit: http://localhost:4000

## 4. Verify It Works

**Check indexer health:**
```bash
curl http://localhost:3000/health
# Should return: OK
```

**Check frontend:**
```bash
curl http://localhost:4000
# Should return HTML
```

**Test WebSocket (in browser console at http://localhost:4000):**
```javascript
const ws = new WebSocket("ws://localhost:3000/ws/market?symbol=ETH/USDC")
ws.onmessage = (e) => console.log(JSON.parse(e.data))
```

## Features

✅ Real-time orderbook updates via WebSocket
✅ Real-time OHLCV candlestick chart
✅ Historical data from REST API
✅ Multiple timeframes (1m, 5m, 15m, 30m, 1h, 4h, 1d)
✅ Toggle between TradingView and Indexer chart
✅ Runs on port 4000 (no conflict with indexer on port 3000)

## Using Individual Components

```tsx
import { IndexerChart } from "@/components/indexer-chart"
import { OrderBook } from "@/components/order-book"

// Real-time chart
<IndexerChart symbol="ETH/USDC" interval="15m" />

// Real-time orderbook
<OrderBook useIndexer={true} symbol="ETH/USDC" />
```

## Using API Clients Directly

```typescript
import { IndexerRestClient, IndexerWebSocketClient } from "@/lib/indexer"
import { env } from "@/lib/env"

// REST Client
const rest = new IndexerRestClient(env.INDEXER_URL)
const candles = await rest.getCandles({
  symbol: "ETH/USDC",
  start_time: Math.floor(Date.now() / 1000) - 3600,
  end_time: Math.floor(Date.now() / 1000),
  interval: "15m"
})

// WebSocket Client
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

## Troubleshooting

**"Invalid environment variables" error?**
- Make sure you restart the dev server after updating `.env`
- Variables should have `NEXT_PUBLIC_` prefix
- Check that `.env` file exists in `frontend/` directory

**WebSocket not connecting?**
- Indexer should be running on port 3000
- Check: `curl http://localhost:3000/health`
- Check browser console for connection errors

**Port already in use?**
- Port 3000: Stop other services using port 3000
- Port 4000: Change in `package.json` scripts

## What's Next?

1. Customize chart styling
2. Add more trading pairs
3. Implement volume bars
4. Add technical indicators
5. Mobile optimization

For detailed documentation:
- [ENV_FIX.md](ENV_FIX.md) - Environment variable fix details
- [INDEXER_INTEGRATION.md](INDEXER_INTEGRATION.md) - Full API documentation
- [INTEGRATION_SUMMARY.md](INTEGRATION_SUMMARY.md) - Architecture overview
