use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, AccountId};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, PartialEq, Eq)]
pub enum RequestStatus {
    Created,
    Doing { to_txn_hash: String },
    Error { to_txn_hash: String, error: String },
    Done { to_txn_hash: String },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, PartialEq, Eq)]
pub struct BridgeRequest {
    to_blockchain: String,
    to_token: String,
    to_address: String,
    from_token_address: Option<String>,
    from_amount_atom: String,
    status: RequestStatus,
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
            status: RequestStatus::Created,
        };
        if request.from_token_address.is_some() {
            unimplemented!()
        } else {
            request.from_amount_atom = env::attached_deposit().to_string();
        }
        if let Some(existing_request) = self.requests.get(&account_id) {
            match existing_request.status {
                RequestStatus::Error { .. } | RequestStatus::Done { .. } => {}
                _ => env::panic_str("unfinished request"),
            }
        }
        self.requests.insert(&account_id, &request);
    }

    pub fn set_request_doing(&mut self, account_id: AccountId, to_txn_hash: String) {
        let predecessor_account_id = env::predecessor_account_id();
        if predecessor_account_id != self.owner {
            env::panic_str("only allowed by owner");
        }

        if let Some(mut existing_request) = self.requests.get(&account_id) {
            match existing_request.status {
                RequestStatus::Created => {
                    existing_request.status = RequestStatus::Doing { to_txn_hash };
                    self.requests.insert(&account_id, &existing_request);
                }
                _ => env::panic_str("expect request to be Created"),
            }
        } else {
            env::panic_str("expect request exist");
        }
    }

    pub fn set_request_done(&mut self, account_id: AccountId) {
        let predecessor_account_id = env::predecessor_account_id();
        if predecessor_account_id != self.owner {
            env::panic_str("only allowed by owner");
        }

        if let Some(mut existing_request) = self.requests.get(&account_id) {
            match existing_request.status {
                RequestStatus::Doing { to_txn_hash } => {
                    existing_request.status = RequestStatus::Done { to_txn_hash };
                    self.requests.insert(&account_id, &existing_request);
                }
                _ => env::panic_str("expect request to be Doing"),
            }
        } else {
            env::panic_str("expect request exist");
        }
    }

    pub fn set_request_error(&mut self, account_id: AccountId, error: String) {
        let predecessor_account_id = env::predecessor_account_id();
        if predecessor_account_id != self.owner {
            env::panic_str("only allowed by owner");
        }

        if let Some(mut existing_request) = self.requests.get(&account_id) {
            match existing_request.status {
                RequestStatus::Doing { to_txn_hash } => {
                    existing_request.status = RequestStatus::Error { to_txn_hash, error };
                    self.requests.insert(&account_id, &existing_request);
                }
                _ => env::panic_str("expect request to be Doing"),
            }
        } else {
            env::panic_str("expect request exist");
        }
    }

    pub fn get_request_status(&self, account_id: AccountId) -> Option<BridgeRequest> {
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
        testing_env!(context.predecessor_account_id(accounts(1)).build());

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
                from_amount_atom: "0".to_string(),
                status: RequestStatus::Created
            }),
            contract.get_request_status(accounts(1))
        );
    }

    #[test]
    #[should_panic(expected = r#"unfinished request"#)]
    fn cannot_set_request_when_pending() {
        let mut context = get_context(accounts(1));
        // Initialize the mocked blockchain
        testing_env!(context.build());

        // Set the testing environment for the subsequent calls
        testing_env!(context.predecessor_account_id(accounts(1)).build());

        let mut contract = HalfmoonCrossBridge::new(accounts(1));
        contract.add_bridge_request(
            "Algorand".to_string(),
            "goNEAR".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
            None,
        );
        contract.add_bridge_request(
            "Algorand".to_string(),
            "goNEAR".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
            None,
        );
    }

    #[test]
    fn owner_set_request_doing() {
        let mut context = get_context(accounts(1));
        // Initialize the mocked blockchain
        testing_env!(context.build());

        // Set the testing environment for the subsequent calls
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        let mut contract = HalfmoonCrossBridge::new(accounts(0));

        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.add_bridge_request(
            "Algorand".to_string(),
            "goNEAR".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
            None,
        );

        testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.set_request_doing(
            accounts(1),
            "TUBMGC74FMITVZOZHI5ASE4AQYDRNHOTJGTO7WKKQDRTA72Q5ISQ".to_string(),
        );
        assert_eq!(
            Some(BridgeRequest {
                to_blockchain: "Algorand".to_string(),
                to_token: "goNEAR".to_string(),
                to_address: "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4"
                    .to_string(),
                from_token_address: None,
                from_amount_atom: "0".to_string(),
                status: RequestStatus::Doing {
                    to_txn_hash: "TUBMGC74FMITVZOZHI5ASE4AQYDRNHOTJGTO7WKKQDRTA72Q5ISQ".to_string()
                }
            }),
            contract.get_request_status(accounts(1))
        );
    }

    #[test]
    fn owner_set_doing_error() {
        let mut context = get_context(accounts(1));
        // Initialize the mocked blockchain
        testing_env!(context.build());

        // Set the testing environment for the subsequent calls
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        let mut contract = HalfmoonCrossBridge::new(accounts(0));

        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.add_bridge_request(
            "Algorand".to_string(),
            "goNEAR".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
            None,
        );

        testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.set_request_doing(
            accounts(1),
            "TUBMGC74FMITVZOZHI5ASE4AQYDRNHOTJGTO7WKKQDRTA72Q5ISQ".to_string(),
        );
        contract.set_request_error(
            accounts(1),
            "network error".to_string(),
        );
        assert_eq!(
            Some(BridgeRequest {
                to_blockchain: "Algorand".to_string(),
                to_token: "goNEAR".to_string(),
                to_address: "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4"
                    .to_string(),
                from_token_address: None,
                from_amount_atom: "0".to_string(),
                status: RequestStatus::Error {
                    to_txn_hash: "TUBMGC74FMITVZOZHI5ASE4AQYDRNHOTJGTO7WKKQDRTA72Q5ISQ".to_string(),
                    error: "network error".to_string()
                }
            }),
            contract.get_request_status(accounts(1))
        );
    }

    #[test]
    fn owner_set_doing_done() {
        let mut context = get_context(accounts(1));
        // Initialize the mocked blockchain
        testing_env!(context.build());

        // Set the testing environment for the subsequent calls
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        let mut contract = HalfmoonCrossBridge::new(accounts(0));

        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.add_bridge_request(
            "Algorand".to_string(),
            "goNEAR".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
            None,
        );

        testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.set_request_doing(
            accounts(1),
            "TUBMGC74FMITVZOZHI5ASE4AQYDRNHOTJGTO7WKKQDRTA72Q5ISQ".to_string(),
        );
        contract.set_request_done(
            accounts(1),
        );
        assert_eq!(
            Some(BridgeRequest {
                to_blockchain: "Algorand".to_string(),
                to_token: "goNEAR".to_string(),
                to_address: "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4"
                    .to_string(),
                from_token_address: None,
                from_amount_atom: "0".to_string(),
                status: RequestStatus::Done {
                    to_txn_hash: "TUBMGC74FMITVZOZHI5ASE4AQYDRNHOTJGTO7WKKQDRTA72Q5ISQ".to_string(),
                }
            }),
            contract.get_request_status(accounts(1))
        );
    }

    #[test]
    #[should_panic(expected = r#"only allowed by owner"#)]
    fn non_owner_update_status_fail() {
        let mut context = get_context(accounts(1));
        // Initialize the mocked blockchain
        testing_env!(context.build());

        // Set the testing environment for the subsequent calls
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        let mut contract = HalfmoonCrossBridge::new(accounts(0));

        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.add_bridge_request(
            "Algorand".to_string(),
            "goNEAR".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
            None,
        );

        // testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.set_request_doing(
            accounts(1),
            "TUBMGC74FMITVZOZHI5ASE4AQYDRNHOTJGTO7WKKQDRTA72Q5ISQ".to_string(),
        );
    }

    #[test]
    #[should_panic(expected = r#"expect request to be Doing"#)]
    fn non_predefined_status_transition_fail() {
        let mut context = get_context(accounts(1));
        // Initialize the mocked blockchain
        testing_env!(context.build());

        // Set the testing environment for the subsequent calls
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        let mut contract = HalfmoonCrossBridge::new(accounts(0));

        testing_env!(context.predecessor_account_id(accounts(1)).build());
        contract.add_bridge_request(
            "Algorand".to_string(),
            "goNEAR".to_string(),
            "4IZRTUO72JY5WH4HKLVDQSKIVF2VSRQX7IFVI3KEOQHHNCQUXCMYPZH7J4".to_string(),
            None,
        );

        testing_env!(context.predecessor_account_id(accounts(0)).build());
        contract.set_request_done(
            accounts(1),
        );
    }


    #[test]
    fn get_nonexistent_request() {
        let contract = HalfmoonCrossBridge::new(accounts(1));
        assert_eq!(
            None,
            contract.get_request_status("francis.near".parse().unwrap())
        );
    }
}
