/**
 * WebSocket client for Indexer real-time market data
 * Handles orderbook and OHLCV updates
 */

import type { MarketDataMessage, OrderbookUpdate, CandleUpdate } from "./types"

export type MarketDataCallback = (message: MarketDataMessage) => void
export type OrderbookCallback = (update: OrderbookUpdate) => void
export type CandleCallback = (update: CandleUpdate) => void

interface SubscriptionOptions {
  orderbook?: boolean
  ohlcv?: boolean
  symbol?: string
  timeframes?: string[]  // e.g., ["1m", "5m"]
}

export class IndexerWebSocketClient {
  private ws: WebSocket | null = null
  private url: string
  private callbacks: Set<MarketDataCallback> = new Set()
  private orderbookCallbacks: Set<OrderbookCallback> = new Set()
  private candleCallbacks: Set<CandleCallback> = new Set()
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000
  private options: SubscriptionOptions

  constructor(baseUrl: string, options: SubscriptionOptions = {}) {
    const {
      orderbook = true,
      ohlcv = true,
      symbol = "ETH/USDC",
      timeframes = ["1m", "5m", "15m", "1h"],
    } = options

    this.options = { orderbook, ohlcv, symbol, timeframes }

    // Build WebSocket URL with query parameters
    const params = new URLSearchParams()
    if (orderbook !== undefined) params.set("orderbook", String(orderbook))
    if (ohlcv !== undefined) params.set("ohlcv", String(ohlcv))
    if (symbol) params.set("symbol", symbol)
    if (timeframes && timeframes.length > 0) params.set("timeframes", timeframes.join(","))

    this.url = `${baseUrl}/ws/market?${params.toString()}`
  }

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url)

        this.ws.onopen = () => {
          console.log("📡 Connected to indexer WebSocket:", this.url)
          this.reconnectAttempts = 0
          resolve()
        }

        this.ws.onmessage = (event) => {
          try {
            const message: MarketDataMessage = JSON.parse(event.data)

            // Notify all general callbacks
            this.callbacks.forEach((cb) => cb(message))

            // Notify specific callbacks
            if (message.type === "orderbook") {
              const update: OrderbookUpdate = {
                symbol: message.symbol,
                time: message.time,
                levels: message.levels,
              }
              this.orderbookCallbacks.forEach((cb) => cb(update))
            } else if (message.type === "candle") {
              const update: CandleUpdate = {
                T: message.T,
                t: message.t,
                o: message.o,
                h: message.h,
                l: message.l,
                c: message.c,
                v: message.v,
                i: message.i,
                s: message.s,
                n: message.n,
              }
              this.candleCallbacks.forEach((cb) => cb(update))
            } else if (message.type === "status") {
              console.log("📊 Indexer status:", message.message)
            }
          } catch (error) {
            console.error("Failed to parse WebSocket message:", error)
          }
        }

        this.ws.onerror = (error) => {
          console.error("WebSocket error:", error)
          reject(error)
        }

        this.ws.onclose = (event) => {
          console.log("WebSocket closed:", event.code, event.reason)
          this.attemptReconnect()
        }
      } catch (error) {
        reject(error)
      }
    })
  }

  private attemptReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error("Max reconnection attempts reached. Giving up.")
      return
    }

    this.reconnectAttempts++
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)

    console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`)

    setTimeout(() => {
      this.connect().catch((error) => {
        console.error("Reconnection failed:", error)
      })
    }, delay)
  }

  disconnect() {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }

  // Subscribe to all market data messages
  onMessage(callback: MarketDataCallback): () => void {
    this.callbacks.add(callback)
    return () => this.callbacks.delete(callback)
  }

  // Subscribe to orderbook updates only
  onOrderbook(callback: OrderbookCallback): () => void {
    this.orderbookCallbacks.add(callback)
    return () => this.orderbookCallbacks.delete(callback)
  }

  // Subscribe to candle updates only
  onCandle(callback: CandleCallback): () => void {
    this.candleCallbacks.add(callback)
    return () => this.candleCallbacks.delete(callback)
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN
  }
}
