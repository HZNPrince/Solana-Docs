use std::fs::read_dir;

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
    instruction::{AuthorityType, initialize_mint, mint_to, set_authority},
    state::{Account, Mint},
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
    .expect("ERROR CONNECTING TO WALLET");

    let new_authority = Keypair::new();

    let mint = Keypair::new();
    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    let create_mint_instruction = create_account(
        &wallet.pubkey(),
        &mint.pubkey(),
        rent,
        space as u64,
        &token_program_id,
    );
    let initialize_mint_instruction = initialize_mint(
        &token_program_id,
        &mint.pubkey(),
        &wallet.pubkey(),
        Some(&wallet.pubkey()),
        2,
    )?;

    let transaction = Transaction::new_signed_with_payer(
        &[create_mint_instruction, initialize_mint_instruction],
        Some(&wallet.pubkey()),
        &[&wallet, &mint],
        latest_blockhash,
    );
    let tx_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!("Transaction : {}", tx_signature);

    let mint_account = client.get_account(&mint.pubkey()).await?;
    let mint_account_data = Mint::unpack(&mint_account.data)?;

    println!("Created mint account : {:#?}", mint_account_data);

    let mint_authority_ix = set_authority(
        &token_program_id,
        &mint.pubkey(),
        Some(&new_authority.pubkey()),
        AuthorityType::MintTokens,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
    )?;

    let freeze_authority_ix = set_authority(
        &token_program_id,
        &mint.pubkey(),
        Some(&new_authority.pubkey()),
        AuthorityType::FreezeAccount,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
    )?;

    let authority_tx = Transaction::new_signed_with_payer(
        &[mint_authority_ix, freeze_authority_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );
    let authority_tx_sign = client.send_and_confirm_transaction(&authority_tx).await?;

    println!("Authority Updated : {}", authority_tx_sign);

    let mint_account = client.get_account(&mint.pubkey()).await?;
    let mint_account_data = Mint::unpack(&mint_account.data)?;
    println!("Authorities changes : \n{:#?}", mint_account_data);

    let revoke_authority_instruction = set_authority(
        &token_program_id,
        &mint.pubkey(),
        None,
        AuthorityType::MintTokens,
        &new_authority.pubkey(),
        &[&new_authority.pubkey()],
    )?;
    let revoke_authority_tx = Transaction::new_signed_with_payer(
        &[revoke_authority_instruction],
        Some(&wallet.pubkey()),
        &[&wallet, &new_authority],
        latest_blockhash,
    );
    let revoke_authority_tx_sign = client
        .send_and_confirm_transaction(&revoke_authority_tx)
        .await?;

    println!("Authority revoked : {}", revoke_authority_tx_sign);

    let mint_account = client.get_account(&mint.pubkey()).await?;
    let mint_account_data = Mint::unpack(&mint_account.data);
    println!(
        "The updated mint authority account \n\n{:#?}",
        mint_account_data
    );
    Ok(())
}
