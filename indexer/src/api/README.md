# 🔥 Orderbook Indexer Backend API

High-performance REST API + WebSocket server for real-time orderbook data and trade execution.

---

## 🏗️ Architecture

```
┌─────────────────┐
│  Substrate Node │
│   (Orderbook)   │
└────────┬────────┘
         │ Events
         ↓
┌─────────────────┐
│ Event Indexer   │
│ (subxt listener)│
└────────┬────────┘
         │ Updates
         ↓
┌─────────────────┐      ┌──────────────┐
│ OrderbookState  │◄─────┤  PostgreSQL  │
│  (In-Memory)    │      │   (Trades)   │
└────────┬────────┘      └──────────────┘
         │ Read
         ↓
┌─────────────────┐
│  Axum Server    │
│  REST + WS      │
└────────┬────────┘
         │
         ↓
    Frontend App
```

---

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- PostgreSQL 14+
- Running Substrate node with orderbook pallet

### Install & Run
```bash
# Clone and navigate
cd indexer

# Set environment variables
export DATABASE_URL="postgresql://user:pass@localhost/orderbook"
export WS_ENDPOINT="ws://127.0.0.1:9944"

# Run migrations
sqlx migrate run

# Start server
cargo run --release
```

Server runs on: `http://localhost:3000`

---

## 📡 REST API Endpoints

### Market Data

#### `GET /api/orderbook`
Get current orderbook snapshot with bids and asks.

**Response:**
```json
{
  "bids": [
    {
      "price": "100.50",
      "quantity": "10.5",
      "orders": 2
    }
  ],
  "asks": [
    {
      "price": "101.00",
      "quantity": "5.0",
      "orders": 1
    }
  ],
  "spread": "0.50",
  "timestamp": 1698765432
}
```

---

#### `GET /api/trades?limit=50&offset=0`
Get recent trades (paginated).

**Query Parameters:**
- `limit` (optional): Number of trades (default: 50, max: 1000)
- `offset` (optional): Pagination offset (default: 0)

**Response:**
```json
{
  "trades": [
    {
      "id": 123,
      "price": "100.50",
      "quantity": "2.5",
      "buyer": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
      "seller": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
      "timestamp": 1698765432,
      "block_number": 12345
    }
  ],
  "total": 500
}
```

---

#### `GET /api/trades/:id`
Get specific trade by ID.

**Response:**
```json
{
  "id": 123,
  "price": "100.50",
  "quantity": "2.5",
  "buyer": "5GrwvaEF...",
  "seller": "5FHneW46...",
  "buy_order_id": 456,
  "sell_order_id": 789,
  "value": "251.25",
  "timestamp": 1698765432,
  "block_number": 12345
}
```

---

#### `GET /api/ohlcv?interval=1m&limit=100`
Get OHLCV candlestick data.

**Query Parameters:**
- `interval`: `1m`, `5m`, `15m`, `1h`, `4h`, `1d` (required)
- `limit`: Number of candles (default: 100, max: 1000)

**Response:**
```json
{
  "candles": [
    {
      "timestamp": 1698765000,
      "open": "100.00",
      "high": "102.50",
      "low": "99.50",
      "close": "101.00",
      "volume": "1250.75"
    }
  ]
}
```

---

#### `GET /api/stats`
Get 24h market statistics.

**Response:**
```json
{
  "last_price": "101.00",
  "price_change_24h": "2.50",
  "price_change_percent_24h": "2.54",
  "high_24h": "105.00",
  "low_24h": "98.00",
  "volume_24h": "15000.50",
  "trades_count_24h": 234
}
```

---

#### `GET /api/order/:id`
Get order details by order ID.

**Response:**
```json
{
  "order_id": 456,
  "trader": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "side": "Buy",
  "asset_id": 1,
  "price": "100.50",
  "quantity": "10.0",
  "filled_quantity": "2.5",
  "remaining_quantity": "7.5",
  "status": "PartiallyFilled",
  "timestamp": 1698765432
}
```

---

### Order Management

#### `POST /api/place-order`
Submit new order to blockchain.

**Request Body:**
```json
{
  "side": "Buy",
  "asset_id": 1,
  "price": "100.50",
  "quantity": "10.0",
  "signer_seed": "//Alice"
}
```

**Response:**
```json
{
  "success": true,
  "order_id": 789,
  "tx_hash": "0x1234...",
  "message": "Order placed successfully"
}
```

---

#### `POST /api/cancel-order`
Cancel existing order.

**Request Body:**
```json
{
  "order_id": 789,
  "signer_seed": "//Alice"
}
```

**Response:**
```json
{
  "success": true,
  "tx_hash": "0x5678...",
  "message": "Order cancelled successfully"
}
```

---

## 🔴 WebSocket Channels

### `/ws/orderbook`
Real-time orderbook updates.

