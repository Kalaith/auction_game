use super::*;

#[test]
fn purchase_cash_needed_includes_deposit_and_fees() {
    assert_eq!(deposit(600_000), 72_000);
    assert_eq!(purchase_fees(600_000), 23_000);
    assert_eq!(cash_needed_to_settle(600_000), 95_000);
}
