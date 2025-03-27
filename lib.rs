#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod linkedin {
    use ink::storage::Mapping;

    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CredentialType {
        WorkExperience,
        Education,
        Certification,
        ProjectContribution,
        SkillEndorsement,
    }

    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Credential {
        issued_to: AccountId,
        issued_by: AccountId,
        credential_type: CredentialType,
        metadata: String,
        timestamp: u64,
    }

    #[ink(storage)]
    pub struct CredentialSystem {
        credentials: Mapping<AccountId, Vec<Credential>>,
        // (user, issuer, type) => metadata
        requests: Mapping<(AccountId, AccountId, CredentialType), String>,
    }

    impl CredentialSystem {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                credentials: Mapping::default(),
                requests: Mapping::default(),
            }
        }

        /// Request a credential from a specific issuer
        #[ink(message, payable)]
        pub fn request_credential(
            &mut self,
            issuer: AccountId,
            cred_type: CredentialType,
            metadata: String,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let fee = self.env().transferred_value();

            if !self.has_identity(caller) || !self.has_identity(issuer) {
                return Err(Error::IdentityRequired);
            }

            if fee < 2_000_000_000_000 {
                return Err(Error::InsufficientFee); // example fee
            }

            self.requests.insert((caller, issuer, cred_type.clone()), &metadata);

            // Send fee directly to issuer
            self.env().transfer(issuer, fee).map_err(|_| Error::TransferFailed)?;

            Ok(())
        }

        #[ink(message)]
        pub fn issue_credential(&mut self, user: AccountId, cred_type: CredentialType) -> Result<(), Error> {
            let caller = self.env().caller(); // issuer
            if !self.has_identity(caller) || !self.has_identity(user) {
                return Err(Error::IdentityRequired);
            }

            let key = (user, caller, cred_type.clone());
            let metadata = self.requests.get(&key).ok_or(Error::NoRequest)?;

            let credential = Credential {
                issued_to: user,
                issued_by: caller,
                credential_type: cred_type,
                metadata,
                timestamp: self.env().block_timestamp(),
            };

            let mut user_creds = self.credentials.get(user).unwrap_or_default();
            user_creds.push(credential);
            self.credentials.insert(user, &user_creds);

            // Clean up the request
            self.requests.remove(&key);

            Ok(())
        }

        #[ink(message)]
        pub fn get_credentials(&self, user: AccountId) -> Vec<Credential> {
            self.credentials.get(user).unwrap_or_default()
        }

        fn has_identity(&self, _account: AccountId) -> bool {
            // Stub: real check should use chain extension to identity pallet
            true
        }
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        IdentityRequired,
        NoRequest,
        InsufficientFee,
        TransferFailed,
    }
}