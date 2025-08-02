use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use sha2::{Digest, Sha256};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ZKProof {
    pub proof_id: String,
    pub proof_data: String, // Base64 encoded proof
    pub public_inputs: Vec<String>,
    pub verification_key: String,
    pub submitter: AccountId,
    pub verified: bool,
    pub submitted_at: u64,
    pub verified_at: Option<u64>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct VerificationResult {
    pub proof_id: String,
    pub is_valid: bool,
    pub verified_at: u64,
    pub verifier: AccountId,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ZKPVerifierContract {
    proofs: LookupMap<String, ZKProof>,
    verification_results: LookupMap<String, VerificationResult>,
    authorized_verifiers: LookupMap<AccountId, bool>,
    owner: AccountId,
}

#[near_bindgen]
impl ZKPVerifierContract {
    #[init]
    pub fn new() -> Self {
        let owner = env::predecessor_account_id();
        let mut authorized_verifiers = LookupMap::new(b"v");
        authorized_verifiers.insert(&owner, &true);
        
        Self {
            proofs: LookupMap::new(b"p"),
            verification_results: LookupMap::new(b"r"),
            authorized_verifiers,
            owner,
        }
    }

    pub fn submit_proof(
        &mut self,
        proof_id: String,
        proof_data: String,
        public_inputs: Vec<String>,
        verification_key: String,
    ) {
        let submitter = env::predecessor_account_id();
        let current_time = env::block_timestamp();
        
        // Ensure proof ID is unique
        assert!(!self.proofs.contains_key(&proof_id), "Proof ID already exists");
        
        let proof = ZKProof {
            proof_id: proof_id.clone(),
            proof_data,
            public_inputs,
            verification_key,
            submitter,
            verified: false,
            submitted_at: current_time,
            verified_at: None,
        };
        
        self.proofs.insert(&proof_id, &proof);
    }

    pub fn verify_proof(&mut self, proof_id: String, is_valid: bool) {
        let verifier = env::predecessor_account_id();
        
        // Check if verifier is authorized
        assert!(
            self.authorized_verifiers.get(&verifier).unwrap_or(false),
            "Not authorized to verify proofs"
        );
        
        // Get the proof
        let mut proof = self.proofs.get(&proof_id).expect("Proof not found");
        
        // Update proof verification status
        proof.verified = true;
        proof.verified_at = Some(env::block_timestamp());
        self.proofs.insert(&proof_id, &proof);
        
        // Store verification result
        let result = VerificationResult {
            proof_id: proof_id.clone(),
            is_valid,
            verified_at: env::block_timestamp(),
            verifier,
        };
        
        self.verification_results.insert(&proof_id, &result);
    }

    pub fn get_proof(&self, proof_id: String) -> Option<ZKProof> {
        self.proofs.get(&proof_id)
    }

    pub fn get_verification_result(&self, proof_id: String) -> Option<VerificationResult> {
        self.verification_results.get(&proof_id)
    }

    pub fn is_proof_valid(&self, proof_id: String) -> Option<bool> {
        self.verification_results.get(&proof_id).map(|result| result.is_valid)
    }

    // Simple hash-based proof verification (placeholder for actual ZKP verification)
    pub fn verify_simple_hash_proof(&mut self, proof_id: String, secret: String, expected_hash: String) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        let result = hasher.finalize();
        let computed_hash = base64::encode(result);
        
        let is_valid = computed_hash == expected_hash;
        
        // Auto-verify this proof
        if let Some(mut proof) = self.proofs.get(&proof_id) {
            proof.verified = true;
            proof.verified_at = Some(env::block_timestamp());
            self.proofs.insert(&proof_id, &proof);
            
            let result = VerificationResult {
                proof_id: proof_id.clone(),
                is_valid,
                verified_at: env::block_timestamp(),
                verifier: env::current_account_id(),
            };
            
            self.verification_results.insert(&proof_id, &result);
        }
        
        is_valid
    }

    // Owner functions
    pub fn add_authorized_verifier(&mut self, verifier: AccountId) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner can add verifiers");
        self.authorized_verifiers.insert(&verifier, &true);
    }

    pub fn remove_authorized_verifier(&mut self, verifier: AccountId) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner can remove verifiers");
        self.authorized_verifiers.remove(&verifier);
    }

    pub fn is_authorized_verifier(&self, verifier: AccountId) -> bool {
        self.authorized_verifiers.get(&verifier).unwrap_or(false)
    }
}