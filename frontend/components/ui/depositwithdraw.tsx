import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { deposit, withdraw } from "@/lib/polkadot/transactions";
import { usePolkadot } from "@/hooks/polkadot-hooks";
import { useToast } from "@/hooks/use-toast";

export function FundDialog() {
  const [showDeposit, setShowDeposit] = useState(false);
  const [showWithdraw, setShowWithdraw] = useState(false);
  const [assetId, setAssetId] = useState("");
  const [amount, setAmount] = useState("");

  const {toast} = useToast();
  const { selectedAccount } = usePolkadot();

  const handleDepositClick = async () => {
    if (!selectedAccount) return toast({ title: "Connect wallet first!" });
    try {
      await deposit(Number(assetId), BigInt(amount), selectedAccount);
      toast({ title: "Deposit successful" });
      setShowDeposit(false);
    } catch (err) {
      toast({ title: "Deposit failed", description: (err as Error).message });
    }
  };

  const handleWithdrawClick = async () => {
    if (!selectedAccount) return toast({ title: "Connect wallet first!" });
    try {
      await withdraw(Number(assetId), BigInt(amount), selectedAccount);
      toast({ title: "Withdraw successful" });
      setShowWithdraw(false);
    } catch (err) {
      toast({ title: "Withdraw failed", description: (err as Error).message });
    }
  };

  return (
    <>
      <div className="flex justify-center items-center px-4 py-3 border-b border-border bg-card">
        <div className="flex gap-20">
          <Button
            variant="secondary"
            className="bg-secondary text-secondary-foreground hover:bg-secondary/90"
            onClick={() => setShowDeposit(true)}
          >
            Deposit
          </Button>
          <Button
            variant="outline"
            className="text-foreground border-muted-foreground/30 hover:bg-muted/50"
            onClick={() => setShowWithdraw(true)}
          >
            Withdraw
          </Button>
        </div>
      </div>

      {/* Deposit Dialog */}
      <Dialog open={showDeposit} onOpenChange={setShowDeposit}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Deposit</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="assetId">Asset ID</Label>
              <Input
                id="assetId"
                type="number"
                value={assetId}
                onChange={(e) => setAssetId(e.target.value)}
              />
            </div>
            <div>
              <Label htmlFor="amount">Amount</Label>
              <Input
                id="amount"
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </div>
            <Button className="w-full" onClick={handleDepositClick}>
              Confirm Deposit
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      {/* Withdraw Dialog */}
      <Dialog open={showWithdraw} onOpenChange={setShowWithdraw}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Withdraw</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="assetId">Asset ID</Label>
              <Input
                id="assetId"
                type="number"
                value={assetId}
                onChange={(e) => setAssetId(e.target.value)}
              />
            </div>
            <div>
              <Label htmlFor="amount">Amount</Label>
              <Input
                id="amount"
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </div>
            <Button className="w-full" onClick={handleWithdrawClick}>
              Confirm Withdraw
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
