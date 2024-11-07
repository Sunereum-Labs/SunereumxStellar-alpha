use bitcoin_splitter::split::script::{IOPair, SplitableScript};
use soroban_sdk::{contractimpl, Address, Env, String};
use std::time::{SystemTime, UNIX_EPOCH};

// Represents the oracle's signed message format for inverter status
#[derive(Clone)]
struct InverterStatusOracle {
    device_id: String,
    timestamp: u64,
    offline_duration: u64,
    oracle_signature: [u8; 64],
}

// BitVM verification for oracle signatures
struct InverterDowntimeVerifier;

impl SplitableScript<9, 3> for InverterDowntimeVerifier {
    fn script() -> Script {
        script! {
            // Verify oracle signature
            { InverterDowntimeVerifier::OP_VERIFY_ORACLE_SIG }
            // Verify downtime threshold
            { InverterDowntimeVerifier::OP_VERIFY_DOWNTIME }
            // Process payout if conditions met
            { InverterDowntimeVerifier::OP_PROCESS_PAYOUT }
        }
    }

    fn OP_VERIFY_ORACLE_SIG() -> Script {
        script! {
            // Load oracle public key
            { ORACLE_PUBKEY }
            // Load signature and message
            { 2 } OP_PICK  // signature
            { 1 } OP_PICK  // message
            // Verify signature
            OP_CHECKSIG
            OP_VERIFY
        }
    }

    fn OP_VERIFY_DOWNTIME() -> Script {
        script! {
            // Load downtime duration
            { 1 } OP_PICK
            // Compare with threshold (4 hours = 14400 seconds)
            { 14400 }
            OP_GREATERTHAN
            OP_VERIFY
        }
    }

    fn OP_PROCESS_PAYOUT() -> Script {
        script! {
            // If all verifications pass, approve payout
            { 1 }
        }
    }
}

// Extension to Virya Integration Contract to handle BitVM oracle duties
#[contractimpl]
impl ViryaIntegrationContract {
    // New method to generate oracle signature for BitVM
    pub fn generate_downtime_proof(
        env: Env,
        device_id: String,
        auth_provider: Address,
    ) -> Result<InverterStatusOracle, String> {
        // Verify authorized provider
        let authorized: Vec<Address> = env.storage().instance().get(&AUTHORIZED_PROVIDERS).unwrap_or(Vec::new(&env));
        if !authorized.contains(&auth_provider) {
            return Err(String::from_str(&env, "Unauthorized provider"));
        }

        let devices: Map<String, IoTDevice> = env.storage().instance().get(&DEVICES).unwrap_or(Map::new(&env));
        
        if let Some(device) = devices.get(device_id.clone()) {
            let current_time = env.ledger().timestamp();
            
            if !device.online_status {
                let offline_duration = current_time - device.last_ping;
                
                // Generate oracle signature
                let msg = format!("{}:{}:{}", device_id, current_time, offline_duration);
                let signature = env.crypto().sign_ed25519(&msg.as_bytes());

                Ok(InverterStatusOracle {
                    device_id: device_id,
                    timestamp: current_time,
                    offline_duration,
                    oracle_signature: signature,
                })
            } else {
                Err(String::from_str(&env, "Device is online"))
            }
        } else {
            Err(String::from_str(&env, "Device not found"))
        }
    }
}

// Example of BitVM reinsurance contract consuming oracle data
fn process_reinsurance_claim(
    oracle_data: InverterStatusOracle,
    policy_data: ReinsurancePolicy,
) -> bool {
    let verifier = InverterDowntimeVerifier::new();
    
    // Prepare input for BitVM verification
    let input = script! {
        // Oracle signature
        { oracle_data.oracle_signature }
        // Timestamp
        { oracle_data.timestamp }
        // Offline duration
        { oracle_data.offline_duration }
        // Policy ID
        { policy_data.policy_id }
    };

    // Run verification
    let result = verifier.verify(input);
    
    // If verification passes, process payout
    if result {
        // Calculate payout amount based on policy terms
        let payout_amount = calculate_payout(
            policy_data.coverage_amount,
            oracle_data.offline_duration
        );
        
        // Trigger BTC transaction for payout
        process_btc_payout(
            policy_data.beneficiary_address,
            payout_amount
        );
        
        true
    } else {
        false
    }
}

// Example usage and test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverter_downtime_claim() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ViryaIntegrationContract);
        let client = ViryaIntegrationContractClient::new(&env, &contract_id);

        // Initialize contract
        client.initialize();

        // Register test inverter
        let device_id = String::from_str(&env, "TEST_INV_001");
        let manufacturer = String::from_str(&env, "SolarEdge");
        let model = String::from_str(&env, "SE7600H");
        client.register_inverter(
            &device_id,
            &String::from_str(&env, "POL001"),
            &manufacturer,
            &model,
            &7600,
        );

        // Simulate inverter going offline
        let provider = Address::random(&env);
        client.authorize_provider(&provider);

        // Set device offline in Soroban contract
        client.update_inverter_data(
            &device_id,
            &provider,
            0,      // No energy production
            0,      // No power
            0,      // All metrics zero
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        );

        // Wait simulated time (4 hours)
        env.ledger().set_timestamp(env.ledger().timestamp() + 14400);

        // Generate oracle proof
        let oracle_data = client.generate_downtime_proof(&device_id, &provider).unwrap();

        // Create test policy data
        let policy_data = ReinsurancePolicy {
            policy_id: "POL001".to_string(),
            coverage_amount: 1_000_000, // 1 BTC in sats
            beneficiary_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
        };

        // Process claim
        let claim_result = process_reinsurance_claim(oracle_data, policy_data);
        assert!(claim_result, "Claim should be approved");
    }
}
