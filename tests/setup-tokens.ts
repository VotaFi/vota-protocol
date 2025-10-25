import { Program, web3 } from "@coral-xyz/anchor";
import {
    createAssociatedTokenAccount,
    createMint,
    getAccount, getAssociatedTokenAddressSync,
    MintLayout,
    mintTo,
} from "@solana/spl-token";
import { expect } from "chai";
import { VoteMarket } from "../target/types/vote_market";

export async function setupTokens(
  program: Program<VoteMarket>,
  payer: web3.Keypair,
  customMintKey: web3.Keypair | null = null
) {
  const mintKey = customMintKey ?? web3.Keypair.generate();
  const mint = mintKey.publicKey;
  let mintAccount = await program.provider.connection.getAccountInfo(mint);
  const ata = getAssociatedTokenAddressSync(mint, program.provider.publicKey);
  if (mintAccount == null) {
      await createMint(
          program.provider.connection,
          payer,
          mintKey.publicKey,
          null,
          9,
          mintKey,
          {
              commitment: "confirmed",
          }
      );
      }
     mintAccount ??= await program.provider.connection.getAccountInfo(mint);
      expect(mintAccount.data.length).to.eql(MintLayout.span);
      await createAssociatedTokenAccount(
          program.provider.connection,
          payer,
          mint,
          program.provider.publicKey
      );
      const sig = await mintTo(
          program.provider.connection,
          payer,
          mint,
          ata,
          mintKey,
          BigInt(1000000000),
          [],
          {
              skipPreflight: true,
          }
      );
      const tokenAccount = await getAccount(program.provider.connection, ata);
      expect(tokenAccount.amount).to.eql(BigInt(1000000000));
  return { mint, ata, mintAuth: mintKey };
}
