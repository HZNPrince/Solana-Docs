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
    instruction::{freeze_account, initialize_mint, mint_to, transfer_checked},
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
        6,
    )?;

    let associated_token_account = get_associated_token_address(&wallet.pubkey(), &mint.pubkey());

    let create_ata_ix = create_associated_token_account(
        &wallet.pubkey(),
        &wallet.pubkey(),
        &mint.pubkey(),
        &token_program_id,
    );

    let mint_supply = 1000_000_000;
    let mint_to_ix = mint_to(
        &token_program_id,
        &mint.pubkey(),
        &associated_token_account,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        mint_supply,
    )?;

    let transaction = Transaction::new_signed_with_payer(
        &[
            create_mint_ix,
            initialize_mint_ix,
            create_ata_ix,
            mint_to_ix,
        ],
        Some(&wallet.pubkey()),
        &[&wallet, &mint],
        latest_blockhash,
    );

    let tx_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!(
        "Mint Address : {}\n\nATA address : {}\n\nTx Signature: {}",
        mint.pubkey(),
        associated_token_account,
        tx_signature
    );

    let freeze_account_ix = freeze_account(
        &token_program_id,
        &associated_token_account,
        &mint.pubkey(),
        &wallet.pubkey(),
        &[&wallet.pubkey()],
    )?;

    let freeze_transaction = Transaction::new_signed_with_payer(
        &[freeze_account_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );
    let freeze_tx_sign = client
        .send_and_confirm_transaction(&freeze_transaction)
        .await?;
    println!("Freezed account successful : {}", freeze_tx_sign);

    let amount = 500_000_000;
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
    let transfer_transaction = Transaction::new_signed_with_payer(
        &[transfer_check_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );
    let verify_signature = client
        .send_and_confirm_transaction(&transfer_transaction)
        .await;
    match verify_signature {
        Ok(tx) => {
            println!("ERROR: Trasaction completed : {tx}");
        }
        Err(error) => {
            println!("Transaction failed : {}", error);
        }
    }

    Ok(())
}
