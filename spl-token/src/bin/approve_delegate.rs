use anyhow::Result;
use solana_client::{nonblocking::rpc_client::RpcClient, nonce_utils::get_account};
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
    instruction::{approve_checked, initialize_mint, mint_to},
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
    .expect("Error reading wallet file");

    //Set delegate keypair
    let delegate_keypair = Keypair::new();

    let airdrop_signature = client
        .request_airdrop(&delegate_keypair.pubkey(), 10_000_000_000)
        .await?;
    let confirm_tx = client.confirm_transaction(&airdrop_signature).await?;
    let airdrop = client
        .request_airdrop(&delegate_keypair.pubkey(), 10_000_000_000)
        .await?;
    loop {
        let confirmed = client.confirm_transaction(&airdrop).await?;
        if confirmed {
            println!("Airdrop successful Tx: {}", &airdrop);
            break;
        }
    }

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

    println!("\nDelegate address : {}", delegate_keypair.pubkey());
    println!("\nToken account address: {}", token_addr);

    let account = client.get_account(&token_addr).await?;
    let token = Account::unpack(&account.data)?;

    println!("{:#?}", token);

    //IMP: To approve delate authority
    let delegate_amount = 10_000;
    let approve_delegate_instruction = approve_checked(
        &token_program_id,
        &token_addr,
        &mint_addr.pubkey(),
        &delegate_keypair.pubkey(),
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        delegate_amount,
        3,
    )?;

    let approve_delegate_transaction = Transaction::new_signed_with_payer(
        &[approve_delegate_instruction],
        Some(&wallet.pubkey()),
        &[&wallet],
        latest_blockhash,
    );
    let approve_delegate_tx_signature = client
        .send_and_confirm_transaction(&approve_delegate_transaction)
        .await?;
    println!(
        "Approve delegate Tx signature: {}",
        approve_delegate_tx_signature
    );

    //Logs
    let token = client.get_token_account(&token_addr).await?;

    println!("Successfully Approved Delegate for 10.0 Tokens ");

    if let Some(token) = token {
        println!("{:#?}", token);
    }

    Ok(())
}
