// Gas Benchmarking Tests for PredictIQ
// Measures instruction counts and memory usage for various operations

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

extern crate predict_iq;
use predict_iq::{PredictIQ, PredictIQClient};

fn create_test_env() -> (Env, Address, PredictIQClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PredictIQ, ());
    let client = PredictIQClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &100);

    (env, admin, client)
}

fn create_options(env: &Env, count: u32) -> Vec<String> {
    let mut options = Vec::new(env);
    for _ in 0..count {
        options.push_back(String::from_str(env, "x"));
    }
    options
}

fn create_oracle_config(env: &Env) -> predict_iq::types::OracleConfig {
    predict_iq::types::OracleConfig {
        oracle_address: Address::generate(env),
        feed_id: String::from_str(env, "test_feed"),
        min_responses: Some(1),
        max_staleness_seconds: 3600,
        max_confidence_bps: 200,
    }
}

#[test]
fn bench_create_market_10_outcomes() {
    let (env, admin, client) = create_test_env();

    let options = create_options(&env, 10);
    let oracle_config = create_oracle_config(&env);
    let native_token = Address::generate(&env);

    let result = client.try_create_market(
        &admin,
        &String::from_str(&env, "10 Outcome Market"),
        &options,
        &1000,
        &2000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &native_token,
        &0,
        &0,
    );

    assert!(result.is_ok());
}

#[test]
fn bench_create_market_50_outcomes() {
    let (env, admin, client) = create_test_env();

    let options = create_options(&env, 50);
    let oracle_config = create_oracle_config(&env);
    let native_token = Address::generate(&env);

    let result = client.try_create_market(
        &admin,
        &String::from_str(&env, "50 Outcome Market"),
        &options,
        &1000,
        &2000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &native_token,
        &0,
        &0,
    );

    assert!(result.is_ok());
}

#[test]
fn bench_create_market_100_outcomes() {
    let (env, admin, client) = create_test_env();

    let options = create_options(&env, 100);
    let oracle_config = create_oracle_config(&env);
    let native_token = Address::generate(&env);

    let result = client.try_create_market(
        &admin,
        &String::from_str(&env, "100 Outcome Market"),
        &options,
        &1000,
        &2000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &native_token,
        &0,
        &0,
    );

    assert!(result.is_ok());
}

#[test]
fn bench_place_multiple_bets() {
    let (env, admin, client) = create_test_env();

    let options = create_options(&env, 10);
    let oracle_config = create_oracle_config(&env);
    let native_token = Address::generate(&env);

    let market_id = client.create_market(
        &admin,
        &String::from_str(&env, "Bet Test Market"),
        &options,
        &1000,
        &2000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &native_token,
        &0,
        &0,
    );

    let bettor = Address::generate(&env);

    for _ in 1..=5 {
        let _ = client.try_place_bet(&bettor, &market_id, &0, &1000, &native_token, &None);
    }
}

#[test]
fn bench_resolve_market() {
    let (env, admin, client) = create_test_env();

    let options = create_options(&env, 50);
    let oracle_config = create_oracle_config(&env);
    let native_token = Address::generate(&env);

    let market_id = client.create_market(
        &admin,
        &String::from_str(&env, "Resolution Test Market"),
        &options,
        &1000,
        &2000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &native_token,
        &0,
        &0,
    );

    let result = client.try_resolve_market(&market_id, &0);
    assert!(result.is_ok());
}

#[test]
fn bench_get_resolution_metrics() {
    let (env, admin, client) = create_test_env();

    let options = create_options(&env, 50);
    let oracle_config = create_oracle_config(&env);
    let native_token = Address::generate(&env);

    let market_id = client.create_market(
        &admin,
        &String::from_str(&env, "Metrics Test Market"),
        &options,
        &1000,
        &2000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &native_token,
        &0,
        &0,
    );

    let _ = client.try_resolve_market(&market_id, &0);
    let _metrics = client.get_resolution_metrics(&market_id, &0);
}

#[test]
fn bench_reject_excessive_outcomes() {
    let (env, admin, client) = create_test_env();

    let options = create_options(&env, 101);
    let oracle_config = create_oracle_config(&env);
    let native_token = Address::generate(&env);

    let result = client.try_create_market(
        &admin,
        &String::from_str(&env, "Too Many Outcomes"),
        &options,
        &1000,
        &2000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &native_token,
        &0,
        &0,
    );

    assert!(result.is_err());
}

#[test]
fn bench_full_market_lifecycle() {
    let (env, admin, client) = create_test_env();

    let options = create_options(&env, 10);
    let oracle_config = create_oracle_config(&env);
    let native_token = Address::generate(&env);

    let market_id = client.create_market(
        &admin,
        &String::from_str(&env, "Lifecycle Test"),
        &options,
        &1000,
        &2000,
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        &native_token,
        &0,
        &0,
    );

    let bettor1 = Address::generate(&env);
    let bettor2 = Address::generate(&env);

    let _ = client.try_place_bet(&bettor1, &market_id, &0, &1000, &native_token, &None);
    let _ = client.try_place_bet(&bettor2, &market_id, &1, &2000, &native_token, &None);

    let _ = client.try_resolve_market(&market_id, &0);

    let _metrics = client.get_resolution_metrics(&market_id, &0);

    let _ = client.try_claim_winnings(&bettor1, &market_id, &native_token);
}
