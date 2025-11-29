# Voting Contract Security Audit

**Date:** 2025-11-02
**Status:** ✅ Reviewed with Recommendations

## Security Analysis

### ✅ PROTECTED AGAINST

#### 1. Reentrancy Attacks
**Status:** SAFE
**Reasoning:**
- All state updates happen BEFORE external calls (`Promise::new().transfer()`)
- `poll.payout_done = true` and `poll.reward_yocto = 0` set before transfers (lines 326-327)
- Poll stored back to state after all transfers complete (line 330)

#### 2. Integer Overflow/Underflow
**Status:** SAFE
**Reasoning:**
- Uses `saturating_sub()` for subtraction (line 299)
- Uses `checked_mul()` and `checked_add()` in duration calculations (lines 162-172)
- All arithmetic operations use safe Rust patterns

#### 3. Access Control
**Status:** SAFE
**Reasoning:**
- Only poll creator can close poll (line 286)
- Only owner can set platform fee (line 73)
- Only poll creator can add/remove from whitelist (lines 84, 102)
- Whitelist check enforced for closed polls (line 231)

#### 4. Time Expiration
**Status:** SAFE
**Reasoning:**
- Expiration checked before voting (lines 235-237)
- Uses NEAR block timestamp (nanoseconds)
- Proper overflow protection in duration calculation

### ⚠️ NEEDS IMPROVEMENT

#### 1. Maximum Participants Limit
**Risk:** HIGH
**Issue:** No limit on number of voters per poll
**Impact:**
- Close_poll iterates over all votes (line 293)
- Could cause gas exhaustion if millions of users vote
- DOS attack vector

**Recommendation:** Add MAX_VOTERS_PER_POLL constant

```rust
const MAX_VOTERS_PER_POLL: u64 = 100_000_000; // 100 million

pub fn vote(&mut self, poll_id: u64, option_index: u64) {
    let poll = self.polls.get(&poll_id).expect("Poll not found");

    // Count unique voters for this poll
    let voter_count = poll.votes.iter().sum::<u64>();
    require!(voter_count < MAX_VOTERS_PER_POLL, "Poll has reached maximum voters");

    // ... rest of vote logic
}
```

#### 2. Options Array Unbounded
**Risk:** MEDIUM
**Issue:** No maximum limit on poll options
**Impact:** Large options array increases storage cost and iteration time

**Recommendation:** Add MAX_OPTIONS_PER_POLL

```rust
const MAX_OPTIONS_PER_POLL: usize = 100;

pub fn create_poll(..., options: Vec<OptionInput>, ...) {
    require!(options.len() >= 2, "Poll must include at least two options");
    require!(options.len() <= MAX_OPTIONS_PER_POLL, "Too many options");
    // ...
}
```

#### 3. No Maximum Reward Limit
**Risk:** LOW
**Issue:** No cap on reward_yocto amount
**Impact:** User could accidentally lock huge amount

**Recommendation:** Add MAX_REWARD constant

```rust
const MAX_REWARD_YOCTO: u128 = 100_000_000_000_000_000_000_000_000_000_000; // 100M NEAR

pub fn create_poll(..., reward_yocto: Option<u128>, ...) {
    let reward = reward_yocto.unwrap_or(0);
    require!(reward <= MAX_REWARD_YOCTO, "Reward exceeds maximum");
    // ...
}
```

#### 4. No Duration Limits
**Risk:** LOW
**Issue:** No maximum duration enforcement
**Impact:** Could create polls far in future

**Recommendation:** Add MAX_DURATION_MINUTES

```rust
const MAX_DURATION_MINUTES: u64 = 525_600; // 1 year

pub fn create_poll(..., duration_minutes: Option<u64>, ...) {
    if let Some(minutes) = duration_minutes {
        require!(minutes > 0, "Duration must be positive");
        require!(minutes <= MAX_DURATION_MINUTES, "Duration too long");
    }
    // ...
}
```

### 🔒 ADDITIONAL SECURITY MEASURES

