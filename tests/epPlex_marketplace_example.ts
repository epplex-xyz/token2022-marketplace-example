import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { IDL, EpplexMarketplaceExample } from "../target/types/epPlex_marketplace_example";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  Transaction,
  sendAndConfirmTransaction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID, 
  createAssociatedTokenAccountIdempotentInstruction, 
  createMintToInstruction, 
  createTransferCheckedInstruction, 
  getAssociatedTokenAddressSync, 
  getMintLen,
} from "@solana/spl-token";

describe("buyer-side-royalties-marketplace", () => {
  // Configure the client to use the local cluster.
  const wallet = anchor.Wallet.local();
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const programId = new PublicKey("AhC8ej2B8LYF86ic16ZFZ4EGAxgcNz7Hvbx1pYdiAHqm");

  const program = new anchor.Program<EpplexMarketplaceExample>(IDL, programId, provider);


  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block
    })
    return signature
  }

  const log = async(signature: string): Promise<string> => {
    console.log(`Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`);
    return signature;
  }
  
  const transferHookProgramId = new PublicKey("GwUqKeSYPfuGq8YAKHNfEKTEfX3rfEz8ygLgGeVBLz8a")
  let mint = new PublicKey("Agg1KhDEp4ciZM8pJMSjunENvRd91BZKunuTiVjt5Ltr");

  // Sender token account address
  const sourceTokenAccount = getAssociatedTokenAddressSync(mint, wallet.publicKey, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);

  const buyer = Keypair.generate();
  const buyerAta = getAssociatedTokenAddressSync(mint, buyer.publicKey, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);

  // ExtraAccountMetaList address
  // Store extra accounts required by the custom transfer hook instruction
  const metasAccountList = PublicKey.findProgramAddressSync(
    [Buffer.from("extra-account-metas"), mint.toBuffer()],
    transferHookProgramId
  )[0];

  const marketplace = PublicKey.findProgramAddressSync(
    [Buffer.from("marketplace"), wallet.publicKey.toBuffer()],
    programId
  )[0];

  const listing = PublicKey.findProgramAddressSync(
    [Buffer.from("listing"), mint.toBuffer(), marketplace.toBuffer()],
    programId
  )[0];

  const listingAta = getAssociatedTokenAddressSync(mint, listing, true, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);

  it("Airdrop", async () => {
    await connection.requestAirdrop(buyer.publicKey, LAMPORTS_PER_SOL * 10).then(confirm).then(log)
  })

  it("Create Token Accounts", async () => {
    const amount = 100;

    const transaction = new Transaction().add(
      createAssociatedTokenAccountIdempotentInstruction(
        wallet.publicKey,
        sourceTokenAccount,
        wallet.publicKey,
        mint,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
      createAssociatedTokenAccountIdempotentInstruction(
        wallet.publicKey,
        listingAta,
        listing,
        mint,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
      createMintToInstruction(
        mint,
        sourceTokenAccount,
        wallet.publicKey,
        amount,
        [],
        TOKEN_2022_PROGRAM_ID,
      ),
    );

    const txSig = await sendAndConfirmTransaction(
      connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true },
    ).then(confirm).then(log);
  });

  it("Initialize the marketplace", async () => {
    await program.methods.initializeMarketplace()
    .accounts({
      admin: wallet.publicKey,
      marketplace,
      systemProgram: SystemProgram.programId,
    })
    .signers([wallet.payer]).rpc({skipPreflight: true}).then(confirm).then(log);
  });

  it("List T22 using CPI", async () => {
    let price = new anchor.BN(10000);

    const cpiListIx = await program.methods.cpiList(price)
    .accounts({
      lister: wallet.publicKey,
      listerAta: sourceTokenAccount,
      marketplace,
      listing,
      listingAta,
      mint,
      metasAccountList,
      transferHookProgramId,
      sysvarInstruction: SYSVAR_INSTRUCTIONS_PUBKEY,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).instruction();
    
    const transaction = new Transaction().add(cpiListIx);
    await sendAndConfirmTransaction(connection, transaction, [wallet.payer], { skipPreflight: true }).then(confirm).then(log);
  });

  xit("Delist T22 using CPI", async () => {
    const createAccountIx = createAssociatedTokenAccountIdempotentInstruction(
      wallet.publicKey,
      buyerAta,
      buyer.publicKey,
      mint,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    const cpiDelistIx = await program.methods.cpiDelist()
    .accounts({
      lister: wallet.publicKey,
      listerAta: sourceTokenAccount,
      marketplace,
      listing,
      listingAta,
      mint,
      metasAccountList,
      transferHookProgramId,
      sysvarInstruction: SYSVAR_INSTRUCTIONS_PUBKEY,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).instruction();
    
    const transaction = new Transaction().add(createAccountIx).add(cpiDelistIx);
    await sendAndConfirmTransaction(connection, transaction, [wallet.payer], { skipPreflight: true }).then(confirm).then(log);
  });

  it("Buy T22 using CPI", async () => {
    let price = new anchor.BN(10000);

    const createAccountIx = createAssociatedTokenAccountIdempotentInstruction(
      wallet.publicKey,
      buyerAta,
      buyer.publicKey,
      mint,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    const payRoyaltiesIx = SystemProgram.transfer(
      {
        fromPubkey: buyer.publicKey,
        toPubkey: PublicKey.default,
        lamports: Number(price) / 100
      }
    );

    const cpiDelistIx = await program.methods.cpiBuy(price)
    .accounts({
      buyer: buyer.publicKey,
      buyerAta,
      lister: wallet.publicKey,
      marketplace,
      listing,
      listingAta,
      mint,
      metasAccountList,
      transferHookProgramId,
      sysvarInstruction: SYSVAR_INSTRUCTIONS_PUBKEY,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).instruction();
    
    const transaction = new Transaction().add(createAccountIx).add(cpiDelistIx).add(payRoyaltiesIx);
    await sendAndConfirmTransaction(connection, transaction, [buyer, wallet.payer], { skipPreflight: true }).then(confirm).then(log);
  });
});
