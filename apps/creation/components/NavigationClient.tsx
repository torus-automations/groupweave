'use client';

import { Navigation, WalletProvider, NavigationItem } from '@repo/ui';
import { usePathname } from 'next/navigation';

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
  const activePath = usePathname();

  const creationNavItems: NavigationItem[] = [
    { name: "Create", href: "/" },
    { name: "Templates", href: "/templates" },
    { name: "My Projects", href: "/projects" },
    { name: "Collaborate", href: "/collaborate" },
  ];

  return (
    <WalletProvider config={walletConfig}>
      <Navigation navItems={creationNavItems} activePath={activePath} />
    </WalletProvider>
  );
}