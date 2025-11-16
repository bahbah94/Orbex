"use client"

import { useState, useEffect, useRef } from "react"
import { IndexerWebSocketClient } from "@/lib/indexer/websocket-client"
import type { OrderbookUpdate, WsPriceLevel } from "@/lib/indexer/types"

export interface OrderBookEntry {
  price: string
  size: string
  total: string
  sizeNum: number
}

export interface OrderBookData {
  asks: OrderBookEntry[]
  bids: OrderBookEntry[]
  spread: string
  spreadPercent: string
  maxSize: number
}

function transformOrderbookData(update: OrderbookUpdate): OrderBookData {
  const [bids, asks] = update.levels

  const transformLevel = (level: WsPriceLevel, runningTotal: number): OrderBookEntry => {
    const sizeNum = parseFloat(level.sz)
    const priceNum = parseFloat(level.px)
    const total = runningTotal + sizeNum * priceNum

    return {
      price: level.px,
      size: level.sz,
      total: total.toFixed(0),
      sizeNum,
    }
  }

  // Transform bids (already sorted descending by price)
  let bidTotal = 0
  const transformedBids = bids.map((level) => {
    const entry = transformLevel(level, bidTotal)
    bidTotal = parseFloat(entry.total)
    return entry
  })

  // Transform asks (need to reverse to show lowest first)
  let askTotal = 0
  const transformedAsks = asks
    .slice()
    .reverse()
    .map((level) => {
      const entry = transformLevel(level, askTotal)
      askTotal = parseFloat(entry.total)
      return entry
    })

  // Calculate max size for heatmap
  const maxSize = Math.max(
    ...transformedBids.map((b) => b.sizeNum),
    ...transformedAsks.map((a) => a.sizeNum),
    0.01 // Prevent division by zero
  )

  // Calculate spread
  const bestBid = transformedBids.length > 0 ? parseFloat(transformedBids[0].price) : 0
  const bestAsk = transformedAsks.length > 0 ? parseFloat(transformedAsks[transformedAsks.length - 1].price) : 0
  const spread = (bestAsk - bestBid).toFixed(2)
  const spreadPercent = bestBid > 0 ? (((bestAsk - bestBid) / bestBid) * 100).toFixed(4) : "0.0000"

  return {
    asks: transformedAsks,
    bids: transformedBids,
    spread,
    spreadPercent,
    maxSize,
  }
}

export function useIndexerOrderbook(
  indexerWsUrl: string,
  symbol = "ETH/USDC"
): OrderBookData | null {
  const [data, setData] = useState<OrderBookData | null>(null)
  const wsClientRef = useRef<IndexerWebSocketClient | null>(null)

  useEffect(() => {
    // Create WebSocket client for orderbook only
    const client = new IndexerWebSocketClient(indexerWsUrl, {
      orderbook: true,
      ohlcv: false,
      symbol,
    })

    wsClientRef.current = client

    // Connect and subscribe to orderbook updates
    client.connect().then(() => {
      client.onOrderbook((update) => {
        const transformedData = transformOrderbookData(update)
        setData(transformedData)
      })
    }).catch((error) => {
      console.error("Failed to connect to indexer WebSocket:", error)
    })

    // Cleanup on unmount
    return () => {
      client.disconnect()
      wsClientRef.current = null
    }
  }, [indexerWsUrl, symbol])

  return data
}
