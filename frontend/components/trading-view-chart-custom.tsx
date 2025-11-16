"use client"

import { useEffect, useRef } from "react"
import { IndexerTradingViewDatafeed } from "@/lib/indexer/tradingview-datafeed"
import { env } from "@/lib/env"

interface TradingViewChartCustomProps {
  symbol: string
}

/**
 * TradingView Chart with Custom Datafeed
 *
 * Note: This component uses a custom datafeed to fetch real-time and historical
 * data from the indexer. However, the TradingView widget embedded via script
 * doesn't support custom datafeeds - that requires the full Charting Library.
 *
 * For now, this creates a simple candlestick chart using canvas.
 * To use TradingView's full features, you'll need to:
 * 1. Purchase the TradingView Charting Library license
 * 2. Install the library locally
 * 3. Replace this implementation with the full library
 */
export function TradingViewChartCustom({ symbol }: TradingViewChartCustomProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const datafeedRef = useRef<IndexerTradingViewDatafeed | null>(null)

  useEffect(() => {
    // Initialize custom datafeed
    const datafeed = new IndexerTradingViewDatafeed(
      env.INDEXER_URL,
      env.INDEXER_WS_URL
    )
    datafeedRef.current = datafeed

    // For demonstration, fetch some initial data
    const initializeChart = async () => {
      const now = Math.floor(Date.now() / 1000)
      const oneDayAgo = now - 24 * 60 * 60

      datafeed.onReady((config) => {
        console.log("Datafeed ready:", config)
      })

      datafeed.resolveSymbol(
        symbol,
        (symbolInfo) => {
          console.log("Symbol resolved:", symbolInfo)

          datafeed.getBars(
            symbolInfo,
            "15",
            { from: oneDayAgo, to: now },
            (bars, meta) => {
              console.log("Received bars:", bars.length, meta)
              if (bars.length > 0) {
                drawChart(bars)
              }
            },
            (error) => {
              console.error("Error fetching bars:", error)
            }
          )
        },
        (error) => {
          console.error("Error resolving symbol:", error)
        }
      )
    }

    initializeChart()

    return () => {
      datafeed.disconnect()
    }
  }, [symbol])

  const drawChart = (bars: any[]) => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext("2d")
    if (!ctx) return

    // Set canvas size
    canvas.width = canvas.offsetWidth
    canvas.height = canvas.offsetHeight

    const width = canvas.width
    const height = canvas.height
    const padding = 40

    // Clear canvas
    ctx.fillStyle = "#161616"
    ctx.fillRect(0, 0, width, height)

    if (bars.length === 0) return

    // Calculate scales
    const prices = bars.flatMap((b) => [b.high, b.low])
    const minPrice = Math.min(...prices)
    const maxPrice = Math.max(...prices)
    const priceRange = maxPrice - minPrice

    const barWidth = (width - 2 * padding) / bars.length
    const candleWidth = Math.max(1, barWidth * 0.6)

    // Draw candlesticks
    bars.forEach((bar, i) => {
      const x = padding + i * barWidth + barWidth / 2

      const openY = height - padding - ((bar.open - minPrice) / priceRange) * (height - 2 * padding)
      const closeY = height - padding - ((bar.close - minPrice) / priceRange) * (height - 2 * padding)
      const highY = height - padding - ((bar.high - minPrice) / priceRange) * (height - 2 * padding)
      const lowY = height - padding - ((bar.low - minPrice) / priceRange) * (height - 2 * padding)

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

    // Draw price labels
    ctx.fillStyle = "#888"
    ctx.font = "12px monospace"
    ctx.textAlign = "right"
    ctx.fillText(maxPrice.toFixed(2), padding - 5, padding)
    ctx.fillText(minPrice.toFixed(2), padding - 5, height - padding)
  }

  return (
    <div className="relative h-full w-full bg-[#161616]">
      <canvas
        ref={canvasRef}
        className="h-full w-full"
        style={{ width: "100%", height: "100%" }}
      />
      <div className="absolute top-4 left-4 text-sm text-gray-400">
        {symbol} - Custom Datafeed (Indexer)
      </div>
    </div>
  )
}