**Message Format:**
```json
{
  "type": "orderbook_update",
  "data": {
    "side": "Buy",
    "price": "100.50",
    "quantity": "10.0",
    "action": "add" | "update" | "remove"
  },
  "timestamp": 1698765432
}
```

**Subscribe:**
```javascript
const ws = new WebSocket('ws://localhost:3000/ws/orderbook');
ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Orderbook update:', update);
};
```

---

### `/ws/trades`
Real-time trade stream.

**Message Format:**
```json
{
  "type": "trade",
  "data": {
    "price": "100.50",
    "quantity": "2.5",
    "side": "buy",
    "value": "251.25",
    "timestamp": 1698765432
  }
}
```

---

### `/ws/ticker`
Real-time price ticker.

**Message Format:**
```json
{
  "type": "ticker",
  "data": {
    "last_price": "101.00",
    "price_change_24h": "2.50",
    "volume_24h": "15000.50"
  },
  "timestamp": 1698765432
}
```

---

### `/ws/user/:account`
User-specific order updates.

**Message Format:**
```json
{
  "type": "user_order_update",
  "data": {
    "order_id": 456,
    "status": "PartiallyFilled",
    "filled_quantity": "2.5",
    "remaining_quantity": "7.5"
  },
  "timestamp": 1698765432
}
```

**Subscribe:**
```javascript
const account = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const ws = new WebSocket(`ws://localhost:3000/ws/user/${account}`);
```

---

## 🛠️ Configuration

### Environment Variables

```bash
# Database
DATABASE_URL="postgresql://user:pass@localhost:5432/orderbook"

# Substrate Node
WS_ENDPOINT="ws://127.0.0.1:9944"

# API Server
API_HOST="0.0.0.0"
API_PORT="3000"

# CORS
ALLOWED_ORIGINS="http://localhost:3001,http://localhost:5173"
```

---

## 📊 Data Flow

```
Blockchain Event → Indexer → In-Memory State
                                    ↓
                          ┌─────────┴─────────┐
                          ↓                   ↓
                    REST Endpoints      WebSocket Broadcast
                          ↓                   ↓
                       Frontend          Live Updates
```

---

## 🔧 Development

### Project Structure
```
api/
├── mod.rs              # Module exports
├── routes.rs           # REST route definitions
├── handlers.rs         # Request handlers
└── websocket.rs        # WebSocket handlers

services/
├── mod.rs
├── market_stats.rs     # 24h statistics calculator
├── candle_builder.rs   # OHLCV aggregator
└── extrinsic_service.rs # Blockchain interaction

models/
├── mod.rs
└── api_types.rs        # Request/response types
```

### Adding New Endpoints

1. **Define route** in `api/routes.rs`:
```rust
.route("/api/new-endpoint", get(handlers::new_handler))
```

2. **Implement handler** in `api/handlers.rs`:
```rust
pub async fn new_handler(
    State(state): State<Arc<Mutex<OrderbookState>>>
) -> impl IntoResponse {
    // Handler logic
}
```

3. **Update types** in `models/api_types.rs` if needed.

---

## 🚨 Error Handling

All endpoints return consistent error format:

```json
{
  "error": "Error description",
  "code": "ERROR_CODE",
  "timestamp": 1698765432
}
```

**Common Error Codes:**
- `INVALID_PARAMETERS` - Bad request parameters
- `ORDER_NOT_FOUND` - Order ID doesn't exist
- `INSUFFICIENT_BALANCE` - Not enough funds
- `INTERNAL_ERROR` - Server error

---

## 📈 Performance

- **In-Memory State**: O(log n) orderbook lookups via BTreeMap
- **WebSocket**: Async broadcast to connected clients
- **Database**: Indexed trades table for fast queries
- **Connection Pooling**: SQLx for efficient DB access

---

## 🔐 Security Considerations

- **CORS**: Configure allowed origins
- **Rate Limiting**: TODO - Add rate limiting middleware
- **Input Validation**: All inputs validated before processing
- **Extrinsic Signing**: Use secure key management (not production-ready with seed phrases)

---

## 🧪 Testing

```bash
# Run tests
cargo test

# Test WebSocket connection
websocat ws://localhost:3000/ws/trades

# Load test REST endpoints
ab -n 1000 -c 10 http://localhost:3000/api/orderbook
```

---

## 📝 TODO

- [ ] Add rate limiting middleware
- [ ] Implement authentication for order submission
- [ ] Add GraphQL endpoint
- [ ] Health check endpoint (`/health`)
- [ ] Metrics endpoint (`/metrics` for Prometheus)
- [ ] Add caching layer (Redis)
- [ ] WebSocket authentication
- [ ] API documentation UI (Swagger/OpenAPI)

---

## 🤝 Contributing

1. Create feature branch from `backend`
2. Implement changes with tests
3. Submit PR with description

---

## 📄 License

MIT License - See LICENSE file for details

---

**Built with using Rust + Axum + subxt**