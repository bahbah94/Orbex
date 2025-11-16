"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { TradingViewChart } from "@/components/trading-view-chart"
import { IndexerChart } from "@/components/indexer-chart"
import { WalletConnect } from "@/components/wallet-connect"
import { OrderBook } from "@/components/order-book"
import { TradingForm } from "@/components/trading-form"
import { AccountTabs } from "@/components/account-tabs"
import { ThemeToggle } from "@/components/theme-toggle"
import { Activity, TrendingUp, TrendingDown } from "lucide-react"

export function TradingDashboardWithIndexer() {
  const [selectedPair, setSelectedPair] = useState("ETH/USDC")
  const [useIndexerChart, setUseIndexerChart] = useState(true)
  const [chartInterval, setChartInterval] = useState("15m")

  const stats = {
    "ETH/USDC": {
      price: "3,922.58",
      change: "+124.50",
      changePercent: "+3.28%",
      high24h: "3,980.00",
      low24h: "3,750.00",
      volume24h: "15.2B",
      isPositive: true,
    },
  }

  const currentStats = stats[selectedPair as keyof typeof stats] || stats["ETH/USDC"]

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
                        <SelectItem value="ETH/USDC">ETH/USDC</SelectItem>
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
                  <div className="flex gap-2 items-center">
                    <Select value={chartInterval} onValueChange={setChartInterval}>
                      <SelectTrigger className="h-7 w-[5rem] text-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="1m">1m</SelectItem>
                        <SelectItem value="5m">5m</SelectItem>
                        <SelectItem value="15m">15m</SelectItem>
                        <SelectItem value="30m">30m</SelectItem>
                        <SelectItem value="1h">1h</SelectItem>
                        <SelectItem value="4h">4h</SelectItem>
                        <SelectItem value="1d">1d</SelectItem>
                      </SelectContent>
                    </Select>
                    <Button
                      variant={useIndexerChart ? "default" : "outline"}
                      size="sm"
                      className="h-7 px-3 text-xs"
                      onClick={() => setUseIndexerChart(true)}
                    >
                      Indexer Chart
                    </Button>
                    <Button
                      variant={!useIndexerChart ? "default" : "outline"}
                      size="sm"
                      className="h-7 px-3 text-xs"
                      onClick={() => setUseIndexerChart(false)}
                    >
                      TradingView
                    </Button>
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
                  {useIndexerChart ? (
                    <IndexerChart symbol={selectedPair} interval={chartInterval} />
                  ) : (
                    <TradingViewChart symbol={selectedPair} />
                  )}
                </div>
              </div>

              {/* Order Book */}
              <div className="w-[24rem] border-r border-border">
                <OrderBook useIndexer={true} symbol={selectedPair} />
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
