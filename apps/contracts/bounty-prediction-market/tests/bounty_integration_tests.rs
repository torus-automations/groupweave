use serde_json::json;
use near_sdk::NearToken;

#[tokio::test]
async fn test_complete_bounty_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;
    
    // Initialize the contract
    let reward_rate = 100u128;
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
    
    assert!(init_outcome.is_success(), "Contract initialization failed");

    // Test complete bounty lifecycle
    test_bounty_creation_and_staking(&sandbox, &contract).await?;
    test_bounty_closure_and_rewards(&sandbox, &contract).await?;
    test_multi_participant_scenario(&sandbox, &contract).await?;
    test_platform_fee_collection(&sandbox, &contract).await?;

    Ok(())
}

async fn test_bounty_creation_and_staking(
    sandbox: &near_workspaces::Worker<near_workspaces::network::Sandbox>,
    contract: &near_workspaces::Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a bounty
    let create_outcome = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Who will win the championship?",
            "description": "Predict the winner of the upcoming championship",
            "options": ["Team A", "Team B", "Team C"],
            "max_stake_per_user": NearToken::from_near(50).as_yoctonear().to_string(),
            "duration_blocks": 1000
        }))
        .transact()
        .await?;
    
    assert!(create_outcome.is_success(), "Bounty creation failed");
    let bounty_id: u64 = create_outcome.json()?;
    assert_eq!(bounty_id, 1);

    // Verify bounty details
    let bounty_outcome = contract
        .view("get_bounty")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?;
    let bounty: Option<serde_json::Value> = bounty_outcome.json()?;
    assert!(bounty.is_some());
    let bounty = bounty.unwrap();
    assert_eq!(bounty["title"], "Who will win the championship?");
    assert_eq!(bounty["options"].as_array().unwrap().len(), 3);
    assert!(bounty["is_active"].as_bool().unwrap());

    // Create user accounts and stake on different options
    let user1 = sandbox.dev_create_account().await?;
    let user2 = sandbox.dev_create_account().await?;
    let user3 = sandbox.dev_create_account().await?;

    // User 1 stakes on option 0
    let stake1_outcome = user1
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 0}))
        .deposit(NearToken::from_near(10))
        .transact()
        .await?;
    assert!(stake1_outcome.is_success(), "User 1 staking failed");

    // User 2 stakes on option 1
    let stake2_outcome = user2
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 1}))
        .deposit(NearToken::from_near(20))
        .transact()
        .await?;
    assert!(stake2_outcome.is_success(), "User 2 staking failed");

    // User 3 stakes on option 0 (same as user 1)
    let stake3_outcome = user3
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 0}))
        .deposit(NearToken::from_near(15))
        .transact()
        .await?;
    assert!(stake3_outcome.is_success(), "User 3 staking failed");

    // Verify stakes
    let stakes_outcome = contract
        .view("get_bounty_stakes")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?;
    let stakes: Vec<String> = stakes_outcome.json()?;
    assert_eq!(stakes[0], NearToken::from_near(25).as_yoctonear().to_string()); // 10 + 15 NEAR
    assert_eq!(stakes[1], NearToken::from_near(20).as_yoctonear().to_string()); // 20 NEAR
    assert_eq!(stakes[2], "0"); // No stakes on option 2

    // Verify individual participant stakes
    let user1_stake_outcome = contract
        .view("get_participant_stake")
        .args_json(json!({"account": user1.id(), "bounty_id": bounty_id}))
        .await?;
    let user1_stake: Option<serde_json::Value> = user1_stake_outcome.json()?;
    assert!(user1_stake.is_some());
    let user1_stake = user1_stake.unwrap();
    assert_eq!(user1_stake["amount"], NearToken::from_near(10).as_yoctonear().to_string());
    assert_eq!(user1_stake["option_index"], 0);

    println!("✅ Bounty creation and staking tests passed");
    Ok(())
}

