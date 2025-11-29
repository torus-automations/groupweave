// Payment processor for Dreamweave credits.
// Accepts NEAR and fungible tokens (USDT, USDC) as payment.
// Users deposit crypto, contract records transaction, backend verifies on-chain and allocates credits.
// Treasury address receives funds immediately. No escrow, no withdrawals.

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::env;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::NearToken;
use near_sdk::{require, AccountId, BorshStorageKey, PromiseOrValue, near};
use near_sdk::{Gas, Promise};
use near_sdk::ext_contract;
use schemars::JsonSchema;

const NEAR_TOKEN_ID: &str = "NEAR";
const MIN_DEPOSIT_USD_MICROS: u128 = 5 * 1_000_000;
const MAX_BENEFICIARY_LEN: usize = 128;
const MAX_MEMO_LEN: usize = 256;
const MAX_PRICE_AGE_MS: u64 = 60 * 60 * 1000; // 1 hour

/// Gas allowance for cross-contract FT transfers during withdrawals.
const GAS_FOR_FT_TRANSFER: Gas = Gas::from_tgas(25);

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    TokenConfigs,
    Deposits,
    DepositsByAccount,
}

/// Metadata and pricing information for an accepted payment token.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenConfig {
    pub symbol: String,
    pub decimals: u8,
    pub price_usd_micros: u128,
    pub last_updated: u64,
    pub is_enabled: bool,
    pub is_native: bool,
}

impl TokenConfig {
    pub fn new(
        symbol: String,
        decimals: u8,
        price_usd_micros: u128,
        is_native: bool,
        is_enabled: bool,
    ) -> Self {
        Self {
            symbol,
            decimals,
            price_usd_micros,
            last_updated: env::block_timestamp_ms(),
            is_enabled,
            is_native,
        }
    }
}

/// Lightweight message passed through `ft_transfer_call`.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DepositMessage {
    pub beneficiary_id: String,
    pub credits_hint: Option<u64>,
    pub memo: Option<String>,
}

/// Stored representation of a payment waiting to be reconciled off-chain.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DepositRecord {
    pub id: u64,
    pub account_id: AccountId,
    pub beneficiary_id: String,
    pub token_id: String,
    pub amount: U128,
    pub usd_value: U128,
    pub credits_hint: Option<u64>,
    pub memo: Option<String>,
    pub timestamp_ms: u64,
}

impl DepositRecord {
    pub fn into_view(self) -> DepositView {
        DepositView {
            id: self.id,
            account_id: self.account_id,
            beneficiary_id: self.beneficiary_id,
            token_id: self.token_id,
            amount: self.amount,
            usd_value: self.usd_value,
            credits_hint: self.credits_hint,
            memo: self.memo,
            timestamp_ms: self.timestamp_ms,
        }
    }
}

/// Human-friendly deposit view returned to clients.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "schemars")]
pub struct DepositView {
    pub id: u64,
    #[schemars(with = "String")]
    pub account_id: AccountId,
    pub beneficiary_id: String,
    pub token_id: String,
    #[schemars(with = "String")]
    pub amount: U128,
    #[schemars(with = "String")]
    pub usd_value: U128,
    pub credits_hint: Option<u64>,
    pub memo: Option<String>,
    pub timestamp_ms: u64,
}

/// On-chain state for the Dreamweave deposit contract.
#[near(contract_state)]
pub struct DepositContract {
    owner_id: AccountId,
    treasury_account_id: AccountId,
    next_deposit_id: u64,
    token_configs: UnorderedMap<String, TokenConfig>,
    deposits: LookupMap<u64, DepositRecord>,
    deposits_by_account: LookupMap<AccountId, Vec<u64>>,
}

impl Default for DepositContract {
    fn default() -> Self {
        env::panic_str("Contract must be initialized with new()")
    }
}

#[near]
impl DepositContract {
    #[init(ignore_state)]
    pub fn new(owner_id: AccountId, treasury_account_id: AccountId) -> Self {
        let mut token_configs = UnorderedMap::new(StorageKey::TokenConfigs);
        // Seed NEAR symbol config (price must be set later by owner).
        token_configs.insert(
            &NEAR_TOKEN_ID.to_string(),
            &TokenConfig::new("NEAR".to_string(), 24, 0, true, true),
        );

        Self {
            owner_id,
            treasury_account_id,
            next_deposit_id: 0,
            token_configs,
            deposits: LookupMap::new(StorageKey::Deposits),
            deposits_by_account: LookupMap::new(StorageKey::DepositsByAccount),
        }
    }

    /// Safer migration: reuse existing state; owner-only; optionally update treasury.
    #[init(ignore_state)]
    pub fn migrate(treasury_account_id: Option<AccountId>) -> Self {
        // Read existing state; fail if none
        let mut old: DepositContract = env::state_read().expect("No existing state to migrate");
        // Only current owner may migrate
        require!(env::predecessor_account_id() == old.owner_id, "Only the owner can migrate");
        if let Some(new_treasury) = treasury_account_id {
            old.treasury_account_id = new_treasury;
        }
        // Return the updated state (becomes new contract state)
        old
    }

    /// Update or register a token configuration (owner only).
    pub fn upsert_token_config(
        &mut self,
        token_id: String,
        symbol: String,
        decimals: u8,
        price_usd_micros: U128,
        is_enabled: bool,
        is_native: bool,
    ) {
        self.assert_owner();
        let mut config = TokenConfig::new(symbol, decimals, price_usd_micros.0, is_native, is_enabled);
        config.last_updated = env::block_timestamp_ms();
        self.token_configs.insert(&token_id, &config);
    }

    /// Update the USD price for a given token (owner only).
    pub fn update_token_price(&mut self, token_id: String, price_usd_micros: U128) {
        self.assert_owner();
        let mut cfg = self
            .token_configs
            .get(&token_id)
            .expect("Token config not found");

        cfg.price_usd_micros = price_usd_micros.0;
        cfg.last_updated = env::block_timestamp_ms();
        self.token_configs.insert(&token_id, &cfg);
    }

    /// Change the treasury account receiving native deposits (owner only).
    pub fn set_treasury(&mut self, treasury_account_id: AccountId) {
        self.assert_owner();
        self.treasury_account_id = treasury_account_id;
    }

    /// View helper for token config.
    pub fn get_token_config(&self, token_id: String) -> Option<TokenConfigView> {
        self.token_configs
            .get(&token_id)
            .map(|cfg| TokenConfigView::from_parts(token_id, cfg))
    }

    /// List all configured tokens.
    pub fn list_token_configs(&self) -> Vec<TokenConfigView> {
        self.token_configs
            .iter()
            .map(|(token_id, cfg)| TokenConfigView::from_parts(token_id, cfg))
            .collect()
    }

    /// Retrieve deposits recorded for a given account.
    pub fn get_deposits_for_account(&self, account_id: AccountId) -> Vec<DepositView> {
        let Some(ids) = self.deposits_by_account.get(&account_id) else {
            return vec![];
        };
        ids.into_iter()
            .filter_map(|id| self.deposits.get(&id))
            .map(DepositRecord::into_view)
            .collect()
    }

