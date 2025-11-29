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
    let max_stake = NearToken::from_near(10000);

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

    // Create a bounty
    let create_outcome = contract
        .call("create_content_bounty")
        .args_json(json!({
            "title": "Who will win?",
            "description": "Predict the winner",
            "requirements": "Submit your best work",
            "base_prize": NearToken::from_near(1).as_yoctonear().to_string(),
            "max_stake_per_user": NearToken::from_near(50).as_yoctonear().to_string(),
            "duration_days": 1
        }))
        .deposit(NearToken::from_near(2)) // 1 NEAR prize + storage
        .transact()
        .await?;

    assert!(create_outcome.is_success(), "Bounty creation failed: {:?}", create_outcome);
    let bounty_id: u64 = create_outcome.json()?;
    assert_eq!(bounty_id, 1);

    // Submit content (creating options/submissions)
    let user1 = sandbox.dev_create_account().await?;
    let user2 = sandbox.dev_create_account().await?;

    // User 1 submits
    let submit1 = user1
        .call(contract.id(), "submit_content")
        .args_json(json!({
            "bounty_id": bounty_id,
            "creation_id": "creation-1",
            "title": "Submission 1",
            "thumbnail_url": "http://url1"
        }))
        .transact()
        .await?;
    assert!(submit1.is_success());

    // User 2 submits
    let submit2 = user2
        .call(contract.id(), "submit_content")
        .args_json(json!({
            "bounty_id": bounty_id,
            "creation_id": "creation-2",
            "title": "Submission 2",
            "thumbnail_url": "http://url2"
        }))
        .transact()
        .await?;
    assert!(submit2.is_success());

    // User 1 stakes on Submission 0 (User 1's submission)
    let stake1_outcome = user1
        .call(contract.id(), "stake_on_submission")
        .args_json(json!({"bounty_id": bounty_id, "submission_index": 0}))
        .deposit(NearToken::from_near(10))
        .transact()
        .await?;
    assert!(stake1_outcome.is_success(), "User 1 staking failed");

    // User 2 stakes on Submission 1 (User 2's submission)
    let stake2_outcome = user2
        .call(contract.id(), "stake_on_submission")
        .args_json(json!({"bounty_id": bounty_id, "submission_index": 1}))
        .deposit(NearToken::from_near(20))
        .transact()
        .await?;
    assert!(stake2_outcome.is_success(), "User 2 staking failed");

    // Verify stakes
    let stakes_outcome = contract
        .view("get_bounty_submission_stakes")
        .args_json(json!({"bounty_id": bounty_id}))
        .await?;
    let stakes: Vec<String> = stakes_outcome.json()?;
    assert_eq!(stakes[0], NearToken::from_near(10).as_yoctonear().to_string());
    assert_eq!(stakes[1], NearToken::from_near(20).as_yoctonear().to_string());

    println!("✅ Bounty creation, submission, and staking tests passed");
    Ok(())
}
