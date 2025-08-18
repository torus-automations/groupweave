'use client';

import React, { ReactNode, useCallback, useEffect, useState } from 'react';
import { setupWalletSelector } from '@near-wallet-selector/core';
import type { WalletSelector, AccountState } from '@near-wallet-selector/core';
import { setupModal } from '@near-wallet-selector/modal-ui';
import type { WalletSelectorModal } from '@near-wallet-selector/modal-ui';
import { setupMyNearWallet } from '@near-wallet-selector/my-near-wallet';
import { setupSender } from '@near-wallet-selector/sender';

const CONTRACT_ID = 'hello.near-examples.testnet';

export const WalletSelectorContext = React.createContext<{
  selector: WalletSelector | null;
  modal: WalletSelectorModal | null;
  accounts: Array<AccountState>;
  accountId: string | null;
}>({
  selector: null,
  modal: null,
  accounts: [],
  accountId: null,
});

export function WalletContextProvider({ children }: { children: ReactNode }) {
  const [selector, setSelector] = useState<WalletSelector | null>(null);
  const [modal, setModal] = useState<WalletSelectorModal | null>(null);
  const [accounts, setAccounts] = useState<Array<AccountState>>([]);

  const init = useCallback(async () => {
    const _selector = await setupWalletSelector({
      network: 'testnet',
      modules: [setupMyNearWallet(), setupSender()],
    });
    const _modal = setupModal(_selector, { contractId: CONTRACT_ID });
    const state = _selector.store.getState();
    setAccounts(state.accounts);
    setSelector(_selector);
    setModal(_modal);
  }, []);

  useEffect(() => {
    init().catch((err) => {
      console.error(err);
      alert('Failed to initialize wallet selector');
    });
  }, [init]);

  useEffect(() => {
    if (!selector) return;

    const subscription = selector.store.subscribe(() => {
      const state = selector.store.getState();
      setAccounts(state.accounts);
    });

    return () => subscription();
  }, [selector]);

  const accountId = accounts.find((account) => account.active)?.accountId || null;

  return (
    <WalletSelectorContext.Provider value={{ selector, modal, accounts, accountId }}>
      {children}
    </WalletSelectorContext.Provider>
  );
}

export function useWalletSelector() {
  const context = React.useContext(WalletSelectorContext);
  if (!context) {
    throw new Error('useWalletSelector must be used within a WalletContextProvider');
  }
  return context;
}
