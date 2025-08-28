'use client';

import React, { createContext, useContext, useEffect, ReactNode } from 'react';
import { useWallet, UseWalletReturn } from './useWallet';
import { WalletConfig } from './WalletService';

interface WalletContextType extends UseWalletReturn {}

const WalletContext = createContext<WalletContextType | undefined>(undefined);

interface WalletProviderProps {
  children: ReactNode;
  config: WalletConfig;
}

export const WalletProvider: React.FC<WalletProviderProps> = ({ children, config }) => {
  const wallet = useWallet();

  useEffect(() => {
    if (!wallet.isInitialized) {
      wallet.initialize(config);
    }
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
    throw new Error('useWalletContext must be used within a WalletProvider');
  }
  return context;
};