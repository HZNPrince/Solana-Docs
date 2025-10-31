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
    instruction::{initialize_mint, mint_to, transfer_checked},
    state::{Account, Mint},
};

#[tokio::main]
async fn main() -> Result<()> {
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );

    let latest_blockhash = client.get_latest_blockhash().await?;

    // Fee payer and owner of source ata
    let wallet_address = read_keypair_file(
        "/Users/aster27/Desktop/dev_creds/DevXPxYms5t88gQQ5w9N8z5ifu8F6F8KPKYjaYRrkQei.json",
    )
    .expect("Error connecting to dev wallet ");

    // Owner of destination ata
    let recipient = Keypair::new();

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
    let airdrop = client
        .request_airdrop(&recipient.pubkey(), 10_000_000_000)
        .await?;
    loop {
        let confirmed = client.confirm_transaction(&airdrop).await?;
        if confirmed {
            println!("Airdrop successful Tx: {}", &airdrop);
            break;
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
    let source_ata_addr =
        get_associated_token_address(&wallet_address.pubkey(), &mint_addr.pubkey());

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

    //Get ATA address for source and destination
    let source_ata = get_associated_token_address(&wallet_address.pubkey(), &mint_addr.pubkey());
    println!("\nSource ata: {}", source_ata);
    let destination_ata = get_associated_token_address(&recipient.pubkey(), &mint_addr.pubkey());
    println!("\nDestination ata: {}", destination_ata);

    let initialize_mint_instruction = initialize_mint(
        &token_program_id,
        &mint_addr.pubkey(),
        &wallet_address.pubkey(),
        Some(&wallet_address.pubkey()),
        2,
    )?;

    let create_ata_source_instruction = create_associated_token_account(
        &wallet_address.pubkey(),
        &wallet_address.pubkey(),
        &mint_addr.pubkey(),
        &token_program_id,
    );

    let create_ata_destination_instruction = create_associated_token_account(
        &recipient.pubkey(),
        &recipient.pubkey(),
        &mint_addr.pubkey(),
        &token_program_id,
    );

    let minting_amount = 100_00;
    let mint_to_instruction = mint_to(
        &token_program_id,
        &mint_addr.pubkey(),
        &source_ata,
        &wallet_address.pubkey(),
        &[&wallet_address.pubkey()],
        minting_amount,
    )?;

    let transaction = Transaction::new_signed_with_payer(
        &[
            create_mint_instruction,
            initialize_mint_instruction,
            create_ata_source_instruction,
            create_ata_destination_instruction,
            mint_to_instruction,
        ],
        Some(&wallet_address.pubkey()),
        &[&wallet_address, &mint_addr, &recipient],
        latest_blockhash,
    );
    let tx_signature = client.send_and_confirm_transaction(&transaction).await?;
    println!("Tx signature : {}", tx_signature);

    let mint_account = client.get_account(&mint_addr.pubkey()).await?;
    let mint_acc_data = Mint::unpack(&mint_account.data)?;
    println!("\n\n\nMint account data created: {:#?}", mint_acc_data);

    let source_account = client.get_account(&source_ata).await?;
    let source_acc_data = Account::unpack(&source_account.data)?;
    println!("Source ATA data created: {:#?}", source_acc_data);

    let destination_account = client.get_account(&destination_ata).await?;
    let destination_acc_data = Account::unpack(&destination_account.data)?;
    println!("Destination ATA data created: {:#?}", destination_acc_data);

    //Amount of Tokens to transfer
    let transfer_amount = 1000;

    // Create transfer_checked instruction to send tokens from wallet ata address to recipient ata address
    let transfer_checked_instruction = transfer_checked(
        &token_program_id,           //Program Id
        &source_ata,                 //Source address which is an ATA
        &mint_addr.pubkey(), // Mint address (Mint address of which the tokens are getting transfered)
        &destination_ata,    //Destination address which is an ATA
        &wallet_address.pubkey(), //Owner of Source address
        &[&wallet_address.pubkey()], //Signers
        transfer_amount,     //Amount
        2,                   //Decimals
    )?;
    let transfer_checked_transaction = Transaction::new_signed_with_payer(
        &[transfer_checked_instruction],
        Some(&wallet_address.pubkey()),
        &[&wallet_address],
        latest_blockhash,
    );
    let transfer_tx_signature = client
        .send_and_confirm_transaction(&transfer_checked_transaction)
        .await?;

    println!("Transfer completed tx: {}", transfer_tx_signature);

    let source_account = client.get_account(&source_ata).await?;
    let source_acc_data = Account::unpack(&source_account.data)?;
    println!("\n\n\nSource ATA data updated: {:#?}", source_acc_data);

    let destination_account = client.get_account(&destination_ata).await?;
    let destination_acc_data = Account::unpack(&destination_account.data)?;
    println!("Destination ATA data updated: {:#?}", destination_acc_data);

    Ok(())
}
