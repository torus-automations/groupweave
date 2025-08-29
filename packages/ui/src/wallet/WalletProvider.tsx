'use client';

import React, { createContext, useContext, useEffect, ReactNode } from 'react';
import { useWallet, UseWalletReturn } from './useWallet';
import { WalletConfig } from './WalletService';

interface WalletContextType extends UseWalletReturn { }

const WalletContext = createContext<WalletContextType | undefined>(undefined);

interface WalletProviderProps {
  children: ReactNode;
  config: WalletConfig;
}

export const WalletProvider: React.FC<WalletProviderProps> = ({ children, config }) => {
  const wallet = useWallet();

  useEffect(() => {
    const initializeWallet = async () => {
      if (!wallet.isInitialized) {
        await wallet.initialize(config);
      }
    };

    initializeWallet();
  }, [config, wallet]);

  return (
    <WalletContext.Provider value={wallet}>
      {children}
    </WalletContext.Provider>
  );
};

export const useWalletContext = (): WalletContextType => {
  const context = useContext(WalletContext);
  if (context === undefined) {
    // Return default context when no provider is available (e.g., during SSR)
    return {
      isConnected: false,
      accountId: null,
      isLoading: false,
      connect: async () => {
        console.warn('Wallet connect called outside of WalletProvider');
      },
      disconnect: async () => {
        console.warn('Wallet disconnect called outside of WalletProvider');
      },
      initialize: async () => {
        console.warn('Wallet initialize called outside of WalletProvider');
      },
      isInitialized: false,
    };
  }
  return context;
};