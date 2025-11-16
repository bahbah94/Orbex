/**
 * REST API client for Indexer historical data
 */

import type { CandleUpdate, OrderbookUpdate, CandleQuery, TradingViewBar } from "./types"

export class IndexerRestClient {
  private baseUrl: string

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl
  }

  /**
   * Get historical OHLCV candles
   * @param params Query parameters
   * @returns Array of candle updates
   */
  async getCandles(params: CandleQuery): Promise<CandleUpdate[]> {
    const queryParams = new URLSearchParams({
      symbol: params.symbol,
      start_time: params.start_time.toString(),
      end_time: params.end_time.toString(),
      interval: params.interval,
    })

    const response = await fetch(`${this.baseUrl}/api/candles?${queryParams.toString()}`)

    if (!response.ok) {
      throw new Error(`Failed to fetch candles: ${response.statusText}`)
    }

    return response.json()
  }

  /**
   * Get historical candles in TradingView format
   * @param params Query parameters
   * @returns Array of TradingView bars
   */
  async getCandlesAsTvBars(params: CandleQuery): Promise<TradingViewBar[]> {
    const candles = await this.getCandles(params)

    return candles.map((candle) => ({
      time: Math.floor(candle.t / 1000), // Convert milliseconds to seconds
      open: parseFloat(candle.o),
      high: parseFloat(candle.h),
      low: parseFloat(candle.l),
      close: parseFloat(candle.c),
      volume: parseFloat(candle.v),
    }))
  }

  /**
   * Get current orderbook snapshot
   * @returns Current orderbook state
   */
  async getOrderbook(): Promise<OrderbookUpdate> {
    const response = await fetch(`${this.baseUrl}/api/orderbook`)

    if (!response.ok) {
      throw new Error(`Failed to fetch orderbook: ${response.statusText}`)
    }

    return response.json()
  }

  /**
   * Health check
   * @returns True if server is healthy
   */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/health`)
      return response.ok
    } catch {
      return false
    }
  }
}
