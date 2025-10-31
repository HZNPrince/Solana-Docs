import { Connection, PublicKey } from "@solana/web3.js"

async function main() {
  const connection = new Connection(
    "https://api.mainnet-beta.solana.com",
    "confirmed"
  )
  const address = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  const accountInfo = await connection.getAccountInfo(address)

  console.log(
    JSON.stringify(
      accountInfo,
      (key, value) => {
        if (key === "data" && value && value.length > 1) {
          return [
            value[0],
            "...truncated, total bytes: " + value.length + "...",
            value[value.length - 1],
          ]
        }
        return value
      },
      2
    )
  )
}

main().catch((err) => {
  console.error(err)
})
