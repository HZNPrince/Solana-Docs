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
    id as token_program_id,
    instruction::initialize_mint,
    state::{Account, Mint},
};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Make Connection with the rpc
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com/".to_string(),
        CommitmentConfig::confirmed(),
    );

    // Get the latest blockhash
    let lastest_blockhash = client.get_latest_blockhash().await?;

    // 1. Load the .env file
    dotenvy::dotenv().expect("Failed to load .env file");

    // 2. Read the path from the environment
    let wallet_path = env::var("WALLET_PATH").expect("WALLET_PATH must be set in .env file");

    //Generate Fee Payer
    let fee_payer = read_keypair_file(&wallet_path).expect("Failed to read keypair from .env file");

    //Airport the fee payer (dont be dumb to forget this)
    let balance = client.get_balance(&fee_payer.pubkey()).await?;
    println!("Wallet Balance: {}", balance);
    if balance < 1_000_000_000 {
        let airdrop_signature = client
            .request_airdrop(&fee_payer.pubkey(), 10_000_000_000)
            .await?;
        let confirmed_signature = client.confirm_transaction(&airdrop_signature).await?;
        loop {
            let confirmed_sign = client.confirm_transaction(&airdrop_signature).await?;
            if confirmed_sign {
                println!("Airdrop Transaction Signature: {}", confirmed_sign);
                break;
            }
        }

        //Give the logs
        println!("Airdrop Transaction Signature: {}", confirmed_signature);
        println!("Airdrop Signature: {}", airdrop_signature);
    }
    //Now that the fee payer is initialized , lets initialize the mint acc
    let mint = Keypair::new();
    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    //We need two instructions to initialize mint account
    //First to create space
    let create_account_instruction = create_account(
        &fee_payer.pubkey(),
        &mint.pubkey(),
        rent,
        space as u64,
        &token_program_id(),
    );

    //Second to initialize the mint account type in that space i.e account created
    let initialize_mint_instruction = initialize_mint(
        &token_program_id(),
        &mint.pubkey(),
        &fee_payer.pubkey(),
        Some(&fee_payer.pubkey()),
        9,
    )?;

    //Now that our instructions are setup we can make our transaction struct
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        lastest_blockhash,
    );

    //And finally we can send the transaction and confirm it
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;
    println!("Mint Address: {}", mint.pubkey());
    println!("Transaction signature: {}", transaction_signature);

    //Lets get the details of the mint-account we created
    let mint_account = client.get_account(&mint.pubkey()).await?;
    let mint_data = Mint::unpack(&mint_account.data)?;
    println!("Mint account Data: {:#?}", mint_data);

    //Now comes the part where we create a Token Account but "ATA"
    //Step 1 : Get the ATA address, its space and rent required also
    let token_addr = get_associated_token_address(&fee_payer.pubkey(), &mint.pubkey());
    println!("ATA address: {}", token_addr);

    // Step 2 : Make the instruction ... ATA instruction does both the function of
    //          creating  space i.e account and initializing the ATA type
    // Create ATA Account
    let create_ata_instruction = create_associated_token_account(
        &fee_payer.pubkey(),
        &fee_payer.pubkey(),
        &mint.pubkey(),
        &token_program_id(),
    );

    // Now that the instructions are made , we can proceed with the transaction
    let ata_transaction = Transaction::new_signed_with_payer(
        &[create_ata_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        lastest_blockhash,
    );

    //Let send and confirm transaction created
    let ata_tx_sign = client
        .send_and_confirm_transaction(&ata_transaction)
        .await?;

    println!("Associated-Token-account created at : {}", token_addr);

    println!("Transaction signature of token creation : {}", ata_tx_sign);

    //Get the data of token account created
    let token_account = client.get_account(&token_addr).await?;
    let token_acc_data = Account::unpack(&token_account.data);
    println!("Token Account Data: {:#?}", token_acc_data);

    Ok(())
}
