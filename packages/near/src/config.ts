// NEAR network configuration
export const NEAR_CONFIG = {
  testnet: {
    networkId: 'testnet',
    nodeUrl: 'https://rpc.testnet.near.org',
    walletUrl: 'https://testnet.mynearwallet.com/',
    helperUrl: 'https://helper.testnet.near.org',
  },
  mainnet: {
    networkId: 'mainnet',
    nodeUrl: 'https://rpc.mainnet.near.org',
    walletUrl: 'https://app.mynearwallet.com/',
    helperUrl: 'https://helper.mainnet.near.org',
  },
} as const;

export type NetworkId = keyof typeof NEAR_CONFIG;