use anchor_lang::prelude::*;
pub mod handler_add_user;
pub mod handler_initialize;
pub mod handler_remove_user;
pub mod handler_claim;
pub mod tokenoperation;
pub mod utils;
pub mod vesting_operations;
use anchor_spl::token::Token;
use anchor_spl::token::{self, SetAuthority};
declare_id!("APxgpqd2EkAzJBBymwss5k7Wp1DSmjjggVx5V6EQc6cR");

#[program]
pub mod vesting_schedule {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, tge_timestamp: u64) -> ProgramResult {
        // good to go
        handler_initialize::process(ctx, tge_timestamp)
    }

    pub fn add_user(
        ctx: Context<AddUser>,
        unlocked_at_tge: u8,
        user_pubkey: Pubkey,
        unlocking_period: u8,
        planned_tokens: u64,
    ) -> ProgramResult {
        handler_add_user::process(
            ctx,
            unlocked_at_tge,
            user_pubkey,
            unlocking_period,
            planned_tokens,
        )
    }

    pub fn remove_user(ctx: Context<RemoveUser>, index: u64) -> ProgramResult {
        handler_remove_user::process(ctx, index)
    }

    pub fn claim(ctx: Context<Claim>, index: u64) -> ProgramResult {
        handler_claim::process(ctx, index)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(signer)]
    pub admin: AccountInfo<'info>,
    #[account(init, payer = admin)]
    pub vesting_data: ProgramAccount<'info, VestingData>,
    #[account(zero)]
    pub vesting_schedule: Loader<'info, VestingSchedule>,
    #[account(mut)]
    pub vesting_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub vesting_schedule: Loader<'info, VestingSchedule>,
    #[account(mut)]
    pub vesting_data: Account<'info, VestingData>,
    pub claim_user: AccountInfo<'info>,
    #[account(mut)]
    pub claim_user_ata: AccountInfo<'info>,
    #[account(mut)]
    pub vesting_vault: AccountInfo<'info>,
    pub vesting_vault_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddUser<'info> {
    #[account(mut)]
    pub vesting_schedule: Loader<'info, VestingSchedule>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveUser<'info> {
    #[account(mut)]
    pub vesting_schedule: Loader<'info, VestingSchedule>,
    pub system_program: Program<'info, System>,
}

#[error]
#[derive(PartialEq, Eq)]
pub enum ErrorCode {
    #[msg("Input Value is not True")]
    InvalidInput,
    #[msg("VestingUsers Data Array is full")]
    VestingUserDataFull,
}

#[account]
#[derive(Debug, Default)]
pub struct VestingData {
    pub tge_timestamp: u64,
    pub vesting_vault: Pubkey,
    pub vesting_vault_authority: Pubkey,
    pub vesting_vault_authority_seed: u8,
    pub total_issued_so_far: u64,
}

#[zero_copy]
#[derive(Debug, PartialEq, Eq)]
pub struct VestingUser {
    // PendingToken : 1 InActive : 0
    pub status: u8,
    // The solana account address
    pub user: Pubkey,
    // how many HBB tokens the user is entitled to
    // since the HBB token has 9 decimals, 1 token = 1.000.000.000
    pub planned_tokens: u64,
    // cumulative amount of tokens claimed
    pub claimed_tokens: u64,
    // percentage 15% is represented as 15, so scaled by 100
    // some people have 20% at TGE, some people have 15% at TGE
    // 15% -> 15 -> 15/100
    pub unlocked_at_tge: u8,
    // number of months: 1, 12, or 18
    // after unlocking_period -> everything should be claimed
    // claimed_tokens === planned_tokens
    pub unlocking_period: u8,
}

impl VestingUser {
    pub fn new(percent: u8, pubkey: Pubkey, period: u8, client_planned_token_amount: u64) -> Self {
        Self {
            status: utils::utils::EventStatus::PendingToken as u8,
            user: pubkey,
            planned_tokens: client_planned_token_amount,
            claimed_tokens: 0,
            unlocked_at_tge: percent,
            unlocking_period: period,
        }
    }
}

#[account(zero_copy)]
pub struct VestingSchedule {
    // TGE = TOKEN GENESIS EVENT
    pub len: u64,
    pub data: [VestingUser; 300],
}

impl Default for VestingSchedule {
    #[cfg(not(test))]
    fn default() -> Self {
        unimplemented!()
    }

    #[cfg(test)]
    #[inline(never)]
    fn default() -> Self {
        let data: [VestingUser; utils::utils::MAX_VESTING_USERS] =
            unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

        VestingSchedule { data, len: 0 }
    }
}