"use client"

import { useState } from "react"
import { useBalances } from "@/hooks/use-balances"
import { usePositions } from "@/hooks/use-positions"
import { useOpenOrders } from "@/hooks/use-open-orders"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent } from "@/components/ui/card"

export function AccountTabs() {
  const [activeTab, setActiveTab] = useState("balances")

  const balances = useBalances()
  const positions = usePositions()
  const openOrders = useOpenOrders()

  return (
    <Card className="border-border bg-card">
      <CardContent className="p-0">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="w-full justify-start rounded-none border-b border-border bg-transparent p-0">
            <TabsTrigger
              value="balances"
              className="rounded-none border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent border-b-2"
            >
              Balances
            </TabsTrigger>
            <TabsTrigger
              value="positions"
              className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              Positions
            </TabsTrigger>
            <TabsTrigger
              value="open-orders"
              className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              Open Orders
            </TabsTrigger>
            <TabsTrigger
              value="twap"
              className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              TWAP
            </TabsTrigger>
            <TabsTrigger
              value="trade-history"
              className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              Trade History
            </TabsTrigger>
            <TabsTrigger
              value="funding-history"
              className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              Funding History
            </TabsTrigger>
            <TabsTrigger
              value="order-history"
              className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              Order History
            </TabsTrigger>
          </TabsList>

          <TabsContent value="balances" className="m-0 p-4">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-border text-left text-xs text-muted-foreground">
                    <th className="pb-2 font-medium">Asset</th>
                    <th className="pb-2 font-medium text-right">Amount</th>
                    <th className="pb-2 font-medium text-right">Available</th>
                    <th className="pb-2 font-medium text-right">Value</th>
                  </tr>
                </thead>
                <tbody>
                  {balances.map((balance) => (
                    <tr key={balance.asset} className="border-b border-border/50">
                      <td className="py-3 font-medium text-foreground">{balance.asset}</td>
                      <td className="py-3 text-right font-mono text-foreground">{balance.amount}</td>
                      <td className="py-3 text-right font-mono text-muted-foreground">{balance.available}</td>
                      <td className="py-3 text-right font-mono text-foreground">{balance.value}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </TabsContent>

          <TabsContent value="positions" className="m-0 p-4">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-border text-left text-xs text-muted-foreground">
                    <th className="pb-2 font-medium">Pair</th>
                    <th className="pb-2 font-medium">Side</th>
                    <th className="pb-2 font-medium text-right">Size</th>
                    <th className="pb-2 font-medium text-right">Entry Price</th>
                    <th className="pb-2 font-medium text-right">Mark Price</th>
                    <th className="pb-2 font-medium text-right">PnL</th>
                  </tr>
                </thead>
                <tbody>
                  {positions.map((position, idx) => (
                    <tr key={idx} className="border-b border-border/50">
                      <td className="py-3 font-medium text-foreground">{position.pair}</td>
                      <td className="py-3">
                        <span className={position.side === "Long" ? "text-primary" : "text-destructive"}>
                          {position.side}
                        </span>
                      </td>
                      <td className="py-3 text-right font-mono text-foreground">{position.size}</td>
                      <td className="py-3 text-right font-mono text-muted-foreground">{position.entryPrice}</td>
                      <td className="py-3 text-right font-mono text-foreground">{position.markPrice}</td>
                      <td className="py-3 text-right font-mono text-primary">
                        {position.pnl} ({position.pnlPercent})
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </TabsContent>

          <TabsContent value="open-orders" className="m-0 p-4">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-border text-left text-xs text-muted-foreground">
                    <th className="pb-2 font-medium">Pair</th>
                    <th className="pb-2 font-medium">Type</th>
                    <th className="pb-2 font-medium">Side</th>
                    <th className="pb-2 font-medium text-right">Price</th>
                    <th className="pb-2 font-medium text-right">Amount</th>
                    <th className="pb-2 font-medium text-right">Filled</th>
                    <th className="pb-2 font-medium text-right">Total</th>
                  </tr>
                </thead>
                <tbody>
                  {openOrders.map((order, idx) => (
                    <tr key={idx} className="border-b border-border/50">
                      <td className="py-3 font-medium text-foreground">{order.pair}</td>
                      <td className="py-3 text-muted-foreground">{order.type}</td>
                      <td className="py-3">
                        <span className={order.side === "Buy" ? "text-primary" : "text-destructive"}>{order.side}</span>
                      </td>
                      <td className="py-3 text-right font-mono text-foreground">{order.price}</td>
                      <td className="py-3 text-right font-mono text-foreground">{order.amount}</td>
                      <td className="py-3 text-right font-mono text-muted-foreground">{order.filled}</td>
                      <td className="py-3 text-right font-mono text-foreground">{order.total}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </TabsContent>

          <TabsContent value="twap" className="m-0 p-4">
            <div className="flex h-32 items-center justify-center text-muted-foreground">No TWAP orders</div>
          </TabsContent>

          <TabsContent value="trade-history" className="m-0 p-4">
            <div className="flex h-32 items-center justify-center text-muted-foreground">No trade history</div>
          </TabsContent>

          <TabsContent value="funding-history" className="m-0 p-4">
            <div className="flex h-32 items-center justify-center text-muted-foreground">No funding history</div>
          </TabsContent>

          <TabsContent value="order-history" className="m-0 p-4">
            <div className="flex h-32 items-center justify-center text-muted-foreground">No order history</div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}
