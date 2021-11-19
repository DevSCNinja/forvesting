use anchor_lang::prelude::Pubkey;

#[derive(Debug)]
pub struct VestingPdaAddress {
    pub key: Pubkey,
    pub seed: u8,
}

pub fn make_vesting_pda_pubkey(owner: &Pubkey, tag: &str, program: &Pubkey) -> VestingPdaAddress {
    let seeds = &[owner.as_ref(), tag.as_ref()];
    let (key, seed) = Pubkey::find_program_address(seeds, program);
    VestingPdaAddress { key, seed }
}

pub fn make_vesting_pda_seeds(owner: &Pubkey, tag: &str) -> [Vec<u8>; 2] {
    let signer_seeds = [owner.as_ref().to_owned(), tag.as_bytes().to_owned()];
    signer_seeds
}
