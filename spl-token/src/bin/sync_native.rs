use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction::transfer;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token_interface::{
    id as token_program_id, instruction::sync_native, native_mint::ID as NATIVE_MINT_ID,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create connection to local validator
    let client = RpcClient::new_with_commitment(
        String::from("http://localhost:8899"),
        CommitmentConfig::confirmed(),
    );
    let latest_blockhash = client.get_latest_blockhash().await?;

    // Generate a new keypair for the fee payer
    let wallet = Keypair::new();

    // Airdrop 2 SOL to fee payer
    let airdrop_signature = client
        .request_airdrop(&wallet.pubkey(), 5_000_000_000)
        .await?;
    client.confirm_transaction(&airdrop_signature).await?;

    loop {
        let confirmed = client.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }

    // Get associated token account address for WSOL
    let associated_token_address = get_associated_token_address(
        &wallet.pubkey(), // owner
        &NATIVE_MINT_ID,  // mint (Wrapped SOL)
    );

    // Instruction to create associated token account for WSOL
    let create_ata_instruction = create_associated_token_account(
        &wallet.pubkey(),    // funding address
        &wallet.pubkey(),    // wallet address
        &NATIVE_MINT_ID,     // mint address
        &token_program_id(), // program id
    );

    // Amount to wrap (1 SOL = 1,000,000,000 lamports)
    let amount = 1_000_000_000;

    // Create transfer instruction to send SOL to the WSOL token account
    let transfer_instruction = transfer(&wallet.pubkey(), &associated_token_address, amount);

    // Create sync native instruction to update WSOL balance
    let sync_native_instruction = sync_native(&token_program_id(), &associated_token_address)?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_ata_instruction,
            transfer_instruction,
            sync_native_instruction,
        ],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );

    // Send and confirm transaction
    client.send_and_confirm_transaction(&transaction).await?;

    let token_account = client.get_token_account(&associated_token_address).await?;

    println!("WSOL Token Account Address: {}", associated_token_address);
    if let Some(token_account) = token_account {
        println!("{:#?}", token_account);
    }

    let transfer_ix = transfer(&wallet.pubkey(), &associated_token_address, amount);
    let transaction_second = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );
    client
        .send_and_confirm_transaction(&transaction_second)
        .await?;

    // ATA should still show 1 wrapped sol cause it hasnt been updated by sync native instruction
    println!("WSOL Token Account Address: {}", associated_token_address);
    let token_account = client.get_token_account(&associated_token_address).await?;
    if let Some(token_account) = token_account {
        println!("{:#?}", token_account);
    }

    let sync_native2_ix = sync_native(&token_program_id(), &associated_token_address)?;
    let transaction = Transaction::new_signed_with_payer(
        &[sync_native2_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );

    client.send_and_confirm_transaction(&transaction).await?;

    println!("WSOL Token Account Address: {}", associated_token_address);
    let token_account = client.get_token_account(&associated_token_address).await?;
    if let Some(token_account) = token_account {
        println!("{:#?}", token_account);
    }

    Ok(())
}
