use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct HalfmoonCrossBridge {
    owner: AccountId,
}

impl Default for HalfmoonCrossBridge {
    fn default() -> Self {
        panic!("HalfmoonCrossBridge should be initialized before usage")
    }
}

#[near_bindgen]
impl HalfmoonCrossBridge {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Owner's account ID is invalid."
        );
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: owner_id,
        }
    }

    #[payable]
    pub fn add_bridge_request(
        &mut self,
        to_blockchain: String,
        to_addr: String,
    ) {}
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    // Allows for modifying the environment of the mocked blockchain
    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn add_get_bridge_request() {
        let mut context = get_context(accounts(1));
        // Initialize the mocked blockchain
        testing_env!(context.build());

        // Set the testing environment for the subsequent calls
        testing_env!(context.predecessor_account_id(accounts(1)).attached_deposit(3 * 10u128.pow(24)).
        build());

        let mut contract = HalfmoonCrossBridge::new(accounts(1));
        contract.add_bridge_request(
            "Algorand".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
        );
    }
}
