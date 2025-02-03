#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod trustbridge_contract {
    use ink::storage::Mapping;

    // Core storage for managing multiple escrows
    #[ink(storage)]
    pub struct TrustbridgeContract {
        escrows: Mapping<u32, EscrowDetails>,
        next_escrow_id: u32,
        admin: AccountId,
    }

    // Details of a single escrow transaction
    #[derive(scale::Decode, scale::Encode, Clone)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq, scale_info::TypeInfo))]
    pub struct EscrowDetails {
        amount: Balance,
        owner: AccountId,
        beneficiary: AccountId,
        arbiter: AccountId,
        is_active: bool,
    }

    // Events emitted during key operations
    #[ink(event)]
    pub struct EscrowCreated {
        #[ink(topic)]
        escrow_id: u32,
        amount: Balance,
    }

    #[ink(event)]
    pub struct FundsReleased {
        #[ink(topic)]
        escrow_id: u32,
        amount: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientFunds,
        NotAuthorized,
        EscrowNotFound,
        EscrowNotActive,
    }

    impl TrustbridgeContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                escrows: Mapping::new(),
                next_escrow_id: 0,
                admin: Self::env().caller(),
            }
        }

        // Main function to create and fund a new escrow
        #[ink(message, payable)]
        pub fn create_escrow(
            &mut self,
            beneficiary: AccountId,
            arbiter: AccountId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();
            let escrow_id = self.next_escrow_id;

            let escrow = EscrowDetails {
                amount,
                owner: caller,
                beneficiary,
                arbiter,
                is_active: true,
            };

            self.escrows.insert(escrow_id, &escrow);
            self.next_escrow_id += 1;
            self.env().emit_event(EscrowCreated { escrow_id, amount });
            Ok(())
        }

        // Function for arbiter to release funds to beneficiary
        #[ink(message)]
        pub fn release_funds(&mut self, escrow_id: u32) -> Result<(), Error> {
            let escrow = self.escrows.get(&escrow_id).ok_or(Error::EscrowNotFound)?;

            if !escrow.is_active || self.env().caller() != escrow.arbiter {
                return Err(Error::NotAuthorized);
            }

            self.env()
                .transfer(escrow.beneficiary, escrow.amount)
                .map_err(|_| Error::InsufficientFunds)?;

            let mut updated_escrow = escrow.clone();
            updated_escrow.is_active = false;
            self.escrows.insert(escrow_id, &updated_escrow);

            self.env().emit_event(FundsReleased {
                escrow_id,
                amount: escrow.amount,
            });
            Ok(())
        }

        // Query function to check escrow status
        #[ink(message)]
        pub fn get_escrow(&self, escrow_id: u32) -> Option<EscrowDetails> {
            self.escrows.get(&escrow_id)
        }
    }
}
