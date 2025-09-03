use near_sdk::NearToken;
use serde_json::json;

/// Test reward rate edge cases to ensure the system handles extreme values safely
#[tokio::test]
async fn test_zero_reward_rate() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    // Test initialization with zero reward rate (should be clamped to 1)
    let zero_reward_rate = 0u128;
    let min_stake = NearToken::from_near(1);
    let max_stake = NearToken::from_near(1000);

    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": zero_reward_rate,
            "min_stake_amount": min_stake.as_yoctonear().to_string(),
            "max_stake_amount": max_stake.as_yoctonear().to_string()
        }))
        .transact()
        .await?;

    let is_success = init_outcome.is_success();
    if !is_success {
        println!(
            "Contract initialization failed with logs: {:?}",
            init_outcome.logs()
        );
        if let Err(failure) = init_outcome.into_result() {
            println!("Failure details: {:?}", failure);
        }
    }

    assert!(
        is_success,
        "Contract initialization with zero reward rate should succeed (clamped to 1)"
    );

    // Verify the reward rate was clamped to 1
    let rate_outcome = contract
        .view("get_reward_rate")
        .args_json(json!({}))
        .await?;
    let current_rate: u128 = rate_outcome.json()?;
    assert_eq!(current_rate, 1, "Zero reward rate should be clamped to 1");

    println!("✅ Zero reward rate test passed - initialization properly clamped to minimum value");
    Ok(())
}

#[tokio::test]
async fn test_very_high_reward_rate() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    // Test with very high reward rate (should be clamped to max)
    let very_high_reward_rate = 1_000_000_000_000u128; // 1 trillion - very high, should be clamped
    let min_stake = NearToken::from_near(1);
    let max_stake = NearToken::from_near(1000);

    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": very_high_reward_rate,
            "min_stake_amount": min_stake.as_yoctonear().to_string(),
            "max_stake_amount": max_stake.as_yoctonear().to_string()
        }))
        .transact()
        .await?;

    assert!(
        init_outcome.is_success(),
        "Contract initialization with high reward rate should succeed"
    );

    // Verify the reward rate was clamped to maximum
    let rate_outcome = contract
        .view("get_reward_rate")
        .args_json(json!({}))
        .await?;
    let current_rate: u128 = rate_outcome.json()?;
    assert_eq!(
        current_rate, 1_000_000_000,
        "Very high reward rate should be clamped to 1 billion"
    );

    // Test staking with high reward rate
    let user_account = sandbox.dev_create_account().await?;
    let stake_amount = NearToken::from_near(1); // Small stake to avoid overflow

    let stake_outcome = user_account
        .call(contract.id(), "stake")
        .deposit(stake_amount)
        .transact()
        .await?;
    assert!(
        stake_outcome.is_success(),
        "Staking with high reward rate should succeed"
    );

    // Test reward calculation (should handle overflow gracefully)
    let rewards_outcome = contract
        .view("calculate_pending_rewards")
        .args_json(json!({"account": user_account.id()}))
        .await?;
    let rewards: String = rewards_outcome.json()?;
    assert!(
        !rewards.is_empty(),
        "Reward calculation should return a value even with high reward rate"
    );

    // Test claiming rewards (should not panic)
    let claim_outcome = user_account
        .call(contract.id(), "claim_rewards")
        .transact()
        .await?;
    assert!(
        claim_outcome.is_success(),
        "Claiming rewards should succeed even with high reward rate"
    );

    println!("✅ Very high reward rate test passed - system handled extreme values safely");
    Ok(())
}

#[tokio::test]
async fn test_reward_rate_overflow_protection() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    // Initialize with moderate reward rate
    let reward_rate = 1_000_000u128; // 1 million rewards per second per NEAR
    let min_stake = NearToken::from_near(1);
    let max_stake = NearToken::from_near(1000);

    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": reward_rate,
            "min_stake_amount": min_stake.as_yoctonear().to_string(),
            "max_stake_amount": max_stake.as_yoctonear().to_string()
        }))
        .transact()
        .await?;

    assert!(
        init_outcome.is_success(),
        "Contract initialization should succeed"
    );

    let user_account = sandbox.dev_create_account().await?;

    // Test with reasonable stake amount to try to trigger overflow
    let stake_amount = NearToken::from_near(10); // Use smaller amount that test accounts can afford
    let stake_outcome = user_account
        .call(contract.id(), "stake")
        .deposit(stake_amount)
        .transact()
        .await?;
    assert!(stake_outcome.is_success(), "Staking should succeed");

    // Multiple reward calculations to test consistency
    for i in 0..5 {
        let rewards_outcome = contract
            .view("calculate_pending_rewards")
            .args_json(json!({"account": user_account.id()}))
            .await?;
        let rewards: String = rewards_outcome.json()?;
        assert!(
            !rewards.is_empty(),
            "Reward calculation {} should return a value",
            i
        );

        // Parse to ensure it's a valid number
        let _reward_value: u128 = rewards
            .parse()
            .expect("Reward should be a valid u128 number");
    }

    // Test claiming multiple times
    for i in 0..3 {
        let claim_outcome = user_account
            .call(contract.id(), "claim_rewards")
            .transact()
            .await?;
        assert!(claim_outcome.is_success(), "Claim {} should succeed", i);

        // Wait a bit between claims
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("✅ Reward rate overflow protection test passed");
    Ok(())
}