async fn test_bounty_closure_and_rewards(
    sandbox: &near_workspaces::Worker<near_workspaces::network::Sandbox>,
    contract: &near_workspaces::Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new bounty for this test
    let create_outcome = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Test Closure Bounty",
            "description": "Testing bounty closure and rewards",
            "options": ["Option A", "Option B"],
            "max_stake_per_user": NearToken::from_near(30).as_yoctonear().to_string(),
            "duration_blocks": 10 // Short duration for testing
        }))
        .transact()
        .await?;
    
    let bounty_id: u64 = create_outcome.json()?;

    // Add participants
    let user1 = sandbox.dev_create_account().await?;
    let user2 = sandbox.dev_create_account().await?;

    // User 1 stakes on option 0 (will lose)
    let _stake1 = user1
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 0}))
        .deposit(NearToken::from_near(5))
        .transact()
        .await?;

    // User 2 stakes on option 1 (will win)
    let _stake2 = user2
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 1}))
        .deposit(NearToken::from_near(15))
        .transact()
        .await?;

    // Wait for bounty to expire (simulate time passage)
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Close the bounty
    let close_outcome = contract
        .call("close_bounty")
        .args_json(json!({"bounty_id": bounty_id}))
        .transact()
        .await?;
    assert!(close_outcome.is_success(), "Bounty closure failed");

    // Verify bounty results
    let results_outcome = contract
        .view("get_bounty_results")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?;
    let results: Option<serde_json::Value> = results_outcome.json()?;
    assert!(results.is_some());
    let results = results.unwrap();
    assert!(results["is_closed"].as_bool().unwrap());
    assert_eq!(results["winning_option"], 1); // Option 1 should win (15 NEAR vs 5 NEAR)

    println!("✅ Bounty closure and rewards tests passed");
    Ok(())
}

async fn test_multi_participant_scenario(
    sandbox: &near_workspaces::Worker<near_workspaces::network::Sandbox>,
    contract: &near_workspaces::Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a bounty with multiple options
    let create_outcome = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Multi-Participant Test",
            "description": "Testing multiple participants across different options",
            "options": ["Red", "Blue", "Green", "Yellow"],
            "max_stake_per_user": NearToken::from_near(25).as_yoctonear().to_string(),
            "duration_blocks": 50
        }))
        .transact()
        .await?;
    
    let bounty_id: u64 = create_outcome.json()?;

    // Create multiple users
    let users = vec![
        sandbox.dev_create_account().await?,
        sandbox.dev_create_account().await?,
        sandbox.dev_create_account().await?,
        sandbox.dev_create_account().await?,
        sandbox.dev_create_account().await?,
    ];

    // Users stake on different options with varying amounts
    let stakes = vec![
        (0, NearToken::from_near(8)),  // Red
        (1, NearToken::from_near(12)), // Blue
        (1, NearToken::from_near(18)), // Blue (will be winning option)
        (2, NearToken::from_near(5)),  // Green
        (0, NearToken::from_near(7)),  // Red
    ];

    for (i, (option, amount)) in stakes.iter().enumerate() {
        let stake_outcome = users[i]
            .call(contract.id(), "stake_on_option")
            .args_json(json!({"bounty_id": bounty_id, "option_index": option}))
            .deposit(*amount)
            .transact()
            .await?;
        assert!(stake_outcome.is_success(), "Staking failed for user {}", i);
    }

    // Verify total stakes per option
    let stakes_outcome = contract
        .view("get_bounty_stakes")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?;
    let option_stakes: Vec<String> = stakes_outcome.json()?;
    
    // Red: 8 + 7 = 15 NEAR
    assert_eq!(option_stakes[0], NearToken::from_near(15).as_yoctonear().to_string());
    // Blue: 12 + 18 = 30 NEAR (winning option)
    assert_eq!(option_stakes[1], NearToken::from_near(30).as_yoctonear().to_string());
    // Green: 5 NEAR
    assert_eq!(option_stakes[2], NearToken::from_near(5).as_yoctonear().to_string());
    // Yellow: 0 NEAR
    assert_eq!(option_stakes[3], "0");

    // Verify user bounties
    let user_bounties_outcome = contract
        .view("get_user_bounties")
        .args_json(json!({"account": users[0].id()}))
        .await?;
    let user_bounties: Vec<serde_json::Value> = user_bounties_outcome.json()?;
    assert!(!user_bounties.is_empty());

    println!("✅ Multi-participant scenario tests passed");
    Ok(())
}

