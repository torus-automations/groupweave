'use client';

import { Navigation, WalletProvider, NavigationItem } from '@repo/ui';
import { usePathname } from 'next/navigation';


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
  const activePath = usePathname();

  const participationNavItems: NavigationItem[] = [
    { name: "Co-Create", href: "/" },
    { name: "My Decisions", href: "/decisions" },
    { name: "Group Progress", href: "/progress" },
    { name: "How It Works", href: "/guide" },
  ];

  return (
    <WalletProvider config={walletConfig}>
      <Navigation navItems={participationNavItems} activePath={activePath} />
    </WalletProvider>
  );
}