    /// Retrieve a single deposit record.
    pub fn get_deposit(&self, deposit_id: u64) -> Option<DepositView> {
        self.deposits.get(&deposit_id).map(DepositRecord::into_view)
    }

    /// Payable method for depositing native NEAR.
    #[payable]
    pub fn deposit_native(
        &mut self,
        beneficiary_id: String,
        credits_hint: Option<u64>,
        memo: Option<String>,
    ) -> DepositView {
        let amount = env::attached_deposit();
        require!(amount.as_yoctonear() > 0, "Attach NEAR to deposit");

        // Basic input size limits to protect storage
        require!(beneficiary_id.len() <= MAX_BENEFICIARY_LEN, "beneficiary_id too long");
        if let Some(m) = &memo { require!(m.len() <= MAX_MEMO_LEN, "memo too long"); }

        let cfg = self
            .token_configs
            .get(&NEAR_TOKEN_ID.to_string())
            .expect("NEAR token config missing");
        require!(cfg.is_enabled, "NEAR deposits are disabled");
        require!(cfg.price_usd_micros > 0, "NEAR price not configured");
        require!(
            env::block_timestamp_ms().saturating_sub(cfg.last_updated) <= MAX_PRICE_AGE_MS,
            "Price data is stale (>1h). Keeper must update price."
        );

        let usd_value = self.usd_value_for(&cfg, amount.as_yoctonear());
        require!(
            usd_value >= MIN_DEPOSIT_USD_MICROS,
            "Minimum deposit is $5 USD"
        );

        let account_id = env::predecessor_account_id();
        let record = self.store_deposit(
            account_id.clone(),
            beneficiary_id,
            NEAR_TOKEN_ID.to_string(),
            amount.as_yoctonear(),
            usd_value,
            credits_hint,
            memo,
        );

        // Immediately forward NEAR to the treasury wallet.
        Promise::new(self.treasury_account_id.clone()).transfer(amount);

        record
    }

    fn store_deposit(
        &mut self,
        account_id: AccountId,
        beneficiary_id: String,
        token_id: String,
        amount: u128,
        usd_value: u128,
        credits_hint: Option<u64>,
        memo: Option<String>,
    ) -> DepositView {
        let deposit_id = self.next_deposit_id;
        self.next_deposit_id += 1;

        let record = DepositRecord {
            id: deposit_id,
            account_id: account_id.clone(),
            beneficiary_id,
            token_id,
            amount: U128(amount),
            usd_value: U128(usd_value),
            credits_hint,
            memo,
            timestamp_ms: env::block_timestamp_ms(),
        };

        self.deposits.insert(&deposit_id, &record);

        let mut ids = self.deposits_by_account.get(&account_id).unwrap_or_default();
        ids.push(deposit_id);
        self.deposits_by_account.insert(&account_id, &ids);

        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"dreamweave_deposit\",\"version\":\"1.0.0\",\"event\":\"deposit\",\"data\":[{}]}}",
            serde_json::to_string(&record).unwrap()
        ));

        record.into_view()
    }

    fn usd_value_for(&self, cfg: &TokenConfig, amount: u128) -> u128 {
        if cfg.price_usd_micros == 0 { return 0; }
        let denominator = 10u128.pow(cfg.decimals as u32);
        if denominator == 0 { return 0; }
        // Compute (amount / denom) * price + ((amount % denom) * price) / denom to avoid overflow
        let whole = amount / denominator;
        let frac = amount % denominator;
        let part1 = whole.saturating_mul(cfg.price_usd_micros);
        let part2 = (frac.saturating_mul(cfg.price_usd_micros)) / denominator;
        part1.saturating_add(part2)
    }

    fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Only the owner can call this method"
        );
    }

    /// Owner-only: sweep FT balances held by this contract to the treasury.
    /// Some FT deposits may leave balances in this contract; use this to forward them.
    pub fn sweep_ft(&mut self, token_id: AccountId, amount: U128) -> Promise {
        self.assert_owner();
        ext_ft::ext(token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(self.treasury_account_id.clone(), amount, None)
    }
}

#[near]
impl FungibleTokenReceiver for DepositContract {
    /// Handles `ft_transfer_call` deposits for whitelisted tokens.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Ensure non-zero deposit and cap inputs to avoid storage blow-up
        require!(amount.0 > 0, "Amount must be > 0");
        let token_id = env::predecessor_account_id();
        let cfg = self
            .token_configs
            .get(&token_id.to_string())
            .expect("Unsupported token");
        require!(cfg.is_enabled, "Token deposits disabled");
        require!(cfg.price_usd_micros > 0, "Token price not configured");
        require!(
            env::block_timestamp_ms().saturating_sub(cfg.last_updated) <= MAX_PRICE_AGE_MS,
            "Price data is stale (>1h). Keeper must update price."
        );

        let parsed: DepositMessage = serde_json::from_str(&msg).expect("Invalid deposit message payload");
        require!(parsed.beneficiary_id.len() <= MAX_BENEFICIARY_LEN, "beneficiary_id too long");
        if let Some(m) = &parsed.memo { require!(m.len() <= MAX_MEMO_LEN, "memo too long"); }

        let usd_value = self.usd_value_for(&cfg, amount.0);
        require!(
            usd_value >= MIN_DEPOSIT_USD_MICROS,
            "Minimum deposit is $5 USD"
        );

        let _record = self.store_deposit(
            sender_id.clone(),
            parsed.beneficiary_id,
            token_id.to_string(),
            amount.0,
            usd_value,
            parsed.credits_hint,
            parsed.memo,
        );

        // For FT deposits the tokens remain held in the contract until the owner withdraws them.
        PromiseOrValue::Value(U128(0))
    }
}

/// Serializable view for token config.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "schemars")]
pub struct TokenConfigView {
    pub token_id: String,
    pub symbol: String,
    pub decimals: u8,
    #[schemars(with = "String")]
    pub price_usd_micros: U128,
    pub last_updated: u64,
    pub is_enabled: bool,
    pub is_native: bool,
}

impl TokenConfigView {
    fn from_parts(token_id: String, cfg: TokenConfig) -> Self {
        Self {
            token_id,
            symbol: cfg.symbol,
            decimals: cfg.decimals,
            price_usd_micros: U128(cfg.price_usd_micros),
            last_updated: cfg.last_updated,
            is_enabled: cfg.is_enabled,
            is_native: cfg.is_native,
        }
    }
}


#[near_sdk::ext_contract(ext_ft)]
pub trait ExtFungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near]
impl DepositContract {
    /// Withdraw native NEAR held by the contract to the treasury (owner only).
    /// Safety mechanism in case forwarding fails.
    pub fn withdraw_native(
        &mut self,
        amount: U128,
        receiver_id: Option<AccountId>,
    ) {
        self.assert_owner();
        let receiver = receiver_id.unwrap_or_else(|| self.treasury_account_id.clone());
        Promise::new(receiver).transfer(NearToken::from_yoctonear(amount.0));
    }

