use scrypto::prelude::*;

#[blueprint]
mod token_sale {
    struct TokenSale {
        // Define what resources and data will be managed by TokenSale components
        useful_tokens_vault: FungibleVault,
        collected_xrd: FungibleVault,
        price_per_token: Decimal,
    }

    impl TokenSale {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate_token_sale(price_per_token: Decimal) -> Global<TokenSale> {

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let bucket_of_useful_tokens = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata! {
                    init {
                        "name" => "Useful Token", locked;
                        "symbol" => "USEFUL", locked;
                    }
                })
                .mint_initial_supply(100);
            
            Self{
                useful_tokens_vault: FungibleVault::with_bucket(bucket_of_useful_tokens),
                collected_xrd: FungibleVault::new(XRD),
                price_per_token: price_per_token,
            }.instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn buy_useful_tokens(&mut self, mut payment: FungibleBucket) -> (FungibleBucket, FungibleBucket) {

            self.collected_xrd.put(payment.take(self.price_per_token));

            (self.useful_tokens_vault.take(1), payment)
        }
    }
}