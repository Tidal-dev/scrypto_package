use scrypto::prelude::ScryptoNonFungibleBucket;
use scrypto_package::launchpad_test::Launchpad;
use scrypto_package::*;
use scrypto_test::{prelude::*, utils::dump_manifest_to_file_system};
// use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Setup the environment
    let mut ledger = LedgerSimulatorBuilder::new().build();
    println!("CHEEEECKK1");

    // Create an account
    let (public_key, _private_key, account) = ledger.new_allocated_account();
    println!("CHEEEECKK2");
    println!("public_key: {}", public_key);

    // Publish package
    let package_address = ledger.compile_and_publish(this_package!());

    // Get the current timestamp using Rust's standard library
    use std::time::{SystemTime, UNIX_EPOCH};

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    println!("current_timestamp: {}", current_timestamp);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .new_badge_fixed(OwnerRole::None, Default::default(), dec!(1))
        .deposit_batch(account)
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    let owner_badge_address = receipt.expect_commit(true).new_resource_addresses()[0];
    println!("owner_badge_address: {}", owner_badge_address.to_hex());

    let sold_token_manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .new_token_fixed(OwnerRole::None, Default::default(), dec!(100000))
        .deposit_batch(account)
        .build();

    let receipt = ledger.execute_manifest(
        sold_token_manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    let sold_token_address = receipt.expect_commit(true).new_resource_addresses()[0];
    // Create a Bech32 encoder for the simulator network
    let bech32_encoder = AddressBech32Encoder::new(&NetworkDefinition::simulator());

    // Encode the address
    let encoded_address = bech32_encoder.encode(&sold_token_address.to_vec());

    println!("sold_token_address: {}", encoded_address.unwrap());

    let pay_token_manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .new_token_fixed(OwnerRole::None, Default::default(), dec!(100000))
        .deposit_batch(account)
        .build();

    let receipt = ledger.execute_manifest(
        pay_token_manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt.expect_commit_success();
    let pay_token_address = receipt.expect_commit(true).new_resource_addresses()[0];
    // println!("pay_token_address: {}", pay_token_address);

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, sold_token_address, dec!(1000))
        .take_from_worktop(sold_token_address, dec!(1000), "sold_token_bucket")
        .create_proof_from_account_of_amount(account, owner_badge_address, dec!(1))
        .call_function(
            package_address,
            "Launchpad",
            "instantiate_launchpad",
            manifest_args!(
                current_timestamp,
                current_timestamp + 100,
                sold_token_address,
                pay_token_address,
                dec!(5),
                "sold_token_bucket"
            ),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    let component = receipt.expect_commit(true).new_component_addresses()[0];
    // println!("component: {}", component);

    // // Test the `free_token` method.
    // let manifest = ManifestBuilder::new()
    //     .lock_fee_from_faucet()
    //     .withdraw_from_account(account, pay_token_address, dec!(1000))
    //     .take_from_worktop(pay_token_address, dec!(1000), "pay_token_bucket")
    //     .call_method(component, "buy", manifest_args!("pay_token_bucket"))
    //     .call_method(
    //         account,
    //         "deposit_batch",
    //         manifest_args!(ManifestExpression::EntireWorktop),
    //     )
    //     .build();
    // let receipt = ledger.execute_manifest(
    //     manifest,
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)],
    // );
    // println!("{:?}\n", receipt);
    // receipt.expect_commit_success();
}

/*
#[test]
fn test_hello_with_test_environment() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let sold_token = ResourceBuilder::new_fungible(OwnerRole::None)
    .mint_initial_supply(dec!(1000000000000000000), &mut env)?;

    let pay_token = ResourceBuilder::new_fungible(OwnerRole::None)
    .mint_initial_supply(dec!(1000000000000000000), &mut env)?;

    // Get the current time
    let current_time = env.get_current_time().seconds_since_unix_epoch;
    println!("current_time: {}", current_time);

    let mut launchpad = Launchpad::instantiate_launchpad(
        current_time,
        current_time + 100,
        sold_token.resource_address(&mut env).unwrap(),
        pay_token.resource_address(&mut env).unwrap(),
        dec!(100),
        sold_token,
        package_address,
        &mut env,
    )?;

    let pay_bucket = pay_token.take(dec!(10000), &mut env).unwrap();

    // Act
    let (nft_bucket, payment) = launchpad.buy(pay_bucket, &mut env)?;
    println!("CHECK 1");
    println!("nft_bucket: {}", nft_bucket.non_fungible_local_id().to_string());
    println!("CHECK 2");
    let vault_amount = launchpad.pay_token_vault_amount(&mut env).unwrap();
    println!("vault_amount: {}", vault_amount);
    assert_eq!(vault_amount, dec!("10000"));

    env.set_current_time(Instant::new(current_time + 200));
    println!("current_time: {}", env.get_current_time().seconds_since_unix_epoch);

    let bucket = launchpad.claim(nft_bucket, &mut env)?;
    println!("sold token remaining: {}", launchpad.sold_token_vault_amount(&mut env).unwrap());
    let amount = bucket.amount(&mut env)?;
    println!("amount bought: {}", amount);
    assert_eq!(amount, dec!("100"));

    Ok(())
} */
