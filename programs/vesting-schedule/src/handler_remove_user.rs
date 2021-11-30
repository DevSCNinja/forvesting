use crate::{vesting_operations, RemoveUser};
use anchor_lang::prelude::*;

pub fn process(ctx: Context<RemoveUser>, index: u64) -> ProgramResult {
    vesting_operations::remove_user_active(
        &mut ctx.accounts.vesting_schedule.load_mut()?,
        index as usize,
    );

    Ok(())
}
