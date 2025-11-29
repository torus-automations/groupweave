use serde_json::json;
use near_sdk::NearToken;

#[tokio::test]
async fn test_participant_limit_enforcement() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    // Initialize contract
    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": 100u128,
            "min_stake_amount": NearToken::from_near(1).as_yoctonear().to_string(),
            "max_stake_amount": NearToken::from_near(10000).as_yoctonear().to_string()
        }))
        .transact()
        .await?;
    assert!(init_outcome.is_success());

    // Create bounty
    let bounty_id: u64 = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Participant Limit Test",
            "description": "Testing maximum participants",
            "options": ["Option A", "Option B"],
            "max_stake_per_user": NearToken::from_near(10).as_yoctonear().to_string(),
            "duration_blocks": 1000
        }))
        .transact()
        .await?
        .json()?;

    // Add participants up to the limit (100)
    let mut users = Vec::new();
    for i in 0..100 {
        let user = sandbox.dev_create_account().await?;
        users.push(user.clone());
        
        let stake_outcome = user
            .call(contract.id(), "stake_on_option")
            .args_json(json!({"bounty_id": bounty_id, "option_index": i % 2}))
            .deposit(NearToken::from_near(1))
            .transact()
            .await?;
        assert!(stake_outcome.is_success(), "Participant {} should succeed", i);
    }

    // Verify we have 100 participants
    let participant_count: u64 = contract
        .view("get_bounty_participant_count")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?
        .json()?;
    assert_eq!(participant_count, 100);

    // Try to add 101st participant - should fail
    let user_101 = sandbox.dev_create_account().await?;
    let stake_outcome_101 = user_101
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 0}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    
    assert!(!stake_outcome_101.is_success(), "101st participant should fail");
    let error_message = format!("{:?}", stake_outcome_101.into_result().unwrap_err());
    assert!(error_message.contains("maximum participant limit"));

    println!("✅ Participant limit test passed");
    Ok(())
}

#[tokio::test]
async fn test_existing_participant_can_change_vote() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    // Initialize contract
    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": 100u128,
            "min_stake_amount": NearToken::from_near(1).as_yoctonear().to_string(),
            "max_stake_amount": NearToken::from_near(10000).as_yoctonear().to_string()
        }))
        .transact()
        .await?;
    assert!(init_outcome.is_success());

    // Create bounty
    let bounty_id: u64 = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Vote Change Test",
            "description": "Testing vote changes don't count as new participants",
            "options": ["Option A", "Option B", "Option C"],
            "max_stake_per_user": NearToken::from_near(10).as_yoctonear().to_string(),
            "duration_blocks": 1000
        }))
        .transact()
        .await?
        .json()?;

    // Add 100 participants
    let mut users = Vec::new();
    for i in 0..100 {
        let user = sandbox.dev_create_account().await?;
        users.push(user.clone());
        
        let stake_outcome = user
            .call(contract.id(), "stake_on_option")
            .args_json(json!({"bounty_id": bounty_id, "option_index": 0}))
            .deposit(NearToken::from_near(1))
            .transact()
            .await?;
        assert!(stake_outcome.is_success());
    }

    // Existing participant changes their vote - should succeed
    let vote_change_outcome = users[0]
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id, "option_index": 1}))
        .deposit(NearToken::from_near(2))
        .transact()
        .await?;
    
    assert!(vote_change_outcome.is_success(), "Existing participant should be able to change vote");

    // Verify participant count is still 100
    let participant_count: u64 = contract
        .view("get_bounty_participant_count")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?
        .json()?;
    assert_eq!(participant_count, 100);

    println!("✅ Vote change test passed");
    Ok(())
}

#[tokio::test]
async fn test_multiple_bounties_independent_limits() -> Result<(), Box<dyn std::error::Error>> {
    let contract_wasm = &near_workspaces::compile_project("./").await?;
    let sandbox = near_workspaces::sandbox().await?;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    // Initialize contract
    let init_outcome = contract
        .call("new")
        .args_json(json!({
            "reward_rate": 100u128,
            "min_stake_amount": NearToken::from_near(1).as_yoctonear().to_string(),
            "max_stake_amount": NearToken::from_near(10000).as_yoctonear().to_string()
        }))
        .transact()
        .await?;
    assert!(init_outcome.is_success());

    // Create two bounties
    let bounty_id1: u64 = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Bounty 1",
            "description": "First bounty",
            "options": ["A", "B"],
            "max_stake_per_user": NearToken::from_near(10).as_yoctonear().to_string(),
            "duration_blocks": 1000
        }))
        .transact()
        .await?
        .json()?;

    let bounty_id2: u64 = contract
        .call("create_bounty")
        .args_json(json!({
            "title": "Bounty 2",
            "description": "Second bounty",
            "options": ["X", "Y"],
            "max_stake_per_user": NearToken::from_near(10).as_yoctonear().to_string(),
            "duration_blocks": 1000
        }))
        .transact()
        .await?
        .json()?;

    // Same user participates in both bounties - should succeed
    let user = sandbox.dev_create_account().await?;
    
    let stake1 = user
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id1, "option_index": 0}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(stake1.is_success());

    let stake2 = user
        .call(contract.id(), "stake_on_option")
        .args_json(json!({"bounty_id": bounty_id2, "option_index": 0}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(stake2.is_success());

    // Verify both bounties have 1 participant each
    let count1: u64 = contract
        .view("get_bounty_participant_count")
        .args_json(json!({"bounty_id": bounty_id1}))
        .await?
        .json()?;
    let count2: u64 = contract
        .view("get_bounty_participant_count")
        .args_json(json!({"bounty_id": bounty_id2}))
        .await?
        .json()?;
    
    assert_eq!(count1, 1);
    assert_eq!(count2, 1);

    println!("✅ Multiple bounties independent limits test passed");
    Ok(())
}
