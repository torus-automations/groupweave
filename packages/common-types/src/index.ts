// User and Authentication Types
export interface User {
  id: string;
  address: string;
  username?: string;
  email?: string;
  avatar?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface AuthSession {
  user: User;
  token: string;
  expiresAt: Date;
}

// Voting and Governance Types
export interface Poll {
  id: number;
  title: string;
  description: string;
  options: string[];
  votes: number[];
  creator: string;
  isActive: boolean;
  createdAt: number;
  endsAt?: number;
  totalVotes: number;
}

export interface Vote {
  pollId: number;
  voter: string;
  optionIndex: number;
  timestamp: number;
}

export interface GovernanceProposal {
  id: number;
  title: string;
  description: string;
  proposer: string;
  votesFor: number;
  votesAgainst: number;
  status: ProposalStatus;
  createdAt: number;
  votingEndsAt: number;
  executionData?: string;
}

export enum ProposalStatus {
  DRAFT = 'draft',
  ACTIVE = 'active',
  PASSED = 'passed',
  REJECTED = 'rejected',
  EXECUTED = 'executed',
  CANCELLED = 'cancelled'
}

// Staking Types
export interface StakeInfo {
  user: string;
  amount: string; // Using string for large numbers
  stakedAt: number;
  lastRewardClaim: number;
  pendingRewards: string;
}

export interface StakingPool {
  id: string;
  name: string;
  totalStaked: string;
  rewardRate: string;
  minStakeAmount: string;
  isActive: boolean;
}

export interface RewardDistribution {
  user: string;
  amount: string;
  timestamp: number;
  txHash: string;
}

// ZKP Types
export interface ZKProof {
  proofId: string;
  proofData: string; // Base64 encoded
  publicInputs: string[];
  verificationKey: string;
  submitter: string;
  verified: boolean;
  submittedAt: number;
  verifiedAt?: number;
}

export interface VerificationResult {
  proofId: string;
  isValid: boolean;
  verifiedAt: number;
  verifier: string;
}

// AI and Analytics Types
export interface VotingAnalysis {
  pollId: number;
  winnerIndex: number;
  winnerOption: string;
  confidenceScore: number;
  totalVotes: number;
  analysis: {
    voteDistribution: Record<string, number>;
    marginOfVictory: number;
    participationRate: 'low' | 'medium' | 'high';
  };
  recommendations: string[];
}

export interface GovernanceInsight {
  id: string;
  type: 'voting_pattern' | 'staking_analysis' | 'participation' | 'security';
  title: string;
  description: string;
  confidence: number;
  data: Record<string, any>;
  timestamp: Date;
}

export interface AIDecision {
  context: string;
  options: string[];
  criteria: string[];
  recommendedOption: string;
  confidence: number;
  reasoning: string;
  analysis: Record<string, any>;
}

// Agent Types
export interface AgentConfig {
  agentId: string;
  network: string;
  rpcEndpoint: string;
  contractAddresses: Record<string, string>;
  pollingInterval: number;
}

export interface AgentAction {
  actionType: 'vote_aggregation' | 'reward_distribution' | 'governance_execution' | 'data_sync' | 'security_check';
  target: string;
  data: Record<string, any>;
  timestamp: number;
}

export interface AgentStatus {
  agentId: string;
  isRunning: boolean;
  lastExecution?: number;
  actionsPerformed: number;
  errors: string[];
}

// API Response Types
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp: number;
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}

// Blockchain Types
export interface Transaction {
  hash: string;
  from: string;
  to: string;
  value: string;
  gasUsed: string;
  gasPrice: string;
  blockNumber: number;
  timestamp: number;
  status: 'pending' | 'confirmed' | 'failed';
}

export interface ContractCall {
  contractAddress: string;
  methodName: string;
  args: Record<string, any>;
  attachedDeposit?: string;
  gas?: string;
}

// Notification Types
export interface Notification {
  id: string;
  userId: string;
  type: 'vote_created' | 'vote_ended' | 'proposal_created' | 'reward_distributed' | 'stake_updated';
  title: string;
  message: string;
  data?: Record<string, any>;
  read: boolean;
  createdAt: Date;
}

// Error Types
export interface AppError {
  code: string;
  message: string;
  details?: Record<string, any>;
  timestamp: Date;
}

// Utility Types
export type NetworkType = 'mainnet' | 'testnet' | 'localnet';
export type WalletType = 'near-wallet' | 'sender-wallet' | 'metamask' | 'wallet-connect';

export interface WalletConnection {
  accountId: string;
  walletType: WalletType;
  isConnected: boolean;
  balance: string;
}

// Event Types
export interface AppEvent {
  type: string;
  payload: Record<string, any>;
  timestamp: number;
}

export interface VotingEvent extends AppEvent {
  type: 'vote_cast' | 'poll_created' | 'poll_ended';
  payload: {
    pollId: number;
    voter?: string;
    optionIndex?: number;
  };
}

export interface StakingEvent extends AppEvent {
  type: 'stake_added' | 'stake_removed' | 'rewards_claimed';
  payload: {
    user: string;
    amount: string;
    poolId?: string;
  };
}