#![no_std] // No standard library for embedded-like environments
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

// Define the contract's storage keys.
// This enum helps organize the persistent data stored on the blockchain.
#[contracttype]
pub enum DataKey {
    Owner,           // The Address of the vault's owner
    TokenId,         // The Address of the token contract this vault holds
    UnlockTimestamp, // The u64 timestamp (ledger close time) when tokens can be withdrawn
    LockedAmount,    // The i128 total amount of tokens currently locked in the vault
}

// Declare the smart contract struct.
// This is a placeholder; the actual contract logic is in the impl block.
#[contract]
pub struct VaultContract;

// Implement the contract's public functions.
// Functions annotated with #[contractimpl] are callable from outside the contract.
#[contractimpl]
impl VaultContract {
    /// Initializes the vault contract.
    /// This function sets up the initial state of the vault and can only be called once.
    ///
    /// # Arguments
    /// * env - The Soroban environment, providing access to ledger, storage, etc.
    /// * owner - The address of the account that will own and control this vault.
    /// * token_id - The address of the token contract that this vault will manage.
    /// * unlock_timestamp - The specific ledger close time (in seconds since epoch)
    ///                        after which the owner can withdraw funds.
    pub fn initialize(env: Env, owner: Address, token_id: Address, unlock_timestamp: u64) {
        // Check if the contract has already been initialized.
        // We use env.storage().instance().has(&DataKey::Owner) to check if the 'Owner' key exists.
        if env.storage().instance().has(&DataKey::Owner) {
            // If it has, panic! and revert the transaction.
            // panic! is Soroban's way of signaling an unrecoverable error and reverting all changes.
            panic!("Vault already initialized");
        }

        // Store the initial state values in instance storage.
        // env.storage().instance().set() writes data persistently to the blockchain.
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::TokenId, &token_id);
        env.storage().instance().set(&DataKey::UnlockTimestamp, &unlock_timestamp);
        // Initialize the locked amount to 0.
        env.storage().instance().set(&DataKey::LockedAmount, &0i128);
    }

    /// Deposits tokens into the vault.
    ///
    /// # Arguments
    /// * env - The Soroban environment.
    /// * from - The address of the account depositing tokens. This account must authorize the call.
    /// * amount - The amount of tokens to deposit. Must be a positive value.
    pub fn deposit(env: Env, from: Address, amount: i128) {
        from.require_auth();

        if amount <= 0 {
            panic!("Deposit amount must be positive");
        }

        // Retrieve the token contract ID from storage.
        let token_id: Address = env.storage().instance().get(&DataKey::TokenId).expect("Token ID not set");
        // Create a client to interact with the token contract.
        let token_client = token::Client::new(&env, &token_id);

        // Transfer tokens from the from account to this contract's address.
        // Note: The from account must have previously `approve`d this contract to spend amount.
        token_client.transfer(&from, &env.current_contract_address(), &amount);

        // Update the total locked amount in the vault.
        let mut locked_amount: i128 = env.storage().instance().get(&DataKey::LockedAmount).expect("Locked amount not set");
        // Use checked_add to prevent integer overflow, which is a common smart contract vulnerability.
        locked_amount = locked_amount.checked_add(amount).expect("Overflow in locked amount");
        env.storage().instance().set(&DataKey::LockedAmount, &locked_amount);
    }

    /// Withdraws tokens from the vault after the unlock timestamp has passed.
    /// Only the vault owner can call this function.
    ///
    /// # Arguments
    /// * env - The Soroban environment.
    /// * to - The address to send the withdrawn tokens to. Typically the owner's address.
    /// * amount - The amount of tokens to withdraw. Must be a positive value.
    pub fn withdraw(env: Env, to: Address, amount: i128) {
        // Retrieve the vault owner's address from storage.
        let owner: Address = env.storage().instance().get(&DataKey::Owner).expect("Owner not set");
        // Ensure that only the owner has authorized this transaction.
        owner.require_auth();

        // Validate the withdrawal amount.
        if amount <= 0 {
            panic!("Withdraw amount must be positive");
        }

        // Check if the current ledger time has passed the unlock timestamp.
        let unlock_timestamp: u64 = env.storage().instance().get(&DataKey::UnlockTimestamp).expect("Unlock timestamp not set");
        let current_ledger_time = env.ledger().timestamp(); // Get the current ledger close time.

        if current_ledger_time < unlock_timestamp {
            panic!("Tokens are still locked");
        }

        // Check for sufficient locked funds in the vault.
        let mut locked_amount: i128 = env.storage().instance().get(&DataKey::LockedAmount).expect("Locked amount not set");
        if amount > locked_amount {
            panic!("Insufficient locked funds");
        }

        // Retrieve the token contract ID and create a client.
        let token_id: Address = env.storage().instance().get(&DataKey::TokenId).expect("Token ID not set");
        let token_client = token::Client::new(&env, &token_id);

        // Transfer tokens from this contract's address to the to address.
        token_client.transfer(&env.current_contract_address(), &to, &amount);

        // Update the total locked amount in the vault.
        // Use checked_sub to prevent integer underflow.
        locked_amount = locked_amount.checked_sub(amount).expect("Underflow in locked amount");
        env.storage().instance().set(&DataKey::LockedAmount, &locked_amount);
    }

    /// Returns the current total locked amount in the vault.
    /// This is a read-only function and doesn't require authorization.
    pub fn get_locked_amount(env: Env) -> i128 {
        // unwrap_or(0) provides a default value if the key isn't found (e.g., before initialization).
        env.storage().instance().get(&DataKey::LockedAmount).unwrap_or(0)
    }

    /// Returns the unlock timestamp for the vault.
    pub fn get_unlock_time(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::UnlockTimestamp).expect("Unlock timestamp not set")
    }

    /// Returns the owner's address of the vault.
    pub fn get_owner(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Owner).expect("Owner not set")
    }

    /// Returns the token ID managed by the vault.
    pub fn get_token_id(env: Env) -> Address {
        env.storage().instance().get(&DataKey::TokenId).expect("Token ID not set")
    }
}