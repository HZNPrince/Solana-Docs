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
    instruction::{approve_checked, initialize_mint, mint_to, revoke},
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
    .expect("Error reading wallet ");

    let delegate_wallet = Keypair::new();

    let airdrop_signature = client
        .request_airdrop(&delegate_wallet.pubkey(), 10_000_000_000)
        .await?;
    client.confirm_transaction(&airdrop_signature).await?;

    loop {
        let confirmed = client.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            println!("Airdrop to delegate wallet : {}", airdrop_signature);
            break;
        }
    }

    let mint = Keypair::new();
    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    let associated_token_addr = get_associated_token_address(&wallet.pubkey(), &mint.pubkey());

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
        4,
    )?;
    let create_ata_instruction = create_associated_token_account(
        &wallet.pubkey(),
        &wallet.pubkey(),
        &mint.pubkey(),
        &token_program_id,
    );

    let mint_supply = 1000_0000;
    let mint_to_instruction = mint_to(
        &token_program_id,
        &mint.pubkey(),
        &associated_token_addr,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        mint_supply,
    )?;

    let delegate_amount = 50_0000;
    let approve_delegate_instruction = approve_checked(
        &token_program_id,
        &associated_token_addr,
        &mint.pubkey(),
        &delegate_wallet.pubkey(),
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        delegate_amount,
        4,
    )?;

    let transaction = Transaction::new_signed_with_payer(
        &[
            create_mint_instruction,
            initialize_mint_instruction,
            create_ata_instruction,
            mint_to_instruction,
            approve_delegate_instruction,
        ],
        Some(&wallet.pubkey()),
        &[&wallet, &mint],
        latest_blockhash,
    );
    let tx_signature = client.send_and_confirm_transaction(&transaction).await?;
    println!("\n Transaction Completed: {}", tx_signature);
    println!(
        "\n\n Mint Account : {}\n ATA created : {}\n Delegate Approved : {}",
        mint.pubkey(),
        associated_token_addr,
        delegate_wallet.pubkey()
    );
    let token = client.get_token_account(&associated_token_addr).await?;
    if let Some(token) = token {
        println!("\n\nToken Account :{:#?}", token);
    }

    let revoke_delegate_instruction = revoke(
        &token_program_id,
        &associated_token_addr,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
    )?;
    let revoke_trasaction = Transaction::new_signed_with_payer(
        &[revoke_delegate_instruction],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );

    let revoke_tx_sign = client
        .send_and_confirm_transaction(&revoke_trasaction)
        .await?;

    println!(
        "\n\nDelegate account successfully revoked : tx   {}",
        revoke_tx_sign
    );

    let token = client.get_token_account(&associated_token_addr).await?;
    if let Some(token) = token {
        println!("\n\n updated ATA:  {:#?}", token)
    }

    Ok(())
}
