# Auction House Tycoon

Auction House Tycoon is a property auction training game about buying under pressure, protecting your margin, renovating carefully, and selling before cashflow turns against you.

You start with limited cash, a few accessible suburbs, and enough borrowing power to make dangerous decisions. The goal is not simply to win auctions. The goal is to buy well, know when to walk away, improve the right properties, and build net worth without letting fees, repairs, debt, and holding costs eat the deal.

## Core Fantasy

Buy undervalued houses at auction, improve them, read the market, and resell before your cashflow collapses.

Every deal asks the same questions:

- Can I afford this property after fees and deposit?
- What is my walk-away price?
- What risk am I accepting?
- Will renovation improve the margin, or just make the loss prettier?
- Is selling now smarter than holding out?

## Current Playable Loop

1. Read the weekly market pulse.
2. Compare available auction listings.
3. Inspect a property and set a walk-away price.
4. Bid against NPC buyers in a live auction.
5. Settle the property if you win.
6. Choose whether to renovate, hold, or sell.
7. Run sale auction and review the profit or loss.
8. Advance weeks toward the campaign goal.

The game is built around the idea that walking away can be a win. Winning the room is not useful if you destroy the deal.

## Main Screens

### Dashboard

Shows your cash, buying power, net worth, campaign progress, market pulse, and the week’s featured opportunities.

The dashboard’s job is to answer: what deserves attention this week?

### Listings

Shows available auction properties with guide price, risk, upside, demand, and a short reason to care.

Filters help narrow the search:

- Low Risk
- High Upside
- Hot Demand
- Cheap Entry

### Property Detail

Shows the property image, research range, guide price, recommended walk-away, risk read, finance status, and due diligence options.

This screen is about setting the number before auction pressure starts.

### Live Auction

The auction screen is the pressure point of the game.

You can:

- Bid
- Hold position
- Walk away

NPC bidders have different personalities and limits. The UI calls out whether the next bid is still safe, thin, over plan, or blocked by finance.

### Portfolio

After buying, the property becomes a cashflow problem.

You can:

- Start a renovation project
- Advance the week while work continues
- Hold for market changes
- Sell when the property is ready

Renovations are now projects rather than instant upgrades. They cost money up front, take time, lock the sale while active, and only affect value once completed.

### Sale Result

The result screen breaks down the final sale, fees, costs, profit or loss, and the lesson from the deal.

The point is not just to show a receipt. The point is to explain why the outcome happened.

## Game Systems

### Walk-Away Price

The walk-away price is the player’s discipline anchor.

Bidding over it is allowed when possible, but the game highlights that the plan has been broken. Sale results also call out how far over the walk-away line the purchase landed.

### Finance

Cash and borrowing power both matter.

A bid can fail even if the player has some cash left, because deposits, fees, debt, and bank headroom all affect whether the purchase can settle.

### Market Pulse

Each week changes market conditions.

Market news affects buyer demand, suburb appeal, fixer-upper interest, and borrowing pressure. The player should not chase the same type of property every week.

### NPC Bidders

Auctions include competing bidders with different behaviour patterns, such as investors, renovators, developers, first-home buyers, and bargain hunters.

The auction should feel like a small contest against people with their own incentives, not a static price ladder.

### Renovation

Renovation is a risk/reward decision.

Current renovation choices consider:

- Build cost
- Holding cost
- Contractor tier
- Project duration
- Permit or delay risk
- Value boost
- Appeal boost
- Sale emotion boost
- Whether the job repairs a defect
- Whether the spend is likely to overcapitalise the deal

The best decision is not always to renovate. Sometimes the correct move is to sell, hold, or accept that the purchase price was the real mistake.

### Campaign Goal

The starter campaign is about growing net worth over a limited number of weeks.

Progress unlocks better suburbs and gives the player more room to make bigger, riskier decisions.

## Design Direction

Auction House Tycoon should teach without lecturing.

The game should keep asking for one clear decision at a time:

- Which property is worth inspecting?
- What is the walk-away price?
- Bid or stop?
- Repair, hold, or sell?
- What did this deal teach?

The best version of the game feels like a tense property auction wrapped around a clear financial lesson. It should reward patience, margin discipline, and reading the market, not just clicking the biggest button.

