"use client"

import { useState, useEffect } from "react"
import { IndexerChart } from "@/components/indexer-chart"
import { OrderBook } from "@/components/order-book"
import { IndexerRestClient } from "@/lib/indexer/rest-client"
import { env } from "@/lib/env"
import { Button } from "@/components/ui/button"
import { CheckCircle2, XCircle, Loader2 } from "lucide-react"

/**
 * Test page to verify indexer integration
 * Navigate to: http://localhost:4000/test-indexer
 */
export default function TestIndexerPage() {
  const [healthStatus, setHealthStatus] = useState<"loading" | "ok" | "error">("loading")
  const [candleCount, setCandleCount] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    testConnection()
  }, [])

  const testConnection = async () => {
    setHealthStatus("loading")
    setError(null)

    try {
      const client = new IndexerRestClient(env.INDEXER_URL)

      // Test health check
      const isHealthy = await client.healthCheck()
      if (!isHealthy) {
        setHealthStatus("error")
        setError("Health check failed")
        return
      }

      setHealthStatus("ok")

      // Test fetching candles
      const now = Math.floor(Date.now() / 1000)
      const oneHourAgo = now - 3600

      const candles = await client.getCandles({
        symbol: "ETH/USDC",
        start_time: oneHourAgo,
        end_time: now,
        interval: "1m",
      })

      setCandleCount(candles.length)
    } catch (err) {
      console.error("Connection test failed:", err)
      setHealthStatus("error")
      setError(err instanceof Error ? err.message : "Unknown error")
    }
  }

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="max-w-7xl mx-auto space-y-8">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold mb-2">Indexer Integration Test</h1>
          <p className="text-muted-foreground">
            Testing connection to indexer at {env.INDEXER_URL}
          </p>
        </div>

        {/* Connection Status */}
        <div className="bg-card p-6 rounded-lg border">
          <h2 className="text-xl font-semibold mb-4">Connection Status</h2>

          <div className="space-y-4">
            {/* Health Check */}
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Health Check</p>
                <p className="text-sm text-muted-foreground">GET {env.INDEXER_URL}/health</p>
              </div>
              <div className="flex items-center gap-2">
                {healthStatus === "loading" && <Loader2 className="h-5 w-5 animate-spin" />}
                {healthStatus === "ok" && <CheckCircle2 className="h-5 w-5 text-green-500" />}
                {healthStatus === "error" && <XCircle className="h-5 w-5 text-red-500" />}
                <span className="text-sm font-medium capitalize">{healthStatus}</span>
              </div>
            </div>

            {/* Candles Check */}
            {candleCount !== null && (
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">Historical Candles</p>
                  <p className="text-sm text-muted-foreground">Last 1 hour, 1m interval</p>
                </div>
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="h-5 w-5 text-green-500" />
                  <span className="text-sm font-medium">{candleCount} candles</span>
                </div>
              </div>
            )}

            {/* WebSocket Check */}
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">WebSocket</p>
                <p className="text-sm text-muted-foreground">{env.INDEXER_WS_URL}/ws/market</p>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">Check components below</span>
              </div>
            </div>

            {/* Error Display */}
            {error && (
              <div className="bg-destructive/10 text-destructive p-4 rounded border border-destructive/20">
                <p className="font-medium">Error:</p>
                <p className="text-sm">{error}</p>
              </div>
            )}

            {/* Retry Button */}
            <Button onClick={testConnection} variant="outline" className="w-full">
              Retry Connection Test
            </Button>
          </div>
        </div>

        {/* Live Components */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {/* Chart */}
          <div className="bg-card rounded-lg border overflow-hidden">
            <div className="p-4 border-b">
              <h2 className="text-lg font-semibold">IndexerChart Component</h2>
              <p className="text-sm text-muted-foreground">Real-time OHLCV data</p>
            </div>
            <div className="h-[400px]">
              <IndexerChart symbol="ETH/USDC" interval="1m" />
            </div>
          </div>

          {/* OrderBook */}
          <div className="bg-card rounded-lg border overflow-hidden">
            <div className="p-4 border-b">
              <h2 className="text-lg font-semibold">OrderBook Component</h2>
              <p className="text-sm text-muted-foreground">Real-time orderbook</p>
            </div>
            <div className="h-[400px]">
              <OrderBook useIndexer={true} symbol="ETH/USDC" />
            </div>
          </div>
        </div>

        {/* Instructions */}
        <div className="bg-card p-6 rounded-lg border">
          <h2 className="text-xl font-semibold mb-4">Instructions</h2>
          <div className="space-y-2 text-sm text-muted-foreground">
            <p>1. Make sure the indexer is running: <code className="text-foreground">cargo run --release --bin orderbook-indexer</code></p>
            <p>2. Verify health endpoint: <code className="text-foreground">curl {env.INDEXER_URL}/health</code></p>
            <p>3. Check browser console for WebSocket connection logs</p>
            <p>4. If everything works, you should see:</p>
            <ul className="ml-6 space-y-1 list-disc">
              <li>Green checkmarks for health and candles</li>
              <li>Live candlestick chart updating in real-time</li>
              <li>Live orderbook with bid/ask prices</li>
            </ul>
          </div>
        </div>

        {/* Links */}
        <div className="flex gap-4">
          <Button asChild variant="outline">
            <a href="/">← Back to Dashboard</a>
          </Button>
          <Button asChild variant="outline">
            <a href={env.INDEXER_URL + "/health"} target="_blank">
              Open Health Endpoint →
            </a>
          </Button>
        </div>
      </div>
    </div>
  )
}
