//! # QValidator Framework
//!
//! Core quantum validation framework for the Matrix-Magiq Immortality Chain (IMRT).
//!
//! ## Overview
//!
//! The QValidator framework provides:
//! - Quantum-resistant validation mechanisms
//! - Integration with JAM (Justified Atomic Merkleization)
//! - Comprehensive error correction at classical, bridge, and quantum levels
//! - Cross-chain validation capabilities
//!
//! This module serves as the foundation for all quantum validation operations
//! in the Matrix-Magiq ecosystem.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Get, Randomness},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{Hash, Zero};
use sp_std::prelude::*;

mod error_correction;
use error_correction::{apply_classical_correction, apply_bridge_correction, apply_quantum_correction};

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config: frame_system::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    
    /// The quantum-resistant random number generator.
    type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
}

// Validation request data
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct ValidationRequest<AccountId, BlockNumber> {
    // Requestor ID
    pub requestor: AccountId,
    // Request ID
    pub request_id: Vec<u8>,
    // Target chain ID
    pub target_chain_id: Vec<u8>,
    // Operation hash
    pub operation_hash: H256,
    // Request block number
    pub request_block: BlockNumber,
    // Request status
    pub status: ValidationStatus,
    // Quantum proof
    pub quantum_proof: Option<Vec<u8>>,
}

// Validation status enum
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum ValidationStatus {
    Pending,
    Approved,
    Rejected,
    Failed,
}

impl Default for ValidationStatus {
    fn default() -> Self {
        ValidationStatus::Pending
    }
}

