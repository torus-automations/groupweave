'use client';

import { Navigation, WalletProvider, NavigationItem } from '@repo/ui';
import { usePathname } from 'next/navigation';

const walletConfig = {
  network: "testnet" as const,
  contractId: "guest-book.testnet", // Replace with your actual contract ID
  walletConnectProjectId: "c4f79cc...", // Replace with your WalletConnect project ID
  dAppMetadata: {
    name: "GroupWeave Dashboard",
    description: "Manage and monitor your collaborative projects",
    url: "https://dashboard.groupweave.com",
    icons: ["https://avatars.githubusercontent.com/u/37784886"],
  },
};

export default function NavigationClient() {
  const activePath = usePathname();

  const dashboardNavItems: NavigationItem[] = [
    { name: "Overview", href: "/" },
    { name: "Analytics", href: "/analytics" },
    { name: "Projects", href: "/projects" },
    { name: "Settings", href: "/settings" },
  ];

  return (
    <WalletProvider config={walletConfig}>
      <Navigation navItems={dashboardNavItems} activePath={activePath} />
    </WalletProvider>
  );
}