"use client"

import { useEffect, useMemo, useRef, useState } from "react"

export type Candle = {
  time: number // unix seconds (UDF-compatible)
  open: number
  high: number
  low: number
  close: number
  volume?: number
}

export type UseSimulatedCandlesOpts = {
  symbol: string
  candleMs?: number      // default 1m
  history?: number       // default 400
  tickMs?: number        // default 1000ms
  volatility?: number    // default 0.002 (0.2% per tick band)
  startPrice?: number    // default derived from symbol hash
}

function mulberry32(seed: number) {
  let t = seed >>> 0
  return () => {
    t += 0x6D2B79F5
    let r = Math.imul(t ^ (t >>> 15), 1 | t)
    r ^= r + Math.imul(r ^ (r >>> 7), 61 | r)
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296
  }
}

function hashStr(s: string) {
  let h = 2166136261 >>> 0
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return h >>> 0
}

export function useSimulatedCandles({
  symbol,
  candleMs = 60_000,
  history = 400,
  tickMs = 1000,
  volatility = 0.002,
  startPrice,
}: UseSimulatedCandlesOpts) {
  const seed = useMemo(() => hashStr(symbol || "SYMBOL"), [symbol])
  const rng = useMemo(() => mulberry32(seed), [seed])

  const [candles, setCandles] = useState<Candle[]>([])

  const lastPriceRef = useRef<number>(startPrice ?? 100)
  const currentStartRef = useRef<number>(Math.floor(Date.now() / 1000))

  // 🔁 (Re)build initial history whenever resolution / symbol / params change
  useEffect(() => {
    const now = Date.now()
    const start = Math.floor((now - history * candleMs) / 1000)
    const base = startPrice ?? (100 + (rng() - 0.5) * 40)

    const arr: Candle[] = []
    let lastClose = base

    for (let i = 0; i < history; i++) {
      const t = start + Math.floor((i * candleMs) / 1000)
      const drift = (rng() - 0.5) * 2 * volatility * 20
      const open = lastClose
      const close = Math.max(0.0001, open * (1 + drift))
      const spread = Math.abs(close - open)
      const high = Math.max(open, close) + spread * (0.2 + rng() * 0.8)
      const low = Math.min(open, close) - spread * (0.2 + rng() * 0.8)
      const vol = 10 + rng() * 90
      arr.push({ time: t, open, high, low, close, volume: vol })
      lastClose = close
    }

    setCandles(arr)
    lastPriceRef.current = lastClose
    currentStartRef.current = arr[arr.length - 1]?.time ?? Math.floor(Date.now() / 1000)
  }, [symbol, candleMs, history, volatility, startPrice, rng])

  // Align the current candle bucket when candleMs changes
  useEffect(() => {
    const nowMs = Date.now()
    const aligned = nowMs - (nowMs % candleMs)
    currentStartRef.current = Math.floor(aligned / 1000)
  }, [candleMs])

  // Live ticking within the latest candle / rolling to next candle
  useEffect(() => {
    const timer = setInterval(() => {
      const nowMs = Date.now()
      const candleStartMs = currentStartRef.current * 1000
      const nextBucket = nowMs >= candleStartMs + candleMs

      const lp = lastPriceRef.current
      const move = (rng() - 0.5) * 2 * volatility
      const next = Math.max(0.0001, lp * (1 + move))
      lastPriceRef.current = next

      setCandles(prev => {
        const last = prev[prev.length - 1]
        if (!last) {
          const t = Math.floor(nowMs / 1000)
          return [{ time: t, open: next, high: next, low: next, close: next, volume: 0 }]
        }

        if (nextBucket) {
          const start = candleStartMs + candleMs
          currentStartRef.current = Math.floor(start / 1000)
          const open = last.close
          const close = next
          const high = Math.max(open, close)
          const low = Math.min(open, close)
          const vol = (last.volume ?? 50) * (0.9 + rng() * 0.2)
          return [
            ...prev,
            {
              time: Math.floor(start / 1000), // ⬅️ still unix seconds (UDF style)
              open,
              high,
              low,
              close,
              volume: vol,
            },
          ]
        } else {
          const updated = { ...last }
          updated.close = next
          updated.high = Math.max(updated.high, next)
          updated.low = Math.min(updated.low, next)
          updated.volume = (updated.volume ?? 0) + (0.2 + rng() * 0.8)
          const copy = prev.slice(0, -1)
          copy.push(updated)
          return copy
        }
      })
    }, tickMs)

    return () => clearInterval(timer)
  }, [tickMs, candleMs, rng, volatility])

  return candles
}
