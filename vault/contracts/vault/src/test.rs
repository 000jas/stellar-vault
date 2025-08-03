#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Env as _}, Address, Env};

#[test]
fn test_initialize_and_deposit() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaultContract);
    let owner = Address::generate(&env);
    let token_id = Address::generate(&env);

    // Call initialize
    VaultContract::initialize(env.clone(), owner.clone(), token_id.clone(), 12345);

    // Check owner
    assert_eq!(VaultContract::get_owner(env.clone()), owner);

    // Check token id
    assert_eq!(VaultContract::get_token_id(env.clone()), token_id);

    // Check unlock time
    assert_eq!(VaultContract::get_unlock_time(env.clone()), 12345);

    // Locked amount should be zero
    assert_eq!(VaultContract::get_locked_amount(env.clone()), 0);
}

#[test]
fn test_vault() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Test deposit
    env.as_contract(&contract_id, || {
        client.deposit(&1000);
        assert_eq!(client.balance(), 1000);
    });

    // Test withdraw
    env.as_contract(&contract_id, || {
        client.withdraw(&500);
        assert_eq!(client.balance(), 500);
    });
}
