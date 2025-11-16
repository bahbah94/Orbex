// src/hooks/usePolkadot.ts
import { useEffect, useState } from "react";
import { web3Enable, web3Accounts } from "@polkadot/extension-dapp";

export function usePolkadot() {
  const [accounts, setAccounts] = useState<{ address: string; meta: any }[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      // Enable Polkadot.js extension
      await web3Enable("OrderbookClob");

      // Get all available accounts
      const accs = await web3Accounts();
      setAccounts(accs);

      // Auto-select first account if available
      if (accs.length > 0) setSelectedAccount(accs[0].address);
    })();
  }, []);

  return { accounts, selectedAccount, setSelectedAccount };
}
