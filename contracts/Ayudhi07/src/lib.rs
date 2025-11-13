#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Symbol, symbol_short};

// Structure to track remittance details
#[contracttype]
#[derive(Clone)]
pub struct Remittance {
    pub sender: Address,
    pub receiver: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub is_completed: bool,
}

// Mapping remittance ID to Remittance data
#[contracttype]
pub enum RemittanceBook {
    Transaction(u64)
}

// Counter for unique remittance IDs
const REMIT_COUNT: Symbol = symbol_short!("R_COUNT");

// Platform fee percentage (in basis points, e.g., 50 = 0.5%)
const FEE_RATE: i128 = 50; // 0.5%
const BASIS_POINTS: i128 = 10000;

#[contract]
pub struct MicroRemittanceContract;

#[contractimpl]
impl MicroRemittanceContract {

    // Initialize a new remittance transaction
    pub fn send_remittance(
        env: Env,
        sender: Address,
        receiver: Address,
        amount: i128
    ) -> u64 {
        // Authenticate sender
        sender.require_auth();

        // Validate amount
        if amount <= 0 {
            log!(&env, "Amount must be greater than zero");
            panic!("Invalid amount");
        }

        // Get and increment remittance counter
        let mut remit_count: u64 = env.storage().instance().get(&REMIT_COUNT).unwrap_or(0);
        remit_count += 1;

        // Get current timestamp
        let timestamp = env.ledger().timestamp();

        // Calculate platform fee
        let fee = (amount * FEE_RATE) / BASIS_POINTS;
        let net_amount = amount - fee;

        // Create remittance record
        let remittance = Remittance {
            sender: sender.clone(),
            receiver: receiver.clone(),
            amount: net_amount,
            timestamp,
            is_completed: false,
        };

        // Store remittance data
        env.storage().instance().set(
            &RemittanceBook::Transaction(remit_count),
            &remittance
        );

        // Update counter
        env.storage().instance().set(&REMIT_COUNT, &remit_count);

        // Extend storage TTL
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Remittance created with ID: {}", remit_count);
        log!(&env, "Net amount after fee: {}", net_amount);

        remit_count
    }

    // Complete a remittance (receiver claims funds)
    pub fn complete_remittance(env: Env, remit_id: u64, receiver: Address) {
        // Authenticate receiver
        receiver.require_auth();

        // Get remittance data
        let key = RemittanceBook::Transaction(remit_id);
        let remittance_opt: Option<Remittance> = env.storage().instance().get(&key);

        // Check if remittance exists
        if remittance_opt.is_none() {
            log!(&env, "Remittance not found");
            panic!("Remittance does not exist");
        }

        let mut remittance = remittance_opt.unwrap();

        // Validate receiver
        if remittance.receiver != receiver {
            log!(&env, "Unauthorized receiver");
            panic!("Only the designated receiver can complete this remittance");
        }

        // Validate not already completed
        if remittance.is_completed {
            log!(&env, "Remittance already completed");
            panic!("Remittance already completed");
        }

        // Mark as completed
        remittance.is_completed = true;

        // Update storage
        env.storage().instance().set(
            &RemittanceBook::Transaction(remit_id),
            &remittance
        );

        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Remittance ID: {} completed successfully", remit_id);
    }

    // View remittance details by ID (returns Option to handle non-existent entries)
    pub fn view_remittance(env: Env, remit_id: u64) -> Option<Remittance> {
        let key = RemittanceBook::Transaction(remit_id);
        env.storage().instance().get(&key)
    }

    // Get total number of remittances
    pub fn get_total_remittances(env: Env) -> u64 {
        env.storage().instance().get(&REMIT_COUNT).unwrap_or(0)
    }
}