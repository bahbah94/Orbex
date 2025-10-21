"use client"

import { useState, useEffect } from "react"

export interface MarketStats {
  markPrice: string
  indexPrice: string
  change24h: string
  volume24h: string
  openInterest: string
  fundingRate: string
  nextFunding: string
}

function generateMarketStats(): MarketStats {
  const markPrice = 3922 + Math.random() * 2
  const indexPrice = markPrice + (Math.random() - 0.5) * 0.5
  const change = (Math.random() - 0.3) * 5
  const volume = 960000000 + Math.random() * 10000000

  return {
    markPrice: `$${markPrice.toFixed(2)}`,
    indexPrice: `$${indexPrice.toFixed(2)}`,
    change24h: `${change >= 0 ? "+" : ""}${change.toFixed(2)}%`,
    volume24h: `$${volume.toLocaleString("en-US", { maximumFractionDigits: 2 })}`,
    openInterest: "$319,420,066.83",
    fundingRate: "0.0012%",
    nextFunding: "15:44",
  }
}

export function useMarketStats(): MarketStats {
  const [stats, setStats] = useState<MarketStats>(generateMarketStats)

  useEffect(() => {
    const interval = setInterval(() => {
      setStats(generateMarketStats())
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  return stats
}