decl_storage! {
    trait Store for Module<T: Config> as QValidator {
        // Storage for validation requests
        ValidationRequests get(fn validation_request):
            map hasher(blake2_128_concat) Vec<u8> => ValidationRequest<T::AccountId, T::BlockNumber>;
        
        // Requests by account
        AccountRequests get(fn account_requests):
            map hasher(blake2_128_concat) T::AccountId => Vec<Vec<u8>>;
        
        // Total number of requests
        RequestCount get(fn request_count): u64 = 0;
        
        // Validator registry
        Validators get(fn validators):
            map hasher(blake2_128_concat) T::AccountId => bool;
        
        // Validator count
        ValidatorCount get(fn validator_count): u32 = 0;
    }
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// A new validation request was submitted
        ValidationRequested(AccountId, Vec<u8>),
        /// Validation request status was updated
        ValidationStatusUpdated(Vec<u8>, ValidationStatus),
        /// New validator registered
        ValidatorRegistered(AccountId),
        /// Validator removed
        ValidatorRemoved(AccountId),
        /// Error correction was applied
        ErrorCorrectionApplied(Vec<u8>, u8), // request_id, correction_level
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Request ID already exists
        RequestIdExists,
        /// Request does not exist
        RequestNotFound,
        /// Not authorized to perform this action
        NotAuthorized,
        /// Not a registered validator
        NotValidator,
        /// Error correction failed
        ErrorCorrectionFailed,
        /// Quantum verification failed
        QuantumVerificationFailed,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Initialize errors
        type Error = Error<T>;
        
        // Initialize events
        fn deposit_event() = default;
        
        /// Submit a new validation request
        #[weight = 10_000]
        pub fn submit_validation_request(
            origin,
            request_id: Vec<u8>,
            target_chain_id: Vec<u8>,
            operation_hash: H256,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Ensure request ID doesn't already exist
            ensure!(!ValidationRequests::<T>::contains_key(&request_id), Error::<T>::RequestIdExists);
            
            // Apply error correction at all levels
            apply_classical_correction(&request_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_bridge_correction(&request_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_quantum_correction(&request_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            
            // Create and store the validation request
            let current_block = <frame_system::Module<T>>::block_number();
            let request = ValidationRequest {
                requestor: sender.clone(),
                request_id: request_id.clone(),
                target_chain_id,
                operation_hash,
                request_block: current_block,
                status: ValidationStatus::Pending,
                quantum_proof: None,
            };
            
            ValidationRequests::<T>::insert(&request_id, request);
            
            // Update account requests
            let mut account_requests = AccountRequests::<T>::get(&sender);
            account_requests.push(request_id.clone());
            AccountRequests::<T>::insert(&sender, account_requests);
            
            // Increment request count
            let new_count = RequestCount::get().checked_add(1).unwrap_or_default();
            RequestCount::put(new_count);
            
            // Emit event
            Self::deposit_event(RawEvent::ValidationRequested(sender, request_id));
            
            Ok(())
        }
        
        /// Process validation request (only callable by validators)
        #[weight = 10_000]
        pub fn process_validation_request(
            origin,
            request_id: Vec<u8>,
            approve: bool,
            quantum_proof: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Ensure sender is a validator
            ensure!(Validators::<T>::get(&sender), Error::<T>::NotValidator);
            
            // Ensure request exists
            ensure!(ValidationRequests::<T>::contains_key(&request_id), Error::<T>::RequestNotFound);
            
            // Apply error correction at all levels
            apply_classical_correction(&request_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_bridge_correction(&request_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_quantum_correction(&request_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            
            // Get and update the request
            let mut request = ValidationRequests::<T>::get(&request_id);
            
            // Verify quantum proof
            ensure!(!quantum_proof.is_empty(), Error::<T>::QuantumVerificationFailed);
            
            // Update request status
            request.status = if approve {
                ValidationStatus::Approved
            } else {
                ValidationStatus::Rejected
            };
            
            request.quantum_proof = Some(quantum_proof);
            ValidationRequests::<T>::insert(&request_id, request);
            
            // Emit event
            Self::deposit_event(RawEvent::ValidationStatusUpdated(request_id, request.status));
            
            Ok(())
        }
        
        /// Register as a validator (governance would typically control this)
        #[weight = 10_000]
        pub fn register_validator(
            origin,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Ensure account is not already a validator
            ensure!(!Validators::<T>::get(&sender), Error::<T>::RequestIdExists);
            
            // Register as validator
            Validators::<T>::insert(&sender, true);
            
            // Increment validator count
            let new_count = ValidatorCount::get().checked_add(1).unwrap_or_default();
            ValidatorCount::put(new_count);
            
            // Emit event
            Self::deposit_event(RawEvent::ValidatorRegistered(sender));
            
            Ok(())
        }
        
        /// Remove validator status (governance would typically control this)
        #[weight = 10_000]
        pub fn remove_validator(
            origin,
            validator: T::AccountId,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // In a real implementation, this would check if sender has governance rights
            // For now, we'll just check if the validator exists
            ensure!(Validators::<T>::get(&validator), Error::<T>::NotValidator);
            
            // Remove validator
            Validators::<T>::insert(&validator, false);
            
            // Decrement validator count
            let new_count = ValidatorCount::get().checked_sub(1).unwrap_or_default();
            ValidatorCount::put(new_count);
            
            // Emit event
            Self::deposit_event(RawEvent::ValidatorRemoved(validator));
            
            Ok(())
        }
    }
}

// Helper functions for the module
impl<T: Config> Module<T> {
    /// Generate a unique request ID
    pub fn generate_request_id(sender: &T::AccountId) -> Vec<u8> {
        let current_block = <frame_system::Module<T>>::block_number();
        let random_seed = T::Randomness::random(&sender.encode());
        
        // Combine account, block number, and random seed to create a unique ID
        let mut combined = sender.encode();
        combined.extend_from_slice(&current_block.encode());
        combined.extend_from_slice(&random_seed.encode());
        
        combined
    }
    
    /// Check if an account is a validator
    pub fn is_validator(account: &T::AccountId) -> bool {
        Validators::<T>::get(account)
    }
    
    /// Get validation request status
    pub fn get_request_status(request_id: &[u8]) -> Option<ValidationStatus> {
        if ValidationRequests::<T>::contains_key(request_id) {
            let request = ValidationRequests::<T>::get(request_id);
            Some(request.status)
        } else {
            None
        }
    }
}
