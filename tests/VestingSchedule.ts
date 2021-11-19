import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { VestingSchedule } from '../target/types/vesting_schedule';
import { 
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  Keypair
 } from "@solana/web3.js";
import { TokenInstructions } from "@project-serum/serum";
import { createMint, setUpAta, createTokenAccount, mintTo, decimalToU64 } from './utils';
import { sleep } from '@project-serum/common';
const serumCmn = require("@project-serum/common");
const { SystemProgram } = anchor.web3;

describe('VestingSchedule', () => {

  // Configure the client to use the local cluster.

  const provider = anchor.Provider.local();

  anchor.setProvider(provider);

  const myAccount = anchor.web3.Keypair.generate();

  let vData = anchor.web3.Keypair.generate();

  const assert = require('assert');

  const web3 = require("@solana/web3.js");

  const program = anchor.workspace.VestingSchedule as Program<VestingSchedule>;

  let mintHBB : any;
  let collateralVaultHBB : any;
  let hbbAta : any;
  let mintTokenAmount = 1000000.0;
  let publishtime = '2021.11.17';

  it('Is initialized!', async () => {
    // Add your test here.
    
    let currentTime = new Date(publishtime).getTime() / 1000;

    mintHBB = await createMint(provider, provider.wallet.publicKey, 6);
  
    collateralVaultHBB = await createTokenAccount(provider, mintHBB, provider.wallet.publicKey);

    const tx = await program.rpc.initialize(
      new anchor.BN(currentTime), {
      accounts: {
        admin: provider.wallet.publicKey,
        vestingSchedule: myAccount.publicKey,
        vestingData: vData.publicKey,
        vestingVault: collateralVaultHBB,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      signers: [myAccount, vData],
    });

    await mintTo(provider, mintHBB, collateralVaultHBB, decimalToU64(mintTokenAmount));

    const fromAccount = await getTokenAccount(provider, collateralVaultHBB);

    assert.ok(fromAccount.amount.eq(new anchor.BN(1000000_000_000)));
  });

  it('Claim Completed', async () => {

    let claimAccount = new web3.PublicKey('29GPMU5gtBDbd368EwquqTmo33tKgvneAK9REmmxkqm8');

    let y: Uint8Array = Uint8Array.from([
      227, 208, 96, 209, 94, 248, 249, 228, 152, 203, 169, 223, 89, 152, 61, 189, 74, 231, 111,
      238, 6, 208, 226, 251, 15, 31, 44, 191, 13, 244, 121, 222, 16, 249, 186, 206, 236, 146, 48,
      246, 2, 163, 91, 119, 79, 228, 118, 207, 67, 138, 98, 53, 182, 219, 33, 68, 109, 246, 221,
      6, 49, 8, 46, 231]);

    let liquidator = Keypair.fromSecretKey(y);
    await provider.connection.requestAirdrop(liquidator.publicKey, 3 * LAMPORTS_PER_SOL);
    await sleep(500);
  
    hbbAta = await setUpAta(provider, liquidator, mintHBB);

    const auth = await program.account.vestingData.fetch(vData.publicKey);

    await program.rpc.claim(
      new anchor.BN(0), {
      accounts: {
        owner: provider.wallet.publicKey,
        vestingSchedule: myAccount.publicKey,
        vestingData: vData.publicKey,
        claimUser: claimAccount,
        claimUserAta: hbbAta,
        vestingVault: auth.vestingVault,
        vestingVaultAuthority: auth.vestingVaultAuthority,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
    });

    let dateTime = new Date().getTime() / 1000;
    let minttime = new Date(publishtime).getTime() / 1000;

    let delaytime = Math.floor((dateTime - minttime) / (24 * 3600));

    let calcclaimtoken = Math.floor(0.15 * decimalToU64(mintTokenAmount) + 0.85 * decimalToU64(mintTokenAmount) * delaytime / (12 * 30.5));

    const fromAccount = await getTokenAccount(provider, hbbAta);
    assert.ok(fromAccount.amount.eq(new anchor.BN(calcclaimtoken)));
  });
});

async function getTokenAccount(provider:any, addr:any) {
  return await serumCmn.getTokenAccount(provider, addr);
}