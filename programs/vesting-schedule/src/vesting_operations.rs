use crate::{utils, ErrorCode, VestingSchedule, VestingUser, tokenoperation, VestingData};
use std::cell::RefMut;
use anchor_lang::prelude::*;
use crate::utils::pda::VestingPdaAddress;

pub fn initialize_users(queue: &mut RefMut<VestingSchedule>) {
    queue.len = 0;
}

pub fn initialize_vesting_data(vesting_data: &mut VestingData, vesting_vault_pubkey: Pubkey, pda_vesting_vault: VestingPdaAddress, tge_timestamp: u64) {
    vesting_data.vesting_vault = vesting_vault_pubkey;
    vesting_data.vesting_vault_authority = pda_vesting_vault.key;
    vesting_data.vesting_vault_authority_seed = pda_vesting_vault.seed;
    vesting_data.total_issued_so_far = 0;
    vesting_data.tge_timestamp = tge_timestamp;
}

pub fn add_user_active(
    add_user: VestingUser,
    queue: &mut RefMut<VestingSchedule>,
) -> Result<(), ErrorCode> {
    let user_index = get_next_index(queue)?;
    queue.len += 1;
    queue.data[user_index as usize] = add_user;
    Ok(())
}

pub fn get_next_index(queue: &mut RefMut<VestingSchedule>) -> Result<usize, ErrorCode> {
    // Gets next available index
    // When we move to derived accounts, this will not longer be an issue
    for (i, user) in (*queue).data.iter().enumerate() {
        if user.status == (utils::utils::EventStatus::Inactive as u8) {
            return Ok(i);
        }
    }
    Err(ErrorCode::VestingUserDataFull)
}

pub fn remove_user_active(queue: &mut RefMut<VestingSchedule>, index: usize) {
    let mut remove_user: VestingUser = (*queue).data[index];

    remove_user.status = utils::utils::EventStatus::Inactive as u8;
    queue.len -= 1;
    queue.data[index] = remove_user;
}

pub fn len(queue: &mut RefMut<VestingSchedule>) -> usize {
    (*queue)
        .data
        .iter()
        .filter(|event| event.status == utils::utils::EventStatus::PendingToken as u8)
        .count()
}


pub fn claim<'info>(
    schedule: &mut RefMut<VestingSchedule>, 
    index: u64, 
    claim_user: Pubkey,
    owner: &AccountInfo<'info>,
    to_vault: &AccountInfo<'info>,
    from_vault: &AccountInfo<'info>,
    from_vault_authority: &AccountInfo<'info>,
    vesting_data: &mut VestingData,
    token_program: &AccountInfo<'info>
) -> ProgramResult {
    if schedule.data[index as usize].user == claim_user
        && schedule.data[index as usize].status == utils::utils::EventStatus::PendingToken as u8
    {
        let user_id = index as usize;
        let now = Clock::get().unwrap().unix_timestamp;
        let can_claim_now = tokenoperation::schedule::calculate_entitled_amount(
            schedule.data[user_id].planned_tokens,
            schedule.data[user_id].unlocked_at_tge as u64,
            schedule.data[user_id].unlocking_period as u64,
            schedule.data[user_id].claimed_tokens,
            vesting_data.tge_timestamp,
            now as u64,
        )?;

        msg!("Can Claim Now {}", can_claim_now);

        if can_claim_now > 0 {
            tokenoperation::tokenoper::vesting_transfer(
                can_claim_now,
                owner,
                to_vault,
                from_vault,
                from_vault_authority,
                vesting_data.vesting_vault_authority_seed,
                token_program,
            );
            vesting_data.total_issued_so_far = vesting_data
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use crate::{VestingSchedule, VestingUser, ErrorCode, utils};
    use anchor_lang::prelude::*;
    use std::str::FromStr;

    #[test]
    pub fn test_add_user() -> Result<(), ErrorCode> {
        let vesting_schedule = RefCell::new(VestingSchedule::default());
        let add_user1 = VestingUser::new(
            15,
            Pubkey::from_str("29GPMU5gtBDbd368EwquqTmo33tKgvneAK9REmmxkqm8").unwrap(),
            12,
            1_000_000_000,
        );
        let add_user2 = VestingUser::new(
            20,
            Pubkey::from_str("8v1DhJaewvhbhDmptNrkYig7YFcExsRKteR3cYjLw2iy").unwrap(),
            8,
            1_000_000_000,
        );
        super::add_user_active(add_user1, &mut vesting_schedule.borrow_mut())?;
        super::add_user_active(add_user2, &mut vesting_schedule.borrow_mut())?;

        assert_eq!(15, vesting_schedule.borrow_mut().data[0].unlocked_at_tge);
        assert_eq!(20, vesting_schedule.borrow_mut().data[1].unlocked_at_tge);
        assert_eq!(Pubkey::from_str("29GPMU5gtBDbd368EwquqTmo33tKgvneAK9REmmxkqm8").unwrap(), vesting_schedule.borrow_mut().data[0].user);
        assert_eq!(Pubkey::from_str("8v1DhJaewvhbhDmptNrkYig7YFcExsRKteR3cYjLw2iy").unwrap(), vesting_schedule.borrow_mut().data[1].user);
        assert_eq!(utils::utils::EventStatus::PendingToken as u8, vesting_schedule.borrow_mut().data[0].status);
        assert_eq!(utils::utils::EventStatus::PendingToken as u8, vesting_schedule.borrow_mut().data[1].status);
        assert_eq!(2, super::len(&mut vesting_schedule.borrow_mut()));
        super::remove_user_active(&mut vesting_schedule.borrow_mut(),1);
        assert_eq!(1, super::len(&mut vesting_schedule.borrow_mut()));
        assert_eq!(utils::utils::EventStatus::PendingToken as u8, vesting_schedule.borrow_mut().data[0].status);
        assert_eq!(utils::utils::EventStatus::Inactive as u8, vesting_schedule.borrow_mut().data[1].status);
        Ok(())
    }
}