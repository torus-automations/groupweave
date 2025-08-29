'use client';

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
    name: "GroupWeave Participation",
    description: "Participate in collaborative decision making",
    url: "https://participation.groupweave.com",
    icons: ["https://avatars.githubusercontent.com/u/37784886"],
  },
};

export default function NavigationClient() {
  const participationNavItems: NavigationItem[] = [
    { name: "Co-Create", href: "/" },
    { name: "My Decisions", href: "/decisions" },
    { name: "Group Progress", href: "/progress" },
    { name: "How It Works", href: "/guide" },
  ];

  return (
    <WalletProvider config={walletConfig}>
      <Navigation navItems={participationNavItems as any} />
    </WalletProvider>
  );
}