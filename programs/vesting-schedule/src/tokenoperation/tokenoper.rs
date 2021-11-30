use crate::utils;
use crate::utils::pda::{self};
use anchor_lang::{prelude::AccountInfo, CpiContext};
use anchor_spl::token::{self, Transfer};

#[allow(clippy::too_many_arguments)]
pub fn vesting_transfer<'info>(
    amount: u64,
    owner: &AccountInfo<'info>,
    to_vault: &AccountInfo<'info>,
    from_vault: &AccountInfo<'info>,
    from_vault_authority: &AccountInfo<'info>,
    from_vault_authority_seed: u8,
    token_program: &AccountInfo<'info>,
) {
    let from_vault_seed: u8 = from_vault_authority_seed;
    let from_vault_authority_bump = vec![from_vault_seed];
    let from_vault_authority_pda_seeds =
        pda::make_vesting_pda_seeds(owner.key, utils::utils::TOKEN_MINT_TAG);
    let seeds = [
        from_vault_authority_pda_seeds[0].as_ref(),
        from_vault_authority_pda_seeds[1].as_ref(),
        from_vault_authority_bump.as_ref(),
    ];
    let signer = &[&seeds[..]];

    let cpi_transfer_accounts = Transfer {
        from: from_vault.clone(),
        to: to_vault.clone(),
        authority: from_vault_authority.clone(),
    };

    let cpi_ctx = CpiContext::new(token_program.clone(), cpi_transfer_accounts).with_signer(signer);
    token::transfer(cpi_ctx, amount).unwrap();
}
