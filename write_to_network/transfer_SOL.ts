import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js"

const connection = new Connection("https://api.devnet.solana.com/", "confirmed")

const sender = new Keypair()
const receiver = new Keypair()

const signature = await connection.requestAirdrop(
  sender.publicKey,
  LAMPORTS_PER_SOL
)

await connection.confirmTransaction(signature, "confirmed")

const transferInstruction = SystemProgram.transfer({
  fromPubkey: sender.publicKey,
  toPubkey: receiver.publicKey,
  lamports: 0.01 * LAMPORTS_PER_SOL,
})
console.log(JSON.stringify(transferInstruction, null, 2))
const transaction = new Transaction().add(transferInstruction)
const transactionSignature = await sendAndConfirmTransaction(
  connection,
  transaction,
  [sender]
)

const compiledMessage = transaction.compileMessage()
console.log(JSON.stringify(compiledMessage, null, 2))

console.log(`Transaction Signature:  ${transactionSignature}`)

const senderBalance = await connection.getBalance(sender.publicKey)
const receiverBalance = await connection.getBalance(receiver.publicKey)

console.log(`Sender's Balance: ${senderBalance}`)
console.log(`Receiver's Balance: ${receiverBalance}`)
