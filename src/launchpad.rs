use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct PurchaseData {
    amount: Decimal,
}

#[blueprint]
mod launchpad {

    const SIMPLE_BADGE: ResourceManager = resource_manager!(
        "resource_sim1t4kwg8fa7ldhwh8exe5w4acjhp9v982svmxp3yqa8ncruad4pf6m22"
    );

    enable_function_auth! {
        instantiate_launchpad => rule!(require(SIMPLE_BADGE.address()));
    }

    enable_method_auth! {
        methods {
            buy => PUBLIC;
            claim => PUBLIC;
            withdraw_funds => restrict_to: [OWNER];
            withdraw_unsold_tokens => restrict_to: [OWNER];
            sold_token_vault_amount => PUBLIC;
            pay_token_vault_amount => PUBLIC;
            start_time => PUBLIC;
            end_time => PUBLIC;
            current_time => PUBLIC;
        }
    }

    struct Launchpad {
        start_time: i64,
        end_time: i64,
        sold_token: ResourceAddress,
        pay_token: ResourceAddress,
        price: Decimal,
        sold_token_vault: Vault,
        pay_token_vault: Vault,
        purchase_nft: ResourceManager,
        launchpad_manager: FungibleVault,
    }

    impl Launchpad {
        pub fn instantiate_launchpad(
            start_time: i64,
            end_time: i64,
            sold_token: ResourceAddress,
            pay_token: ResourceAddress,
            price: Decimal,
            sold_token_bucket: Bucket,
        ) -> Global<Launchpad> {
            assert!(start_time < end_time, "End time must be after start time");
            assert!(price > Decimal::zero(), "Price must be greater than zero");
            assert!(!sold_token_bucket.is_empty(), "Must provide tokens to sell");

            // let owner_role = OwnerRole::Updatable(rule!(require(SIMPLE_BADGE.address())));
            let owner_role = OwnerRole::Updatable(rule!(require(SIMPLE_BADGE.address())));

            // Create launchpad manager badge
            let launchpad_manager_badge = ResourceBuilder::new_fungible(owner_role.clone())
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {
                    init {
                        "name" => "Launchpad Manager Badge", locked;
                        "symbol" => "LMB", locked;
                    }
                })
                .mint_initial_supply(1);

            let purchase_nft = ResourceBuilder::new_ruid_non_fungible::<PurchaseData>(owner_role.clone())
                .metadata(metadata! {
                    init {
                        "name" => "Launchpad Purchase NFT", locked;
                        "symbol" => "LPN", locked;
                    }
                })
                .mint_roles(mint_roles!(
                    minter => rule!(require(launchpad_manager_badge.resource_address()));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(launchpad_manager_badge.resource_address()));
                    burner_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();

            Self {
                start_time,
                end_time,
                sold_token,
                pay_token,
                price,
                sold_token_vault: Vault::with_bucket(sold_token_bucket),
                pay_token_vault: Vault::new(pay_token),
                purchase_nft,
                launchpad_manager: FungibleVault::with_bucket(launchpad_manager_badge),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .globalize()
        }

        pub fn buy(&mut self, mut payment: Bucket) -> (NonFungibleBucket, Bucket) {
            assert!(
                Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch >= self.start_time,
                "Sale has not started yet"
            );
            assert!(
                Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch < self.end_time,
                "Sale has already ended"
            );
            assert!(
                payment.resource_address() == self.pay_token,
                "Invalid token for purchase"
            );

            let amount = payment.amount() / self.price;
            assert!(
                amount <= self.sold_token_vault.amount(),
                "Not enough tokens available for sale"
            );

            self.pay_token_vault.put(payment.take(amount * self.price));

            let purchase_nft = self.launchpad_manager.authorize_with_amount(1, || {
                self.purchase_nft.mint_ruid_non_fungible(PurchaseData { amount })
            }).as_non_fungible();

            (purchase_nft, payment)
        }

        pub fn claim(&mut self, purchase_nft: NonFungibleBucket) -> Bucket {
            assert!(
                Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch >= self.end_time,
                "Sale has not ended yet"
            );
            assert!(
                purchase_nft.resource_address() == self.purchase_nft.address(),
                "Invalid purchase NFT"
            );

            let purchase_data: PurchaseData = purchase_nft.non_fungible().data();
            purchase_nft.burn();

            self.sold_token_vault.take(purchase_data.amount)
        }

        pub fn withdraw_funds(&mut self) -> Bucket {
            assert!(
                Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch >= self.end_time,
                "Sale has not ended yet"
            );

            self.pay_token_vault.take_all()
        }

        pub fn withdraw_unsold_tokens(&mut self) -> Bucket {
            assert!(
                Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch >= self.end_time,
                "Sale has not ended yet"
            );

            self.sold_token_vault.take_all()
        }

        pub fn sold_token_vault_amount(&self) -> Decimal {
            self.sold_token_vault.amount()
        }

        pub fn pay_token_vault_amount(&self) -> Decimal {
            self.pay_token_vault.amount()
        }

        pub fn start_time(&self) -> i64 {
            self.start_time
        }

        pub fn end_time(&self) -> i64 {
            self.end_time
        }

        pub fn current_time(&self) -> i64 {
            Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch
        }
    }
}