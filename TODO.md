# TODO — Auction House Tycoon

## Auction tension

- Rival bidders routinely run past the profitable ceiling, so early lots read as unwinnable unless the player overpays; retune `auction_bidders` aggression against reserve and appraised value.
- Give bidding tactics weight — a jump bid should visibly rattle or embolden rivals rather than just advancing the price.
- Let the schedule occasionally offer a genuine bargain so patience and walking away are rewarded.
- Make going over budget pay off sometimes, so the decision is a gamble rather than a tax.

## Presentation

- The screens read flat; the auction needs pacing, callouts, and rival reactions to carry the tension the sim already produces.

## Testing

- `src/sim/auction_sim.rs` and `src/sim/auction_bidders.rs` have no tests. Add deterministic coverage for tied bids, last-moment bids, pass-in below reserve, and bidder budget ceilings.
