import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as serumCmn from "@project-serum/common";
import * as anchor from '@project-serum/anchor';
import { VestingSchedule } from '../target/types/vesting_schedule';
import { TokenInstructions } from "@project-serum/serum";
import { Program } from '@project-serum/anchor';
import { createMint, createTokenAccount, setUpAta } from './utils';
import { sleep } from '@project-serum/common';

const program = anchor.workspace.VestingSchedule as Program<VestingSchedule>;

export type InitializeParameterInterface = {
    vesting_schedule: PublicKey,
    vesting_data: Keypair,
    mint_hbb: PublicKey,
    vesting_vault_hbb: PublicKey
};

export async function getInitilizeParameter(provider:anchor.Provider) : Promise<InitializeParameterInterface> {

    const vesting_schedule = (
        await serumCmn.createAccountRentExempt(
            provider,
            program.programId,
            program.account.vestingSchedule.size
        )
    ).publicKey;

    const vData = anchor.web3.Keypair.generate();

    const mint_hbb = await createMint(provider, provider.wallet.publicKey, 6);
  
    const vesting_vault_hbb = await createTokenAccount(provider, mint_hbb, provider.wallet.publicKey);

    return {
        vesting_schedule,
        vesting_data: vData,
        mint_hbb,
        vesting_vault_hbb
    };
    
}

export async function initialize(
    current_time : number,
    admin: PublicKey,
    vesting_schedule: PublicKey,
    vesting_data: Keypair,
    vesting_vault: PublicKey,
) {
    await program.rpc.initialize(
        new anchor.BN(current_time), {
        accounts: {
            admin: admin,
            vestingData: vesting_data.publicKey,
            vestingSchedule: vesting_schedule,
            vestingVault: vesting_vault,
            tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
        },
        signers: [vesting_data],
    });
}

export async function claim(
    index: number,
    admin: PublicKey,
    vesting_schedule: PublicKey,
    vesting_data: Keypair,
    claim_account: PublicKey,
    claim_user_ata: PublicKey,
) {

    const auth = await program.account.vestingData.fetch(vesting_data.publicKey);

    await program.rpc.claim(
      new anchor.BN(index), {
      accounts: {
        owner: admin,
        vestingSchedule: vesting_schedule,
        vestingData: vesting_data.publicKey,
        claimUser: claim_account,
        claimUserAta: claim_user_ata,
        vestingVault: auth.vestingVault,
        vestingVaultAuthority: auth.vestingVaultAuthority,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
}

export async function getUserAta(user:Uint8Array, provider:anchor.Provider, mint_hbb: PublicKey) : Promise<PublicKey> {
    const user_liquidator = Keypair.fromSecretKey(user);
    await provider.connection.requestAirdrop(user_liquidator.publicKey, 3 * LAMPORTS_PER_SOL);
    await sleep(500);
    const user_ata = await setUpAta(provider, user_liquidator, mint_hbb);
    return user_ata;
}

export async function addUser(
    unlock_percent: number,
    user_pubkey: PublicKey,
    unlock_period: number,
    planned_tokens: number,
    vesting_schedule: PublicKey,
) {
    await program.rpc.addUser(
        new anchor.BN(unlock_percent),
        user_pubkey,
        new anchor.BN(unlock_period),
        new anchor.BN(planned_tokens), 
        {
            accounts: {
                vestingSchedule: vesting_schedule,
                systemProgram: anchor.web3.SystemProgram.programId,
            },
        }
    );
}

export async function removeUser(
    index: number,
    vesting_schedule: PublicKey,
) {
    await program.rpc.removeUser(
        new anchor.BN(index),
        {
            accounts: {
                vestingSchedule: vesting_schedule,
                systemProgram: anchor.web3.SystemProgram.programId,
            },
        }
    );
}