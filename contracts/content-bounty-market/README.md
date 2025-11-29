# Content Bounty Market

A NEAR smart contract for content creation competitions where creators submit work, the community stakes NEAR on their favorites, and winners share the prize pool.

## Overview

This contract enables bounty-based content competitions:
- **Bounty Creators** post bounties with requirements and base prize
- **Content Creators** submit their work (characters, art, stories, etc.)
- **Community Members** stake NEAR on submissions they think should win
- **Winners** are determined by most NEAR staked
- **Rewards** split between winning creator (90% default, configurable) and their backers (10% default)

## Key Features

### Content-Focused Design
- ✅ Submissions track creator, creation ID, title, and thumbnail
- ✅ Links to Dreamweave database for full content details
- ✅ Prevents duplicate submissions per creator
- ✅ Supports up to 100 submissions per bounty

### Creator/Backer Reward Splits
- ✅ Configurable creator share (30-90%, default 90%)
- ✅ Configurable backer share (10-70%, default 10%)
- ✅ Platform fee: 5% of total prize
- ✅ Backers split their pool proportionally to their stakes

### Economic Model
- ✅ Base prize from bounty creator (minimum 1 NEAR)
- ✅ Community stakes add to total prize pool
- ✅ Winner determined by submission with most stakes
- ✅ Fair distribution: creator gets majority, backers share remainder

## Contract Methods

### Create Content Bounty
```rust
create_content_bounty(
    title: String,              // Bounty title (max 200 chars)
    description: String,        // What you're looking for (max 1000 chars)
    requirements: String,       // Specific requirements (max 2000 chars)
    base_prize: NearToken,      // Initial prize (minimum 1 NEAR)
    max_stake_per_user: NearToken, // Max a user can stake (0.1-10000 NEAR)
    creator_share: Option<u8>,  // % to creator (30-90, default 90)
    backer_share: Option<u8>,   // % to backers (10-70, default 10)
    duration_days: u64          // How long until closing (1-365 days)
) -> u64  // Returns bounty ID
```

**Attached Deposit Required:** `base_prize + storage_cost`

**Example (using defaults - 90% creator, 10% backers):**
```bash
near call content-bounty.testnet create_content_bounty \
  '{"title":"Cyberpunk Character Design","description":"Need a futuristic warrior","requirements":"1024x1024, high detail","base_prize":"10000000000000000000000000","max_stake_per_user":"5000000000000000000000000","duration_days":7}' \
  --accountId creator.testnet \
  --amount 10.1
```

**Example (custom split - 70% creator, 30% backers):**
```bash
near call content-bounty.testnet create_content_bounty \
  '{"title":"Cyberpunk Character Design","description":"Need a futuristic warrior","requirements":"1024x1024, high detail","base_prize":"10000000000000000000000000","max_stake_per_user":"5000000000000000000000000","creator_share":70,"backer_share":30,"duration_days":7}' \
  --accountId creator.testnet \
  --amount 10.1
```

### Submit Content
```rust
submit_content(
    bounty_id: u64,           // Which bounty to submit to
    creation_id: String,      // Dreamweave database Creation ID
    title: String,            // Submission title (max 200 chars)
    thumbnail_url: String     // Preview image URL
) -> u64  // Returns submission index
```

**Example:**
```bash
near call content-bounty.testnet submit_content \
  '{"bounty_id":1,"creation_id":"cm4abc123","title":"Neon Warrior","thumbnail_url":"https://..."}' \
  --accountId artist.testnet
```

### Stake on Submission
```rust
stake_on_submission(
    bounty_id: u64,           // Which bounty
    submission_index: u64     // Which submission to back
)
```

**Attached Deposit Required:** Amount to stake (respects `max_stake_per_user`)

**Example:**
```bash
near call content-bounty.testnet stake_on_submission \
  '{"bounty_id":1,"submission_index":0}' \
  --accountId backer.testnet \
  --amount 2
```

**Note:** Users can change their stake to support a different submission. Previous stake is returned.

### Close Bounty & Distribute Rewards
```rust
close_bounty(bounty_id: u64)
```

**Who can call:** Bounty creator or contract owner  
**When:** After bounty expiry (duration_days passed)

**Example:**
```bash
near call content-bounty.testnet close_bounty \
  '{"bounty_id":1}' \
  --accountId creator.testnet
```

