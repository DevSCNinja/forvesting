use crate::ErrorCode;
pub fn calculate_entitled_amount(
    total_amount_for_user: u64,
    tge_unlock_percent: u64,
    vesting_period: u64,
    tokens_claimed: u64,
    tge_timestamp: u64,
    now_timestamp: u64,
) -> Result<u64, ErrorCode> {
    if vesting_period == 0 && tge_unlock_percent < 100 || tge_unlock_percent > 100 {
        return Err(ErrorCode::InvalidInput.into());
    }

    if now_timestamp <= tge_timestamp {
        return Ok(0);
    }

    let amount_at_tge = total_amount_for_user
        .checked_mul(tge_unlock_percent)
        .unwrap()
        .checked_div(100)
        .unwrap();
    let total_minutes_in_period = 305 * vesting_period * 60 * 24 / 10;
    let total_minutes_so_far = now_timestamp
        .checked_sub(tge_timestamp)
        .unwrap()
        .checked_div(60)
        .unwrap();
    let amount_after_tge = total_amount_for_user
        .checked_sub(amount_at_tge)
        .unwrap()
        .checked_mul(u64::min(total_minutes_so_far, total_minutes_in_period))
        .unwrap()
        .checked_div(total_minutes_in_period)
        .unwrap_or(0);

    let amount_entitled = amount_at_tge.checked_add(amount_after_tge).unwrap();
    let amount_now = amount_entitled.checked_sub(tokens_claimed).unwrap();
    Ok(amount_now)
}
#[cfg(test)]
mod tests {

    use crate::ErrorCode;
    use quickcheck_macros::quickcheck;

    #[test]
    fn test_vesting_period() {
        let err = super::calculate_entitled_amount(1_000_000, 20, 0, 0, 1, 4);
        assert_eq!(err.err(), Some(ErrorCode::InvalidInput.into()));
    }

