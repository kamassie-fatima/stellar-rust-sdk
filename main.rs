use std::error::Error;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use stellar_base::amount::Amount;
use stellar_base::asset::Asset;
use stellar_base::crypto::{DalekKeyPair, PublicKey};
use stellar_base::network::Network;
use stellar_base::operations::Operation;
use stellar_base::transaction::{Transaction, MIN_BASE_FEE};
use stellar_sdk::Server;

const HORIZON_TESTNET: &str = "https://horizon-testnet.stellar.org";

/// Opens a trustline from source_kp's account to the given asset.
fn establish_trustline(
    server: &Server,
    source_kp: &DalekKeyPair,
    asset: &Asset,
) -> Result<(), Box<dyn Error>> {
    let account_id = source_kp.public_key().account_id();
    let sequence: i64 = server.load_account(&account_id)?.sequence_number().parse()?;

    let trust_op = Operation::new_change_trust()
        .with_asset(asset.clone().into())
        .with_limit(None::<&str>)?
        .build()?;

    let mut trust_tx = Transaction::builder(source_kp.public_key(), sequence, MIN_BASE_FEE)
        .add_operation(trust_op)
        .into_transaction()?;

    trust_tx.sign(source_kp.as_ref(), &Network::new_test());
    server.submit_transaction(trust_tx)?;

    println!("Trustline submitted for {}", account_id);
    Ok(())
}

/// Sends `amount` of `asset` from source_kp to destination.
/// Call this only after the trustline transaction has been confirmed.
fn send_asset(
    server: &Server,
    source_kp: &DalekKeyPair,
    destination: &PublicKey,
    asset: &Asset,
    amount: &str,
) -> Result<(), Box<dyn Error>> {
    let account_id = source_kp.public_key().account_id();
    let sequence: i64 = server.load_account(&account_id)?.sequence_number().parse()?;

    let payment_op = Operation::new_payment()
        .with_destination(destination.clone())
        .with_amount(Amount::from_str(amount)?)?
        .with_asset(asset.clone())
        .build()?;

    let mut payment_tx = Transaction::builder(source_kp.public_key(), sequence, MIN_BASE_FEE)
        .add_operation(payment_op)
        .into_transaction()?;

    payment_tx.sign(source_kp.as_ref(), &Network::new_test());
    server.submit_transaction(payment_tx)?;

    println!("Sent {} {} to {}", amount, asset.code(), destination.account_id());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let server = Server::new(String::from(HORIZON_TESTNET), None)?;

    let source_kp = DalekKeyPair::random()?;
    let issuer = PublicKey::from_account_id(
        "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
    )?;
    let destination = PublicKey::from_account_id("G...RECIPIENT")?;
    let credit_asset = Asset::new_credit("USDC", issuer)?;

    establish_trustline(&server, &source_kp, &credit_asset)?;

    // Give the trustline time to confirm on the ledger before paying.
    // A payment fails with op_no_trust if it's submitted too early.
    sleep(Duration::from_secs(6));

    send_asset(&server, &source_kp, &destination, &credit_asset, "25")?;

    Ok(())
}
