// Example setup for each app in /apps/dashboard, /apps/contracts, /apps/creation

import React from 'react';
import { WalletProvider, Navigation } from '@repo/ui';

// App-specific configuration
const walletConfig = {
  network: "testnet" as const, // or "mainnet"
  contractId: "guest-book.testnet", // Replace with your contract ID
  walletConnectProjectId: "c4f79cc...", // Replace with your WalletConnect project ID
  unityProjectId: "c4f79cc...", // Replace with your Unity project ID (optional)
  dAppMetadata: {
    name: "Your dApp Name", // Customize per app
    description: "Your dApp Description",
    url: "https://your-app-url.com",
    icons: ["https://your-app-icon.com/icon.png"],
  },
};

// Example App Layout Component
export function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <WalletProvider config={walletConfig}>
      <div className="min-h-screen bg-gray-50">
        <Navigation />
        <main className="pt-20">
          {children}
        </main>
      </div>
    </WalletProvider>
  );
}

// Example usage in your app's root component (App.tsx or layout.tsx)
export function App() {
  return (
    <AppLayout>
      <div className="container mx-auto px-4 py-8">
        <h1>Your App Content</h1>
        {/* Your app content here */}
      </div>
    </AppLayout>
  );
}

// Example of using wallet functionality in a component
import { useWalletContext } from '@repo/ui';

export function WalletInfo() {
  const { isConnected, accountId, connect, disconnect } = useWalletContext();

  return (
    <div className="p-4 border rounded-lg">
      {isConnected ? (
        <div>
          <p>Connected as: {accountId}</p>
          <button onClick={disconnect}>Disconnect</button>
        </div>
      ) : (
        <div>
          <p>Not connected</p>
          <button onClick={connect}>Connect Wallet</button>
        </div>
      )}
    </div>
  );
}