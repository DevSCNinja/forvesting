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
    let schedule = &mut ctx.accounts.vesting_schedule.load_init()?;

    vesting_operations::initialize_vesting_data(vesting_data, ctx.accounts.vesting_vault.key(), pda_vesting_vault, tge_timestamp);
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
