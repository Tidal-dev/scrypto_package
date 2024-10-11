use scrypto::prelude::*;

#[blueprint]
mod radiswap{
    struct Radiswap{
        vault_a : FungibleVault,
        vault_b : FungibleVault,
        pool_units_resource_manager : ResourceManager,
        fee: Decimal,
    }

    impl Radiswap{
        pub fn instantiate_radiswap(
            bucket_a : FungibleBucket,
            bucket_b : FungibleBucket,
            fee : Decimal,
        ) -> (Global<Radiswap>, FungibleBucket) {
        
            assert!(
                !bucket_a.is_empty() && !bucket_b.is_empty(),
                "You must pass in an initial supply of each token"
            );
            assert!(
                fee >= dec!("0") && fee <= dec!("1"),
                "Invalid fee in thousandths"
            );

            let(address_reservation, component_address) = 
                Runtime::allocate_component_address(Radiswap::blueprint_id());
            
            let pool_units: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Pool Units", locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                ))
                .mint_initial_supply(100);


            let radiswap = Self{
                vault_a: FungibleVault::with_bucket(bucket_a),
                vault_b: FungibleVault::with_bucket(bucket_b),
                pool_units_resource_manager: pool_units.resource_manager(),
                fee: fee,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .globalize();

            (radiswap, pool_units)
        }

        pub fn swap(&mut self, input_tokens: FungibleBucket) -> FungibleBucket {
            let(input_tokens_vault, output_tokens_vault): (&mut FungibleVault, &mut FungibleVault) = 
                if input_tokens.resource_address() == self.vault_a.resource_address() {
                    (&mut self.vault_a, &mut self.vault_b)
                } else if input_tokens.resource_address() == self.vault_b.resource_address() {
                    (&mut self.vault_b, &mut self.vault_a)
                } else {
                    panic!("Invalid input token")
                };

            let output_amount: Decimal = (output_tokens_vault.amount() 
            * (dec!("1") - self.fee) 
            * input_tokens.amount()) 
            / (input_tokens_vault.amount() + (dec!("1") - self.fee) 
            * input_tokens.amount());

            input_tokens_vault.put(input_tokens);
            output_tokens_vault.take(output_amount)
        }

        pub fn add_liquidity(&mut self, bucket_a: FungibleBucket, bucket_b: FungibleBucket) -> (FungibleBucket, FungibleBucket, FungibleBucket){
            let (mut bucket_a, mut bucket_b) : (FungibleBucket, FungibleBucket) = 
            if bucket_a.resource_address() == self.vault_a.resource_address()  
                && bucket_b.resource_address() == self.vault_b.resource_address() {
                (bucket_a, bucket_b)
            } else if bucket_a.resource_address() == self.vault_b.resource_address()  
                && bucket_b.resource_address() == self.vault_a.resource_address() {
                (bucket_b, bucket_a)
            } else {
                panic!("Invalid input tokens")
            };

            // Getting the values of `dm` and `dn` based on the sorted buckets
            let dm: Decimal = bucket_a.amount();
            let dn: Decimal = bucket_b.amount();

            // Getting the values of m and n from the liquidity pool vaults
            let m: Decimal = self.vault_a.amount();
            let n: Decimal = self.vault_b.amount();
            
            let vault_ratio: Decimal = m / n;
            let input_ratio: Decimal = dm / dn;
            let(optimal_a, optimal_b): (Decimal, Decimal)= 
                if (m == Decimal::zero()) | (n == Decimal::zero()) | (input_ratio == vault_ratio){
                    (dm, dn)
                } else if input_ratio > vault_ratio {
                    ( dn * m / n, dn)
                } else if input_ratio < vault_ratio {
                    ( dm, dm * n / m)
                } else {
                    panic!("Invalid input ratio")
                };
            
            self.vault_a.put(bucket_a.take(optimal_a));
            self.vault_b.put(bucket_b.take(optimal_b));

            let pool_units_amount: Decimal = 
                if self.pool_units_resource_manager.total_supply().unwrap() == Decimal::zero() {
                    dec!("100.00")
                } else if (dm / m) <= (dn / n) {
                    dm * self.pool_units_resource_manager.total_supply().unwrap() / m
                } else {
                    dn * self.pool_units_resource_manager.total_supply().unwrap() / n
                };

            let pool_units: FungibleBucket = self.pool_units_resource_manager.mint(pool_units_amount).as_fungible();

            (bucket_a, bucket_b, pool_units)
        }

        pub fn remove_liquidity(&mut self, pool_units: FungibleBucket) -> (FungibleBucket, FungibleBucket){
            assert!(
                pool_units.resource_address() == self.pool_units_resource_manager.address(),
                "Invalid pool units"
            );

            let pool_units_amount: Decimal = pool_units.amount();
            let m: Decimal = self.vault_a.amount();
            let n: Decimal = self.vault_b.amount();
            let total_supply: Decimal = self.pool_units_resource_manager.total_supply().unwrap();
            let dm: Decimal = m * pool_units_amount / total_supply;
            let dn: Decimal = n * pool_units_amount / total_supply;

            let bucket_a: FungibleBucket = self.vault_a.take(dm);
            let bucket_b: FungibleBucket = self.vault_b.take(dn);

            self.pool_units_resource_manager.burn(pool_units);

            (bucket_a, bucket_b)
        }
    }
}