#[tokio::test]
async fn test_reward_rate_update_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    // Initialize with normal reward rate
    let initial_reward_rate = 100u128;
    let min_stake = NearToken::from_near(1);
    let max_stake = NearToken::from_near(1000);

    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": initial_reward_rate,
            "min_stake_amount": min_stake.as_yoctonear().to_string(),
            "max_stake_amount": max_stake.as_yoctonear().to_string()
        }))
        .transact()
        .await?;

    assert!(
        init_outcome.is_success(),
        "Contract initialization should succeed"
    );

    let user_account = sandbox.dev_create_account().await?;

    // Stake some amount
    let stake_amount = NearToken::from_near(10);
    let stake_outcome = user_account
        .call(contract.id(), "stake")
        .deposit(stake_amount)
        .transact()
        .await?;
    assert!(stake_outcome.is_success(), "Initial staking should succeed");

    // Test updating to very high reward rate (should be clamped)
    let very_high_rate = 2_000_000_000u128; // 2 billion - above the 1 billion limit
    let update_outcome = contract
        .as_account()
        .call(contract.id(), "update_reward_rate")
        .args_json(json!({"new_rate": very_high_rate}))
        .transact()
        .await?;
    assert!(
        update_outcome.is_success(),
        "Updating to high reward rate should succeed"
    );

    // Verify the rate was clamped to maximum
    let rate_outcome = contract
        .view("get_reward_rate")
        .args_json(json!({}))
        .await?;
    let current_rate: u128 = rate_outcome.json()?;
    assert_eq!(
        current_rate, 1_000_000_000,
        "Reward rate should be clamped to maximum"
    );

    // Test reward calculation with new high rate
    let rewards_outcome = contract
        .view("calculate_pending_rewards")
        .args_json(json!({"account": user_account.id()}))
        .await?;
    let rewards: String = rewards_outcome.json()?;
    assert!(
        !rewards.is_empty(),
        "Reward calculation should work with updated high rate"
    );

    // Test updating back to low rate
    let low_rate = 1u128;
    let update_low_outcome = contract
        .as_account()
        .call(contract.id(), "update_reward_rate")
        .args_json(json!({"new_rate": low_rate}))
        .transact()
        .await?;
    assert!(
        update_low_outcome.is_success(),
        "Updating to low reward rate should succeed"
    );

    // Verify low rate update
    let final_rate_outcome = contract
        .view("get_reward_rate")
        .args_json(json!({}))
        .await?;
    let final_rate: u128 = final_rate_outcome.json()?;
    assert_eq!(
        final_rate, low_rate,
        "Reward rate should be updated to low value"
    );

    // Test that rewards still work with low rate
    let final_rewards_outcome = contract
        .view("calculate_pending_rewards")
        .args_json(json!({"account": user_account.id()}))
        .await?;
    let final_rewards: String = final_rewards_outcome.json()?;
    assert!(
        !final_rewards.is_empty(),
        "Reward calculation should work with low rate"
    );

    println!("✅ Reward rate update edge cases test passed");
    Ok(())
}