### View Methods (No gas required)

**Get Bounty Details:**
```rust
get_bounty(bounty_id: u64) -> BountyView
```

**List Active Bounties:**
```rust
get_active_bounties() -> Vec<BountyView>
```

**Get User's Stake:**
```rust
get_participant_stake(account: AccountId, bounty_id: u64) -> ParticipantStakeView
```

**Get User's All Bounties:**
```rust
get_user_bounties(account: AccountId) -> Vec<ParticipantStakeView>
```

**Get Bounty Participants:**
```rust
get_bounty_participants(bounty_id: u64) -> Vec<AccountId>
```

**Get Stakes Per Submission:**
```rust
get_bounty_submission_stakes(bounty_id: u64) -> Vec<U128>
```

**Get Platform Fee Rate:**
```rust
get_platform_fee_rate() -> u128  // Returns basis points (500 = 5%)
```

## Data Structures

### BountyView
```rust
{
  id: u64,
  title: String,
  description: String,
  requirements: String,
  submissions: Vec<ContentSubmissionView>,
  creator: AccountId,
  base_prize: U128,              // In yoctoNEAR
  max_stake_per_user: U128,      // In yoctoNEAR
  creator_share: u8,             // Percentage (e.g., 60)
  backer_share: u8,              // Percentage (e.g., 40)
  is_active: bool,
  created_at: u64,               // Nanoseconds timestamp
  ends_at: u64,                  // Nanoseconds timestamp
  total_staked: U128,            // Community stakes (not base_prize)
  is_closed: bool,
  winning_submission: Option<u64>
}
```

### ContentSubmissionView
```rust
{
  creator: AccountId,
  creation_id: String,           // Links to Dreamweave Creation
  title: String,
  thumbnail_url: String,
  total_staked: U128,            // In yoctoNEAR
  submitted_at: u64              // Nanoseconds timestamp
}
```

### ParticipantStakeView
```rust
{
  bounty_id: u64,
  submission_index: u64,
  amount: U128,                  // In yoctoNEAR
  staked_at: u64                 // Nanoseconds timestamp
}
```

## Economic Example

**Scenario:**
- Base Prize: 10 NEAR (from bounty creator)
- Community Stakes:
  - Submission A: 15 NEAR ← winner
  - Submission B: 5 NEAR
- Creator Share: 90% (default)
- Backer Share: 10%
- Platform Fee: 5%

**Distribution:**
1. **Total Prize:** 10 + 20 = 30 NEAR
2. **Platform Fee (5%):** 1.5 NEAR → platform treasury
3. **Prize After Fee:** 28.5 NEAR
4. **Winner Creator:** 28.5 × 90% = **25.65 NEAR**
5. **Backer Pool:** 28.5 × 10% = **2.85 NEAR** (split proportionally)

**Backer ROI Example:**
- Alice staked 10 NEAR on Submission A (out of 15 total)
- Alice receives: (10/15) × 2.85 = **1.9 NEAR**
- Alice's ROI: **19%** (1.9 profit on 10 stake)
- Note: Lower backer share incentivizes creators more, while backers still profit

## Building & Testing

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Build Contract
```bash
cd contracts/content-bounty-market
cargo build --target wasm32-unknown-unknown --release
```

**Output:** `target/wasm32-unknown-unknown/release/content_bounty_market.wasm`

### Run Tests
```bash
cargo test
```

## Deployment

### Deploy to Testnet
```bash
near deploy your-account.testnet \
  target/wasm32-unknown-unknown/release/content_bounty_market.wasm \
  --initFunction new \
  --initArgs '{"owner":"your-account.testnet","platform_fee_rate":500}'
```

**Init Parameters:**
- `owner` - Account that can update platform fee and close bounties
- `platform_fee_rate` - Fee in basis points (500 = 5%, max 1000 = 10%)

### Deploy to Mainnet
```bash
near deploy your-account.near \
  target/wasm32-unknown-unknown/release/content_bounty_market.wasm \
  --initFunction new \
  --initArgs '{"owner":"your-account.near","platform_fee_rate":500}'
```

**Security Recommendations:**
- Use multi-sig for owner account on mainnet
- Set platform_fee_rate ≤ 500 (5%)
- Test thoroughly on testnet first
- Audit contract before mainnet deployment

