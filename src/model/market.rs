use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct MarketEvent {
    pub title: String,
    pub items: Vec<String>,
    pub suburb_modifiers: HashMap<String, f32>,
    pub renovator_modifier: f32,
    pub buyer_budget_modifier: f32,
    pub strategy_effect: String,
}

impl MarketEvent {
    pub fn suburb_modifier(&self, suburb: &str) -> f32 {
        self.suburb_modifiers.get(suburb).copied().unwrap_or(0.0)
    }
}
