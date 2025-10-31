use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
};
use solana_system_interface::instruction::create_account;
use spl_token_interface::{
    id as token_program_id,
    instruction::{initialize_account, initialize_mint},
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

    //Now comes the part where we create a Token Account
    //Step 1 : make Keypair for token account, its space and rent required also
    let token_addr = Keypair::new();
    let token_acc_space = Account::LEN;
    let token_acc_rent = client
        .get_minimum_balance_for_rent_exemption(token_acc_space)
        .await?;

    // Step 2 : Make the instructions
    // Create Account
    let create_token_acc_instruction = create_account(
        &fee_payer.pubkey(),
        &token_addr.pubkey(),
        token_acc_rent,
        token_acc_space as u64,
        &token_program_id(),
    );

    //Initialize Token-account type on the created space
    let initialize_token_acc_instruction = initialize_account(
        &token_program_id(),
        &token_addr.pubkey(),
        &mint.pubkey(),
        &fee_payer.pubkey(),
    )?;

    // Now that the instructions are made , we can proceed with the transaction
    let token_acc_transaction = Transaction::new_signed_with_payer(
        &[
            create_token_acc_instruction,
            initialize_token_acc_instruction,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &token_addr],
        lastest_blockhash,
    );

    //Let send and confirm transaction created
    let token_acc_tx_sign = client
        .send_and_confirm_transaction(&token_acc_transaction)
        .await?;
    println!("Token account created at : {}", token_addr.pubkey());
    println!(
        "Transaction signature of token creation : {}",
        token_acc_tx_sign
    );

    //Get the data of token account created
    let token_account = client.get_account(&token_addr.pubkey()).await?;
    let token_acc_data = Account::unpack(&token_account.data);
    println!("Token Account Data: {:#?}", token_acc_data);

    Ok(())
}
