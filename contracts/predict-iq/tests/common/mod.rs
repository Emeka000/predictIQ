// Common test utilities and helpers

use predict_iq::{PredictIQ, PredictIQClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env, String, Vec};

/// Setup test environment with initialized contract
pub fn setup() -> (Env, PredictIQClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PredictIQ, ());
    let client = PredictIQClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &100);

    (env, client, admin)
}

/// Setup test environment with token contract
pub fn setup_with_token() -> (Env, PredictIQClient<'static>, Address, Address) {
    let (env, client, admin) = setup();

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();

    (env, client, admin, token_address)
}

/// Create a simple test market
pub fn create_market(
    client: &PredictIQClient,
    env: &Env,
    creator: &Address,
    token: &Address,
) -> u64 {
    let options = Vec::from_array(
        env,
        [String::from_str(env, "Yes"), String::from_str(env, "No")],
    );

    let oracle_config = predict_iq::types::OracleConfig {
        oracle_address: Address::generate(env),
        feed_id: String::from_str(env, "test_feed"),
        min_responses: Some(1),
        max_staleness_seconds: 3600,
        max_confidence_bps: 200,
    };

    client.create_market(
        creator,
        &String::from_str(env, "Test Market"),
        &options,
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &oracle_config,
        &predict_iq::types::MarketTier::Basic,
        token,
        &0,
        &0,
    )
}

/// Setup user with token balance
pub fn setup_user_with_balance(env: &Env, token: &Address, amount: i128) -> Address {
    let user = Address::generate(env);
    let token_client = token::StellarAssetClient::new(env, token);
    token_client.mint(&user, &amount);
    user
}

/// Advance ledger time
pub fn advance_time(env: &Env, seconds: u64) {
    use soroban_sdk::testutils::Ledger;
    env.ledger().with_mut(|li| {
        li.timestamp += seconds;
    });
}

/// Assert market status
pub fn assert_market_status(
    client: &PredictIQClient,
    market_id: u64,
    expected_status: predict_iq::types::MarketStatus,
) {
    let market = client.get_market(&market_id).unwrap();
    assert_eq!(market.status, expected_status);
}

/// Create multiple users with balances
pub fn create_users(env: &Env, token: &Address, count: u32, balance: i128) -> Vec<Address> {
    let mut users = Vec::new(env);
    let token_client = token::StellarAssetClient::new(env, token);

    for _ in 0..count {
        let user = Address::generate(env);
        token_client.mint(&user, &balance);
        users.push_back(user);
    }

    users
}

/// Setup guardians for governance
pub fn setup_guardians(
    client: &PredictIQClient,
    env: &Env,
    count: u32,
) -> Vec<predict_iq::types::Guardian> {
    let mut guardians = Vec::new(env);

    for _ in 0..count {
        guardians.push_back(predict_iq::types::Guardian {
            address: Address::generate(env),
            voting_power: 1,
        });
    }

    client.initialize_guardians(&guardians);
    guardians
}

/// Place bet and return bet amount
pub fn place_bet_helper(
    client: &PredictIQClient,
    user: &Address,
    market_id: u64,
    outcome: u32,
    amount: i128,
    token: &Address,
) -> i128 {
    client.place_bet(user, &market_id, &outcome, &amount, token, &None);
    amount
}

/// Resolve market and verify status
pub fn resolve_and_verify(client: &PredictIQClient, market_id: u64, winning_outcome: u32) {
    client.resolve_market(&market_id, &winning_outcome);
    assert_market_status(client, market_id, predict_iq::types::MarketStatus::Resolved);
}
