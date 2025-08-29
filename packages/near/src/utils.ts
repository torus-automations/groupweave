import { JsonRpcProvider } from "@near-js/providers";

// NEAR utility functions
export function formatNearAmount(amount: string): string {
  // Add utility functions for NEAR amount formatting, etc.
  return amount;
}

export function parseNearAmount(amount: string): string {
  // Add utility functions for NEAR amount parsing, etc.
  return amount;
}

// NEAR service utilities
export async function createNearConnection(networkId: string, nodeUrl: string) {
  const provider = new JsonRpcProvider({ url: nodeUrl });
  return { provider, networkId };
}