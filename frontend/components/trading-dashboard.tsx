"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { TradingViewChart } from "@/components/trading-view-chart"
import { WalletConnect } from "@/components/wallet-connect"
import { OrderBook } from "@/components/order-book"
import { TradingForm } from "@/components/trading-form"
import { AccountTabs } from "@/components/account-tabs"
import { ThemeToggle } from "@/components/theme-toggle"
import { Activity, TrendingUp, TrendingDown } from "lucide-react"

export function TradingDashboard() {
  const [selectedPair, setSelectedPair] = useState("BTC/USD")

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
    <div className="flex h-screen flex-col bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="flex h-14 items-center justify-between px-4">
          <div className="flex items-center gap-8">
            <div className="flex items-center gap-2">
              <Activity className="h-5 w-5 text-primary" />
              <h1 className="text-lg font-bold text-foreground">Orbex</h1>
            </div>
            <nav className="hidden items-center gap-6 md:flex">
              <Button variant="ghost" className="text-muted-foreground hover:text-foreground">
                Trade
              </Button>
            </nav>
          </div>
          <div className="flex items-center gap-2">
            <ThemeToggle />
            <WalletConnect />
          </div>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Middle Section - Chart, Order Book, and Trading Form */}
        <div className="flex flex-1 overflow-hidden">
          {/* Left Section - Chart and Order Book */}
          <div className="flex flex-1 flex-col">
            {/* Chart and Order Book Row */}
            <div className="flex flex-1 overflow-hidden">
              {/* Trading Chart */}
              <div className="flex flex-1 flex-col border-r border-border">
                <div className="flex items-center justify-between border-b border-border px-4 py-2">
                  <div className="flex items-center gap-3">
                    <Select value={selectedPair} onValueChange={setSelectedPair}>
                      <SelectTrigger className="h-8 w-[9rem] border-none bg-transparent text-sm font-semibold">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="BTC/USD">BTC/USD</SelectItem>
                        <SelectItem value="ETH/USD">ETH/USD</SelectItem>
                        <SelectItem value="SOL/USD">SOL/USD</SelectItem>
                      </SelectContent>
                    </Select>
                    <div className="flex items-baseline gap-2">
                      <span className="text-lg font-bold text-card-foreground">${currentStats.price}</span>
                      <div
                        className={`flex items-center gap-1 text-xs font-medium ${
                          currentStats.isPositive ? "text-primary" : "text-destructive"
                        }`}
                      >
                        {currentStats.isPositive ? (
                          <TrendingUp className="h-3 w-3" />
                        ) : (
                          <TrendingDown className="h-3 w-3" />
                        )}
                        {currentStats.changePercent}
                      </div>
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 px-3 text-xs text-muted-foreground hover:text-foreground"
                    >
                      24h High: ${currentStats.high24h}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 px-3 text-xs text-muted-foreground hover:text-foreground"
                    >
                      24h Vol: ${currentStats.volume24h}
                    </Button>
                  </div>
                </div>
                <div className="flex-1">
                  <TradingViewChart symbol={selectedPair} />
                </div>
              </div>

              {/* Order Book */}
              <div className="w-[24rem] border-r border-border">
                <OrderBook />
              </div>
            </div>

            <div className="border-t border-border">
              <AccountTabs />
            </div>
          </div>

          {/* Right Column - Trading Form */}
          <div className="w-[22rem]">
            <TradingForm selectedPair={selectedPair} />
          </div>
        </div>
      </div>
    </div>
  )
}
