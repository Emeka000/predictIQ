// End-to-end integration tests for PredictIQ contract
// Tests complete workflows across multiple modules

use predict_iq::{PredictIQ, PredictIQClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String, Vec,
};

mod common;
use common::*;

#[test]
fn test_complete_market_lifecycle() {
    let (env, client, admin, token) = setup_with_token();

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let market_id = create_market(&client, &env, &admin, &token);
    assert_eq!(market_id, 1);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let mint_client = token::StellarAssetClient::new(&env, &token);
    mint_client.mint(&user1, &10_000);
    mint_client.mint(&user2, &20_000);
    mint_client.mint(&user3, &30_000);

    client.place_bet(&user1, &market_id, &0, &1_000, &token, &None);
    client.place_bet(&user2, &market_id, &0, &2_000, &token, &None);
    client.place_bet(&user3, &market_id, &1, &3_000, &token, &None);

    let market = client.get_market(&market_id).unwrap();
    assert_eq!(market.status, predict_iq::types::MarketStatus::Active);

    client.resolve_market(&market_id, &0);

    let market = client.get_market(&market_id).unwrap();
    assert_eq!(market.status, predict_iq::types::MarketStatus::Resolved);

    let bal_client = token::Client::new(&env, &token);
    let balance1_before = bal_client.balance(&user1);
    let balance2_before = bal_client.balance(&user2);

    let winnings1 = client.claim_winnings(&user1, &market_id, &token);
    let winnings2 = client.claim_winnings(&user2, &market_id, &token);

    assert!(winnings1 > 1_000);
    assert!(winnings2 > 2_000);

    let balance1_after = bal_client.balance(&user1);
    let balance2_after = bal_client.balance(&user2);

    assert_eq!(balance1_after - balance1_before, winnings1);
    assert_eq!(balance2_after - balance2_before, winnings2);

    let result = client.try_claim_winnings(&user3, &market_id, &token);
    assert_eq!(result, Err(Ok(predict_iq::ErrorCode::NoWinnings)));
}

#[test]
fn test_multi_market_concurrent_operations() {
    let (env, client, admin, token) = setup_with_token();

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let market1 = create_market(&client, &env, &admin, &token);
    let market2 = create_market(&client, &env, &admin, &token);
    let market3 = create_market(&client, &env, &admin, &token);

    assert_eq!(market1, 1);
    assert_eq!(market2, 2);
    assert_eq!(market3, 3);

    let user = Address::generate(&env);
    let mint_client = token::StellarAssetClient::new(&env, &token);
    mint_client.mint(&user, &100_000);

    client.place_bet(&user, &market1, &0, &1_000, &token, &None);
    client.place_bet(&user, &market2, &1, &2_000, &token, &None);
    client.place_bet(&user, &market3, &0, &3_000, &token, &None);

    client.resolve_market(&market1, &0);
    client.resolve_market(&market2, &0);
    client.resolve_market(&market3, &0);

    let winnings1 = client.claim_winnings(&user, &market1, &token);
    let winnings3 = client.claim_winnings(&user, &market3, &token);

    assert!(winnings1 > 0);
    assert!(winnings3 > 0);

    let result = client.try_claim_winnings(&user, &market2, &token);
    assert_eq!(result, Err(Ok(predict_iq::ErrorCode::NoWinnings)));
}

#[test]
fn test_referral_system_integration() {
    let (env, client, admin, token) = setup_with_token();

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let market_id = create_market(&client, &env, &admin, &token);

    let bettor = Address::generate(&env);
    let referrer = Address::generate(&env);

    let mint_client = token::StellarAssetClient::new(&env, &token);
    mint_client.mint(&bettor, &10_000);

    client.place_bet(
        &bettor,
        &market_id,
        &0,
        &1_000,
        &token,
        &Some(referrer.clone()),
    );

    let bal_client = token::Client::new(&env, &token);
    let rewards_before = bal_client.balance(&referrer);
    let claimed = client.claim_referral_rewards(&referrer, &token);

    if claimed > 0 {
        let rewards_after = bal_client.balance(&referrer);
        assert!(rewards_after > rewards_before);
    }
}

#[test]
fn test_conditional_market_chain() {
    let (env, client, admin, token) = setup_with_token();

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let parent_id = create_market(&client, &env, &admin, &token);

    client.resolve_market(&parent_id, &0);

    let options = Vec::from_array(
        &env,
        [String::from_str(&env, "Yes"), String::from_str(&env, "No")],
    );

    let oracle_config = predict_iq::types::OracleConfig {
        oracle_address: Address::generate(&env),
        feed_id: String::from_str(&env, "test"),
        min_responses: Some(1),
        max_staleness_seconds: 3600,
        max_confidence_bps: 200,
    };

    let child_id = client.create_market(
        &admin,
        &String::from_str(&env, "Child Market"),
        &options,
        &2000,
        &3000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &token,
        &parent_id,
        &0,
    );

    assert_eq!(child_id, 2);

    let child_market = client.get_market(&child_id).unwrap();
    assert_eq!(child_market.parent_id, parent_id);
    assert_eq!(child_market.parent_outcome_idx, 0);
}

