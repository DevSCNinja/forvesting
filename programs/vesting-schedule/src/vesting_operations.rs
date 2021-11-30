use crate::{utils, ErrorCode, VestingSchedule, VestingUser};
use std::cell::RefMut;
pub const MAX_VESTING_USERS: usize = 300;

pub fn initialize_users(queue: &mut RefMut<VestingSchedule>) {
    queue.len = 0;
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

mod tests {
    use std::cell::RefCell;
    use crate::{VestingSchedule, VestingUser, ErrorCode, VestingData};
    use anchor_lang::prelude::*;
    use std::str::FromStr;

    #[test]
    pub fn test_add_user() {
        let vesting_schedule = RefCell::new(VestingSchedule::default());
        let add_user1 = VestingUser::new(
            15,
            Pubkey::from_str("29GPMU5gtBDbd368EwquqTmo33tKgvneAK9REmmxkqm8").unwrap(),
            12,
            1_000_000_000,
        );
        let add_user2 = VestingUser::new(
            15,
            Pubkey::from_str("8v1DhJaewvhbhDmptNrkYig7YFcExsRKteR3cYjLw2iy").unwrap(),
            12,
            1_000_000_000,
        );
        super::add_user_active(add_user1, &mut vesting_schedule.borrow_mut());
        super::add_user_active(add_user2, &mut vesting_schedule.borrow_mut());

        assert_eq!(2, super::len(&mut vesting_schedule.borrow_mut()));
        super::remove_user_active(&mut vesting_schedule.borrow_mut(),1);
        assert_eq!(1, super::len(&mut vesting_schedule.borrow_mut()));
    }
}