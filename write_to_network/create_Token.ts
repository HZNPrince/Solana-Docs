import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js"
import {
  createInitializeMint2Instruction,
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  getMint,
} from "@solana/spl-token"

const connection = new Connection("https://api.devnet.solana.com", "confirmed")

const wallet = new Keypair()
// Fund the wallet with SOL
const signature = await connection.requestAirdrop(
  wallet.publicKey,
  LAMPORTS_PER_SOL
)

await connection.confirmTransaction(signature, "confirmed")

// Generate Keypair to use as address of mint account
const mintAcc = Keypair.generate()

// Calculate lamports required for Rent Exemption
const rentExemptionLamports = await getMinimumBalanceForRentExemptMint(
  connection
)

// Instruction to create new account with space for new mint account
const createAccountInstruction = SystemProgram.createAccount({
  fromPubkey: wallet.publicKey,
  newAccountPubkey: mintAcc.publicKey,
  space: MINT_SIZE,
  lamports: rentExemptionLamports,
  programId: TOKEN_2022_PROGRAM_ID,
})

// Instruction to initialize mint account
const initializeMintInstruction = createInitializeMint2Instruction(
  mintAcc.publicKey,
  2, // decimals
  wallet.publicKey, // mint authority
  wallet.publicKey, // freeze authority
  TOKEN_2022_PROGRAM_ID
)

// Build transactions with instructions to create new account and initialize mint account
const transactions = new Transaction().add(
  createAccountInstruction,
  initializeMintInstruction
)
const transacitonSignature = await sendAndConfirmTransaction(
  connection,
  transactions,
  [
    wallet, // Payer
    mintAcc, // Mint Address Keypair
  ]
)
console.log(`Transaction Signature ${transacitonSignature}`)

const mintData = await getMint(
  connection,
  mintAcc.publicKey,
  "confirmed",
  TOKEN_2022_PROGRAM_ID
)
console.log(
  "Mint Account:",
  JSON.stringify(
    mintData,
    (key, value) => {
      // Convert BigInt to String
      if (typeof value === "bigint") {
        return value.toString()
      }
      // Handle Buffer objects
      if (Buffer.isBuffer(value)) {
        return `<Buffer ${value.toString("hex")}>`
      }
      return value
    },
    2
  )
)