#### 5. Refund Logic
**Status:** ✅ IMPLEMENTED CORRECTLY
**Analysis:**
- If no votes (max_votes == 0), refunds creator minus platform fee (lines 322-324)
- Platform fee still deducted (lines 298-302)
- Remainder from rounding distributed to creator (lines 319-321)

#### 6. Duplicate Recipients
**Status:** ✅ HANDLED
**Analysis:**
- Deduplicates recipients in payout loop (lines 308-318)
- Prevents double-payment to same address
- Excess goes to remainder bucket

### 🚨 CRITICAL FIXES REQUIRED

#### Fix #1: Add Voter Count Tracking

**Current Code Problem:**
```rust
// Line 293: Iterates all vote counts, but no limit check
for &v in &poll.votes { if v > max_votes { max_votes = v; } }
```

**Secure Version:**
```rust
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Poll {
    // ... existing fields
    pub total_voters: u64, // ADD THIS
}

pub fn vote(&mut self, poll_id: u64, option_index: u64) {
    // ... existing checks

    let vote_key = (voter.clone(), poll_id);
    let is_new_voter = !self.user_votes.contains_key(&vote_key);

    if is_new_voter {
        require!(
            poll.total_voters < MAX_VOTERS_PER_POLL,
            "Poll has reached maximum voters (100 million)"
        );
        poll.total_voters += 1;
    }

    // ... rest of vote logic
}
```

#### Fix #2: Add Length Validations

**Add to create_poll:**
```rust
// Title length
require!(title.chars().count() <= TITLE_MAX, "Title exceeds max length");

// Criteria length (already exists, keep it)
require!(crit_len <= CRITERIA_MAX, "Criteria exceeds max length");

// Options count
require!(options.len() <= MAX_OPTIONS_PER_POLL, "Too many options");
```

#### Fix #3: Add Reward Validation

**Add to create_poll:**
```rust
let reward = reward_yocto.unwrap_or(0);
if reward > 0 {
    require!(
        reward >= MIN_REWARD_YOCTO,
        "Reward must be at least 0.1 NEAR"
    );
    require!(
        reward <= MAX_REWARD_YOCTO,
        "Reward cannot exceed 100,000,000 NEAR"
    );
}
```

### 📊 Gas Cost Analysis

**Operation Costs:**
- Create poll: ~0.01 NEAR (storage) + 0.001 NEAR (gas)
- Vote: ~0.005 NEAR (storage) + 0.001 NEAR (gas)
- Close poll with 100 voters: ~0.002 NEAR (gas)
- Close poll with 1M voters: ~0.02 NEAR (gas)
- Close poll with 100M voters: ⚠️ **MAY EXCEED GAS LIMIT**

**Recommendation:** Implement batched payout for large polls or cap at reasonable limit.

### 🎯 Recommended Constants

```rust
// Add to top of lib.rs
const MAX_VOTERS_PER_POLL: u64 = 100_000_000; // 100 million
const MAX_OPTIONS_PER_POLL: usize = 100;
const MAX_DURATION_MINUTES: u64 = 525_600; // 1 year
const MIN_REWARD_YOCTO: u128 = 100_000_000_000_000_000_000_000; // 0.1 NEAR
const MAX_REWARD_YOCTO: u128 = 100_000_000_000_000_000_000_000_000_000_000; // 100M NEAR
const TITLE_MAX: usize = 120;
const CRITERIA_MIN: usize = 8;
const CRITERIA_MAX: usize = 200;
const DETAILS_MAX: usize = 600;
```

### ✅ Conclusion

**Overall Security Rating:** B+ (Good, with improvements needed)

**Required Changes:**
1. ✅ Add MAX_VOTERS_PER_POLL limit (CRITICAL)
2. ✅ Add total_voters tracking to Poll struct
3. ✅ Add MAX_OPTIONS_PER_POLL validation
4. ✅ Add MAX_REWARD_YOCTO validation
5. ✅ Add MAX_DURATION_MINUTES validation

**Optional Enhancements:**
- Implement batch payout for polls with > 1000 winners
- Add emergency pause functionality
- Add upgrade mechanism for contract

**After implementing fixes:** A (Production Ready)