async fn test_platform_fee_collection(
    sandbox: &near_workspaces::Worker<near_workspaces::network::Sandbox>,
    contract: &near_workspaces::Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    // Test platform fee rate management
    let fee_rate_outcome = contract
        .view("get_platform_fee_rate")
        .args_json(json!({}))
        .await?;
    let fee_rate: u128 = fee_rate_outcome.json()?;
    assert_eq!(fee_rate, 500); // Should be 5% (500 basis points)

    // Update platform fee rate
    let update_fee_outcome = contract
        .call("update_platform_fee_rate")
        .args_json(json!({"new_rate": 300})) // 3%
        .transact()
        .await?;
    assert!(update_fee_outcome.is_success(), "Fee rate update failed");

    // Verify updated fee rate
    let new_fee_rate_outcome = contract
        .view("get_platform_fee_rate")
        .args_json(json!({}))
        .await?;
    let new_fee_rate: u128 = new_fee_rate_outcome.json()?;
    assert_eq!(new_fee_rate, 300);

    // Test contract pause/unpause functionality
    let pause_outcome = contract
        .call("pause_contract")
        .args_json(json!({}))
        .transact()
        .await?;
    assert!(pause_outcome.is_success(), "Contract pause failed");

    let is_paused_outcome = contract
        .view("is_contract_paused")
        .args_json(json!({}))
        .await?;
    let is_paused: bool = is_paused_outcome.json()?;
    assert!(is_paused);

    // Try to create bounty while paused (should fail)
    let create_while_paused = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Should Fail",
            "description": "This should fail because contract is paused",
            "options": ["A", "B"],
            "max_stake_per_user": NearToken::from_near(10).as_yoctonear().to_string(),
            "duration_blocks": 100
        }))
        .transact()
        .await?;
    assert!(!create_while_paused.is_success(), "Bounty creation should fail when paused");

    // Unpause contract
    let unpause_outcome = contract
        .call("unpause_contract")
        .args_json(json!({}))
        .transact()
        .await?;
    assert!(unpause_outcome.is_success(), "Contract unpause failed");

    let is_paused_after_outcome = contract
        .view("is_contract_paused")
        .args_json(json!({}))
        .await?;
    let is_paused_after: bool = is_paused_after_outcome.json()?;
    assert!(!is_paused_after);

    println!("✅ Platform fee collection tests passed");
    Ok(())
}

#[tokio::test]
async fn test_single_participant_bounty() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;
    
    // Initialize contract
    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": 100u128,
            "min_stake_amount": NearToken::from_near(1).as_yoctonear().to_string(),
            "max_stake_amount": NearToken::from_near(1000).as_yoctonear().to_string()
        }))
        .transact()
        .await?;
    assert!(init_outcome.is_success());

    // Create bounty
    let bounty_id: u64 = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Single Participant Test",
            "description": "Testing single participant scenario",
            "options": ["Yes", "No"],
            "max_stake_per_user": NearToken::from_near(20).as_yoctonear().to_string(),
            "duration_blocks": 10
        }))
        .transact()
        .await?
        .json()?;

    // Single user stakes
    let user = sandbox.dev_create_account().await?;
    let stake_outcome = user
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 0}))
        .deposit(NearToken::from_near(10))
        .transact()
        .await?;
    assert!(stake_outcome.is_success());

    // Wait and close bounty
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    let close_outcome = contract
        .call("close_bounty")
        .args_json(json!({"bounty_id": bounty_id}))
        .transact()
        .await?;
    assert!(close_outcome.is_success());

    // Verify bounty is closed
    let bounty_outcome = contract
        .view("get_bounty")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?;
    let bounty: Option<serde_json::Value> = bounty_outcome.json()?;
    let bounty = bounty.unwrap();
    assert!(bounty["is_closed"].as_bool().unwrap());

    println!("✅ Single participant bounty test passed");
    Ok(())
}

#[tokio::test]
async fn test_emergency_functions() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;
    
    // Initialize contract
    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": 100u128,
            "min_stake_amount": NearToken::from_near(1).as_yoctonear().to_string(),
            "max_stake_amount": NearToken::from_near(1000).as_yoctonear().to_string()
        }))
        .transact()
        .await?;
    assert!(init_outcome.is_success());

    // Create bounty
    let bounty_id: u64 = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Emergency Test",
            "description": "Testing emergency functions",
            "options": ["A", "B"],
            "max_stake_per_user": NearToken::from_near(15).as_yoctonear().to_string(),
            "duration_blocks": 1000 // Long duration
        }))
        .transact()
        .await?
        .json()?;

    // Add participant
    let user = sandbox.dev_create_account().await?;
    let _stake = user
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 0}))
        .deposit(NearToken::from_near(8))
        .transact()
        .await?;

    // Emergency close bounty (before expiration)
    let emergency_close_outcome = contract
        .call("emergency_close_bounty")
        .args_json(json!({"bounty_id": bounty_id}))
        .transact()
        .await?;
    assert!(emergency_close_outcome.is_success());

    // Verify bounty is closed
    let bounty_outcome = contract
        .view("get_bounty")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?;
    let bounty: Option<serde_json::Value> = bounty_outcome.json()?;
    let bounty = bounty.unwrap();
    assert!(bounty["is_closed"].as_bool().unwrap());

    println!("✅ Emergency functions test passed");
    Ok(())
}