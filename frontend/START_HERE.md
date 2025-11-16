# 🚀 START HERE - Indexer Integration

## ✅ FIXED: Environment Variables Issue

The environment variables now work correctly with the `NEXT_PUBLIC_` prefix.

## Quick Start (2 Steps)

### 1. Start the Indexer (Port 3000)
```bash
cargo run --release --bin orderbook-indexer
```

### 2. Start the Frontend (Port 4000)
```bash
cd frontend
npm run dev
```

## URLs

- **Frontend**: http://localhost:4000
- **Test Page**: http://localhost:4000/test-indexer ← Start here!
- **Indexer API**: http://localhost:3000
- **Indexer Health**: http://localhost:3000/health

## What's Working

✅ Environment variables configured (`NEXT_PUBLIC_*` prefix)
✅ Frontend runs on port 4000 (no conflict with indexer)
✅ WebSocket client for real-time orderbook updates
✅ REST client for historical OHLCV data
✅ Custom candlestick chart component
✅ OrderBook component with live data
✅ Test page to verify everything works

## Test It

1. **Quick Health Check**:
   ```bash
   curl http://localhost:3000/health
   # Should return: OK
   ```

2. **Visit Test Page**:
   Open http://localhost:4000/test-indexer
   - Should show green checkmarks
   - Should display live chart
   - Should display live orderbook

3. **Use the New Dashboard** (Optional):

   Edit `frontend/app/page.tsx`:
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

## File Structure

```
frontend/
├── .env                              ✅ Fixed (NEXT_PUBLIC_* vars)
├── package.json                      ✅ Fixed (port 4000)
├── lib/
│   ├── env.ts                        ✅ Fixed (validates NEXT_PUBLIC_* vars)
│   └── indexer/                      ✨ NEW
│       ├── types.ts                  - TypeScript types
│       ├── rest-client.ts            - REST API client
│       ├── websocket-client.ts       - WebSocket client
│       ├── tradingview-datafeed.ts   - TradingView integration
│       └── index.ts                  - Exports
├── hooks/
│   └── use-indexer-orderbook.ts      ✨ NEW - Orderbook hook
├── components/
│   ├── indexer-chart.tsx             ✨ NEW - OHLCV chart
│   ├── order-book.tsx                ✅ Updated - Indexer support
│   └── trading-dashboard-with-indexer.tsx  ✨ NEW - Complete UI
└── app/
    └── test-indexer/
        └── page.tsx                  ✨ NEW - Test page
```

## What Was Fixed

### Issue
```
❌ Invalid environment variables:
  - SERVER_URL: Required
  - INDEXER_URL: Required
  - INDEXER_WS_URL: Required
```

### Solution
1. Added `NEXT_PUBLIC_` prefix to all environment variables
2. Updated `lib/env.ts` to validate `NEXT_PUBLIC_*` variables
3. Changed frontend port from 3000 to 4000
4. Updated all imports to use clean names via `env` export

### Before
```bash
# .env (WRONG - doesn't work in Next.js client)
INDEXER_URL=http://localhost:3000
```

### After
```bash
# .env (CORRECT - works in Next.js client)
NEXT_PUBLIC_INDEXER_URL=http://localhost:3000
```

## Documentation

- **[START_HERE.md](START_HERE.md)** ← You are here
- **[ENV_FIX.md](ENV_FIX.md)** - Details about the environment fix
- **[UPDATED_QUICKSTART.md](UPDATED_QUICKSTART.md)** - Updated quick start guide
- **[INDEXER_INTEGRATION.md](INDEXER_INTEGRATION.md)** - Full API documentation
- **[INTEGRATION_SUMMARY.md](INTEGRATION_SUMMARY.md)** - Architecture overview

## Components You Can Use

### IndexerChart - Real-time Candlestick Chart
```tsx
import { IndexerChart } from "@/components/indexer-chart"

<IndexerChart symbol="ETH/USDC" interval="15m" />
```

### OrderBook - Real-time Order Book
```tsx
import { OrderBook } from "@/components/order-book"

<OrderBook useIndexer={true} symbol="ETH/USDC" />
```

### Full Dashboard
```tsx
import { TradingDashboardWithIndexer } from "@/components/trading-dashboard-with-indexer"

<TradingDashboardWithIndexer />
```

## API Clients

### REST Client
```typescript
import { IndexerRestClient } from "@/lib/indexer"
import { env } from "@/lib/env"

const client = new IndexerRestClient(env.INDEXER_URL)
const candles = await client.getCandles({
  symbol: "ETH/USDC",
  start_time: Math.floor(Date.now() / 1000) - 3600,
  end_time: Math.floor(Date.now() / 1000),
  interval: "15m"
})
```

### WebSocket Client
```typescript
import { IndexerWebSocketClient } from "@/lib/indexer"
import { env } from "@/lib/env"

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

### "Invalid environment variables" error?
**Solution**: Restart your dev server
```bash
# Press Ctrl+C to stop
# Then start again:
npm run dev
```

### WebSocket not connecting?
**Check**: Is the indexer running?
```bash
curl http://localhost:3000/health
```

### Port already in use?
**Frontend (4000)**:
```bash
lsof -ti:4000 | xargs kill -9  # Kill process on port 4000
npm run dev
```

**Indexer (3000)**:
```bash
lsof -ti:3000 | xargs kill -9  # Kill process on port 3000
```

## Next Steps

1. ✅ Visit test page: http://localhost:4000/test-indexer
2. ✅ Verify green checkmarks
3. ✅ Watch live chart and orderbook update
4. 🎨 Customize the dashboard (optional)
5. 🚀 Build your trading features!

## Support

If you need help:
1. Check the [ENV_FIX.md](ENV_FIX.md) for environment variable details
2. Read [UPDATED_QUICKSTART.md](UPDATED_QUICKSTART.md) for usage examples
3. See [INDEXER_INTEGRATION.md](INDEXER_INTEGRATION.md) for complete API docs

---

**Everything is ready! Just run `npm run dev` in the frontend directory.** 🎉
