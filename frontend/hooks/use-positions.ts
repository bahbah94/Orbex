"use client"

import { useState, useEffect } from "react"

export interface Position {
  pair: string
  side: "Long" | "Short"
  size: string
  entryPrice: string
  markPrice: string
  pnl: string
  pnlPercent: string
}

function generatePositions(): Position[] {
  const ethMarkPrice = 3450 + Math.random() * 20
  const ethEntry = 3450
  const ethPnl = (ethMarkPrice - ethEntry) * 1.5
  const ethPnlPercent = ((ethPnl / (ethEntry * 1.5)) * 100).toFixed(2)

  const btcMarkPrice = 67500 - Math.random() * 500
  const btcEntry = 67500
  const btcPnl = (btcEntry - btcMarkPrice) * 0.05
  const btcPnlPercent = ((btcPnl / (btcEntry * 0.05)) * 100).toFixed(2)

  return [
    {
      pair: "ETH/USD",
      side: "Long",
      size: "1.5",
      entryPrice: "3,450.00",
      markPrice: ethMarkPrice.toFixed(2),
      pnl: ethPnl >= 0 ? `+${ethPnl.toFixed(2)}` : ethPnl.toFixed(2),
      pnlPercent: ethPnl >= 0 ? `+${ethPnlPercent}%` : `${ethPnlPercent}%`,
    },
    {
      pair: "BTC/USD",
      side: "Short",
      size: "0.05",
      entryPrice: "67,500.00",
      markPrice: btcMarkPrice.toFixed(2),
      pnl: btcPnl >= 0 ? `+${btcPnl.toFixed(2)}` : btcPnl.toFixed(2),
      pnlPercent: btcPnl >= 0 ? `+${btcPnlPercent}%` : `${btcPnlPercent}%`,
    },
  ]
}

export function usePositions(): Position[] {
  const [positions, setPositions] = useState<Position[]>(generatePositions)

  useEffect(() => {
    const interval = setInterval(() => {
      setPositions(generatePositions())
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  return positions
}