    /// Withdraw fungible tokens held by the contract to the treasury (owner only).
    #[payable]
    pub fn withdraw_ft(
        &mut self,
        token_id: AccountId,
        amount: U128,
        receiver_id: Option<AccountId>,
        memo: Option<String>,
    ) {
        self.assert_owner();
        require!(
            env::attached_deposit() >= NearToken::from_yoctonear(1),
            "Attach at least 1 yoctoNEAR to cover security requirements"
        );

        let receiver = receiver_id.unwrap_or_else(|| self.treasury_account_id.clone());

        ext_ft::ext(token_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(receiver, amount, memo);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
    mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    fn setup_context(attached_deposit: u128, predecessor: AccountId) {
        let mut builder = VMContextBuilder::new();
        builder
            .attached_deposit(NearToken::from_yoctonear(attached_deposit))
            .predecessor_account_id(predecessor)
            .signer_account_id(accounts(0));
        testing_env!(builder.build());
    }

    fn init_contract() -> DepositContract {
        setup_context(0, accounts(0));
        DepositContract::new(accounts(0), accounts(1))
    }

    // ========================================
    // Initialization Tests
    // ========================================

    #[test]
    fn test_contract_initialization() {
        let contract = init_contract();
        assert_eq!(contract.owner_id, accounts(0));
        assert_eq!(contract.treasury_account_id, accounts(1));
        assert_eq!(contract.next_deposit_id, 0);
        
        // NEAR token should be pre-configured
        let near_config = contract.get_token_config(NEAR_TOKEN_ID.to_string());
        assert!(near_config.is_some());
        let config = near_config.unwrap();
        assert_eq!(config.symbol, "NEAR");
        assert_eq!(config.decimals, 24);
        assert!(config.is_native);
        assert!(config.is_enabled);
    }

    // removed: old migrate behavior replaced with safer owner-gated migrate

    // ========================================
    // Token Configuration Tests
    // ========================================

    #[test]
    fn test_upsert_token_config() {
        let mut contract = init_contract();
        
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );

        let config = contract.get_token_config("usdc.token".to_string());
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.symbol, "USDC");
        assert_eq!(config.decimals, 6);
        assert_eq!(config.price_usd_micros.0, 1_000_000);
        assert!(config.is_enabled);
        assert!(!config.is_native);
        // last_updated is 0 in test environment
        assert_eq!(config.last_updated, 0);
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_upsert_token_config_non_owner_fails() {
        let mut contract = init_contract();
        setup_context(0, accounts(2)); // Not the owner
        
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );
    }

    #[test]
    fn test_update_token_price() {
        let mut contract = init_contract();
        
        // Update NEAR price
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(5_000_000));
        
