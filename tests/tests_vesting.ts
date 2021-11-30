import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { TokenInstructions } from "@project-serum/serum";
import { mintTo, decimalToU64, getTokenAccount } from './utils';
import { initialize, getInitilizeParameter, claim, getUserAta, addUser, removeUser } from './vesting_instruction';
import { VestingSchedule } from '../target/types/vesting_schedule';
import { Keypair } from "@solana/web3.js";
const { SystemProgram } = anchor.web3;

describe('VestingSchedule', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.local();
  const assert = require('assert');
  const program = anchor.workspace.VestingSchedule as Program<VestingSchedule>;

  anchor.setProvider(provider);

  it('Is initialized!', async () => {
    const publish_time = '2021.8.17';
    const mint_token_amount = 1_000_000.0;
    const current_time = new Date(publish_time).getTime() / 1000;
    const { vesting_schedule, vesting_data, mint_hbb, vesting_vault_hbb } = await getInitilizeParameter(provider);

    await initialize(
      current_time,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      vesting_vault_hbb,
    );

    const initializeToken = await getTokenAccount(provider, vesting_vault_hbb);
    assert.ok(initializeToken.amount.eq(new anchor.BN(0)));

    const get_total_issue = await program.account.vestingData.fetch(vesting_data.publicKey);
    assert.ok(get_total_issue.totalIssuedSoFar.eq(new anchor.BN(0)));

    await mintTo(provider, mint_hbb, vesting_vault_hbb, decimalToU64(mint_token_amount));

    const mintToken = await getTokenAccount(provider, vesting_vault_hbb);
    assert.ok(mintToken.amount.eq(new anchor.BN(1_000_000_000_000)));
  });

  it('Add User Completed', async () => {

    // Add 2 Clients
    // Check PendingToken Status Client Number match 2
    // Claim
    // Calc how much will be claimed for 2 Clients
    // Compare token amounts

    const { vesting_schedule, vesting_data, mint_hbb, vesting_vault_hbb } = await getInitilizeParameter(provider);
    const publish_time = '2021.8.17';
    const mint_token_amount = 1_000_000.0;
    const tge_time = new Date(publish_time).getTime() / 1000;
    const userClaimToken = 1_000;
    let user1 = Keypair.generate();
    let user2 = Keypair.generate();

    await initialize(
      tge_time,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      vesting_vault_hbb,
    );

    await mintTo(provider, mint_hbb, vesting_vault_hbb, decimalToU64(mint_token_amount));

    await addUser(
      15,
      user1.publicKey,
      12,
      1_000_000_000,
      vesting_schedule,
    );

    await addUser(
      20,
      user2.publicKey,
      8,
      1_000_000_000,
      vesting_schedule,
    );

    const get_active_users = await program.account.vestingSchedule.fetch(vesting_schedule);
    assert.ok(get_active_users.len, 2);

    const user_ata1 = await getUserAta(user1.secretKey, provider, mint_hbb);
    const user_ata2 = await getUserAta(user2.secretKey, provider, mint_hbb);

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );


    let claim_time = new Date().getTime() / 1000;
    let mint_time = new Date(publish_time).getTime() / 1000;

    let delay_minutes = Math.floor((claim_time - mint_time) / 60);

    let calc_user_claim_token1 = Math.floor(0.15 * decimalToU64(userClaimToken) + 0.85 * decimalToU64(userClaimToken) * delay_minutes / (12 * 30.5 * 24 * 60));
    let calc_user_claim_token2 = Math.floor(0.2 * decimalToU64(userClaimToken) + 0.8 * decimalToU64(userClaimToken) * delay_minutes / (8 * 30.5 * 24 * 60));

    let token_amount_account1 = await getTokenAccount(provider, user_ata1);
    assert.ok(token_amount_account1.amount.eq(new anchor.BN(calc_user_claim_token1)));

    let token_amount_account2 = await getTokenAccount(provider, user_ata2);
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(calc_user_claim_token2)));
  });

  it('Locked User Claim Completed', async () => {

    // Add 2 clients
    // First User Set Inactive Status
    // Check PendingToken Status Client Number match 1
    // Claim
    // First User Get 0, Second User Get Calculate Amounts
    // First User Set PendingToken Status
    // Check PendingToken Status Client Number match 2
    // Claim
    // First User Get Calculate Amounts, Second User Get 0

    const { vesting_schedule, vesting_data, mint_hbb, vesting_vault_hbb } = await getInitilizeParameter(provider);
    const publish_time = '2021.8.17';
    const mint_token_amount = 1_000_000.0;
    const tge_time = new Date(publish_time).getTime() / 1000;
    const userClaimToken = 1_000;
    let user1 = Keypair.generate();
    let user2 = Keypair.generate();

    await initialize(
      tge_time,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      vesting_vault_hbb,
    );

    await mintTo(provider, mint_hbb, vesting_vault_hbb, decimalToU64(mint_token_amount));

    await addUser(
      15,
      user1.publicKey,
      12,
      1_000_000_000,
      vesting_schedule,
    );

    await addUser(
      20,
      user2.publicKey,
      8,
      1_000_000_000,
      vesting_schedule,
    );

    await removeUser(0, vesting_schedule);
    let get_active_users = await program.account.vestingSchedule.fetch(vesting_schedule);
    assert.ok(get_active_users.len, 1);

    const user_ata1 = await getUserAta(user1.secretKey, provider, mint_hbb);
    const user_ata2 = await getUserAta(user2.secretKey, provider, mint_hbb);

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );

    let claim_time = new Date().getTime() / 1000;
    let mint_time = new Date(publish_time).getTime() / 1000;

    let delay_minutes = Math.floor((claim_time - mint_time) / 60);

    let calc_user_claim_token1 = 0;
    let calc_user_claim_token2 = Math.floor(0.2 * decimalToU64(userClaimToken) + 0.8 * decimalToU64(userClaimToken) * delay_minutes / (8 * 30.5 * 24 * 60));

    let token_amount_account1 = await getTokenAccount(provider, user_ata1);
    assert.ok(token_amount_account1.amount.eq(new anchor.BN(calc_user_claim_token1)));

    let token_amount_account2 = await getTokenAccount(provider, user_ata2);
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(calc_user_claim_token2)));

    await addUser(
      15,
      user1.publicKey,
      12,
      1_000_000_000,
      vesting_schedule,
    );

    get_active_users = await program.account.vestingSchedule.fetch(vesting_schedule);
    assert.ok(get_active_users.len, 2);

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );

    claim_time = new Date().getTime() / 1000;
    delay_minutes = Math.floor((claim_time - mint_time) / 60);

    calc_user_claim_token1 = Math.floor(0.15 * decimalToU64(userClaimToken) + 0.85 * decimalToU64(userClaimToken) * delay_minutes / (12 * 30.5 * 24 * 60));
    token_amount_account1 = await getTokenAccount(provider, user_ata1);
    assert.ok(token_amount_account1.amount.eq(new anchor.BN(calc_user_claim_token1)));

    token_amount_account2 = await getTokenAccount(provider, user_ata2);
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(calc_user_claim_token2)));
  });

  it('Claim Two Times Completed', async () => {

    // Add 2 clients
    // Check PendingToken Status Client Number match 2
    // Claim
    // Clients Get Calculated Amount
    // Total Issued So Far = First Client's Claimed Tokens + Second Client's Claimed Tokens
    // Check Mint Token Amount reduced.
    // Claim Again
    // Clients Amount Same with First Claim
    // Total Issued So Far did not change
    // Mint Token Amount did not change

    const { vesting_schedule, vesting_data, mint_hbb, vesting_vault_hbb } = await getInitilizeParameter(provider);
    const publish_time = '2021.8.17';
    const mint_token_amount = 1_000_000.0;
    const tge_time = new Date(publish_time).getTime() / 1000;
    const userClaimToken = 1_000;
    let user1 = Keypair.generate();
    let user2 = Keypair.generate();

    await initialize(
      tge_time,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      vesting_vault_hbb,
    );

    await mintTo(provider, mint_hbb, vesting_vault_hbb, decimalToU64(mint_token_amount));

    await addUser(
      15,
      user1.publicKey,
      12,
      1_000_000_000,
      vesting_schedule,
    );

    await addUser(
      20,
      user2.publicKey,
      8,
      1_000_000_000,
      vesting_schedule,
    );

    const user_ata1 = await getUserAta(user1.secretKey, provider, mint_hbb);
    const user_ata2 = await getUserAta(user2.secretKey, provider, mint_hbb);

    let get_active_users = await program.account.vestingSchedule.fetch(vesting_schedule);
    assert.ok(get_active_users.len, 2);

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );


    let claim_time = new Date().getTime() / 1000;
    let mint_time = new Date(publish_time).getTime() / 1000;

    let delay_minutes = Math.floor((claim_time - mint_time) / 60);

    let calc_user_claim_token1 = Math.floor(0.15 * decimalToU64(userClaimToken) + 0.85 * decimalToU64(userClaimToken) * delay_minutes / (12 * 30.5 * 24 * 60));
    let calc_user_claim_token2 = Math.floor(0.2 * decimalToU64(userClaimToken) + 0.8 * decimalToU64(userClaimToken) * delay_minutes / (8 * 30.5 * 24 * 60));

    let token_amount_account1 = await getTokenAccount(provider, user_ata1);
    assert.ok(token_amount_account1.amount.eq(new anchor.BN(calc_user_claim_token1)));

    let token_amount_account2 = await getTokenAccount(provider, user_ata2);
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(calc_user_claim_token2)));

    let get_total_issue = await program.account.vestingData.fetch(vesting_data.publicKey);
    assert.ok(get_total_issue.totalIssuedSoFar.eq(new anchor.BN(calc_user_claim_token1 + calc_user_claim_token2)));

    let token_amount_vault_account = await getTokenAccount(provider, vesting_vault_hbb);
    assert.ok(token_amount_vault_account.amount.eq(new anchor.BN(1_000_000_000_000 - calc_user_claim_token1 - calc_user_claim_token2)));

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );

    assert.ok(token_amount_account1.amount.eq(new anchor.BN(calc_user_claim_token1)));
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(calc_user_claim_token2)));
    assert.ok(get_total_issue.totalIssuedSoFar.eq(new anchor.BN(calc_user_claim_token1 + calc_user_claim_token2)));
    assert.ok(token_amount_vault_account.amount.eq(new anchor.BN(1_000_000_000_000 - calc_user_claim_token1 - calc_user_claim_token2)));
  });

  it('After Period Claim Completed', async () => {

    // Add 2 clients
    // Check PendingToken Status Client Number match 2
    // Claim
    // All Clients Get Total Amount
    // Claim Again
    // Clients Amount Same with Total Amount

    const { vesting_schedule, vesting_data, mint_hbb, vesting_vault_hbb } = await getInitilizeParameter(provider);
    const publish_time = '2019.8.17';
    const mint_token_amount = 1_000_000.0;
    const tge_time = new Date(publish_time).getTime() / 1000;
    let user1 = Keypair.generate();
    let user2 = Keypair.generate();

    await initialize(
      tge_time,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      vesting_vault_hbb,
    );

    await mintTo(provider, mint_hbb, vesting_vault_hbb, decimalToU64(mint_token_amount));

    await addUser(
      15,
      user1.publicKey,
      12,
      1_000_000_000,
      vesting_schedule,
    );

    await addUser(
      20,
      user2.publicKey,
      8,
      1_000_000_000,
      vesting_schedule,
    );

    const user_ata1 = await getUserAta(user1.secretKey, provider, mint_hbb);
    const user_ata2 = await getUserAta(user2.secretKey, provider, mint_hbb);

    let get_active_users = await program.account.vestingSchedule.fetch(vesting_schedule);
    assert.ok(get_active_users.len, 2);

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );

    let token_amount_account1 = await getTokenAccount(provider, user_ata1);
    assert.ok(token_amount_account1.amount.eq(new anchor.BN(1_000_000_000)));

    let token_amount_account2 = await getTokenAccount(provider, user_ata2);
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(1_000_000_000)));

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );

    assert.ok(token_amount_account1.amount.eq(new anchor.BN(1_000_000_000)));
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(1_000_000_000)));
  });

  it('Before Publish Claim Completed', async () => {

    // Add 2 clients
    // Check PendingToken Status Client Number match 2
    // Claim
    // All Clients Get 0 Tokens
    // Claim Again
    // All Clients Also Get 0 Tokens

    const { vesting_schedule, vesting_data, mint_hbb, vesting_vault_hbb } = await getInitilizeParameter(provider);
    const publish_time = '2022.1.11';
    const mint_token_amount = 1_000_000.0;
    const tge_time = new Date(publish_time).getTime() / 1000;
    let user1 = Keypair.generate();
    let user2 = Keypair.generate();

    await initialize(
      tge_time,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      vesting_vault_hbb,
    );

    await mintTo(provider, mint_hbb, vesting_vault_hbb, decimalToU64(mint_token_amount));

    await addUser(
      15,
      user1.publicKey,
      12,
      1_000_000_000,
      vesting_schedule,
    );

    await addUser(
      20,
      user2.publicKey,
      8,
      1_000_000_000,
      vesting_schedule,
    );

    const user_ata1 = await getUserAta(user1.secretKey, provider, mint_hbb);
    const user_ata2 = await getUserAta(user2.secretKey, provider, mint_hbb);

    let get_active_users = await program.account.vestingSchedule.fetch(vesting_schedule);
    assert.ok(get_active_users.len, 2);

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );

    let token_amount_account1 = await getTokenAccount(provider, user_ata1);
    assert.ok(token_amount_account1.amount.eq(new anchor.BN(0)));

    let token_amount_account2 = await getTokenAccount(provider, user_ata2);
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(0)));

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );

    assert.ok(token_amount_account1.amount.eq(new anchor.BN(0)));
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(0)));
  });

  it('Non Vesting Period Claim Completed', async () => {

    // Add First client (Vesting Period = 0, Unlock Percent = 100)
    // Check PendingToken Status Client Number match 1
    // Claim
    // Client Get 1000 Tokens
    // Claim Again
    // Client Token Amount Still 1000 Tokens
    // Add Second client (Vesting Period = 0, Unlock Percent = 100)
    // Set Second Client to InActive
    // Claim
    // Second Clients Token Amount 0

    const { vesting_schedule, vesting_data, mint_hbb, vesting_vault_hbb } = await getInitilizeParameter(provider);
    const publish_time = '2021.1.11';
    const mint_token_amount = 1_000_000.0;
    const tge_time = new Date(publish_time).getTime() / 1000;
    let user1 = Keypair.generate();

    await initialize(
      tge_time,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      vesting_vault_hbb,
    );

    await mintTo(provider, mint_hbb, vesting_vault_hbb, decimalToU64(mint_token_amount));

    await addUser(
      100,
      user1.publicKey,
      0,
      1_000_000_000,
      vesting_schedule,
    );

    const user_ata1 = await getUserAta(user1.secretKey, provider, mint_hbb);

    let get_active_users = await program.account.vestingSchedule.fetch(vesting_schedule);
    assert.ok(get_active_users.len, 1);

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    let token_amount_account1 = await getTokenAccount(provider, user_ata1);
    assert.ok(token_amount_account1.amount.eq(new anchor.BN(1_000_000_000)));

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    assert.ok(token_amount_account1.amount.eq(new anchor.BN(1_000_000_000)));

    let user2 = Keypair.generate();
    const user_ata2 = await getUserAta(user2.secretKey, provider, mint_hbb);
    await addUser(
      100,
      user2.publicKey,
      0,
      1_000_000_000,
      vesting_schedule,
    );

    await removeUser(1, vesting_schedule);
    await claim(
      1,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user2.publicKey,
      user_ata2,
    );

    let token_amount_account2 = await getTokenAccount(provider, user_ata2);
    assert.ok(token_amount_account2.amount.eq(new anchor.BN(0)));
  });

  it('After Publish 1 Second Claim Completed', async () => {

    // Add client
    // Check PendingToken Status Client Number match 1
    // Claim
    // Client Get unlock_period * total_amount Tokens
    // Claim Again
    // Client Token Amount Still Same

    const { vesting_schedule, vesting_data, mint_hbb, vesting_vault_hbb } = await getInitilizeParameter(provider);
    const mint_token_amount = 1_000_000.0;
    const tge_time = new Date().getTime() / 1000 - 1;
    let user1 = Keypair.generate();
    const userClaimToken = 1_000;

    await initialize(
      tge_time,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      vesting_vault_hbb,
    );

    await mintTo(provider, mint_hbb, vesting_vault_hbb, decimalToU64(mint_token_amount));

    await addUser(
      20,
      user1.publicKey,
      12,
      1_000_000_000,
      vesting_schedule,
    );

    const user_ata1 = await getUserAta(user1.secretKey, provider, mint_hbb);

    let get_active_users = await program.account.vestingSchedule.fetch(vesting_schedule);
    assert.ok(get_active_users.len, 1);

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    let token_amount_account1 = await getTokenAccount(provider, user_ata1);
    assert.ok(token_amount_account1.amount.eq(new anchor.BN(1_000_000_000 * 0.2)));

    await claim(
      0,
      provider.wallet.publicKey,
      vesting_schedule,
      vesting_data,
      user1.publicKey,
      user_ata1,
    );

    assert.ok(token_amount_account1.amount.eq(new anchor.BN(1_000_000_000 * 0.2)));
  });
});
