use crate::{tokenoperation, utils, Claim};
use anchor_lang::prelude::*;

pub fn process(ctx: Context<Claim>, index: u64) -> ProgramResult {
    let schedule = &mut ctx.accounts.vesting_schedule.load_mut()?;

    if schedule.data[index as usize].user == ctx.accounts.claim_user.key()
        && schedule.data[index as usize].status == utils::utils::EventStatus::PendingToken as u8
    {
        let user_id = index as usize;

        let now = Clock::get().unwrap().unix_timestamp;
        let calc_claim_tokens = tokenoperation::schedule::calculate_entitled_amount(
            schedule.data[user_id].planned_tokens,
            schedule.data[user_id].unlocked_at_tge as u64,
            schedule.data[user_id].unlocking_period as u64,
            schedule.data[user_id].claimed_tokens,
            ctx.accounts.vesting_data.tge_timestamp,
            now as u64,
        );

        if calc_claim_tokens == Err(crate::ErrorCode::InvalidInput.into()) {
            return Err(crate::ErrorCode::InvalidInput.into());
        }

        let can_claim_now = calc_claim_tokens.unwrap();

        msg!("Can Claim Now {}", can_claim_now);

        if can_claim_now > 0 {
            tokenoperation::tokenoper::vesting_transfer(
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

        schedule.data[user_id].claimed_tokens = schedule.data[user_id]
            .claimed_tokens
            .checked_add(can_claim_now)
            .unwrap();
    }

    Ok(())
}
