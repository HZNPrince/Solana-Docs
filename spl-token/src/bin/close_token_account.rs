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
    instruction::{close_account, initialize_mint},
    state::Mint,
};

#[tokio::main]
async fn main() -> Result<()> {
    let client = RpcClient::new_with_commitment(
        "http:/localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );
    let latest_blockhash = client.get_latest_blockhash().await?;

    let wallet = read_keypair_file(
        "/Users/aster27/Desktop/dev_creds/DevXPxYms5t88gQQ5w9N8z5ifu8F6F8KPKYjaYRrkQei.json",
    )
    .expect("Error Connecting to wallet");

    let mint = Keypair::new();
    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    let create_mint_ix = create_account(
        &wallet.pubkey(),
        &mint.pubkey(),
        rent,
        space as u64,
        &token_program_id,
    );

    let initialize_mint_ix = initialize_mint(
        &token_program_id,
        &mint.pubkey(),
        &wallet.pubkey(),
        Some(&wallet.pubkey()),
        5,
    )?;

    let associated_token_account = get_associated_token_address(&wallet.pubkey(), &mint.pubkey());

    let create_ata_ix = create_associated_token_account(
        &wallet.pubkey(),
        &wallet.pubkey(),
        &mint.pubkey(),
        &token_program_id,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[create_mint_ix, initialize_mint_ix, create_ata_ix],
        Some(&wallet.pubkey()),
        &[&wallet, &mint],
        latest_blockhash,
    );

    let tx_sign = client.send_and_confirm_transaction(&transaction).await?;

    println!(
        "ATA Created : {}\n\nof Mint Address : {}\n\n Tx signature : {}",
        associated_token_account,
        mint.pubkey(),
        tx_sign
    );
    let token = client.get_token_account(&associated_token_account).await?;
    if let Some(token) = token {
        println!("\n\n Token account data: {:#?}", token);
    }

    let close_account_ix = close_account(
        &token_program_id,
        &associated_token_account,
        &wallet.pubkey(),
        &wallet.pubkey(),
        &[&wallet.pubkey()],
    )?;

    let close_account_transaction = Transaction::new_signed_with_payer(
        &[close_account_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );
    client
        .send_and_confirm_transaction(&close_account_transaction)
        .await?;

    let token = client.get_account(&associated_token_account).await;
    match token {
        Ok(token) => {
            println!("ERROR: Token account has not been closed {:#?}", token);
        }
        Err(error) => {
            println!("Token account has been closed : {}", error);
        }
    }

    Ok(())
}