    #[test]
    fn test_vesting_period_all() {
        let actual = super::calculate_entitled_amount(1_000_000, 100, 0, 0, 1, 4).unwrap();
        let expected = 1_000_000;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_nothing_yet_before() {
        let actual = super::calculate_entitled_amount(1_000_000, 20, 12, 0, 1, 0).unwrap();
        let expected = 0;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_nothing_yet_same() {
        let actual = super::calculate_entitled_amount(1_000_000, 20, 12, 0, 0, 0).unwrap();
        let expected = 0;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_only_tge() {
        let actual = super::calculate_entitled_amount(1_000_000, 20, 12, 0, 0, 1).unwrap();
        let expected = 200_000;
        println!("Actual {}, expected {}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_after_some_time_first_passed() {
        let tge_timestamp = 0;
        let tge_unlock_percent = 20;
        let vesting_months = 12;

        let months_passed = 4;
        let now = tge_timestamp + months_passed * 732 * 60 * 60;

        let actual = super::calculate_entitled_amount(
            1_000_000,
            tge_unlock_percent,
            vesting_months,
            0,
            tge_timestamp,
            now,
        )
        .unwrap();
        let expected = 466_666;
        println!("Actual {}, expected {}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_after_some_time_passed() {
        let tge_timestamp = 0;
        let tge_unlock_percent = 20;
        let vesting_months = 12;

        let months_passed = 6;
        let now = tge_timestamp + months_passed * 732 * 60 * 60;

        let actual = super::calculate_entitled_amount(
            1_000_000,
            tge_unlock_percent,
            vesting_months,
            0,
            tge_timestamp,
            now,
        )
        .unwrap();
        let expected = 600_000;
        println!("Actual {}, expected {}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_after_all_time_passed() {
        let tge_timestamp = 0;
        let tge_unlock_percent = 20;
        let vesting_months = 12;

        let months_passed = 20;
        let now = tge_timestamp + months_passed * 30 * 24 * 60 * 60;

        let actual = super::calculate_entitled_amount(
            1_000_000,
            tge_unlock_percent,
            vesting_months,
            0,
            tge_timestamp,
            now,
        )
        .unwrap();
        let expected = 1_000_000;
        println!("Actual {}, expected {}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_step_claim() {
        let tge_timestamp = 0;
        let tge_unlock_percent = 20;
        let vesting_months = 12;

        let first_months_passed = 1;
        let second_months_passed = 2;
        let third_months_passed = 4;
        let mut now = tge_timestamp + first_months_passed * 732 * 60 * 60;

        let first_actual = super::calculate_entitled_amount(
            1_000_000,
            tge_unlock_percent,
            vesting_months,
            0,
            tge_timestamp,
            now,
        )
        .unwrap();

        now = tge_timestamp + second_months_passed * 732 * 60 * 60;

        let second_actual = super::calculate_entitled_amount(
            1_000_000,
            tge_unlock_percent,
            vesting_months,
            first_actual,
            tge_timestamp,
            now,
        )
        .unwrap();

        now = tge_timestamp + third_months_passed * 732 * 60 * 60;

        let third_actual = super::calculate_entitled_amount(
            1_000_000,
            tge_unlock_percent,
            vesting_months,
            second_actual + first_actual,
            tge_timestamp,
            now,
        )
        .unwrap();

        let fianl_actual = super::calculate_entitled_amount(
            1_000_000,
            tge_unlock_percent,
            vesting_months,
            0,
            tge_timestamp,
            now,
        )
        .unwrap();

        println!(
            "Step Actual {}, expected {}",
            first_actual + second_actual + third_actual,
            fianl_actual
        );
        assert_eq!(first_actual + second_actual + third_actual, fianl_actual);
    }

    #[test]
    fn test_idempotence_all() {
        let total_user_amount = 1_000_000;
        let unlock_percent = 100;
        let vesting_period = 0;
        let token_claimed = 0;
        let now = 1;
        let can_claim_now = super::calculate_entitled_amount(
            total_user_amount,
            unlock_percent,
            vesting_period,
            token_claimed,
            0,
            now,
        )
        .unwrap();
        // try again
        let extra = super::calculate_entitled_amount(
            total_user_amount,
            unlock_percent,
            vesting_period,
            token_claimed + can_claim_now,
            0,
            now,
        )
        .unwrap();
        assert_eq!(extra, 0);
    }

    #[quickcheck]
    fn test_never_more_than_issued(
        tge_unlock_percent: u32,
        vesting_period: u32,
        tge_timestamp: u32,
        now_timestamp: u32,
    ) {
        let total_user_amount = 1_000_000;
        let token_claimed = 0;
        if tge_unlock_percent > 100 || vesting_period == 0 && tge_unlock_percent < 100 {
            let err = super::calculate_entitled_amount(
                total_user_amount,
                tge_unlock_percent as u64,
                vesting_period as u64,
                token_claimed,
                tge_timestamp as u64,
                now_timestamp as u64,
            );
            assert_eq!(err.err(), Some(ErrorCode::InvalidInput.into()));
        } else {
            let actual = super::calculate_entitled_amount(
                total_user_amount,
                tge_unlock_percent as u64,
                vesting_period as u64,
                token_claimed,
                tge_timestamp as u64,
                now_timestamp as u64,
            )
            .unwrap();
            assert!(actual <= total_user_amount);
        }
    }

    #[quickcheck]
    fn test_idempotence(
        tge_unlock_percent: u32,
        vesting_period: u32,
        tge_timestamp: u32,
        now_timestamp: u32,
    ) {
        let total_user_amount = 1_000_000;
        let token_claimed = 0;

        if vesting_period == 0 && tge_unlock_percent < 100 || tge_unlock_percent > 100 {
            let err = super::calculate_entitled_amount(
                total_user_amount,
                tge_unlock_percent as u64,
                vesting_period as u64,
                token_claimed,
                tge_timestamp as u64,
                now_timestamp as u64,
            );
            assert_eq!(err.err(), Some(ErrorCode::InvalidInput.into()));
        } else {
            let can_claim_now = super::calculate_entitled_amount(
                total_user_amount,
                tge_unlock_percent as u64,
                vesting_period as u64,
                token_claimed,
                tge_timestamp as u64,
                now_timestamp as u64,
            )
            .unwrap();
            let extra = super::calculate_entitled_amount(
                total_user_amount,
                tge_unlock_percent as u64,
                vesting_period as u64,
                token_claimed + can_claim_now,
                tge_timestamp as u64,
                now_timestamp as u64,
            )
            .unwrap();
            assert!(extra == 0);
        }
    }
}
