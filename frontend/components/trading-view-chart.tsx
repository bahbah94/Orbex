"use client"

import { useEffect, useRef, memo, useMemo, useState } from "react"

interface TradingViewChartProps {
  symbol: string
}

export const TradingViewChart = memo(function TradingViewChart({ symbol }: TradingViewChartProps) {
  const container = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!container.current) return

    // Clear previous widget
    container.current.innerHTML = ""

    const script = document.createElement("script")
    script.src = "https://s3.tradingview.com/external-embedding/embed-widget-advanced-chart.js"
    script.type = "text/javascript"
    script.async = true
    script.innerHTML = JSON.stringify({
      autosize: true,
      symbol: symbol.replace("/", ""),
      interval: "D",
      timezone: "Etc/UTC",
      theme: "dark",
      style: "1",
      locale: "en",
      enable_publishing: false,
      backgroundColor: "rgba(22, 22, 22, 1)",
      gridColor: "rgba(42, 42, 42, 1)",
      hide_top_toolbar: false,
      hide_legend: false,
      save_image: false,
      container_id: "tradingview_chart",
      height: "600",
      width: "100%",
    })

    container.current.appendChild(script)

    return () => {
      if (container.current) {
        container.current.innerHTML = ""
      }
    }
  }, [symbol])

  return (
    <div className="tradingview-widget-container h-[600px] w-full" ref={container}>
      <div id="tradingview_chart" className="h-full w-full" />
    </div>
  )
})

import {
  createChart,
  CandlestickSeries,
  type IChartApi,
  type ISeriesApi,
  type CandlestickData,
  type Time,
} from "lightweight-charts"
import { useSimulatedCandles } from "../hooks/use-simulated-candles"

type SimulatedTVChartProps = {
  symbol: string
}

type Timeframe = "1m" | "30m" | "1h" | "4h" | "1D"

export function SimulatedTVChart({ symbol }: SimulatedTVChartProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const chartRef = useRef<IChartApi | null>(null)
  const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null)

  const [timeframe, setTimeframe] = useState<Timeframe>("1D")

  const candleMs = useMemo(() => {
    switch (timeframe) {
      case "1m":
        return 60_000
      case "30m":
        return 30 * 60_000
      case "1h":
        return 60 * 60_000
      case "4h":
        return 4 * 60 * 60_000
      case "1D":
      default:
        return 24 * 60 * 60_000
    }
  }, [timeframe])

  const candles = useSimulatedCandles({
    symbol,
    candleMs,
    history: 30,
    tickMs: 1000,
  })

  const lastCandle = useMemo(() => {
    if (!candles.length) return null
    return candles[candles.length - 1]
  }, [candles])

  const initialZoomDoneRef = useRef(false)

  useEffect(() => {
    initialZoomDoneRef.current = false
  }, [timeframe])

  useEffect(() => {
    if (!containerRef.current || chartRef.current) return

    const chart = createChart(containerRef.current, {
      autoSize: true,
      layout: {
        background: { color: "rgba(22, 22, 22, 1)" },
        textColor: "#d1d4dc",
      },
      grid: {
        vertLines: { color: "rgba(42, 42, 42, 1)" },
        horzLines: { color: "rgba(42, 42, 42, 1)" },
      },
      
      rightPriceScale: {
        borderColor: "rgba(42, 42, 42, 1)",
      },
      timeScale: {
        borderColor: "rgba(42, 42, 42, 1)",
        timeVisible: true,
        secondsVisible: false,
        rightOffset: 5,
        barSpacing: 10,
      },
      crosshair: {
        mode: 1,
        vertLine: {
          color: "rgba(197, 203, 206, 0.6)",
          width: 1,
        },
        horzLine: {
          color: "rgba(197, 203, 206, 0.6)",
          width: 1,
        },
      },
    })

    chartRef.current = chart

    const candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: "#22c55e",
      downColor: "#ef4444",
      borderUpColor: "#22c55e",
      borderDownColor: "#ef4444",
      wickUpColor: "#22c55e",
      wickDownColor: "#ef4444",
    })

    seriesRef.current = candleSeries

    return () => {
      chart.remove()
      chartRef.current = null
      seriesRef.current = null
    }
  }, [])

  useEffect(() => {
    if (!seriesRef.current) return

    const data: CandlestickData[] = candles.map(c => ({
      time: c.time as Time,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
    }))

    seriesRef.current.setData(data)

    if (!chartRef.current || data.length === 0) return

    if (!initialZoomDoneRef.current) {
      const total = data.length

      // How many *real* candles you want visible:
      const visible = 20
      // How many "empty slots" you want to the right:
      const emptyRight = 10

      const actualVisible = Math.min(visible, total)
      const from = Math.max(0, total - actualVisible)
      const to = from + actualVisible - 1 + emptyRight

      chartRef.current.timeScale().setVisibleLogicalRange({ from, to })
      initialZoomDoneRef.current = true
    }
  }, [candles])


  const close = lastCandle?.close ?? 0
  const open = lastCandle?.open ?? 0
  const high = lastCandle?.high ?? 0
  const low = lastCandle?.low ?? 0
  const diff = close - open
  const pct = open ? (diff / open) * 100 : 0
  const isUp = diff >= 0

  const timeframes: Timeframe[] = ["1m", "30m", "1h", "4h", "1D"]

  return (
    <div className="w-full rounded-xl border border-slate-800 bg-[#020617] text-slate-50 overflow-hidden">
      {/* Header bar */}
      <div className="flex items-center justify-between border-b border-slate-800 bg-[#020617] px-4 py-2">
        <div className="flex items-center gap-3">
          <div className="rounded-md bg-slate-900 px-3 py-1 text-sm font-semibold">
            {symbol}
          </div>
          <div className="flex items-center gap-3 text-xs sm:text-sm">
            <span className="font-semibold">{close.toFixed(2)}</span>
            <span className={isUp ? "text-emerald-400" : "text-red-400"}>
              {isUp ? "+" : ""}
              {diff.toFixed(2)} ({pct.toFixed(2)}%)
            </span>
            <span className="hidden md:inline text-slate-400">
              O {open.toFixed(2)} · H {high.toFixed(2)} · L {low.toFixed(2)} · C {close.toFixed(2)}
            </span>
          </div>
        </div>

        {/* Timeframe buttons */}
        <div className="flex items-center gap-1 text-[11px] sm:text-xs text-slate-300">
          {timeframes.map(tf => (
            <button
              key={tf}
              type="button"
              onClick={() => setTimeframe(tf)}
              className={`rounded px-2 py-1 hover:bg-slate-800 transition ${
                timeframe === tf ? "bg-slate-800 text-white" : ""
              }`}
            >
              {tf}
            </button>
          ))}
        </div>
      </div>

      {/* Chart area */}
      <div
        ref={containerRef}
        className="tradingview-widget-container w-full h-[320px] sm:h-[420px] lg:h-[520px]"
        style={{ backgroundColor: "rgba(22, 22, 22, 1)" }}
      />
    </div>
  )
}
