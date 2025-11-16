/**
 * Types for Indexer WebSocket and REST API
 * Matches the Rust backend types from indexer/src/api/websocket/messages.rs
 */

// Price level for orderbook (Hyperliquid format)
export interface WsPriceLevel {
  px: string  // Price
  sz: string  // Size
  n: number   // Number of orders
}

// Orderbook update message
export interface OrderbookUpdate {
  symbol: string
  time: number  // timestamp in milliseconds
  levels: [WsPriceLevel[], WsPriceLevel[]]  // [bids, asks]
}

// OHLCV candle update (Hyperliquid format)
export interface CandleUpdate {
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

// Status message
export interface StatusMessage {
  message: string
}

// Unified WebSocket message envelope
export type MarketDataMessage =
  | { type: "orderbook"; symbol: string; time: number; levels: [WsPriceLevel[], WsPriceLevel[]] }
  | { type: "candle"; T: number; t: number; o: string; h: string; l: string; c: string; v: string; i: string; s: string; n: number }
  | { type: "status"; message: string }

// REST API query parameters
export interface CandleQuery {
  symbol: string
  start_time: number  // Unix timestamp in seconds
  end_time: number    // Unix timestamp in seconds
  interval: string    // "1m", "5m", "15m", "30m", "1h", "4h", "1d", "1w", "1M"
}

// TradingView Bar format
export interface TradingViewBar {
  time: number      // Unix timestamp in seconds
  open: number
  high: number
  low: number
  close: number
  volume?: number
}