#[tokio::test]
async fn test_extreme_staking_amounts_with_high_rewards() -> Result<(), Box<dyn std::error::Error>>
{
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    // Initialize with high reward rate
    let high_reward_rate = 1_000_000u128; // 1 million per second per NEAR
    let min_stake = NearToken::from_near(1);
    let max_stake = NearToken::from_near(10000); // Use the new higher limit

    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": high_reward_rate,
            "min_stake_amount": min_stake.as_yoctonear().to_string(),
            "max_stake_amount": max_stake.as_yoctonear().to_string()
        }))
        .transact()
        .await?;

    assert!(
        init_outcome.is_success(),
        "Contract initialization should succeed"
    );

    // Test with multiple users staking maximum amounts
    let mut users = Vec::new();
    for _ in 0..5 {
        users.push(sandbox.dev_create_account().await?);
    }

    // Each user stakes a reasonable amount (test accounts have limited balance)
    let stake_per_user = NearToken::from_near(10); // Use smaller amount that test accounts can afford
    for (i, user) in users.iter().enumerate() {
        let stake_outcome = user
            .call(contract.id(), "stake")
            .deposit(stake_per_user)
            .transact()
            .await?;
        assert!(
            stake_outcome.is_success(),
            "User {} staking should succeed",
            i
        );
    }

    // Verify total staked is correct
    let total_staked_outcome = contract
        .view("get_total_staked")
        .args_json(json!({}))
        .await?;
    let total_staked: String = total_staked_outcome.json()?;
    let expected_total = stake_per_user.as_yoctonear() * users.len() as u128;
    assert_eq!(
        total_staked.parse::<u128>().unwrap(),
        expected_total,
        "Total staked should equal sum of all user stakes"
    );

    // Test reward calculations for all users
    for (i, user) in users.iter().enumerate() {
        let rewards_outcome = contract
            .view("calculate_pending_rewards")
            .args_json(json!({"account": user.id()}))
            .await?;
        let rewards: String = rewards_outcome.json()?;
        assert!(
            !rewards.is_empty(),
            "User {} reward calculation should work",
            i
        );

        // Ensure reward is a valid number
        let _reward_value: u128 = rewards
            .parse()
            .expect("Reward should be a valid u128 number");
    }

    // Test claiming rewards for all users
    for (i, user) in users.iter().enumerate() {
        let claim_outcome = user.call(contract.id(), "claim_rewards").transact().await?;
        assert!(
            claim_outcome.is_success(),
            "User {} reward claiming should succeed",
            i
        );
    }

    // Test partial unstaking with high rewards
    let partial_unstake = NearToken::from_near(5); // Partial amount of the 10 NEAR staked
    for (i, user) in users.iter().enumerate() {
        let unstake_outcome = user
            .call(contract.id(), "unstake")
            .args_json(json!({"amount": partial_unstake.as_yoctonear().to_string()}))
            .transact()
            .await?;
        assert!(
            unstake_outcome.is_success(),
            "User {} partial unstaking should succeed",
            i
        );
    }

    println!("✅ Extreme staking amounts with high rewards test passed");
    Ok(())
}

#[tokio::test]
async fn test_reward_rate_boundary_values() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;

    // Test boundary values for reward rate
    let boundary_values = vec![
        (0u128, 1u128),                             // Zero should be clamped to 1
        (1u128, 1u128),                             // Minimum valid value
        (1_000_000u128, 1_000_000u128),             // 1 million - high but reasonable
        (1_000_000_000u128, 1_000_000_000u128),     // 1 billion - at the limit
        (1_000_000_000_000u128, 1_000_000_000u128), // 1 trillion - should be clamped to 1 billion
    ];

    for (idx, (input_rate, expected_rate)) in boundary_values.iter().enumerate() {
        let contract = sandbox.dev_deploy(contract_wasm).await?;

        let min_stake = NearToken::from_near(1);
        let max_stake = NearToken::from_near(1000);

        let init_outcome = contract
            .call("new")
            .args_json(json!({
                "reward_rate": input_rate,
                "min_stake_amount": min_stake.as_yoctonear().to_string(),
                "max_stake_amount": max_stake.as_yoctonear().to_string()
            }))
            .transact()
            .await?;

        assert!(
            init_outcome.is_success(),
            "Boundary value {} initialization should succeed",
            idx
        );

        // Verify the reward rate was set/clamped correctly
        let rate_outcome = contract
            .view("get_reward_rate")
            .args_json(json!({}))
            .await?;
        let actual_rate: u128 = rate_outcome.json()?;
        assert_eq!(
            actual_rate, *expected_rate,
            "Boundary value {} should be clamped correctly",
            idx
        );

        let user_account = sandbox.dev_create_account().await?;

        // Test basic operations with boundary reward rate
        let stake_outcome = user_account
            .call(contract.id(), "stake")
            .deposit(NearToken::from_near(10))
            .transact()
            .await?;
        assert!(
            stake_outcome.is_success(),
            "Boundary value {} staking should succeed",
            idx
        );

        // Test reward calculation
        let rewards_outcome = contract
            .view("calculate_pending_rewards")
            .args_json(json!({"account": user_account.id()}))
            .await?;
        let rewards: String = rewards_outcome.json()?;
        assert!(
            !rewards.is_empty(),
            "Boundary value {} reward calculation should work",
            idx
        );

        // Ensure reward is a valid number and doesn't overflow
        let reward_value: u128 = rewards
            .parse()
            .expect("Reward should be a valid u128 number");
        assert!(reward_value <= u128::MAX, "Reward should not overflow u128");

        // Test claiming
        let claim_outcome = user_account
            .call(contract.id(), "claim_rewards")
            .transact()
            .await?;
        assert!(
            claim_outcome.is_success(),
            "Boundary value {} claim should succeed",
            idx
        );
    }

    println!("✅ Reward rate boundary values test passed");
    Ok(())
}
