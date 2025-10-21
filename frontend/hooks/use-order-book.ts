"use client"

import { useState, useEffect } from "react"

export interface OrderBookEntry {
  price: string
  size: string
  total: string
  sizeNum: number // Added numeric size for heatmap calculations
}

export interface OrderBookData {
  asks: OrderBookEntry[]
  bids: OrderBookEntry[]
  spread: string
  spreadPercent: string
  maxSize: number // Added max size for heatmap normalization
}

function generateOrderBookData(basePrice: number): OrderBookData {
  const asks: OrderBookEntry[] = []
  const bids: OrderBookEntry[] = []
  let maxSize = 0

  // Generate asks (sell orders)
  for (let i = 0; i < 10; i++) {
    const price = basePrice + 0.01 * (i + 1) + Math.random() * 0.5
    const sizeNum = Math.random() * 50 + 0.01
    const total = price * sizeNum
    maxSize = Math.max(maxSize, sizeNum)
    asks.push({
      price: price.toFixed(2),
      size: sizeNum.toFixed(4),
      total: total.toFixed(0),
      sizeNum,
    })
  }

  // Generate bids (buy orders)
  for (let i = 0; i < 10; i++) {
    const price = basePrice - 0.01 * (i + 1) - Math.random() * 0.5
    const sizeNum = Math.random() * 50 + 0.01
    const total = price * sizeNum
    maxSize = Math.max(maxSize, sizeNum)
    bids.push({
      price: price.toFixed(2),
      size: sizeNum.toFixed(4),
      total: total.toFixed(0),
      sizeNum,
    })
  }

  const spread = (Number.parseFloat(asks[asks.length - 1].price) - Number.parseFloat(bids[0].price)).toFixed(2)
  const spreadPercent = ((Number.parseFloat(spread) / Number.parseFloat(bids[0].price)) * 100).toFixed(4)

  return {
    asks: asks.reverse(),
    bids,
    spread,
    spreadPercent,
    maxSize,
  }
}

export function useOrderBook(basePrice = 3922.58): OrderBookData {
  const [data, setData] = useState<OrderBookData>(() => generateOrderBookData(basePrice))

  useEffect(() => {
    const interval = setInterval(() => {
      setData(generateOrderBookData(basePrice + (Math.random() - 0.5) * 2))
    }, 1000)

    return () => clearInterval(interval)
  }, [basePrice])

  return data
}
