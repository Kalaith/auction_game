use super::*;

#[test]
fn an_older_hammer_debrief_gains_empty_portfolio_underwriting_fields() {
    let legacy = r#"{
        "address":"1 Legacy Lane",
        "purchase_price":500000,
        "estimated_resale":540000,
        "fees":22000,
        "cash_to_settle":72000,
        "cash_after_settle":148000,
        "renovation_allowance":12000,
        "walkaway_delta":0,
        "projected_profit":6000,
        "lesson":"Legacy lesson"
    }"#;

    let debrief: PurchaseDebrief =
        serde_json::from_str(legacy).expect("legacy debrief should load");

    assert_eq!(debrief.purchase_price, 500_000);
    assert_eq!(debrief.contract_deposit, 0);
    assert_eq!(debrief.loan_amount, 0);
    assert_eq!(debrief.weekly_rent, 0);
    assert_eq!(debrief.weekly_rental_cashflow, 0);
}
