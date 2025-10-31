import { Keypair, Connection, LAMPORTS_PER_SOL } from "@solana/web3.js"

async function main() {
  const keypair = Keypair.generate()
  console.log(`Keypair Public Address: ${keypair.publicKey}`)

  const connection = new Connection(
    "https://api.devnet.solana.com/",
    "confirmed"
  )

  //Funding an address with SOL automatically creates an address
  const signature = await connection.requestAirdrop(
    keypair.publicKey,
    LAMPORTS_PER_SOL
  )

  await connection.confirmTransaction(signature, "confirmed")

  const accountInfo = await connection.getAccountInfo(keypair.publicKey)
  console.log(JSON.stringify(accountInfo, null, 10))
}

main().catch((err) => {
  console.error(err)
})
