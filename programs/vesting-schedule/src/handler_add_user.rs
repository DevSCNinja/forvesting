use crate::{vesting_operations, AddUser, VestingUser};
use anchor_lang::prelude::*;

pub fn process(
    ctx: Context<AddUser>,
    unlocked_at_tge: u8,
    user_pubkey: Pubkey,
    unlocking_period: u8,
    planned_tokens: u64,
) -> ProgramResult {
    let active_user = VestingUser::new(
        unlocked_at_tge,
        user_pubkey,
        unlocking_period,
        planned_tokens,
    );

    vesting_operations::add_user_active(
        active_user,
        &mut ctx.accounts.vesting_schedule.load_mut()?,
    )?;

    Ok(())
}
