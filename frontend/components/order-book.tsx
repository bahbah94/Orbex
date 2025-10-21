"use client"

import { useState, useEffect, useRef } from "react"
import { useOrderBook } from "@/hooks/use-order-book"
import { useTrades } from "@/hooks/use-trades"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"

function OrderBookRow({
  price,
  size,
  total,
  sizeNum,
  maxSize,
  type,
  priceGrouping,
}: {
  price: string
  size: string
  total: string
  sizeNum: number
  maxSize: number
  type: "ask" | "bid"
  priceGrouping: number
}) {
  const [flash, setFlash] = useState(false)
  const prevSizeRef = useRef(sizeNum)

  useEffect(() => {
    // Detect size changes and trigger flash animation
    if (prevSizeRef.current !== sizeNum) {
      setFlash(true)
      const timer = setTimeout(() => setFlash(false), 300)
      prevSizeRef.current = sizeNum
      return () => clearTimeout(timer)
    }
  }, [sizeNum])

  // Calculate heatmap intensity (0-1)
  const intensity = maxSize > 0 ? sizeNum / maxSize : 0

  const bgGradient =
    type === "ask"
      ? `linear-gradient(to right, transparent ${100 - intensity * 100}%, rgba(239, 68, 68, ${intensity * 0.3}) ${100 - intensity * 100}%)`
      : `linear-gradient(to right, transparent ${100 - intensity * 100}%, rgba(34, 197, 94, ${intensity * 0.3}) ${100 - intensity * 100}%)`

  const formatPrice = (priceStr: string, grouping: number): string => {
    const priceNum = Number(priceStr)

    if (grouping >= 1) {
      // Round up to nearest multiple of grouping
      const rounded = Math.ceil(priceNum / grouping) * grouping
      return rounded.toFixed(0)
    } else {
      // For decimal groupings, use decimal places
      const decimals = Math.max(0, -Math.log10(grouping))
      return priceNum.toFixed(decimals)
    }
  }

  const formattedPrice = formatPrice(price, priceGrouping)

  return (
    <div
      className={`relative grid grid-cols-3 gap-2 py-0.5 font-mono text-[0.75rem] transition-all ${
        flash ? (type === "ask" ? "bg-red-500/40" : "bg-green-500/40") : ""
      } hover:bg-accent/50`}
      style={{ background: flash ? undefined : bgGradient }}
    >
      <div className={type === "ask" ? "text-red-500" : "text-green-500"}>{formattedPrice}</div>
      <div className="text-right text-foreground">{size}</div>
      <div className="text-right text-muted-foreground">{total}</div>
    </div>
  )
}

export function OrderBook() {
  const [activeTab, setActiveTab] = useState<"orderbook" | "trades">("orderbook")
  const [priceGrouping, setPriceGrouping] = useState<number>(0.01)

  const { asks, bids, spread, spreadPercent, maxSize } = useOrderBook()
  const trades = useTrades()

  return (
    <div className="flex h-full flex-col bg-card">
      <div className="flex border-b border-border">
        <button
          onClick={() => setActiveTab("orderbook")}
          className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
            activeTab === "orderbook"
              ? "border-b-2 border-primary text-foreground"
              : "text-muted-foreground hover:text-foreground"
          }`}
        >
          Order Book
        </button>
        <button
          onClick={() => setActiveTab("trades")}
          className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
            activeTab === "trades"
              ? "border-b-2 border-primary text-foreground"
              : "text-muted-foreground hover:text-foreground"
          }`}
        >
          Trades
        </button>
      </div>

      {activeTab === "orderbook" ? (
        <>
          <div className="flex items-center justify-between border-b border-border px-4 py-2">
            <Select value={priceGrouping.toString()} onValueChange={(value) => setPriceGrouping(Number(value))}>
              <SelectTrigger className="h-8 w-[7.5rem] text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0.01">Price - 0.01</SelectItem>
                <SelectItem value="0.1">Price - 0.1</SelectItem>
                <SelectItem value="1">Price - 1</SelectItem>
                <SelectItem value="10">Price - 10</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="grid grid-cols-3 gap-2 border-b border-border px-4 py-2 text-[0.6875rem] font-medium text-muted-foreground">
            <div>Price</div>
            <div className="text-right">
              Size <span className="text-green-500">ETH</span>
            </div>
            <div className="text-right">Total</div>
          </div>

          <div className="flex-1 overflow-y-auto">
            <div className="px-4 py-1">
              {asks.map((ask, i) => (
                <OrderBookRow
                  key={`ask-${i}`}
                  price={ask.price}
                  size={ask.size}
                  total={ask.total}
                  sizeNum={ask.sizeNum}
                  maxSize={maxSize}
                  type="ask"
                  priceGrouping={priceGrouping}
                />
              ))}
            </div>

            <div className="bg-accent/30 px-4 py-2">
              <div className="flex items-center justify-center gap-2 text-[0.6875rem]">
                <span className="text-muted-foreground">{spread}</span>
                <span className="text-muted-foreground">Spread</span>
                <span className="text-muted-foreground">{spreadPercent}%</span>
              </div>
            </div>

            <div className="px-4 py-1">
              {bids.map((bid, i) => (
                <OrderBookRow
                  key={`bid-${i}`}
                  price={bid.price}
                  size={bid.size}
                  total={bid.total}
                  sizeNum={bid.sizeNum}
                  maxSize={maxSize}
                  type="bid"
                  priceGrouping={priceGrouping}
                />
              ))}
            </div>
          </div>
        </>
      ) : (
        <>
          <div className="grid grid-cols-3 gap-2 border-b border-border px-4 py-2 text-[0.6875rem] font-medium text-muted-foreground">
            <div>Price</div>
            <div className="text-right">Size</div>
            <div className="text-right">Time</div>
          </div>

          <div className="flex-1 overflow-y-auto px-4 py-1">
            {trades.map((trade, i) => (
              <div
                key={`trade-${i}`}
                className="grid grid-cols-3 gap-2 py-0.5 font-mono text-[0.75rem] transition-colors hover:bg-accent/50"
              >
                <div className={trade.isBuy ? "text-green-500" : "text-red-500"}>{trade.price}</div>
                <div className="text-right text-foreground">{trade.size}</div>
                <div className="text-right text-muted-foreground">{trade.time}</div>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  )
}
