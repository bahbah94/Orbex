"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Slider } from "@/components/ui/slider"
import { useToast } from "@/hooks/use-toast"

interface TradingFormProps {
  selectedPair: string
}

export function TradingForm({ selectedPair }: TradingFormProps) {
  const [buyAmount, setBuyAmount] = useState("")
  const [buyPrice, setBuyPrice] = useState("")
  const [sellAmount, setSellAmount] = useState("")
  const [sellPrice, setSellPrice] = useState("")
  const [buyPercentage, setBuyPercentage] = useState([0])
  const [sellPercentage, setSellPercentage] = useState([0])
  const { toast } = useToast()

  const handleBuy = () => {
    toast({
      title: "Buy Order Placed",
      description: `Buying ${buyAmount} ${selectedPair.split("/")[0]} at $${buyPrice}`,
    })
    setBuyAmount("")
    setBuyPrice("")
    setBuyPercentage([0])
  }

  const handleSell = () => {
    toast({
      title: "Sell Order Placed",
      description: `Selling ${sellAmount} ${selectedPair.split("/")[0]} at $${sellPrice}`,
    })
    setSellAmount("")
    setSellPrice("")
    setSellPercentage([0])
  }

  return (
    <div className="flex h-full flex-col border-border bg-card">
      <div className="border-b border-border px-4 py-3">
        <div className="text-base font-semibold text-card-foreground">Trade</div>
      </div>
      <div className="flex-1 overflow-y-auto px-4 py-4">
        <Tabs defaultValue="buy" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger
              value="buy"
              className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground"
            >
              Buy
            </TabsTrigger>
            <TabsTrigger
              value="sell"
              className="data-[state=active]:bg-destructive data-[state=active]:text-destructive-foreground"
            >
              Sell
            </TabsTrigger>
          </TabsList>

          <TabsContent value="buy" className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="buy-price" className="text-card-foreground">
                Price (USD)
              </Label>
              <Input
                id="buy-price"
                type="number"
                placeholder="0.00"
                value={buyPrice}
                onChange={(e) => setBuyPrice(e.target.value)}
                className="bg-background text-foreground"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="buy-amount" className="text-card-foreground">
                Amount ({selectedPair.split("/")[0]})
              </Label>
              <Input
                id="buy-amount"
                type="number"
                placeholder="0.00"
                value={buyAmount}
                onChange={(e) => setBuyAmount(e.target.value)}
                className="bg-background text-foreground"
              />
            </div>

            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <Label className="text-card-foreground">Amount</Label>
                <span className="text-muted-foreground">{buyPercentage[0]}%</span>
              </div>
              <Slider value={buyPercentage} onValueChange={setBuyPercentage} max={100} step={25} className="w-full" />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>0%</span>
                <span>25%</span>
                <span>50%</span>
                <span>75%</span>
                <span>100%</span>
              </div>
            </div>

            <div className="space-y-2 rounded-lg bg-muted px-4 py-3">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Available Balance:</span>
                <span className="font-medium text-card-foreground">10,000.00 USD</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Total:</span>
                <span className="font-medium text-card-foreground">
                  {buyAmount && buyPrice
                    ? (Number.parseFloat(buyAmount) * Number.parseFloat(buyPrice)).toFixed(2)
                    : "0.00"}{" "}
                  USD
                </span>
              </div>
            </div>

            <Button
              onClick={handleBuy}
              className="w-full bg-primary text-primary-foreground hover:bg-primary/90"
              size="lg"
            >
              Buy {selectedPair.split("/")[0]}
            </Button>
          </TabsContent>

          <TabsContent value="sell" className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="sell-price" className="text-card-foreground">
                Price (USD)
              </Label>
              <Input
                id="sell-price"
                type="number"
                placeholder="0.00"
                value={sellPrice}
                onChange={(e) => setSellPrice(e.target.value)}
                className="bg-background text-foreground"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="sell-amount" className="text-card-foreground">
                Amount ({selectedPair.split("/")[0]})
              </Label>
              <Input
                id="sell-amount"
                type="number"
                placeholder="0.00"
                value={sellAmount}
                onChange={(e) => setSellAmount(e.target.value)}
                className="bg-background text-foreground"
              />
            </div>

            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <Label className="text-card-foreground">Amount</Label>
                <span className="text-muted-foreground">{sellPercentage[0]}%</span>
              </div>
              <Slider value={sellPercentage} onValueChange={setSellPercentage} max={100} step={25} className="w-full" />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>0%</span>
                <span>25%</span>
                <span>50%</span>
                <span>75%</span>
                <span>100%</span>
              </div>
            </div>

            <div className="space-y-2 rounded-lg bg-muted px-4 py-3">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Available Balance:</span>
                <span className="font-medium text-card-foreground">0.5234 {selectedPair.split("/")[0]}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Total:</span>
                <span className="font-medium text-card-foreground">
                  {sellAmount && sellPrice
                    ? (Number.parseFloat(sellAmount) * Number.parseFloat(sellPrice)).toFixed(2)
                    : "0.00"}{" "}
                  USD
                </span>
              </div>
            </div>

            <Button
              onClick={handleSell}
              className="w-full bg-destructive text-destructive-foreground hover:bg-destructive/90"
              size="lg"
            >
              Sell {selectedPair.split("/")[0]}
            </Button>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}
