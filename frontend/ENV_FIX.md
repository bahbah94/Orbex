# Environment Variables Fix

## Issue

The environment variables weren't accessible because Next.js requires client-side environment variables to be prefixed with `NEXT_PUBLIC_`.

## What Was Fixed

### 1. Environment Variables (`.env`)
All variables now have the `NEXT_PUBLIC_` prefix:

```bash
NEXT_PUBLIC_NODE_WS_URL=ws://127.0.0.1:9944
NEXT_PUBLIC_SERVER_URL=http://localhost:4000
NEXT_PUBLIC_INDEXER_URL=http://localhost:3000
NEXT_PUBLIC_INDEXER_WS_URL=ws://localhost:3000
NEXT_PUBLIC_APP_ENV=dev
```

### 2. Environment Schema (`lib/env.ts`)
Updated to:
- Validate `NEXT_PUBLIC_*` variables
- Export clean names via `env` object:
  ```typescript
  import { env } from "@/lib/env"

  // Use like this:
  env.INDEXER_URL        // instead of process.env.NEXT_PUBLIC_INDEXER_URL
  env.INDEXER_WS_URL     // instead of process.env.NEXT_PUBLIC_INDEXER_WS_URL
  ```

### 3. Port Configuration (`package.json`)
Frontend now runs on port **4000**:
```json
{
  "scripts": {
    "dev": "next dev -p 4000",
    "start": "next start -p 4000"
  }
}
```

## Port Summary

- **Indexer**: `http://localhost:3000` (WebSocket: `ws://localhost:3000`)
- **Frontend**: `http://localhost:4000`
- **Polkadot Node**: `ws://127.0.0.1:9944`

## How to Run

1. **Start Indexer** (runs on port 3000):
   ```bash
   cd indexer
   cargo run --release
   ```

2. **Start Frontend** (runs on port 4000):
   ```bash
   cd frontend
   npm run dev
   ```

3. **Access the app**:
   - Frontend: http://localhost:4000
   - Test page: http://localhost:4000/test-indexer
   - Indexer health: http://localhost:3000/health

## No Action Required

Everything is already fixed! Just restart your frontend dev server and it should work:

```bash
cd frontend
npm run dev
```

The app will now start on port 4000 and correctly connect to the indexer on port 3000.
