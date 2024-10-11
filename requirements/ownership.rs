use scrypto::prelude::*;

#[blueprint]
mod pair {
    pub struct Pair {
        pub pool: Global<TwoResourcePool>,
        pub pool_manager: FungibleVault,
    }

    impl Pair {
        pub fn instantiate(
            owner_role: OwnerRole,
            resource_addresses: (ResourceAddress, ResourceAddress),
        ) -> (Global<Pair>, FungibleBucket) {
            // ✅ The owner role is propagated to all of the resources created
            // in the instantiation function that should belong to the same
            // owner.
            let pool_manager_badge =
                ResourceBuilder::new_fungible(owner_role.clone())
                    .divisibility(DIVISIBILITY_NONE)
                    .mint_initial_supply(1);
            let admin_badge = ResourceBuilder::new_fungible(owner_role.clone())
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            // ✅ The owner role is propagated to all of the components created
            // in the instantiation function.
            let pool = Blueprint::<TwoResourcePool>::instantiate(
                owner_role.clone(),
                rule!(require(pool_manager_badge.resource_address())),
                resource_addresses,
                None,
            );

            // ✅ The owner role is used as the owner of the Pair being
            // instantiated.
            let pair = Self {
                pool,
                pool_manager: FungibleVault::with_bucket(pool_manager_badge),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .globalize();

            (pair, admin_badge)
        }
    }
}