import { setupWalletSelector, WalletSelector } from "@near-wallet-selector/core";
import { setupModal, WalletSelectorModal } from "@near-wallet-selector/modal-ui";
import "@near-wallet-selector/modal-ui/styles.css";
import { setupArepaWallet } from "@near-wallet-selector/arepa-wallet";
import { setupBitgetWallet } from "@near-wallet-selector/bitget-wallet";
import { setupBitteWallet } from "@near-wallet-selector/bitte-wallet";
import { setupCoin98Wallet } from "@near-wallet-selector/coin98-wallet";
import { setupEthereumWallets } from "@near-wallet-selector/ethereum-wallets"; // Requires wagmiConfig
import { setupHereWallet } from "@near-wallet-selector/here-wallet";
import { setupHotWallet } from "@near-wallet-selector/hot-wallet";
import { setupIntearWallet } from "@near-wallet-selector/intear-wallet";
import { setupLedger } from "@near-wallet-selector/ledger"; // Disabled due to Node.js dependencies
import { setupMathWallet } from "@near-wallet-selector/math-wallet";
import { setupMeteorWallet } from "@near-wallet-selector/meteor-wallet";
import { setupMeteorWalletApp } from "@near-wallet-selector/meteor-wallet-app";
import { setupMyNearWallet } from "@near-wallet-selector/my-near-wallet";
import { setupNarwallets } from "@near-wallet-selector/narwallets";
import { setupNearMobileWallet } from "@near-wallet-selector/near-mobile-wallet";
import { setupNearSnap } from "@near-wallet-selector/near-snap"; // Disabled due to Node.js dependencies
import { setupNightly } from "@near-wallet-selector/nightly";
import { setupOKXWallet } from "@near-wallet-selector/okx-wallet";
import { setupRamperWallet } from "@near-wallet-selector/ramper-wallet";
import { setupSender } from "@near-wallet-selector/sender";
import { setupUnityWallet } from "@near-wallet-selector/unity-wallet";
import { setupWalletConnect } from "@near-wallet-selector/wallet-connect";
import { setupWelldoneWallet } from "@near-wallet-selector/welldone-wallet";
import { setupXDEFI } from "@near-wallet-selector/xdefi";

export interface WalletConfig {
  network: "testnet" | "mainnet";
  contractId: string;
  walletConnectProjectId?: string;
  unityProjectId?: string;
  dAppMetadata?: {
    name: string;
    description: string;
    url: string;
    icons: string[];
  };
}

class WalletService {
  private selector: WalletSelector | null = null;
  private modal: WalletSelectorModal | null = null;
  private config: WalletConfig | null = null;

  async initialize(config: WalletConfig) {
    this.config = config;
    
    const defaultMetadata = {
      name: "Torus GroupWeave",
      description: "Decentralized collaboration platform",
      url: "https://torus.groupweave.com",
      icons: ["https://avatars.githubusercontent.com/u/37784886"],
    };

    const metadata = config.dAppMetadata || defaultMetadata;

    try {
      this.selector = await setupWalletSelector({
        network: config.network,
        modules: [
          setupArepaWallet(),
          setupBitgetWallet(),
          setupBitteWallet() as any, // Type compatibility issue with wallet selector
          setupCoin98Wallet(),
          // setupEthereumWallets requires wagmiConfig, skip for now
          // setupEthereumWallets({
          //   wagmiConfig: config.wagmiConfig,
          // }),
          setupHereWallet(),
          setupHotWallet(),
          setupIntearWallet(),
          setupLedger(), // Disabled due to potential Node.js dependencies
          setupMathWallet(),
          setupMeteorWallet(),
          setupMeteorWalletApp({ contractId: config.contractId }),
          setupMyNearWallet(),
          setupNarwallets(),
          setupNearMobileWallet(),
          setupNearSnap(), // Disabled due to Node.js dependencies
          setupNightly(),
          setupOKXWallet(),
          setupRamperWallet(),
          setupSender(),
          ...(config.unityProjectId ? [setupUnityWallet({
            projectId: config.unityProjectId,
            metadata,
          })] : []),
          ...(config.walletConnectProjectId ? [setupWalletConnect({
            projectId: config.walletConnectProjectId,
            metadata,
          })] : []),
          setupWelldoneWallet(),
          setupXDEFI(),
        ],
      });

      this.modal = setupModal(this.selector, {
        contractId: config.contractId,
      });

      return { selector: this.selector, modal: this.modal };
    } catch (error) {
      console.error("Failed to initialize wallet selector:", error);
      throw error;
    }
  }

  getSelector() {
    return this.selector;
  }

  getModal() {
    return this.modal;
  }

  async connect() {
    if (!this.modal) {
      throw new Error("Wallet service not initialized");
    }
    this.modal.show();
  }

  async disconnect() {
    if (!this.selector) {
      throw new Error("Wallet service not initialized");
    }
    
    const wallet = await this.selector.wallet();
    await wallet.signOut();
  }

  async getAccountId() {
    if (!this.selector) {
      return null;
    }
    
    const wallet = await this.selector.wallet();
    const accounts = await wallet.getAccounts();
    return accounts.length > 0 ? accounts[0]?.accountId || null : null;
  }

  isInitialized() {
    return this.selector !== null && this.modal !== null;
  }
}

export const walletService = new WalletService();
export default walletService;