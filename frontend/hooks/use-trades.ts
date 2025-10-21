"use client"

import { useState, useEffect } from "react"

export interface Trade {
  price: string
  size: string
  time: string
  isBuy: boolean
}

function generateTrades(basePrice: number, count: number): Trade[] {
  const trades: Trade[] = []

  for (let i = 0; i < count; i++) {
    const price = basePrice + (Math.random() - 0.5) * 2
    const size = Math.random() * 10 + 0.01
    const isBuy = Math.random() > 0.5
    const time = new Date(Date.now() - i * 60000).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    })

    trades.push({
      price: price.toFixed(2),
      size: size.toFixed(4),
      time,
      isBuy,
    })
  }

  return trades
}

export function useTrades(basePrice = 3922.58, count = 20): Trade[] {
  const [trades, setTrades] = useState<Trade[]>(() => generateTrades(basePrice, count))

  useEffect(() => {
    const interval = setInterval(() => {
      setTrades(generateTrades(basePrice + (Math.random() - 0.5) * 2, count))
    }, 1000)

    return () => clearInterval(interval)
  }, [basePrice, count])

  return trades
}
