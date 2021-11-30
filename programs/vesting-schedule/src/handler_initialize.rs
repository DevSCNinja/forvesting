use super::*;
use crate::utils::pda::VestingPdaAddress;
use crate::{utils, vesting_operations, Initialize};

pub fn process(ctx: Context<Initialize>, tge_timestamp: u64) -> ProgramResult {
    let pda_vesting_vault = utils::pda::make_vesting_pda_pubkey(
        ctx.accounts.admin.key,
        utils::utils::TOKEN_MINT_TAG,
        ctx.program_id,
    );

    transfer_set_authority(&ctx, &pda_vesting_vault);

    let vesting_data = &mut ctx.accounts.vesting_data;
    vesting_data.vesting_vault = ctx.accounts.vesting_vault.key();
    vesting_data.vesting_vault_authority = pda_vesting_vault.key;
    vesting_data.vesting_vault_authority_seed = pda_vesting_vault.seed;
    vesting_data.total_issued_so_far = 0;
    vesting_data.tge_timestamp = tge_timestamp;

    let schedule = &mut ctx.accounts.vesting_schedule.load_init()?;

    vesting_operations::initialize_users(schedule);

    Ok(())
}

pub fn transfer_set_authority(ctx: &Context<Initialize>, authority_pda: &VestingPdaAddress) {
    token::set_authority(
        ctx.accounts.to_set_authority(),
        spl_token::instruction::AuthorityType::AccountOwner,
        Some(authority_pda.key),
    )
    .unwrap();
}
