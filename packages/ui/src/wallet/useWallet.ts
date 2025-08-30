'use client';

import { useEffect, useState } from 'react';
import { walletService, WalletConfig } from './WalletService';

export interface UseWalletReturn {
  isConnected: boolean;
  accountId: string | null;
  isLoading: boolean;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  initialize: (config: WalletConfig) => Promise<void>;
  isInitialized: boolean;
}

export const useWallet = (): UseWalletReturn => {
  const [isConnected, setIsConnected] = useState(false);
  const [accountId, setAccountId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);

  const initialize = async (config: WalletConfig) => {
    try {
      setIsLoading(true);
      await walletService.initialize(config);
      setIsInitialized(true);
      
      // Check if already connected
      const currentAccountId = await walletService.getAccountId();
      if (currentAccountId) {
        setAccountId(currentAccountId);
        setIsConnected(true);
      }
    } catch (error) {
      console.error('Failed to initialize wallet:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const connect = async () => {
    try {
      setIsLoading(true);
      await walletService.connect();
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const disconnect = async () => {
    try {
      setIsLoading(true);
      await walletService.disconnect();
      setIsConnected(false);
      setAccountId(null);
    } catch (error) {
      console.error('Failed to disconnect wallet:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    const checkConnection = async () => {
      if (walletService.isInitialized()) {
        const currentAccountId = await walletService.getAccountId();
        setIsConnected(!!currentAccountId);
        setAccountId(currentAccountId);
        setIsInitialized(true);
      }
    };

    checkConnection();
  }, []);

  return {
    isConnected,
    accountId,
    isLoading,
    connect,
    disconnect,
    initialize,
    isInitialized: isInitialized && walletService.isInitialized(),
  };
};