'use client';

import { useEffect, useState } from 'react';
import { Navigation, WalletProvider } from '@repo/ui';

// Define NavigationItem type locally since it's not exported in the built package
interface NavigationItem {
  name: string;
  href: string;
}

const walletConfig = {
  network: "testnet" as const,
  contractId: "guest-book.testnet", // Replace with your actual contract ID
  walletConnectProjectId: "c4f79cc...", // Replace with your WalletConnect project ID
  dAppMetadata: {
    name: "GroupWeave Creation",
    description: "Create and manage collaborative projects",
    url: "https://creation.groupweave.com",
    icons: ["https://avatars.githubusercontent.com/u/37784886"],
  },
};

export default function NavigationClient() {
  const [isClient, setIsClient] = useState(false);
  
  useEffect(() => {
    setIsClient(true);
  }, []);

  const creationNavItems: NavigationItem[] = [
    { name: "Create", href: "/" },
    { name: "Templates", href: "/templates" },
    { name: "My Projects", href: "/projects" },
    { name: "Collaborate", href: "/collaborate" },
  ];

  // Only render wallet provider on client side to avoid SSR issues
  if (!isClient) {
    return <Navigation navItems={creationNavItems as any} />;
  }

  return (
    <WalletProvider config={walletConfig}>
      <Navigation navItems={creationNavItems as any} />
    </WalletProvider>
  );
}