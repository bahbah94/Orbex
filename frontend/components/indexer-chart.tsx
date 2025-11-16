"use client"

import { useEffect, useRef, useState } from "react"
import { IndexerRestClient } from "@/lib/indexer/rest-client"
import { IndexerWebSocketClient } from "@/lib/indexer/websocket-client"
import type { TradingViewBar } from "@/lib/indexer/types"
import { env } from "@/lib/env"

interface IndexerChartProps {
  symbol: string
  interval?: string
}

/**
 * Custom candlestick chart using Indexer data
 * This is a simple canvas-based implementation that displays real-time OHLCV data
 */
export function IndexerChart({ symbol, interval = "5m" }: IndexerChartProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [bars, setBars] = useState<TradingViewBar[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const restClientRef = useRef<IndexerRestClient | null>(null)
  const wsClientRef = useRef<IndexerWebSocketClient | null>(null)
  const animationFrameRef = useRef<number | null>(null)

  // Fetch initial historical data
  useEffect(() => {
    const restClient = new IndexerRestClient(env.INDEXER_URL)
    restClientRef.current = restClient

    const fetchHistoricalData = async () => {
      try {
        setLoading(true)
        setError(null)

        const now = Math.floor(Date.now() / 1000)
        const oneDayAgo = now - 24 * 60 * 60

        const data = await restClient.getCandlesAsTvBars({
          symbol,
          start_time: oneDayAgo,
          end_time: now,
          interval,
        })

        setBars(data)
        setLoading(false)
      } catch (err) {
        console.error("Error fetching historical data:", err)
        setError(err instanceof Error ? err.message : "Failed to fetch data")
        setLoading(false)
      }
    }

    fetchHistoricalData()
  }, [symbol, interval])

  // Subscribe to real-time updates
  useEffect(() => {
    const wsClient = new IndexerWebSocketClient(env.INDEXER_WS_URL, {
      orderbook: false,
      ohlcv: true,
      symbol,
      timeframes: [interval],
    })

    wsClientRef.current = wsClient

    wsClient.connect().then(() => {
      wsClient.onCandle((candle) => {
        // Check if this candle matches our interval
        if (candle.i !== interval || candle.s !== symbol) return

        const newBar: TradingViewBar = {
          time: Math.floor(candle.t / 1000),
          open: parseFloat(candle.o),
          high: parseFloat(candle.h),
          low: parseFloat(candle.l),
          close: parseFloat(candle.c),
          volume: parseFloat(candle.v),
        }

        setBars((prevBars) => {
          const lastBar = prevBars[prevBars.length - 1]

          if (lastBar && lastBar.time === newBar.time) {
            // Update existing bar
            return [...prevBars.slice(0, -1), newBar]
          } else {
            // Add new bar
            return [...prevBars, newBar].slice(-100) // Keep last 100 bars
          }
        })
      })
    }).catch((err) => {
      console.error("WebSocket connection error:", err)
    })

    return () => {
      wsClient.disconnect()
    }
  }, [symbol, interval])

  // Draw chart on canvas
  useEffect(() => {
    if (bars.length === 0) return

    const drawChart = () => {
      const canvas = canvasRef.current
      if (!canvas) return

      const ctx = canvas.getContext("2d")
      if (!ctx) return

      // Set canvas size
      const dpr = window.devicePixelRatio || 1
      const rect = canvas.getBoundingClientRect()
      canvas.width = rect.width * dpr
      canvas.height = rect.height * dpr
      ctx.scale(dpr, dpr)

      const width = rect.width
      const height = rect.height
      const padding = { top: 20, right: 80, bottom: 30, left: 10 }

      // Clear canvas
      ctx.fillStyle = "#161616"
      ctx.fillRect(0, 0, width, height)

      // Calculate scales
      const prices = bars.flatMap((b) => [b.high, b.low])
      const minPrice = Math.min(...prices)
      const maxPrice = Math.max(...prices)
      const priceRange = maxPrice - minPrice || 1

      const chartWidth = width - padding.left - padding.right
      const chartHeight = height - padding.top - padding.bottom
      const barWidth = chartWidth / bars.length
      const candleWidth = Math.max(1, barWidth * 0.7)

      // Draw grid lines
      ctx.strokeStyle = "#2a2a2a"
      ctx.lineWidth = 1
      for (let i = 0; i <= 5; i++) {
        const y = padding.top + (chartHeight / 5) * i
        ctx.beginPath()
        ctx.moveTo(padding.left, y)
        ctx.lineTo(width - padding.right, y)
        ctx.stroke()
      }

      // Draw candlesticks
      bars.forEach((bar, i) => {
        const x = padding.left + i * barWidth + barWidth / 2

        const openY = padding.top + chartHeight - ((bar.open - minPrice) / priceRange) * chartHeight
        const closeY = padding.top + chartHeight - ((bar.close - minPrice) / priceRange) * chartHeight
        const highY = padding.top + chartHeight - ((bar.high - minPrice) / priceRange) * chartHeight
        const lowY = padding.top + chartHeight - ((bar.low - minPrice) / priceRange) * chartHeight

        const isGreen = bar.close >= bar.open

        // Draw wick
        ctx.strokeStyle = isGreen ? "#22c55e" : "#ef4444"
        ctx.lineWidth = 1
        ctx.beginPath()
        ctx.moveTo(x, highY)
        ctx.lineTo(x, lowY)
        ctx.stroke()

        // Draw body
        ctx.fillStyle = isGreen ? "#22c55e" : "#ef4444"
        const bodyTop = Math.min(openY, closeY)
        const bodyHeight = Math.abs(closeY - openY) || 1
        ctx.fillRect(x - candleWidth / 2, bodyTop, candleWidth, bodyHeight)
      })

      // Draw price labels on the right
      ctx.fillStyle = "#888"
      ctx.font = "11px monospace"
      ctx.textAlign = "left"

      for (let i = 0; i <= 5; i++) {
        const price = maxPrice - (priceRange / 5) * i
        const y = padding.top + (chartHeight / 5) * i
        ctx.fillText(price.toFixed(2), width - padding.right + 5, y + 4)
      }

      // Draw current price indicator
      const lastBar = bars[bars.length - 1]
      if (lastBar) {
        const currentPriceY = padding.top + chartHeight - ((lastBar.close - minPrice) / priceRange) * chartHeight
        const isGreen = lastBar.close >= lastBar.open

        ctx.fillStyle = isGreen ? "#22c55e" : "#ef4444"
        ctx.fillRect(width - padding.right, currentPriceY - 10, padding.right - 5, 20)

        ctx.fillStyle = "#000"
        ctx.font = "12px monospace"
        ctx.textAlign = "center"
        ctx.fillText(lastBar.close.toFixed(2), width - padding.right / 2, currentPriceY + 4)
      }
    }

    drawChart()
    animationFrameRef.current = requestAnimationFrame(drawChart)

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current)
      }
    }
  }, [bars])

  if (loading) {
    return (
      <div className="flex h-full w-full items-center justify-center bg-[#161616] text-gray-400">
        Loading chart data...
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex h-full w-full items-center justify-center bg-[#161616] text-red-400">
        Error: {error}
      </div>
    )
  }

  if (bars.length === 0) {
    return (
      <div className="flex h-full w-full items-center justify-center bg-[#161616] text-gray-400">
        No data available
      </div>
    )
  }

  return (
    <div className="relative h-full w-full bg-[#161616]">
      <canvas
        ref={canvasRef}
        className="h-full w-full"
        style={{ width: "100%", height: "100%" }}
      />
      <div className="absolute top-2 left-2 flex items-center gap-2 text-xs text-gray-400">
        <div className="h-2 w-2 rounded-full bg-green-500"></div>
        <span>
          {symbol} · {interval} · Live
        </span>
      </div>
    </div>
  )
}
