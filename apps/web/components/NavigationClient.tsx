'use client';

import { Navigation } from '@repo/ui/navigation';
import { useWalletSelector } from './WalletContextProvider';

export default function NavigationClient() {
  const { selector, modal, accountId } = useWalletSelector();

  const handleConnect = () => {
    if (modal) {
      modal.show();
    }
  };

  const handleDisconnect = async () => {
    if (selector) {
      const wallet = await selector.wallet();
      await wallet.signOut();
    }
  };

  return (
    <Navigation
      accountId={accountId}
      onConnect={handleConnect}
      onDisconnect={handleDisconnect}
    />
  );
}