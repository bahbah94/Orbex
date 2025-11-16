import { http, createConfig } from "wagmi"
import { mainnet, arbitrum, optimism, polygon } from "wagmi/chains"
import { injected, walletConnect, coinbaseWallet } from "wagmi/connectors"

// WalletConnect Project ID - get from https://cloud.walletconnect.com
const projectId = process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || "demo-project-id"

export const config = createConfig({
  chains: [mainnet, arbitrum, optimism, polygon],
  connectors: [injected(), walletConnect({ projectId }), coinbaseWallet({ appName: "Orbex" })],
  transports: {
    [mainnet.id]: http(),
    [arbitrum.id]: http(),
    [optimism.id]: http(),
    [polygon.id]: http(),
  },
})
