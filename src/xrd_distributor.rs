use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct ClaimData {
  #[mutable]
  claimed: bool
}

#[blueprint]
mod token_giveaway {
    struct TokenGiveaway {
        minter_badge_vault: FungibleVault,
        vault: FungibleVault,
        claim_badge_manager: ResourceManager,
    }
  
    impl TokenGiveaway {
        pub fn instantiate_token_giveaway() -> Global<TokenGiveaway> {
            
            let minter_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(1);

            let tokens = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(1000);

            let claim_badge = 
                ResourceBuilder::new_ruid_non_fungible::<ClaimData>(OwnerRole::None)
                .mint_roles(mint_roles!(
                    minter => rule!(require(minter_badge.resource_address()));
                    minter_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(minter_badge.resource_address()));
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();
            
            Self {
                minter_badge_vault: FungibleVault::with_bucket(minter_badge),
                vault: FungibleVault::with_bucket(tokens),
                claim_badge_manager: claim_badge,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        
        }

        pub fn mint_claim_badge(&mut self) -> NonFungibleBucket {
            
            let claim_badge = self.minter_badge_vault.authorize_with_amount(1, || {
                self.claim_badge_manager
                    .mint_ruid_non_fungible(
                        ClaimData { claimed: false }
                    )
                }
            ).as_non_fungible();

            return claim_badge

        }

        pub fn claim_tokens(&mut self, claim_badge_proof: NonFungibleProof) -> FungibleBucket {
            
            let checked_proof = 
            claim_badge_proof.check_with_message(
                self.claim_badge_manager.address(), "Incorrect proof!"
            ); 
        
            let nft = checked_proof.non_fungible::<ClaimData>();
            let nft_data = nft.data();
            
            // Asserting that the claimed field does not have a value of true.
            assert_ne!(
              nft_data.claimed, true, 
              "You have already claimed your tokens"
            ); 
        
            self.minter_badge_vault.authorize_with_amount(1, || {  
                self.claim_badge_manager.update_non_fungible_data(
                    &nft.local_id(),
                    "xrd_claimed",
                    true
                )
            });
        
            return self.vault.take(10);
        }
    }
}