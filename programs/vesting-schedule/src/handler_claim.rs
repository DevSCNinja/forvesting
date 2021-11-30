use crate::{vesting_operations, Claim};
use anchor_lang::prelude::*;

pub fn process(ctx: Context<Claim>,index: u64) -> ProgramResult {
    let schedule = &mut ctx.accounts.vesting_schedule.load_mut()?;
    let vesting_data = &mut ctx.accounts.vesting_data;
    vesting_operations::claim(
        schedule, 
        index, 
        ctx.accounts.claim_user.key(),
        &ctx.accounts.owner,
        &ctx.accounts.claim_user_ata,
        &ctx.accounts.vesting_vault,
        &ctx.accounts.vesting_vault_authority,
        vesting_data,
        &ctx.accounts.token_program.to_account_info()
    )
}
