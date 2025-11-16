"use client"

import { Card, CardContent } from "@/components/ui/card"
import { TrendingUp, TrendingDown } from "lucide-react"

interface MarketStatsProps {
  selectedPair: string
}

export function MarketStats({ selectedPair }: MarketStatsProps) {
  // Mock data - in production, fetch from API
  const stats = {
    "BTC/USD": {
      price: "67,234.50",
      change: "+2.34",
      changePercent: "+3.61%",
      high24h: "68,450.00",
      low24h: "65,120.00",
      volume24h: "28.5B",
      isPositive: true,
    },
    "ETH/USD": {
      price: "3,456.78",
      change: "-45.23",
      changePercent: "-1.29%",
      high24h: "3,520.00",
      low24h: "3,401.00",
      volume24h: "12.3B",
      isPositive: false,
    },
    "SOL/USD": {
      price: "142.56",
      change: "+8.92",
      changePercent: "+6.68%",
      high24h: "145.00",
      low24h: "135.20",
      volume24h: "2.1B",
      isPositive: true,
    },
  }

  const currentStats = stats[selectedPair as keyof typeof stats]

  return (
    <div className="grid gap-2 md:grid-cols-2 lg:grid-cols-4">
      <Card className="border-border bg-card">
        <CardContent className="p-3">
          <div className="space-y-1">
            <p className="text-xs text-muted-foreground">Price</p>
            <div className="flex items-baseline gap-2">
              <p className="text-lg font-bold text-card-foreground">${currentStats.price}</p>
              <div
                className={`flex items-center gap-1 text-xs font-medium ${
                  currentStats.isPositive ? "text-primary" : "text-destructive"
                }`}
              >
                {currentStats.isPositive ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
                {currentStats.changePercent}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="border-border bg-card">
        <CardContent className="p-3">
          <div className="space-y-1">
            <p className="text-xs text-muted-foreground">24h Change</p>
            <p className={`text-lg font-bold ${currentStats.isPositive ? "text-primary" : "text-destructive"}`}>
              ${currentStats.change}
            </p>
          </div>
        </CardContent>
      </Card>

      <Card className="border-border bg-card">
        <CardContent className="p-3">
          <div className="space-y-1">
            <p className="text-xs text-muted-foreground">24h High</p>
            <p className="text-lg font-bold text-card-foreground">${currentStats.high24h}</p>
          </div>
        </CardContent>
      </Card>

      <Card className="border-border bg-card">
        <CardContent className="p-3">
          <div className="space-y-1">
            <p className="text-xs text-muted-foreground">24h Volume</p>
            <p className="text-lg font-bold text-card-foreground">${currentStats.volume24h}</p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