## Integration with Dreamweave

### TypeScript Integration
```typescript
import { getBounty, getActiveBounties } from '@/lib/near-bounty'
import { 
  yoctoToNear, 
  calculateCreatorReward, 
  calculateBackerReward 
} from '@/lib/near-bounty-utils'

// Fetch bounty
const bounty = await getBounty(1)
console.log(`Total Prize: ${yoctoToNear(bounty.base_prize) + yoctoToNear(bounty.total_staked)} NEAR`)

// Calculate rewards
const creatorReward = calculateCreatorReward(bounty, 500) // 500 = 5% platform fee
console.log(`Winner gets: ${creatorReward} NEAR`)
```

### Environment Variables
```bash
# .env
NEAR_BOUNTY_CONTRACT_ID="your-content-bounty-market.testnet"
NEAR_RPC_URL="https://test.rpc.fastnear.com"
```

### tRPC Procedures
- `bounty.listOnChainBounties` - List all active bounties with reward calculations
- `bounty.getOnChainBounty` - Get single bounty with user stake and potential rewards
- `bounty.getUserOnChainBounties` - Get all bounties user has staked on

## Safety Features

### Validations
- ✅ Title length ≤ 200 chars
- ✅ Description ≤ 1000 chars
- ✅ Requirements ≤ 2000 chars
- ✅ Base prize ≥ 1 NEAR
- ✅ Max stake per user: 0.1 - 10,000 NEAR
- ✅ Creator share: 30-90%
- ✅ Creator + backer shares must equal 100%
- ✅ Duration: 1-365 days
- ✅ Max 100 submissions per bounty
- ✅ Max 150 participants per bounty (prevents DoS)

### Security
- ✅ All math uses checked operations (prevents overflow)
- ✅ Reentrancy protection via state updates before transfers
- ✅ Base prize locked in contract when bounty created
- ✅ Only bounty creator or owner can close bounties
- ✅ Only closed bounties pay rewards
- ✅ Duplicate submission prevention
- ✅ Storage cost validation

### Error Handling
- Insufficient deposit for base prize
- Bounty not found
- Bounty expired or already closed
- Invalid submission index
- Stake exceeds max allowed
- Duplicate submission by same creator
- Creation ID already submitted
- Platform fee exceeds 10%

## Admin Functions

### Update Platform Fee (Owner Only)
```rust
update_platform_fee_rate(new_rate: u128)
```

**Example:**
```bash
near call content-bounty.testnet update_platform_fee_rate \
  '{"new_rate":500}' \
  --accountId owner.testnet
```

**Constraints:**
- Only contract owner can call
- Rate ≤ 1000 (10%)
- Doesn't affect existing bounties

### Pause Contract (Owner Only)
```rust
pause_contract()
```

### Resume Contract (Owner Only)
```rust
resume_contract()
```

## Storage Costs

**Typical costs per operation:**
- Create bounty: ~0.1 NEAR (depends on title/description length)
- Submit content: ~0.05 NEAR (one-time per submission)
- Stake: Gas only (~0.0003 NEAR)

**Storage refunds:**
- Excess attached deposit is refunded automatically
- Storage costs calculated precisely and refunded

## Version History

### v0.1.0 (Current)
- Content-focused bounty system
- Creator/backer reward splits
- Base prize + community stakes model
- Submission tracking with creation IDs
- Configurable reward percentages
- Full TypeScript integration

## License

MIT License - See LICENSE file for details

## Support

- **Documentation:** See `/contracts/CONTENT_BOUNTY_ADAPTATION.md`
- **Smart Contracts Guide:** See `/SMART_CONTRACTS_EXPLAINED.md`
- **TypeScript Integration:** See `/INTEGRATION_COMPLETE.md`
- **Issues:** Report bugs via GitHub issues

## Contributing

Contributions welcome! Please:
1. Test changes thoroughly on testnet
2. Follow Rust/NEAR best practices
3. Update documentation for new features
4. Add tests for new functionality

---

**Contract Name:** `content-bounty-market`  
**Language:** Rust  
**Framework:** NEAR SDK v5.17.2  
**Status:** ✅ Ready for deployment  
**Last Updated:** 2025-10-14
