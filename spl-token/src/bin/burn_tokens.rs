use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
};
use solana_system_interface::instruction::create_account;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token_interface::{
    ID as token_program_id,
    instruction::{burn_checked, initialize_mint, mint_to},
    state::Mint,
};

#[tokio::main]
async fn main() -> Result<()> {
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );
    let latest_blockhash = client.get_latest_blockhash().await?;

    let wallet = read_keypair_file(
        "/Users/aster27/Desktop/dev_creds/DevXPxYms5t88gQQ5w9N8z5ifu8F6F8KPKYjaYRrkQei.json",
    )
    .expect("Error reading wallet file");

    // Mint account setup
    let mint_addr = Keypair::new();
    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    //Setup ATA for wallet keypair
    let token_addr = get_associated_token_address(&wallet.pubkey(), &mint_addr.pubkey());

    // Setup Instructions
    // Firstly for Mint account
    let create_mint_instruction = create_account(
        &wallet.pubkey(),
        &mint_addr.pubkey(),
        rent,
        space as u64,
        &token_program_id,
    );

    let initialize_mint_instruction = initialize_mint(
        &token_program_id,
        &mint_addr.pubkey(),
        &wallet.pubkey(),
        Some(&wallet.pubkey()),
        3,
    )?;

    //Second: ATA instruction
    let create_ata_instruction = create_associated_token_account(
        &wallet.pubkey(),
        &wallet.pubkey(),
        &mint_addr.pubkey(),
        &token_program_id,
    );

    //Mint_to to supply tokens
    let mint_supply = 100_000;
    let mint_to_instruction = mint_to(
        &token_program_id,
        &mint_addr.pubkey(),
        &token_addr,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        mint_supply,
    )?;

    //Setup transaction
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_mint_instruction,
            initialize_mint_instruction,
            create_ata_instruction,
            mint_to_instruction,
        ],
        Some(&wallet.pubkey()),
        &[&wallet, &mint_addr],
        latest_blockhash,
    );
    let tx_signature = client.send_and_confirm_transaction(&transaction).await?;
    println!("Transaction signature : {}", tx_signature);

    let token = client.get_token_account(&token_addr).await?;
    if let Some(token) = token {
        println!("\n\n{:#?}", token);
    }

    let burn_amount = 30_000;
    let burn_token_ix = burn_checked(
        &token_program_id,
        &token_addr,
        &mint_addr.pubkey(),
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        burn_amount,
        3,
    )?;
    let burn_tx = Transaction::new_signed_with_payer(
        &[burn_token_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );
    let burn_tx_sign = client.send_and_confirm_transaction(&burn_tx).await?;
    println!("Burnt tokens successfully : {}", burn_tx_sign);

    let token = client.get_token_account(&token_addr).await?;
    if let Some(token) = token {
        println!(
            "Token account amount after burning: {:#?}",
            token.token_amount.ui_amount_string
        );
    }

    Ok(())
}
