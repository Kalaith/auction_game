use super::*;
use crate::model::{Condition, DealArchetype, MarketEvent, Property, ResearchLevel, WalkawayStyle};
use std::collections::HashMap;

fn test_property() -> Property {
    Property {
        id: 7,
        address: "12 Test Street".to_string(),
        suburb: "Ridgefield".to_string(),
        bedrooms: 3,
        bathrooms: 1,
        condition: Condition::Rough,
        land_size: 640,
        market_value: 560_000,
        guide_price: 470_000,
        reserve_price: 505_000,
        appeal: 48,
        renovation_potential: 84,
        hidden_defect_risk: 0.32,
        holding_cost_per_week: 2_000,
        buyer_demand: 68,
        deal_archetype: DealArchetype::RiskyFixer,
        thesis: "Test thesis".to_string(),
        main_risk: "Test risk".to_string(),
        best_strategy: "Test strategy".to_string(),
        bad_strategy: "Test mistake".to_string(),
        notes: "Test fixture".to_string(),
    }
}

fn test_market() -> MarketEvent {
    MarketEvent {
        title: "Test".to_string(),
        items: vec![],
        suburb_modifiers: HashMap::new(),
        renovator_modifier: 0.04,
        buyer_budget_modifier: 0.0,
        strategy_effect: "Test strategy effect".to_string(),
    }
}

#[test]
fn renovation_project_completes_after_weekly_progress() {
    let upgrade = UpgradeData {
        id: "paint_clean".to_string(),
        name: "Paint and Clean".to_string(),
        cost: 9_000,
        value_boost: 18_000,
        appeal_boost: 8,
        sale_emotion_boost: 4,
        removes_defect: false,
        description: "Presentation lift.".to_string(),
    };
    let owned = OwnedProperty::new(
        test_property(),
        430_000,
        16_000,
        52_000,
        378_000,
        530_000,
        ResearchLevel::BuildingInspection,
        WalkawayStyle::Balanced,
    );
    let quote = quote_renovation(
        &owned,
        &upgrade,
        ContractorTier::Reliable,
        &test_market(),
        0,
    );
    let project = start_upgrade_project(&quote, 1, 0);
    let mut player = Player::new();
    player.properties.push(owned);
    player.properties[0].active_renovation = Some(project.clone());

    for _ in 1..project.weeks_total {
        assert!(progress_player_renovations(&mut player).is_empty());
        assert!(player.properties[0].active_renovation.is_some());
    }

    let messages = progress_player_renovations(&mut player);
    assert_eq!(messages.len(), 1);
    assert!(player.properties[0].active_renovation.is_none());
    assert!(player.properties[0].has_upgrade("paint_clean"));
}
