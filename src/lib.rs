use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, AccountId};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, PartialEq, Eq)]
pub struct BridgeRequest {
    to_blockchain: String,
    to_token: String,
    to_address: String,
    from_token_address: Option<String>,
    from_amount_atom: String,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct HalfmoonCrossBridge {
    owner: AccountId,
    requests: LookupMap<AccountId, BridgeRequest>,
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
            requests: LookupMap::new(b"r".to_vec()),
        }
    }

    #[payable]
    pub fn add_bridge_request(
        &mut self,
        to_blockchain: String,
        to_token: String,
        to_address: String,
        from_token_address: Option<String>,
    ) {
        let account_id = env::predecessor_account_id();
        let mut request = BridgeRequest {
            to_token,
            to_address,
            to_blockchain,
            from_token_address,
            from_amount_atom: "".to_string(),
        };
        if request.from_token_address.is_some() {
            unimplemented!()
        } else {
            request.from_amount_atom = env::attached_deposit().to_string();
        }

        self.requests.insert(&account_id, &request);
    }

    pub fn get_request(&self, account_id: AccountId) -> Option<BridgeRequest> {
        return self.requests.get(&account_id);
    }
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
            "goNEAR".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
            None,
        );
        assert_eq!(
            Some(BridgeRequest {
                to_blockchain: "Algorand".to_string(),
                to_token: "goNEAR".to_string(),
                to_address: "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4"
                    .to_string(),
                from_token_address: None,
                from_amount_atom: "3000000000000000000000000".to_string(),
            }),
            contract.get_request(accounts(1))
        );
    }

    #[test]
    fn get_nonexistent_request() {
        let contract = HalfmoonCrossBridge::new(accounts(1));
        assert_eq!(
            None,
            contract.get_request("francis.near".parse().unwrap())
        );
    }
}