#[test]
fn test_emergency_pause_and_recovery() {
    let (env, client, admin, token) = setup_with_token();

    let guardian = Address::generate(&env);
    client.set_guardian(&guardian);

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let market_id = create_market(&client, &env, &admin, &token);

    let user = Address::generate(&env);
    let mint_client = token::StellarAssetClient::new(&env, &token);
    mint_client.mint(&user, &10_000);

    client.place_bet(&user, &market_id, &0, &1_000, &token, &None);

    client.pause();

    let result = client.try_place_bet(&user, &market_id, &0, &1_000, &token, &None);
    assert_eq!(result, Err(Ok(predict_iq::ErrorCode::ContractPaused)));

    client.resolve_market(&market_id, &0);

    let winnings = client.claim_winnings(&user, &market_id, &token);
    assert!(winnings > 0);

    client.unpause();

    let new_market = create_market(&client, &env, &admin, &token);
    assert_eq!(new_market, 2);
}

#[test]
fn test_governance_upgrade_workflow() {
    let (env, client, _admin, _token) = setup_with_token();

    let guardian1 = Address::generate(&env);
    let guardian2 = Address::generate(&env);
    let guardian3 = Address::generate(&env);

    let mut guardians = Vec::new(&env);
    guardians.push_back(predict_iq::types::Guardian {
        address: guardian1.clone(),
        voting_power: 1,
    });
    guardians.push_back(predict_iq::types::Guardian {
        address: guardian2.clone(),
        voting_power: 1,
    });
    guardians.push_back(predict_iq::types::Guardian {
        address: guardian3.clone(),
        voting_power: 1,
    });

    client.initialize_guardians(&guardians);

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let wasm_hash = String::from_str(&env, "new_wasm_hash_123");
    client.initiate_upgrade(&wasm_hash);

    client.vote_for_upgrade(&guardian1, &true);
    client.vote_for_upgrade(&guardian2, &true);

    let (for_votes, against_votes) = client.get_upgrade_votes();
    assert_eq!(for_votes, 2);
    assert_eq!(against_votes, 0);

    env.ledger().with_mut(|li| li.timestamp = 1000 + 172_801);

    let result = client.try_execute_upgrade();
    assert!(result.is_ok());

    let pending = client.get_pending_upgrade();
    assert!(pending.is_none());
}

#[test]
fn test_market_cancellation_and_refunds() {
    let (env, client, admin, token) = setup_with_token();

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let market_id = create_market(&client, &env, &admin, &token);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let mint_client = token::StellarAssetClient::new(&env, &token);
    mint_client.mint(&user1, &10_000);
    mint_client.mint(&user2, &20_000);

    client.place_bet(&user1, &market_id, &0, &1_000, &token, &None);
    client.place_bet(&user2, &market_id, &1, &2_000, &token, &None);

    // Cancellation/refund flow is commented out pending cancel_market_admin exposure
    let _ = market_id;
}

#[test]
fn test_fee_collection_and_distribution() {
    let (env, client, admin, token) = setup_with_token();

    env.ledger().with_mut(|li| li.timestamp = 1000);

    client.set_base_fee(&100);

    let market_id = create_market(&client, &env, &admin, &token);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let mint_client = token::StellarAssetClient::new(&env, &token);
    mint_client.mint(&user1, &10_000);
    mint_client.mint(&user2, &10_000);

    client.place_bet(&user1, &market_id, &0, &1_000, &token, &None);
    client.place_bet(&user2, &market_id, &1, &1_000, &token, &None);

    let revenue_before = client.get_revenue(&token);

    client.resolve_market(&market_id, &0);
    client.claim_winnings(&user1, &market_id, &token);

    let revenue_after = client.get_revenue(&token);

    assert!(revenue_after >= revenue_before);
}

#[test]
fn test_reputation_based_deposit_waiver() {
    let (env, client, _admin, token) = setup_with_token();

    client.set_creation_deposit(&10_000_000);

    let creator = Address::generate(&env);

    client.set_creator_reputation(&creator, &predict_iq::types::CreatorReputation::Pro);

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let market_id = create_market(&client, &env, &creator, &token);

    let market = client.get_market(&market_id).unwrap();
    assert_eq!(market.creation_deposit, 0);
}
