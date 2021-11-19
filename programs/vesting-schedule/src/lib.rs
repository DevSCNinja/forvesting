use anchor_lang::prelude::*;
pub mod pda;
pub mod tokenoper;
use anchor_spl::token::Token;
use anchor_spl::token::{self, SetAuthority};
declare_id!("APxgpqd2EkAzJBBymwss5k7Wp1DSmjjggVx5V6EQc6cR");

#[program]
pub mod vesting_schedule {
    use super::*;

    pub const TOKEN_MINT_TAG: &str = "tmt";

    pub fn initialize(ctx: Context<Initialize>, tge_timestamp: u64) -> ProgramResult {
        // This instruction happens once
        // and is invoked by the admin

        let pda_vesting_vault =
            pda::make_vesting_pda_pubkey(ctx.accounts.admin.key, TOKEN_MINT_TAG, ctx.program_id);

        utils::transfer_set_authority(&ctx, &pda_vesting_vault);

        let vesting_data = &mut ctx.accounts.vesting_data;
        vesting_data.vesting_vault = ctx.accounts.vesting_vault.key();
        vesting_data.vesting_vault_authority = pda_vesting_vault.key;
        vesting_data.vesting_vault_authority_seed = pda_vesting_vault.seed;
        vesting_data.total_issued_so_far = 0;
        vesting_data.tge_timestamp = tge_timestamp;

        let schedule = &mut ctx.accounts.vesting_schedule;

        let hardcoded_schedules = utils::get_hardcoded_schedule();

        // basically set them all here upfront on the chain
        for (i, hardcoded_schedule) in hardcoded_schedules.iter().enumerate() {
            schedule.data[i].user = hardcoded_schedule.user;
            schedule.data[i].planned_tokens = hardcoded_schedule.planned_tokens;
            schedule.data[i].claimed_tokens = 0;
            schedule.data[i].unlocked_at_tge = hardcoded_schedule.unlocked_at_tge;
            schedule.data[i].unlocking_period = hardcoded_schedule.unlocking_period;
        }

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, index: u64) -> ProgramResult {
        // check how much the user has claimed
        // see how much is outstanding based on the

        // find user id
        let schedule = &mut ctx.accounts.vesting_schedule;
        if schedule.data[index as usize].user == ctx.accounts.claim_user.key() {
            let user_id = index as usize;

            let now = Clock::get().unwrap().unix_timestamp;
            let tge_timestamp = ctx.accounts.vesting_data.tge_timestamp;
            let tokens_claimed = schedule.data[user_id].claimed_tokens;
            let tokens_entitled_until_now = utils::calculate_entitled_amount(
                schedule.data[user_id].planned_tokens,
                schedule.data[user_id].unlocked_at_tge as u64,
                tge_timestamp,
                schedule.data[user_id].unlocking_period as u64,
                now as u64,
            );

            let can_claim_now = if tokens_entitled_until_now > schedule.data[user_id].planned_tokens
            {
                schedule.data[user_id]
                    .planned_tokens
                    .checked_sub(tokens_claimed)
                    .unwrap()
            } else {
                tokens_entitled_until_now
                    .checked_sub(tokens_claimed)
                    .unwrap()
            };

            schedule.data[user_id].claimed_tokens = schedule.data[user_id]
                .claimed_tokens
                .checked_add(can_claim_now)
                .unwrap();

            msg!("Can Claim Now {}", can_claim_now);

            // Look at hubble: spltoken transfer
            if can_claim_now > 0 {
                tokenoper::vesting_transfer(
                    can_claim_now,
                    &ctx.accounts.owner,
                    &ctx.accounts.claim_user_ata,
                    &ctx.accounts.vesting_vault,
                    &ctx.accounts.vesting_vault_authority,
                    ctx.accounts.vesting_data.vesting_vault_authority_seed,
                    &ctx.accounts.token_program.to_account_info(),
                );
                ctx.accounts.vesting_data.total_issued_so_far = ctx
                    .accounts
                    .vesting_data
                    .total_issued_so_far
                    .checked_add(can_claim_now)
                    .unwrap();
            }
        }

        Ok(())
    }
}

impl<'a, 'b, 'c, 'info> Initialize<'info> {
    pub fn to_set_authority(&self) -> CpiContext<'a, 'b, 'c, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vesting_vault.clone(),
            current_authority: self.admin.clone(),
        };
        let cpi_program = self.token_program.to_account_info().clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(signer)]
    pub admin: AccountInfo<'info>,
    #[account(init, payer = admin)]
    pub vesting_schedule: Account<'info, VestingSchedule>,
    #[account(init, payer = admin)]
    pub vesting_data: Account<'info, VestingData>,
    #[account(mut)]
    pub vesting_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub vesting_schedule: Account<'info, VestingSchedule>,
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

