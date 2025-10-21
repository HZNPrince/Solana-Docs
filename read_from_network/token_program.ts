import { Connection, PublicKey } from "@solana/web3.js"

async function main() {
  const connection = new Connection(
    "https://api.mainnet-beta.solana.com",
    "confirmed"
  )

  const address = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
  const accountInfo = await connection.getAccountInfo(address)
  console.log(
    JSON.stringify(
      accountInfo,
      (key, value) => {
        if (key === "data" && value && value.length > 1) {
          return [
            value[0],
            "... truncated, total bytes : " + value.length + "...",
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
