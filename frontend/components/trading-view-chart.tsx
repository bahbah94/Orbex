"use client"

import { useEffect, useRef, memo } from "react"

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
