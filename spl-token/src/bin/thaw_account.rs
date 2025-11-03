use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, read_keypair_file},
    signer::Signer,
    transaction::Transaction,
};
use solana_system_interface::instruction::create_account;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token_interface::{
    ID as token_program_id,
    instruction::{freeze_account, initialize_mint, mint_to, thaw_account, transfer_checked},
    state::Mint,
};

#[tokio::main]
async fn main() -> Result<()> {
    // --- 1. Setup Connection and Wallet ---
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(), // Corrected the URL
        CommitmentConfig::confirmed(),
    );

    // Load your local wallet
    let wallet = read_keypair_file(
        "/Users/aster27/Desktop/dev_creds/DevXPxYms5t88gQQ5w9N8z5ifu8F6F8KPKYjaYRrkQei.json",
    )
    .expect("Error Connecting to wallet");

    println!("Wallet loaded: {}", wallet.pubkey());
    println!("Minting and freezing tokens...");

    // --- 2. Transaction 1: Create Mint, ATA, Mint Tokens, and Freeze ---

    // Generate a new keypair for the Mint account
    let mint = Keypair::new();
    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    // Instruction: Create the mint account
    let create_mint_ix = create_account(
        &wallet.pubkey(), // Payer
        &mint.pubkey(),   // New account address
        rent,
        space as u64,
        &token_program_id, // Owner
    );

    // Instruction: Initialize the mint with 6 decimals
    let initialize_mint_ix = initialize_mint(
        &token_program_id,
        &mint.pubkey(),
        &wallet.pubkey(),       // Mint authority
        Some(&wallet.pubkey()), // Freeze authority
        6,                      // Decimals
    )?;

    // Get the address for the wallet's Associated Token Account (ATA)
    let associated_token_account = get_associated_token_address(&wallet.pubkey(), &mint.pubkey());

    // Instruction: Create the ATA
    let create_ata_ix = create_associated_token_account(
        &wallet.pubkey(), // Payer
        &wallet.pubkey(), // Owner of the ATA
        &mint.pubkey(),
        &token_program_id,
    );

    // Instruction: Mint 1,000 tokens (1000 * 10^6)
    let mint_supply = 1_000_000_000;
    let mint_to_ix = mint_to(
        &token_program_id,
        &mint.pubkey(),
        &associated_token_account,
        &wallet.pubkey(), // Mint authority
        &[&wallet.pubkey()],
        mint_supply,
    )?;

    // Instruction: Freeze the ATA (since wallet is the freeze authority)
    let freeze_account_ix = freeze_account(
        &token_program_id,
        &associated_token_account, // Account to freeze
        &mint.pubkey(),
        &wallet.pubkey(), // Freeze authority
        &[&wallet.pubkey()],
    )?;

    // Get a fresh blockhash for this transaction
    let latest_blockhash = client.get_latest_blockhash().await?;

    // Build and send the transaction
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_mint_ix,
            initialize_mint_ix,
            create_ata_ix,
            mint_to_ix,
            freeze_account_ix,
        ],
        Some(&wallet.pubkey()),
        &[&wallet, &mint], // Mint keypair must also sign (since it's a new account)
        latest_blockhash,
    );
    let sig1 = client.send_and_confirm_transaction(&transaction).await?;
    println!("Transaction 1 (Create, Mint, Freeze) successful: {}", sig1);

    // --- 3. Transaction 2: Attempt Transfer (Expected to Fail) ---
    println!("\nAttempting transfer while frozen (this should fail)...");

    // Amount to transfer: 500 tokens
    let amount = 500_000_000;
    let transfer_check_ix = transfer_checked(
        &token_program_id,
        &associated_token_account, // From
        &mint.pubkey(),
        &associated_token_account, // To (sending to self)
        &wallet.pubkey(),          // Owner
        &[&wallet.pubkey()],
        amount,
        6, // Decimals
    )?;

    // Get a fresh blockhash for this transaction
    let latest_blockhash = client.get_latest_blockhash().await?;

    let transfer_transaction = Transaction::new_signed_with_payer(
        &[transfer_check_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );

    // We expect this to fail, so we use a `match` to catch the error
    let verify_signature = client
        .send_and_confirm_transaction(&transfer_transaction)
        .await;

    match verify_signature {
        Ok(tx) => {
            println!("Error: Transaction completed (it should have failed): {tx}");
        }
        Err(error) => {
            println!("Success: Transaction failed as expected: {}", error);
        }
    }

    // --- 4. Transaction 3: Thaw the Account ---
    println!("\nThawing the token account...");

    // Instruction: Thaw the ATA
    let thaw_account_ix = thaw_account(
        &token_program_id,
        &associated_token_account, // Account to thaw
        &mint.pubkey(),
        &wallet.pubkey(), // Freeze authority
        &[&wallet.pubkey()],
    )?;

    // Get a fresh blockhash for this transaction
    let latest_blockhash = client.get_latest_blockhash().await?;

    let thaw_tx = Transaction::new_signed_with_payer(
        &[thaw_account_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );

    let thaw_tx_sign = client.send_and_confirm_transaction(&thaw_tx).await?;
    println!("Transaction 3 (Thaw) successful: {}", thaw_tx_sign);

    // --- 5. Transaction 4: Attempt Transfer (Expected to Succeed) ---
    println!("\nAttempting transfer while thawed (this should succeed)...");

    // Instruction: Transfer 500 tokens (same as before)
    let transfer_check_ix = transfer_checked(
        &token_program_id,
        &associated_token_account,
        &mint.pubkey(),
        &associated_token_account,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        amount,
        6,
    )?;

    // Get a fresh blockhash for this transaction
    let latest_blockhash = client.get_latest_blockhash().await?;

    let transfer_transaction = Transaction::new_signed_with_payer(
        &[transfer_check_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );

    // We expect this to succeed
    let verify_signature = client
        .send_and_confirm_transaction(&transfer_transaction)
        .await;

    match verify_signature {
        Ok(tx) => {
            println!("Success: Transaction 4 (Transfer) completed: {tx}");
        }
        Err(error) => {
            println!(
                "Error: Transaction failed (it should have succeeded): {}",
                error
            );
        }
    }

    Ok(())
}
