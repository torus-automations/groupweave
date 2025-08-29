// Re-export specific NEAR.js classes and functions
export { Account } from "@near-js/accounts";
export { JsonRpcProvider } from "@near-js/providers";
export { KeyPairSigner } from "@near-js/signers";

// Re-export all other exports for flexibility
export * from '@near-js/accounts';
export * from '@near-js/providers';
export * from '@near-js/signers';

// Add your custom NEAR utilities here
export * from './utils';
export * from './config';