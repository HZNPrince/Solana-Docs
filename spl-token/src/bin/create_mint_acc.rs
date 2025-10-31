use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction::create_account;
use spl_token_interface::{id as token_program_id, instruction::initialize_mint, state::Mint};

#[tokio::main]
async fn main() -> Result<()> {
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com/".to_string(),
        CommitmentConfig::confirmed(),
    );
    let latest_blockhash = client.get_latest_blockhash().await?;

    //Generates a new keypair for FeePayer
    let fee_payer = Keypair::new();

    let airdrop_signature = client
        .request_airdrop(&fee_payer.pubkey(), 1_000_000_000)
        .await?;

    loop {
        let confirmed = client.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }

    //Generate a keypair to use address as mint
    let mint = Keypair::new();

    let space = Mint::LEN;
    let rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    //Create Account Instruction
    let create_account_instruction = create_account(
        &fee_payer.pubkey(),
        &mint.pubkey(),
        rent,
        space as u64,
        &token_program_id(),
    );

    //Initialize Mint Instruction
    let initialize_mint_instruction = initialize_mint(
        &token_program_id(),
        &mint.pubkey(),
        &fee_payer.pubkey(),
        Some(&fee_payer.pubkey()),
        9,
    )?;

    //create Transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );

    //Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;
    println!("Mint Address: {}", mint.pubkey());
    println!("Transaction Signature: {}", transaction_signature);

    // Get mint account
    let mint_account = client.get_account(&mint.pubkey()).await?;
    let mint = Mint::unpack(&mint_account.data)?;
    println!("\n{:#?}", mint);
    Ok(())
}
