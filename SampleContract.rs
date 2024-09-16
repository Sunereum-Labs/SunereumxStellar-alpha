#![no_std]
use soroban_sdk::{contractimpl, symbol_short, Address, Env, String, Symbol, Vec};

#[derive(Clone)]
pub struct InsurancePolicy {
    policy_id: String,
    policy_holder: Address,
    premium: u32,
    coverage_amount: u32,
    is_active: bool,
}

const POLICIES: Symbol = symbol_short!("POLICIES");
const RISK_LEVEL: Symbol = symbol_short!("RISK_LEVEL");

pub struct InsuranceContract;

#[contractimpl]
impl InsuranceContract {
    pub fn initialize(env: Env) {
        env.storage().instance().set(&POLICIES, &Vec::new(&env));
        env.storage().instance().set(&RISK_LEVEL, &String::from_str(&env, "LOW"));
    }

    pub fn create_policy(env: Env, policy_holder: Address, premium: u32, coverage_amount: u32) -> String {
        let policy_id = generate_policy_id(&env);
        let policy = InsurancePolicy {
            policy_id: policy_id.clone(),
            policy_holder,
            premium,
            coverage_amount,
            is_active: true,
        };
        
        let mut policies: Vec<InsurancePolicy> = env.storage().instance().get(&POLICIES).unwrap_or(Vec::new(&env));
        policies.push_back(policy);
        env.storage().instance().set(&POLICIES, &policies);
        
        policy_id
    }

    pub fn get_policy(env: Env, policy_id: String) -> Option<InsurancePolicy> {
        let policies: Vec<InsurancePolicy> = env.storage().instance().get(&POLICIES).unwrap_or(Vec::new(&env));
        policies.iter().find(|p| p.policy_id == policy_id).cloned()
    }

    pub fn update_risk_level(env: Env, new_risk_level: String) {
        env.storage().instance().set(&RISK_LEVEL, &new_risk_level);
    }

    pub fn get_risk_level(env: Env) -> String {
        env.storage().instance().get(&RISK_LEVEL).unwrap()
    }

    pub fn file_claim(env: Env, policy_id: String, claim_amount: u32) -> bool {
        let mut policies: Vec<InsurancePolicy> = env.storage().instance().get(&POLICIES).unwrap_or(Vec::new(&env));
        if let Some(index) = policies.iter().position(|p| p.policy_id == policy_id) {
            let mut policy = policies.get(index).unwrap().clone();
            if policy.is_active && claim_amount <= policy.coverage_amount {
                policy.is_active = false;
                policies.set(index, policy);
                env.storage().instance().set(&POLICIES, &policies);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

fn generate_policy_id(env: &Env) -> String {
    let random_bytes = env.crypto().random_bytes(16);
    String::from_slice(env, &random_bytes)
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, Address};

    #[test]
    fn test_insurance_contract() {
        let env = Env::default();
        let contract_id = env.register_contract(None, InsuranceContract);
        let client = InsuranceContractClient::new(&env, &contract_id);

        client.initialize();

        let policy_holder = Address::random(&env);
        let policy_id = client.create_policy(&policy_holder, &1000, &10000);

        let policy = client.get_policy(&policy_id).unwrap();
        assert_eq!(policy.policy_holder, policy_holder);
        assert_eq!(policy.premium, 1000);
        assert_eq!(policy.coverage_amount, 10000);
        assert_eq!(policy.is_active, true);

        client.update_risk_level(&String::from_str(&env, "MEDIUM"));
        assert_eq!(client.get_risk_level(), String::from_str(&env, "MEDIUM"));

        let claim_result = client.file_claim(&policy_id, &5000);
        assert_eq!(claim_result, true);

        let updated_policy = client.get_policy(&policy_id).unwrap();
        assert_eq!(updated_policy.is_active, false);
    }
}
