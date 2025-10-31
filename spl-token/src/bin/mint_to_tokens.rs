use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer, read_keypair_file},
    transaction::{self, Transaction},
};
use solana_system_interface::instruction::create_account;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token_interface::{
    ID as token_program_id,
    instruction::{initialize_mint, mint_to},
    state::{Account, Mint},
};

#[tokio::main]
async fn main() -> Result<()> {
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com/".to_string(),
        CommitmentConfig::confirmed(),
    );

    let latest_blockhash = client.get_latest_blockhash().await?;

    // Fee payer
    let wallet_address = read_keypair_file(
        "/Users/aster27/Desktop/dev_creds/DevXPxYms5t88gQQ5w9N8z5ifu8F6F8KPKYjaYRrkQei.json",
    )
    .expect("Error connecting to dev wallet ");

    let balance = client.get_balance(&wallet_address.pubkey()).await?;
    if balance < 1_000_000_000 {
        let airdrop = client
            .request_airdrop(&wallet_address.pubkey(), 10_000_000_000)
            .await?;
        loop {
            let confirmed = client.confirm_transaction(&airdrop).await?;
            if confirmed {
                println!("Airdrop successful Tx: {}", &airdrop);
                break;
            }
        }
    }

    //Mint account
    let mint_addr = Keypair::new();
    let mint_addr_space = Mint::LEN;
    let mint_addr_rent = client
        .get_minimum_balance_for_rent_exemption(mint_addr_space)
        .await?;
    println!(
        "The space required to create mint accounts are {}",
        mint_addr_space
    );

    // ATA account
    let token_addr = get_associated_token_address(&wallet_address.pubkey(), &mint_addr.pubkey());

    //Instructions : 1) Create mint account
    //               2) Initialize Mint account
    //               3) create Associated-Token-Account
    let create_mint_instruction = create_account(
        &wallet_address.pubkey(),
        &mint_addr.pubkey(),
        mint_addr_rent,
        mint_addr_space as u64,
        &token_program_id,
    );

    let initialize_mint_instruction = initialize_mint(
        &token_program_id,
        &mint_addr.pubkey(),
        &wallet_address.pubkey(),
        Some(&wallet_address.pubkey()),
        9,
    )?;

    let create_ata_instruction = create_associated_token_account(
        &wallet_address.pubkey(),
        &wallet_address.pubkey(),
        &mint_addr.pubkey(),
        &token_program_id,
    );

    //Transaction
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_mint_instruction,
            initialize_mint_instruction,
            create_ata_instruction,
        ],
        Some(&wallet_address.pubkey()),
        &[&wallet_address, &mint_addr],
        latest_blockhash,
    );
    let tx_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!("Tx Signature: {}", tx_signature);
    println!("Mint Account address: {}", mint_addr.pubkey());
    println!("Token Account address: {}", token_addr);

    let mint_account = client.get_account(&mint_addr.pubkey()).await?;
    let mint_acc_data = Mint::unpack(&mint_account.data)?;
    println!("Mint Account data : {:#?}", mint_acc_data);

    let token_account = client.get_account(&token_addr).await?;
    let token_acc_data = Account::unpack(&token_account.data)?;
    println!("ATA data : {:#?}", token_acc_data);

    //Amount of tokens to mint (100 tokens with 2 decimal spaces )
    let mint_amount = 1_000_000_000;

    //Create mint_to instruction to mint tokens to ATA
    let mint_to_instruction = mint_to(
        &token_program_id,
        &mint_addr.pubkey(),
        &token_addr,
        &wallet_address.pubkey(),
        &[&wallet_address.pubkey()],
        mint_amount,
    )?;

    //Create Transaction for minting tokens
    let mint_transaction = Transaction::new_signed_with_payer(
        &[mint_to_instruction],
        Some(&wallet_address.pubkey()),
        &[wallet_address],
        latest_blockhash,
    );

    // Send and confirm the tx created
    let mint_tx_signature = client
        .send_and_confirm_transaction(&mint_transaction)
        .await?;
    println!(
        "Transaction to mint_to instruction  : {}",
        mint_tx_signature
    );

    println!("Minted 1 token to the Associated-token-account");

    let updated_mint_acc = client.get_account(&mint_addr.pubkey()).await?;
    let updated_mint_acc_data = Mint::unpack(&updated_mint_acc.data)?;

    let updated_ata_acc = client.get_account(&token_addr).await?;
    let updated_ata_acc_data = Account::unpack(&updated_ata_acc.data)?;
    println!("Mint account updated data : {:#?}", updated_mint_acc_data);
    println!("ATA updated data : {:#?}", updated_ata_acc_data);

    Ok(())
}
