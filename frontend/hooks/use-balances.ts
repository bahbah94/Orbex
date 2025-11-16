"use client"

import { useState, useEffect } from "react"

export interface Balance {
  asset: string
  amount: string
  value: string
  available: string
}

function generateBalances(): Balance[] {
  const ethAmount = (2.5 + Math.random() * 0.1).toFixed(4)
  const usdcAmount = (15000 + Math.random() * 500).toFixed(2)
  const btcAmount = (0.12 + Math.random() * 0.01).toFixed(4)

  return [
    {
      asset: "ETH",
      amount: ethAmount,
      value: `$${(Number.parseFloat(ethAmount) * 3922).toFixed(2)}`,
      available: ethAmount,
    },
    { asset: "USDC", amount: usdcAmount, value: `$${usdcAmount}`, available: usdcAmount },
    {
      asset: "BTC",
      amount: btcAmount,
      value: `$${(Number.parseFloat(btcAmount) * 67234).toFixed(2)}`,
      available: btcAmount,
    },
  ]
}

export function useBalances(): Balance[] {
  const [balances, setBalances] = useState<Balance[]>(generateBalances)

  useEffect(() => {
    const interval = setInterval(() => {
      setBalances(generateBalances())
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  return balances
}