#[account]
#[derive(Debug, Default)]
pub struct VestingData {
    pub tge_timestamp: u64,
    pub vesting_vault: Pubkey,
    pub vesting_vault_authority: Pubkey,
    pub vesting_vault_authority_seed: u8,
    pub total_issued_so_far: u64,
}

//#[account]
#[derive(Clone, AnchorDeserialize, AnchorSerialize, Copy, Debug, Default, PartialEq, Eq)]
pub struct VestingUser {
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
    pub fn new(percent: u8, pubkey: Pubkey, period: u8, total_token: u64) -> Self {
        Self {
            user: pubkey,
            planned_tokens: total_token,
            claimed_tokens: 0,
            unlocked_at_tge: percent,
            unlocking_period: period,
        }
    }
}

#[account]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct VestingSchedule {
    // TGE = TOKEN GENESIS EVENT
    pub data: [VestingUser; 10],
}

#[error]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
}

mod utils {

    use super::*;
    use crate::pda::VestingPdaAddress;
    use crate::VestingUser;
    use std::str::FromStr;

    pub fn get_hardcoded_schedule() -> Vec<VestingUser> {
        let hardcoded_schedule = vec![
            VestingUser::new(
                15,
                Pubkey::from_str("29GPMU5gtBDbd368EwquqTmo33tKgvneAK9REmmxkqm8").unwrap(),
                12,
                1_000_000_000_000,
            ),
            VestingUser::new(
                20,
                Pubkey::from_str("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L").unwrap(),
                12,
                100_000_000_000,
            ),
            VestingUser::new(
                15,
                Pubkey::from_str("Ej8ZpQbTn53oT9pmAXSZp5Yju67uHP2dM4scW6Cd5sNe").unwrap(),
                12,
                10_000_000_000,
            ),
            VestingUser::new(
                15,
                Pubkey::from_str("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L").unwrap(),
                12,
                10_000_000_000,
            ),
            VestingUser::new(
                15,
                Pubkey::from_str("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L").unwrap(),
                18,
                10_000_000_000,
            ),
            VestingUser::new(
                15,
                Pubkey::from_str("5xot9PVkphiX2adznghwrAuxGs2zeWisNSxMW6hU6Hkj").unwrap(),
                18,
                10_000_000_000,
            ),
            VestingUser::new(
                15,
                Pubkey::from_str("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L").unwrap(),
                18,
                10_000_000_000,
            ),
            VestingUser::new(
                15,
                Pubkey::from_str("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L").unwrap(),
                18,
                10_000_000_000,
            ),
            VestingUser::new(
                15,
                Pubkey::from_str("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L").unwrap(),
                18,
                10_000_000_000,
            ),
            VestingUser::new(
                15,
                Pubkey::from_str("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L").unwrap(),
                18,
                10_000_000_000,
            ),
        ];
        hardcoded_schedule
    }

    pub fn calculate_entitled_amount(
        tge_amount: u64,
        tge_unlock: u64,
        tge_timestamp: u64,
        vesting_period: u64,
        now_timestamp: u64,
    ) -> u64 {
        if tge_timestamp >= now_timestamp {
            0
        } else {
            let basic_tge = tge_amount
                .checked_mul(tge_unlock)
                .unwrap()
                .checked_div(100)
                .unwrap();
            let time_passed = now_timestamp
                .checked_sub(tge_timestamp)
                .unwrap()
                .checked_div(3600 * 24)
                .unwrap();
            let remain_tge = tge_amount
                .checked_mul(100 - tge_unlock)
                .unwrap()
                .checked_div(100)
                .unwrap();
            let monthday = 305;
            basic_tge
                .checked_add(
                    remain_tge
                        .checked_mul(time_passed)
                        .unwrap()
                        .checked_mul(10)
                        .unwrap()
                        .checked_div(vesting_period.checked_mul(monthday).unwrap())
                        .unwrap(),
                )
                .unwrap()
            //            return (basic_tge as f64 + remain_tge as f64 * time_passed as f64 / (vesting_period as f64 * 30.5)) as u64;
        }
    }

    pub fn transfer_set_authority(ctx: &Context<Initialize>, authority_pda: &VestingPdaAddress) {
        token::set_authority(
            ctx.accounts.to_set_authority(),
            spl_token::instruction::AuthorityType::AccountOwner,
            Some(authority_pda.key),
        )
        .unwrap();
    }
}
