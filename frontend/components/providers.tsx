'use client'

import type React from 'react'
import { useMemo, useState } from 'react'

import { WagmiProvider, createConfig, http } from 'wagmi'
import { mainnet } from 'wagmi/chains'
// IMPORTANT: use injected() instead of metaMask() to avoid pulling @metamask/sdk
import { injected } from 'wagmi/connectors'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient())

  // Create config on the client; injected() does not touch indexedDB/RN storage.
  const config = useMemo(() => {
    return createConfig({
      chains: [mainnet],
      transports: { [mainnet.id]: http() },
      connectors: [injected()],
      // Do NOT enable any persistence layer that uses indexedDB on module load.
    })
  }, [])

  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </WagmiProvider>
  )
}
