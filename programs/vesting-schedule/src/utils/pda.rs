use anchor_lang::prelude::Pubkey;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::{utils};

    #[test]
    fn test_generate_seed() {
        let owner = Pubkey::from_str("BSKmmWSyV42Pw3AwZHRFyiHpcBpQ3FyCYeHVecUanb6y").unwrap();
        let program_id = Pubkey::from_str("7SeC6f66GuxEEE1PHmAabu1SYbLnayJkWNE6127BNUYc").unwrap();
        let pdapubkey = make_vesting_pda_pubkey(
            &owner.clone(),
            utils::utils::TOKEN_MINT_TAG,
            &program_id,
        );
        println!("pubkey {:?}", &pdapubkey.key);
        println!("pubkey {:?}", &pdapubkey.seed);
        assert_eq!(Pubkey::from_str("6yUirNTpsj1jfpfaT1pyBcc772mYC2fwRSpKZ52pS6CM").unwrap(), pdapubkey.key);
    }
    #[test]
    fn test_create_borrow_account() {
        // Expecting 72DY6RSrCjYXEgPt2Yaz5jm3CdUDVrsemgtzxcEUj2fj
        let owner = Pubkey::from_str("BSKmmWSyV42Pw3AwZHRFyiHpcBpQ3FyCYeHVecUanb6y").unwrap();
        let program_id = Pubkey::from_str("7SeC6f66GuxEEE1PHmAabu1SYbLnayJkWNE6127BNUYc").unwrap();
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"vesting_schedule_seed"], &program_id);
        let x = Pubkey::create_with_seed(&owner, "vesting_schedule_seed", &program_id);
        println!("key {:?}", pda);
        println!("x {:?}", x);
    }
}
