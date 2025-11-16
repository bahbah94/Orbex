"use client"

import { useState, useEffect } from "react"

export interface OpenOrder {
  pair: string
  type: string
  side: "Buy" | "Sell"
  price: string
  amount: string
  filled: string
  total: string
}

function generateOpenOrders(): OpenOrder[] {
  const ethPrice = (3400 + Math.random() * 10).toFixed(2)
  const solPrice = (145 + Math.random() * 5).toFixed(2)
  const ethFilled = Math.floor(Math.random() * 30)
  const solFilled = Math.floor(Math.random() * 20)

  return [
    {
      pair: "ETH/USD",
      type: "Limit",
      side: "Buy",
      price: ethPrice,
      amount: "0.5",
      filled: `${ethFilled}%`,
      total: (Number.parseFloat(ethPrice) * 0.5).toFixed(2),
    },
    {
      pair: "SOL/USD",
      type: "Limit",
      side: "Sell",
      price: solPrice,
      amount: "10",
      filled: `${solFilled}%`,
      total: (Number.parseFloat(solPrice) * 10).toFixed(2),
    },
  ]
}

export function useOpenOrders(): OpenOrder[] {
  const [orders, setOrders] = useState<OpenOrder[]>(generateOpenOrders)

  useEffect(() => {
    const interval = setInterval(() => {
      setOrders(generateOpenOrders())
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  return orders
}