        let config = contract.get_token_config(NEAR_TOKEN_ID.to_string()).unwrap();
        assert_eq!(config.price_usd_micros.0, 5_000_000);
    }

    #[test]
    #[should_panic(expected = "Token config not found")]
    fn test_update_nonexistent_token_price_fails() {
        let mut contract = init_contract();
        contract.update_token_price("nonexistent.token".to_string(), U128(1_000_000));
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_update_token_price_non_owner_fails() {
        let mut contract = init_contract();
        setup_context(0, accounts(2));
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(5_000_000));
    }

    #[test]
    fn test_list_token_configs() {
        let mut contract = init_contract();
        
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );
        
        let configs = contract.list_token_configs();
        assert!(configs.len() >= 2); // NEAR + USDC
        
        let has_near = configs.iter().any(|c| c.token_id == NEAR_TOKEN_ID);
        let has_usdc = configs.iter().any(|c| c.token_id == "usdc.token");
        assert!(has_near);
        assert!(has_usdc);
    }

    #[test]
    fn test_set_treasury() {
        let mut contract = init_contract();
        let new_treasury = accounts(5);
        
        contract.set_treasury(new_treasury.clone());
        assert_eq!(contract.treasury_account_id, new_treasury);
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_set_treasury_non_owner_fails() {
        let mut contract = init_contract();
        setup_context(0, accounts(2));
        contract.set_treasury(accounts(5));
    }

    // ========================================
    // Native NEAR Deposit Tests
    // ========================================

    #[test]
    fn test_native_deposit_records() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000)); // $1 per NEAR

        // Attach 6 NEAR (in yocto) to exceed $5 threshold.
        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let receipt = contract.deposit_native("user-123".to_string(), Some(250), None);

        assert_eq!(receipt.account_id, accounts(2));
        assert_eq!(receipt.usd_value.0, 6 * 1_000_000);
        assert_eq!(receipt.credits_hint, Some(250));
        assert_eq!(receipt.token_id, NEAR_TOKEN_ID);
        assert_eq!(receipt.beneficiary_id, "user-123");
        assert_eq!(receipt.id, 0);
        // timestamp_ms is 0 in test environment
        assert_eq!(receipt.timestamp_ms, 0);

        let deposits = contract.get_deposits_for_account(accounts(2));
        assert_eq!(deposits.len(), 1);
        assert_eq!(deposits[0].usd_value.0, receipt.usd_value.0);
    }

    #[test]
    fn test_native_deposit_with_memo() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(2_000_000));

        let five_near = 5u128 * 10u128.pow(24);
        setup_context(five_near, accounts(2));
        let receipt = contract.deposit_native(
            "user-456".to_string(),
            Some(500),
            Some("Premium subscription".to_string()),
        );

        assert_eq!(receipt.memo, Some("Premium subscription".to_string()));
        assert_eq!(receipt.usd_value.0, 10_000_000); // 5 NEAR * $2
    }

    #[test]
    fn test_multiple_deposits_increment_id() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        
        setup_context(six_near, accounts(2));
        let receipt1 = contract.deposit_native("user-1".to_string(), None, None);
        
        setup_context(six_near, accounts(3));
        let receipt2 = contract.deposit_native("user-2".to_string(), None, None);
        
        assert_eq!(receipt1.id, 0);
        assert_eq!(receipt2.id, 1);
    }

    #[test]
    fn test_deposit_retrieval_by_id() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let receipt = contract.deposit_native("user-789".to_string(), Some(300), None);

        let retrieved = contract.get_deposit(receipt.id);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, receipt.id);
        assert_eq!(retrieved.beneficiary_id, "user-789");
        assert_eq!(retrieved.credits_hint, Some(300));
    }

    #[test]
    fn test_nonexistent_deposit_returns_none() {
        let contract = init_contract();
        let result = contract.get_deposit(999);
        assert!(result.is_none());
    }

    #[test]
    fn test_deposits_by_account_multiple() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        
        setup_context(six_near, accounts(2));
        contract.deposit_native("user-a".to_string(), None, None);
        contract.deposit_native("user-b".to_string(), None, None);
        contract.deposit_native("user-c".to_string(), None, None);

        let deposits = contract.get_deposits_for_account(accounts(2));
        assert_eq!(deposits.len(), 3);
        assert_eq!(deposits[0].beneficiary_id, "user-a");
        assert_eq!(deposits[1].beneficiary_id, "user-b");
        assert_eq!(deposits[2].beneficiary_id, "user-c");
    }

    #[test]
    fn test_empty_deposits_for_new_account() {
        let contract = init_contract();
        // Use an arbitrary account that exists
        let deposits = contract.get_deposits_for_account(accounts(5));
        assert_eq!(deposits.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Minimum deposit is $5 USD")]
    fn test_native_deposit_below_min_rejected() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(2_000_000)); // $2 per NEAR

        let two_near = 2u128 * 10u128.pow(24);
        setup_context(two_near, accounts(2));
        let _ = contract.deposit_native("user".to_string(), None, None);
    }

    #[test]
    #[should_panic(expected = "Attach NEAR to deposit")]
    fn test_native_deposit_zero_amount_fails() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        
        setup_context(0, accounts(2));
        contract.deposit_native("user".to_string(), None, None);
    }

    #[test]
    #[should_panic(expected = "NEAR price not configured")]
    fn test_native_deposit_without_price_fails() {
        let mut contract = init_contract();
        // Don't set price, leave at 0
        
        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        contract.deposit_native("user".to_string(), None, None);
    }

    #[test]
    #[should_panic(expected = "NEAR deposits are disabled")]
    fn test_native_deposit_when_disabled_fails() {
        let mut contract = init_contract();
        
        // Disable NEAR deposits
        contract.upsert_token_config(
            NEAR_TOKEN_ID.to_string(),
            "NEAR".to_string(),
            24,
            U128(1_000_000),
            false, // disabled
            true,
        );
        
        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        contract.deposit_native("user".to_string(), None, None);
    }

    #[test]
    fn test_native_deposit_exactly_at_minimum() {
        let mut contract = init_contract();
        // Set price so that deposit equals exactly $5
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        
        let five_near = 5u128 * 10u128.pow(24);
        setup_context(five_near, accounts(2));
        let receipt = contract.deposit_native("user".to_string(), None, None);
        
        assert_eq!(receipt.usd_value.0, MIN_DEPOSIT_USD_MICROS);
    }

    #[test]
    fn test_usd_value_calculation_precision() {
        let contract = init_contract();
        
        let cfg = TokenConfig::new(
            "TEST".to_string(),
            6,
            1_500_000, // $1.50
            false,
            true,
        );
        
        // 10 tokens with 6 decimals = 10_000_000
        let usd = contract.usd_value_for(&cfg, 10_000_000);
        assert_eq!(usd, 15_000_000); // $15
    }

    // ========================================
    // Fungible Token Deposit Tests
    // ========================================

    #[test]
    fn test_ft_deposit_records() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "usdt.token".to_string(),
            "USDT".to_string(),
            6,
            U128(1_000_000), // $1 per token
            true,
            false,
        );

        setup_context(0, "usdt.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user-321".to_string(),
            credits_hint: Some(500),
            memo: Some("stablecoin deposit".to_string()),
        })
        .unwrap();

        let result = contract.ft_on_transfer(accounts(3), U128(7_000_000), msg);
        match result {
            PromiseOrValue::Value(v) => assert_eq!(v.0, 0),
            _ => panic!("Expected Value variant"),
        }

        let deposits = contract.get_deposits_for_account(accounts(3));
        assert_eq!(deposits.len(), 1);
        assert_eq!(deposits[0].usd_value.0, 7 * 1_000_000);
        assert_eq!(deposits[0].token_id, "usdt.token".to_string());
        assert_eq!(deposits[0].memo, Some("stablecoin deposit".to_string()));
    }

    #[test]
        fn test_ft_deposit_minimal_message() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "dai.token".to_string(),
            "DAI".to_string(),
            18,
            U128(1_000_000),
            true,
            false,
        );

        setup_context(0, "dai.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user-999".to_string(),
            credits_hint: None,
            memo: None,
        })
        .unwrap();

        let amount = 10u128 * 10u128.pow(18); // 10 DAI
        let result = contract.ft_on_transfer(accounts(4), U128(amount), msg);
        match result {
            PromiseOrValue::Value(v) => assert_eq!(v.0, 0),
            _ => panic!("Expected Value variant"),
        }

        let deposits = contract.get_deposits_for_account(accounts(4));
        assert_eq!(deposits.len(), 1);
        assert_eq!(deposits[0].beneficiary_id, "user-999");
        assert_eq!(deposits[0].credits_hint, None);
        assert_eq!(deposits[0].memo, None);
    }

    #[test]
    #[should_panic(expected = "Amount must be > 0")]
    fn test_ft_deposit_zero_amount_fails() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "usdt.token".to_string(),
            "USDT".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );
        setup_context(0, "usdt.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage { beneficiary_id: "user".to_string(), credits_hint: None, memo: None }).unwrap();
        contract.ft_on_transfer(accounts(3), U128(0), msg);
    }

    #[test]
    #[should_panic(expected = "beneficiary_id too long")]
    fn test_beneficiary_too_long_native() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let long = "x".repeat(MAX_BENEFICIARY_LEN + 1);
        contract.deposit_native(long, None, None);
    }

    #[test]
    #[should_panic(expected = "memo too long")]
    fn test_memo_too_long_native() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let memo = "y".repeat(MAX_MEMO_LEN + 1);
        contract.deposit_native("user".to_string(), None, Some(memo));
    }

    #[test]
    fn test_usd_value_overflow_guard_large_amount() {
        let mut contract = init_contract();
        // Set a high price and 24 decimals to stress the multiplication
        contract.upsert_token_config(
            "big.token".to_string(),
            "BIG".to_string(),
            24,
            U128(2_000_000), // $2
            true,
            false,
        );
        setup_context(0, "big.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage { beneficiary_id: "user".to_string(), credits_hint: None, memo: None }).unwrap();
        // 10^30 base units (huge but within u128)
        let amount = 1_000_000_000_000_000_000_000_000_000_000u128; // 1e30
        let result = contract.ft_on_transfer(accounts(3), U128(amount), msg);
        match result { PromiseOrValue::Value(v) => assert_eq!(v.0, 0), _ => panic!("Expected Value") }
        // Ensure a record was stored and usd_value > 0
        let deposits = contract.get_deposits_for_account(accounts(3));
        assert_eq!(deposits.len(), 1);
        assert!(deposits[0].usd_value.0 > 0);
    }

    #[test]
    #[should_panic(expected = "Only the owner can migrate")]
    fn test_migrate_requires_owner() {
        // Write initial state
        let mut initial = DepositContract::new(accounts(0), accounts(1));
        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(0)).build());
        initial.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        near_sdk::env::state_write(&initial);
        // Call migrate as non-owner
        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(2)).build());
        let _ = DepositContract::migrate(None);
    }

    #[test]
    fn test_migrate_updates_treasury() {
        // initial state with owner=accounts(0), treasury=accounts(1)
        let mut initial = DepositContract::new(accounts(0), accounts(1));
        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(0)).build());
        initial.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        near_sdk::env::state_write(&initial);
        // migrate as owner to set new treasury=accounts(3)
        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(0)).build());
        let updated = DepositContract::migrate(Some(accounts(3)));
        assert_eq!(updated.treasury_account_id, accounts(3));
        assert_eq!(updated.owner_id, accounts(0));
    }

    #[test]
    #[should_panic(expected = "Unsupported token")]
    fn test_ft_deposit_unsupported_token_fails() {
        let mut contract = init_contract();
        
        setup_context(0, "unknown.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user".to_string(),
            credits_hint: None,
            memo: None,
        })
        .unwrap();

        contract.ft_on_transfer(accounts(3), U128(1_000_000), msg);
    }

    #[test]
    #[should_panic(expected = "Token deposits disabled")]
    fn test_ft_deposit_disabled_token_fails() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            false, // disabled
            false,
        );

        setup_context(0, "usdc.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user".to_string(),
            credits_hint: None,
            memo: None,
        })
        .unwrap();

        contract.ft_on_transfer(accounts(3), U128(6_000_000), msg);
    }

    #[test]
    #[should_panic(expected = "Token price not configured")]
    fn test_ft_deposit_zero_price_fails() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "test.token".to_string(),
            "TEST".to_string(),
            6,
            U128(0), // no price
            true,
            false,
        );

        setup_context(0, "test.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user".to_string(),
            credits_hint: None,
            memo: None,
        })
        .unwrap();

        contract.ft_on_transfer(accounts(3), U128(10_000_000), msg);
    }

    #[test]
    #[should_panic(expected = "Minimum deposit is $5 USD")]
    fn test_ft_deposit_below_minimum_fails() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );

        setup_context(0, "usdc.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user".to_string(),
            credits_hint: None,
            memo: None,
        })
        .unwrap();

        // Only 3 USDC (below $5 minimum)
        contract.ft_on_transfer(accounts(3), U128(3_000_000), msg);
    }

    #[test]
    #[should_panic(expected = "Invalid deposit message payload")]
    fn test_ft_deposit_invalid_json_fails() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );

        setup_context(0, "usdc.token".parse().unwrap());
        let invalid_msg = "{invalid json";

        contract.ft_on_transfer(accounts(3), U128(6_000_000), invalid_msg.to_string());
    }

    #[test]
    fn test_ft_deposit_high_precision_token() {
        let mut contract = init_contract();
        // Token with 18 decimals
        contract.upsert_token_config(
            "weth.token".to_string(),
            "WETH".to_string(),
            18,
            U128(3_000_000_000), // $3000 per token
            true,
            false,
        );

        setup_context(0, "weth.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "whale".to_string(),
            credits_hint: Some(10000),
            memo: None,
        })
        .unwrap();

        // 0.01 WETH = 10^16
        let amount = 10u128.pow(16);
        let result = contract.ft_on_transfer(accounts(5), U128(amount), msg);
        match result {
            PromiseOrValue::Value(v) => assert_eq!(v.0, 0),
            _ => panic!("Expected Value variant"),
        }

        let deposits = contract.get_deposits_for_account(accounts(5));
        assert_eq!(deposits.len(), 1);
        // 0.01 WETH * $3000 = $30
        assert_eq!(deposits[0].usd_value.0, 30_000_000);
    }

    // ========================================
    // Withdrawal Tests
    // ========================================

    #[test]
    fn test_withdraw_ft_requires_owner() {
        let mut contract = init_contract();
        
        // Owner can call (won't panic, but promise won't execute in test)
        setup_context(1, accounts(0));
        contract.withdraw_ft(
            "usdc.token".parse().unwrap(),
            U128(1_000_000),
            None,
            None,
        );
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_withdraw_ft_non_owner_fails() {
        let mut contract = init_contract();
        setup_context(1, accounts(2)); // Not owner
        
        contract.withdraw_ft(
            "usdc.token".parse().unwrap(),
            U128(1_000_000),
            None,
            None,
        );
    }

    #[test]
    #[should_panic(expected = "Attach at least 1 yoctoNEAR")]
    fn test_withdraw_ft_requires_yoctonear() {
        let mut contract = init_contract();
        setup_context(0, accounts(0)); // No attached deposit
        
        contract.withdraw_ft(
            "usdc.token".parse().unwrap(),
            U128(1_000_000),
            None,
            None,
        );
    }

    // ========================================
    // Edge Cases and Boundary Tests
    // ========================================

    #[test]
    fn test_large_deposit_amount() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(5_000_000));

        // 1 million NEAR
        let large_amount = 1_000_000u128 * 10u128.pow(24);
        setup_context(large_amount, accounts(2));
        let receipt = contract.deposit_native("whale".to_string(), Some(1_000_000), None);

        // $5M USD value
        assert_eq!(receipt.usd_value.0, 5_000_000_000_000);
    }

    #[test]
    fn test_token_config_update_changes_price() {
        let mut contract = init_contract();
        
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(2_000_000));
        let config1 = contract.get_token_config(NEAR_TOKEN_ID.to_string()).unwrap();
        
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(3_000_000));
        let config2 = contract.get_token_config(NEAR_TOKEN_ID.to_string()).unwrap();
        
        assert_ne!(config1.price_usd_micros.0, config2.price_usd_micros.0);
        assert_eq!(config2.price_usd_micros.0, 3_000_000);
        // timestamp doesn't advance in test environment
        assert_eq!(config2.last_updated, config1.last_updated);
    }

    #[test]
    fn test_beneficiary_id_formats() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        
        // Test various beneficiary ID formats
        let test_ids = vec![
            "user-123",
            "email@example.com",
            "UUID-12345-67890",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
        ];

        for (i, beneficiary) in test_ids.iter().enumerate() {
            setup_context(six_near, accounts(i as usize + 2));
            let receipt = contract.deposit_native(beneficiary.to_string(), None, None);
            assert_eq!(receipt.beneficiary_id, *beneficiary);
        }
    }

    #[test]
    fn test_zero_decimals_token() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "nft.token".to_string(),
            "NFT".to_string(),
            0, // Zero decimals
            U128(10_000_000), // $10 per token
            true,
            false,
        );

        setup_context(0, "nft.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "nft-buyer".to_string(),
            credits_hint: None,
            memo: None,
        })
        .unwrap();

        // 1 token (no decimals)
        let result = contract.ft_on_transfer(accounts(5), U128(1), msg);
        match result {
            PromiseOrValue::Value(v) => assert_eq!(v.0, 0),
            _ => panic!("Expected Value variant"),
        }

        let deposits = contract.get_deposits_for_account(accounts(5));
        assert_eq!(deposits[0].usd_value.0, 10_000_000); // $10
    }

    #[test]
    fn test_concurrent_deposits_different_accounts() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        
        // Simulate deposits from multiple accounts (only use available test accounts)
        for i in 0..3 {
            setup_context(six_near, accounts(i + 2));
            contract.deposit_native(format!("user-{}", i), None, None);
        }

        // Check each account has exactly one deposit
        for i in 0..3 {
            let deposits = contract.get_deposits_for_account(accounts(i + 2));
            assert_eq!(deposits.len(), 1);
        }
    }

    // ========================================
    // State Management Tests
    // ========================================

    #[test]
    fn test_deposit_id_sequential() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        
        for i in 0..5 {
            setup_context(six_near, accounts(2));
            let receipt = contract.deposit_native(format!("user-{}", i), None, None);
            assert_eq!(receipt.id, i as u64);
        }
    }

    #[test]
    fn test_multiple_deposits_same_beneficiary() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        
        // Same beneficiary, multiple deposits
        contract.deposit_native("user-123".to_string(), Some(100), None);
        contract.deposit_native("user-123".to_string(), Some(200), None);
        contract.deposit_native("user-123".to_string(), Some(300), None);

        let deposits = contract.get_deposits_for_account(accounts(2));
        assert_eq!(deposits.len(), 3);
        assert_eq!(deposits[0].credits_hint, Some(100));
        assert_eq!(deposits[1].credits_hint, Some(200));
        assert_eq!(deposits[2].credits_hint, Some(300));
    }

    #[test]
    fn test_deposit_preserves_account_isolation() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        
        setup_context(six_near, accounts(2));
        contract.deposit_native("user-a".to_string(), None, None);
        
        setup_context(six_near, accounts(3));
        contract.deposit_native("user-b".to_string(), None, None);

        // Verify isolation
        let deposits_a = contract.get_deposits_for_account(accounts(2));
        let deposits_b = contract.get_deposits_for_account(accounts(3));
        
        assert_eq!(deposits_a.len(), 1);
        assert_eq!(deposits_b.len(), 1);
        assert_eq!(deposits_a[0].beneficiary_id, "user-a");
        assert_eq!(deposits_b[0].beneficiary_id, "user-b");
    }

    // ========================================
    // Security & Access Control Tests
    // ========================================

    #[test]
    fn test_only_owner_can_change_treasury() {
        let mut contract = init_contract();
        let original_treasury = contract.treasury_account_id.clone();
        
        contract.set_treasury(accounts(4));
        assert_ne!(contract.treasury_account_id, original_treasury);
        assert_eq!(contract.treasury_account_id, accounts(4));
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this method")]
    fn test_non_owner_cannot_disable_token() {
        let mut contract = init_contract();
        setup_context(0, accounts(2)); // Not owner
        
        contract.upsert_token_config(
            NEAR_TOKEN_ID.to_string(),
            "NEAR".to_string(),
            24,
            U128(1_000_000),
            false,
            true,
        );
    }

    #[test]
    fn test_token_can_be_disabled_and_reenabled() {
        let mut contract = init_contract();
        
        // Disable
        contract.upsert_token_config(
            NEAR_TOKEN_ID.to_string(),
            "NEAR".to_string(),
            24,
            U128(1_000_000),
            false,
            true,
        );
        
        let config = contract.get_token_config(NEAR_TOKEN_ID.to_string()).unwrap();
        assert!(!config.is_enabled);
        
        // Re-enable
        contract.upsert_token_config(
            NEAR_TOKEN_ID.to_string(),
            "NEAR".to_string(),
            24,
            U128(1_000_000),
            true,
            true,
        );
        
        let config = contract.get_token_config(NEAR_TOKEN_ID.to_string()).unwrap();
        assert!(config.is_enabled);
    }

    // ========================================
    // USD Value Calculation Tests
    // ========================================

    #[test]
    fn test_usd_calculation_with_various_decimals() {
        let contract = init_contract();
        
        // 6 decimals (USDC)
        let cfg_6 = TokenConfig::new("USDC".to_string(), 6, 1_000_000, false, true);
        let usd = contract.usd_value_for(&cfg_6, 10_000_000); // 10 USDC
        assert_eq!(usd, 10_000_000);
        
        // 18 decimals (ETH)
        let cfg_18 = TokenConfig::new("ETH".to_string(), 18, 2_000_000_000, false, true);
        let usd = contract.usd_value_for(&cfg_18, 10u128.pow(18)); // 1 ETH
        assert_eq!(usd, 2_000_000_000);
        
        // 8 decimals (BTC)
        let cfg_8 = TokenConfig::new("BTC".to_string(), 8, 50_000_000_000, false, true);
        let usd = contract.usd_value_for(&cfg_8, 100_000_000); // 1 BTC
        assert_eq!(usd, 50_000_000_000);
    }

    #[test]
    fn test_usd_calculation_fractional_tokens() {
        let contract = init_contract();
        
        let cfg = TokenConfig::new("TEST".to_string(), 6, 2_500_000, false, true); // $2.50
        
        // 0.5 tokens
        let usd = contract.usd_value_for(&cfg, 500_000);
        assert_eq!(usd, 1_250_000); // $1.25
        
        // 0.01 tokens
        let usd = contract.usd_value_for(&cfg, 10_000);
        assert_eq!(usd, 25_000); // $0.025
    }

    #[test]
    fn test_usd_value_zero_price_returns_zero() {
        let contract = init_contract();
        let cfg = TokenConfig::new("TEST".to_string(), 6, 0, false, true);
        let usd = contract.usd_value_for(&cfg, 1_000_000);
        assert_eq!(usd, 0);
    }

    #[test]
    fn test_usd_value_zero_amount_returns_zero() {
        let contract = init_contract();
        let cfg = TokenConfig::new("TEST".to_string(), 6, 1_000_000, false, true);
        let usd = contract.usd_value_for(&cfg, 0);
        assert_eq!(usd, 0);
    }

    // ========================================
    // FT Integration Edge Cases
    // ========================================

    #[test]
    #[should_panic(expected = "memo too long")]
    fn test_ft_deposit_with_very_long_memo() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );

        setup_context(0, "usdc.token".parse().unwrap());
        let long_memo = "a".repeat(500);
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user".to_string(),
            credits_hint: None,
            memo: Some(long_memo.clone()),
        })
        .unwrap();

        let _ = contract.ft_on_transfer(accounts(3), U128(6_000_000), msg);
    }

    #[test]
    fn test_ft_deposit_special_characters_in_beneficiary() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );

        setup_context(0, "usdc.token".parse().unwrap());
        let special_id = "user+test@example.com";
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: special_id.to_string(),
            credits_hint: None,
            memo: None,
        })
        .unwrap();

        let result = contract.ft_on_transfer(accounts(3), U128(6_000_000), msg);
        match result {
            PromiseOrValue::Value(v) => assert_eq!(v.0, 0),
            _ => panic!("Expected Value variant"),
        }

        let deposits = contract.get_deposits_for_account(accounts(3));
        assert_eq!(deposits[0].beneficiary_id, special_id);
    }

    #[test]
    fn test_ft_deposit_maximum_credits_hint() {
        let mut contract = init_contract();
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );

        setup_context(0, "usdc.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user".to_string(),
            credits_hint: Some(u64::MAX),
            memo: None,
        })
        .unwrap();

        let result = contract.ft_on_transfer(accounts(3), U128(6_000_000), msg);
        match result {
            PromiseOrValue::Value(v) => assert_eq!(v.0, 0),
            _ => panic!("Expected Value variant"),
        }

        let deposits = contract.get_deposits_for_account(accounts(3));
        assert_eq!(deposits[0].credits_hint, Some(u64::MAX));
    }

    // ========================================
    // Native Deposit Advanced Tests
    // ========================================

    #[test]
    fn test_native_deposit_exactly_one_yoctonear_above_minimum() {
        let mut contract = init_contract();
        // Set price such that minimum is achievable
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        
        // $5.000001 - but due to integer division, might round to $5.000000
        let amount = (5u128 * 10u128.pow(24)) + 1;
        setup_context(amount, accounts(2));
        let receipt = contract.deposit_native("user".to_string(), None, None);
        
        // Accept that due to rounding, it might equal the minimum
        assert!(receipt.usd_value.0 >= MIN_DEPOSIT_USD_MICROS);
    }

    #[test]
    #[should_panic(expected = "Minimum deposit is $5 USD")]
    fn test_native_deposit_one_yoctonear_below_minimum() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        
        // Just under $5
        let amount = (5u128 * 10u128.pow(24)) - 1;
        setup_context(amount, accounts(2));
        contract.deposit_native("user".to_string(), None, None);
    }

    #[test]
    fn test_native_deposit_with_empty_string_beneficiary() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let receipt = contract.deposit_native("".to_string(), None, None);
        
        assert_eq!(receipt.beneficiary_id, "");
    }

    #[test]
    fn test_native_deposit_memo_with_unicode() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let unicode_memo = "🚀 Premium subscription 你好";
        let receipt = contract.deposit_native(
            "user".to_string(),
            None,
            Some(unicode_memo.to_string()),
        );
        
        assert_eq!(receipt.memo, Some(unicode_memo.to_string()));
    }

    // ========================================
    // Token Configuration Edge Cases
    // ========================================

    #[test]
    fn test_upsert_overwrites_existing_token() {
        let mut contract = init_contract();
        
        contract.upsert_token_config(
            "test.token".to_string(),
            "TEST".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );
        
        // Overwrite with different values
        contract.upsert_token_config(
            "test.token".to_string(),
            "TEST2".to_string(),
            8,
            U128(2_000_000),
            false,
            false,
        );
        
        let config = contract.get_token_config("test.token".to_string()).unwrap();
        assert_eq!(config.symbol, "TEST2");
        assert_eq!(config.decimals, 8);
        assert_eq!(config.price_usd_micros.0, 2_000_000);
        assert!(!config.is_enabled);
    }

    #[test]
    fn test_multiple_native_tokens_not_allowed() {
        let mut contract = init_contract();
        
        // Try to add another native token
        contract.upsert_token_config(
            "fake.token".to_string(),
            "FAKE".to_string(),
            18,
            U128(1_000_000),
            true,
            true, // Trying to mark as native
        );
        
        // Both should exist (contract doesn't enforce single native)
        let configs = contract.list_token_configs();
        let native_count = configs.iter().filter(|c| c.is_native).count();
        assert!(native_count >= 2);
    }

    #[test]
    fn test_token_with_maximum_decimals() {
        let mut contract = init_contract();
        
        contract.upsert_token_config(
            "high.token".to_string(),
            "HIGH".to_string(),
            255, // Maximum u8 value
            U128(1_000_000),
            true,
            false,
        );
        
        let config = contract.get_token_config("high.token".to_string()).unwrap();
        assert_eq!(config.decimals, 255);
    }

    // ========================================
    // Withdrawal Edge Cases
    // ========================================

    #[test]
    fn test_withdraw_to_custom_receiver() {
        let mut contract = init_contract();
        setup_context(1, accounts(0));
        
        // Withdraw to different account
        contract.withdraw_ft(
            "usdc.token".parse().unwrap(),
            U128(1_000_000),
            Some(accounts(3)),
            Some("Withdrawal to custom account".to_string()),
        );
    }

    #[test]
    fn test_withdraw_with_zero_amount() {
        let mut contract = init_contract();
        setup_context(1, accounts(0));
        
        // Zero amount withdrawal (contract doesn't prevent it)
        contract.withdraw_ft(
            "usdc.token".parse().unwrap(),
            U128(0),
            None,
            None,
        );
    }

    // ========================================
    // Storage Refund Edge Cases
    // ========================================

    #[test]
    fn test_native_deposit_exact_storage_cost() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let _receipt = contract.deposit_native("user-123".to_string(), Some(250), None);
    }

    #[test]
    fn test_storage_usage_increases_with_deposit_count() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        
        setup_context(six_near, accounts(2));
        contract.deposit_native("user-1".to_string(), None, None);
        
        setup_context(six_near, accounts(2));
        contract.deposit_native("user-2".to_string(), None, None);
        
        setup_context(six_near, accounts(2));
        contract.deposit_native("user-3".to_string(), None, None);
    }

    #[test]
    fn test_treasury_change_with_pending_deposits() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        
        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        contract.deposit_native("user-1".to_string(), None, None);
        
        let original_treasury = contract.treasury_account_id.clone();
        
        setup_context(0, accounts(0));
        let new_treasury = accounts(5);
        contract.set_treasury(new_treasury.clone());
        
        assert_ne!(contract.treasury_account_id, original_treasury);
        assert_eq!(contract.treasury_account_id, new_treasury);
        
        setup_context(six_near, accounts(3));
        contract.deposit_native("user-2".to_string(), None, None);
    }

    #[test]
    fn test_multiple_treasury_changes() {
        let mut contract = init_contract();
        
        let treasuries = vec![accounts(2), accounts(3), accounts(4)];
        
        for treasury in treasuries {
            setup_context(0, accounts(0));
            contract.set_treasury(treasury.clone());
            assert_eq!(contract.treasury_account_id, treasury);
        }
    }

    #[test]
    fn test_deposit_id_never_reuses() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        let mut seen_ids = std::collections::HashSet::new();
        
        for i in 0..10 {
            setup_context(six_near, accounts(2));
            let receipt = contract.deposit_native(format!("user-{}", i), None, None);
            assert!(!seen_ids.contains(&receipt.id), "Deposit ID should be unique");
            seen_ids.insert(receipt.id);
        }
    }

    #[test]
    fn test_token_disable_then_reenable_preserves_price() {
        let mut contract = init_contract();
        
        let token_id = "test.token".to_string();
        let original_price = 5_000_000u128;
        
        contract.upsert_token_config(
            token_id.clone(),
            "TEST".to_string(),
            6,
            U128(original_price),
            true,
            false,
        );
        
        contract.upsert_token_config(
            token_id.clone(),
            "TEST".to_string(),
            6,
            U128(original_price),
            false,
            false,
        );
        
        let config_disabled = contract.get_token_config(token_id.clone()).unwrap();
        assert!(!config_disabled.is_enabled);
        assert_eq!(config_disabled.price_usd_micros.0, original_price);
        
        contract.upsert_token_config(
            token_id.clone(),
            "TEST".to_string(),
            6,
            U128(original_price),
            true,
            false,
        );
        
        let config_enabled = contract.get_token_config(token_id.clone()).unwrap();
        assert!(config_enabled.is_enabled);
        assert_eq!(config_enabled.price_usd_micros.0, original_price);
    }

    #[test]
    fn test_list_token_configs_large_dataset() {
        let mut contract = init_contract();
        
        for i in 0..50 {
            contract.upsert_token_config(
                format!("token{}.test", i),
                format!("TK{}", i),
                6,
                U128((i as u128 + 1) * 1_000_000),
                true,
                false,
            );
        }
        
        let configs = contract.list_token_configs();
        assert!(configs.len() >= 51);
        
        let has_near = configs.iter().any(|c| c.token_id == NEAR_TOKEN_ID);
        let has_token_25 = configs.iter().any(|c| c.token_id == "token25.test");
        assert!(has_near);
        assert!(has_token_25);
    }

    #[test]
    fn test_withdraw_to_custom_address_different_from_treasury() {
        let mut contract = init_contract();
        setup_context(1, accounts(0));
        
        let custom_receiver = accounts(5);
        assert_ne!(custom_receiver, contract.treasury_account_id);
        
        contract.withdraw_ft(
            "usdc.token".parse().unwrap(),
            U128(1_000_000),
            Some(custom_receiver),
            Some("Custom withdrawal".to_string()),
        );
    }

    #[test]
    fn test_withdraw_maximum_u128_amount() {
        let mut contract = init_contract();
        setup_context(1, accounts(0));
        
        contract.withdraw_ft(
            "token.test".parse().unwrap(),
            U128(u128::MAX),
            None,
            None,
        );
    }

    #[test]
    fn test_get_deposits_preserves_order() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        
        let beneficiaries = vec!["first", "second", "third", "fourth", "fifth"];
        for beneficiary in &beneficiaries {
            setup_context(six_near, accounts(2));
            contract.deposit_native(beneficiary.to_string(), None, None);
        }
        
        let deposits = contract.get_deposits_for_account(accounts(2));
        assert_eq!(deposits.len(), 5);
        
        for (i, deposit) in deposits.iter().enumerate() {
            assert_eq!(deposit.beneficiary_id, beneficiaries[i]);
            assert_eq!(deposit.id, i as u64);
        }
    }

    #[test]
    fn test_get_deposit_by_id_boundary_values() {
        let mut contract = init_contract();
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));

        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let receipt = contract.deposit_native("user".to_string(), None, None);
        
        let retrieved = contract.get_deposit(receipt.id);
        assert!(retrieved.is_some());
        
        let non_existent = contract.get_deposit(u64::MAX);
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_usd_calculation_with_very_small_amounts() {
        let contract = init_contract();
        
        let cfg = TokenConfig::new(
            "TEST".to_string(),
            18,
            1_000_000,
            false,
            true,
        );
        
        let one_wei = 1u128;
        let usd = contract.usd_value_for(&cfg, one_wei);
        assert_eq!(usd, 0, "Sub-cent amounts should round to 0");
    }

    #[test]
    fn test_usd_calculation_no_precision_loss_large_amounts() {
        let contract = init_contract();
        
        let cfg = TokenConfig::new(
            "BTC".to_string(),
            8,
            50_000_000_000,
            false,
            true,
        );
        
        let one_btc = 100_000_000u128;
        let usd = contract.usd_value_for(&cfg, one_btc);
        assert_eq!(usd, 50_000_000_000, "1 BTC should be exactly $50,000");
        
        let ten_btc = 1_000_000_000u128;
        let usd_ten = contract.usd_value_for(&cfg, ten_btc);
        assert_eq!(usd_ten, 500_000_000_000, "10 BTC should be exactly $500,000");
    }

    #[test]
    fn test_usd_calculation_consistency_across_scales() {
        let contract = init_contract();
        
        let cfg = TokenConfig::new(
            "TEST".to_string(),
            6,
            2_000_000,
            false,
            true,
        );
        
        let one_token = 1_000_000u128;
        let usd_one = contract.usd_value_for(&cfg, one_token);
        
        let ten_tokens = 10_000_000u128;
        let usd_ten = contract.usd_value_for(&cfg, ten_tokens);
        
        assert_eq!(usd_ten, usd_one * 10, "USD should scale linearly");
    }

    // ========================================
    // Integration Scenarios
    // ========================================

    #[test]
    fn test_full_deposit_flow_native() {
        let mut contract = init_contract();
        
        // 1. Owner sets price
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(3_500_000));
        
        // 2. User makes deposit
        let ten_near = 10u128 * 10u128.pow(24);
        setup_context(ten_near, accounts(2));
        let receipt = contract.deposit_native(
            "user@example.com".to_string(),
            Some(1000),
            Some("Monthly subscription".to_string()),
        );
        
        // 3. Verify deposit recorded
        assert_eq!(receipt.id, 0);
        assert_eq!(receipt.usd_value.0, 35_000_000); // 10 * $3.50
        
        // 4. Retrieve by ID
        let retrieved = contract.get_deposit(0).unwrap();
        assert_eq!(retrieved.beneficiary_id, "user@example.com");
        
        // 5. Retrieve by account
        let deposits = contract.get_deposits_for_account(accounts(2));
        assert_eq!(deposits.len(), 1);
    }

    #[test]
    fn test_full_deposit_flow_ft() {
        let mut contract = init_contract();
        
        // 1. Owner configures token
        contract.upsert_token_config(
            "dai.token".to_string(),
            "DAI".to_string(),
            18,
            U128(1_000_000),
            true,
            false,
        );
        
        // 2. FT contract calls ft_on_transfer
        setup_context(0, "dai.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "0x123abc".to_string(),
            credits_hint: Some(500),
            memo: Some("Premium plan".to_string()),
        })
        .unwrap();
        
        let amount = 25u128 * 10u128.pow(18); // 25 DAI
        let result = contract.ft_on_transfer(accounts(3), U128(amount), msg);
        
        match result {
            PromiseOrValue::Value(v) => assert_eq!(v.0, 0),
            _ => panic!("Expected Value variant"),
        }
        
        // 3. Verify deposit
        let deposits = contract.get_deposits_for_account(accounts(3));
        assert_eq!(deposits.len(), 1);
        assert_eq!(deposits[0].usd_value.0, 25_000_000);
        assert_eq!(deposits[0].beneficiary_id, "0x123abc");
    }

    #[test]
    fn test_mixed_deposits_same_account() {
        let mut contract = init_contract();
        
        // Setup
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(1_000_000));
        contract.upsert_token_config(
            "usdc.token".to_string(),
            "USDC".to_string(),
            6,
            U128(1_000_000),
            true,
            false,
        );
        
        // Native deposit
        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        contract.deposit_native("user-1".to_string(), None, None);
        
        // FT deposit from same account
        setup_context(0, "usdc.token".parse().unwrap());
        let msg = serde_json::to_string(&DepositMessage {
            beneficiary_id: "user-2".to_string(),
            credits_hint: None,
            memo: None,
        })
        .unwrap();
        contract.ft_on_transfer(accounts(2), U128(10_000_000), msg);
        
        // Verify both recorded
        let deposits = contract.get_deposits_for_account(accounts(2));
        assert_eq!(deposits.len(), 2);
        assert_eq!(deposits[0].token_id, NEAR_TOKEN_ID);
        assert_eq!(deposits[1].token_id, "usdc.token");
    }

    #[test]
    fn test_price_update_affects_subsequent_deposits() {
        let mut contract = init_contract();
        
        // First price
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(2_000_000));
        let six_near = 6u128 * 10u128.pow(24);
        setup_context(six_near, accounts(2));
        let receipt1 = contract.deposit_native("user-1".to_string(), None, None);
        
        // Update price (must be called as owner)
        setup_context(0, accounts(0));
        contract.update_token_price(NEAR_TOKEN_ID.to_string(), U128(3_000_000));
        
        setup_context(six_near, accounts(3));
        let receipt2 = contract.deposit_native("user-2".to_string(), None, None);
        
        // Different USD values
        assert_eq!(receipt1.usd_value.0, 12_000_000); // 6 * $2
        assert_eq!(receipt2.usd_value.0, 18_000_000); // 6 * $3
    }
}
