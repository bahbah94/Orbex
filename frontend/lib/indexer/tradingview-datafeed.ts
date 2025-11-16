/**
 * TradingView Charting Library Datafeed Implementation
 * Connects TradingView charts to the Indexer API for real-time and historical OHLCV data
 */

import { IndexerRestClient } from "./rest-client"
import { IndexerWebSocketClient } from "./websocket-client"
import type { TradingViewBar, CandleUpdate } from "./types"

// TradingView resolution to indexer interval mapping
const RESOLUTION_MAP: Record<string, string> = {
  "1": "1m",
  "5": "5m",
  "15": "15m",
  "30": "30m",
  "60": "1h",
  "240": "4h",
  "D": "1d",
  "1D": "1d",
  "W": "1w",
  "1W": "1w",
  "M": "1M",
  "1M": "1M",
}

interface SubscriberInfo {
  symbolInfo: any
  resolution: string
  lastBar: TradingViewBar | null
  listener: (bar: TradingViewBar) => void
}

/**
 * TradingView Datafeed implementation using Indexer API
 */
export class IndexerTradingViewDatafeed {
  private restClient: IndexerRestClient
  private wsClient: IndexerWebSocketClient | null = null
  private subscribers: Map<string, SubscriberInfo> = new Map()

  constructor(indexerUrl: string, indexerWsUrl: string) {
    this.restClient = new IndexerRestClient(indexerUrl)

    // Initialize WebSocket for real-time updates
    this.wsClient = new IndexerWebSocketClient(indexerWsUrl, {
      orderbook: false,
      ohlcv: true,
      symbol: "ETH/USDC",
      timeframes: ["1m", "5m", "15m", "30m", "1h", "4h", "1d"],
    })

    this.wsClient.connect().then(() => {
      // Subscribe to candle updates
      this.wsClient?.onCandle((candle) => {
        this.handleRealtimeUpdate(candle)
      })
    }).catch((error) => {
      console.error("Failed to connect to indexer WebSocket for datafeed:", error)
    })
  }

  /**
   * This method is called by the chart library to get information about the datafeed
   */
  onReady(callback: (config: any) => void) {
    setTimeout(() => {
      callback({
        supported_resolutions: ["1", "5", "15", "30", "60", "240", "D", "W", "M"],
        supports_marks: false,
        supports_timescale_marks: false,
        supports_time: true,
      })
    }, 0)
  }

  /**
   * Resolve symbol information
   */
  resolveSymbol(
    symbolName: string,
    onResolve: (symbolInfo: any) => void,
    onError: (error: string) => void
  ) {
    setTimeout(() => {
      const symbolInfo = {
        name: symbolName,
        ticker: symbolName,
        description: symbolName,
        type: "crypto",
        session: "24x7",
        timezone: "Etc/UTC",
        exchange: "Orbex",
        minmov: 1,
        pricescale: 100,
        has_intraday: true,
        has_daily: true,
        has_weekly_and_monthly: true,
        supported_resolutions: ["1", "5", "15", "30", "60", "240", "D", "W", "M"],
        volume_precision: 2,
        data_status: "streaming",
      }
      onResolve(symbolInfo)
    }, 0)
  }

  /**
   * Get historical bars
   */
  async getBars(
    symbolInfo: any,
    resolution: string,
    periodParams: any,
    onResult: (bars: TradingViewBar[], meta: { noData: boolean }) => void,
    onError: (error: string) => void
  ) {
    try {
      const interval = RESOLUTION_MAP[resolution] || "1m"

      const bars = await this.restClient.getCandlesAsTvBars({
        symbol: symbolInfo.name,
        start_time: periodParams.from,
        end_time: periodParams.to,
        interval,
      })

      if (bars.length === 0) {
        onResult([], { noData: true })
      } else {
        onResult(bars, { noData: false })
      }
    } catch (error) {
      console.error("Error fetching bars:", error)
      onError(String(error))
    }
  }

  /**
   * Subscribe to real-time updates
   */
  subscribeBars(
    symbolInfo: any,
    resolution: string,
    onRealtimeCallback: (bar: TradingViewBar) => void,
    subscriberUID: string,
    onResetCacheNeededCallback: () => void
  ) {
    const interval = RESOLUTION_MAP[resolution] || "1m"

    this.subscribers.set(subscriberUID, {
      symbolInfo,
      resolution: interval,
      lastBar: null,
      listener: onRealtimeCallback,
    })
  }

  /**
   * Unsubscribe from real-time updates
   */
  unsubscribeBars(subscriberUID: string) {
    this.subscribers.delete(subscriberUID)
  }

  /**
   * Handle real-time candle updates from WebSocket
   */
  private handleRealtimeUpdate(candle: CandleUpdate) {
    this.subscribers.forEach((subscriber) => {
      // Check if this update matches the subscriber's symbol and interval
      if (
        subscriber.symbolInfo.name === candle.s &&
        subscriber.resolution === candle.i
      ) {
        const bar: TradingViewBar = {
          time: Math.floor(candle.t / 1000), // Convert to seconds
          open: parseFloat(candle.o),
          high: parseFloat(candle.h),
          low: parseFloat(candle.l),
          close: parseFloat(candle.c),
          volume: parseFloat(candle.v),
        }

        // Update or create new bar
        if (subscriber.lastBar && subscriber.lastBar.time === bar.time) {
          // Update existing bar
          subscriber.lastBar = bar
        } else {
          // New bar
          subscriber.lastBar = bar
        }

        subscriber.listener(bar)
      }
    })
  }

  /**
   * Cleanup
   */
  disconnect() {
    this.wsClient?.disconnect()
    this.subscribers.clear()
  }
}
