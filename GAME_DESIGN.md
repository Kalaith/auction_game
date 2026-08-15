# Auction House Tycoon — Rebuild Design

## Player fantasy

The player is a disciplined small property investor growing from cash and a bank pre-approval into a three-home portfolio. The game should reproduce the useful parts of a real house auction: misleading guides, imperfect research, public pressure, rival behaviour, finance limits, settlement costs, and the fact that winning can be the worst outcome.

## Campaign promise

By the end of week 24 the player must simultaneously:

- own at least three homes;
- collect at least $1,500 in gross weekly rent;
- hold at least $240,000 in net worth.

The three requirements prevent a single degenerate strategy. Cheap but poor assets may achieve the door count without the rent. Leveraged rentals may achieve rent without equity. Repeated flips may create equity without a portfolio.

## Weekly loop

1. Read the market pulse and rotating Saturday schedule.
2. Inspect listings, buy due diligence, and choose a walk-away style.
3. Spend up to two registrations on live auctions—or deliberately spend none.
4. Review the unconditional auction terms, deposit exposure, finance at the cap, and rival room; then tap START AUCTION CALLS.
5. Read rival tells and choose between RAISE, one ASSERT jump, WAIT & READ ROOM, and WALK AWAY.
6. Settle a win, then repair, improve, lease, hold, or sell the asset.
7. Advance the week to collect rent, pay management, ownership costs, and interest, progress works, rotate the market, and refresh registrations.

## Decision rules

- The guide is advertising, not valuation.
- Research buys narrower uncertainty, not a guaranteed answer.
- A walk-away is a commitment device, not a hard lock.
- The live walk-away panel underwrites rental cashflow at that exact price after management, property outgoings, and loan interest; raising the ceiling can turn a sound rental into a weekly loss.
- The same cashflow forecast follows the next executable bid into the auction room so the operating thesis remains visible under pressure.
- Registration assigns a deterministic visible paddle, and the live price always identifies whether that paddle, a rival, or a declared vendor bid currently leads.
- The hammer debrief turns price into an ownership position: contract deposit, new loan, settlement cash, fees, rent appraisal, and leveraged weekly cashflow are visible before SETTLE PURCHASE.
- The vendor's true reserve stays hidden; research provides an estimate and the auctioneer announces when bidding crosses the real line.
- The room opens in $10,000 rises. Crossing reserve triggers the on-market call and the auctioneer tightens every later raise to $5,000, making late discipline more granular.
- One declared vendor bid may move the auction below reserve, but can never buy the property or put it on the market.
- A pass-in moves to a private vendor counteroffer below reserve; accepting still requires finance and can still violate the player's walk-away.
- Before meeting that counter, the player may test the vendor once at the passed-in price. Seller flexibility follows demand and disclosed risk, and a rejection leaves the counteroffer available.
- A hammer win requires a 10% contract deposit plus separately visible acquisition costs; both are included in settlement cash and the deal remains unconditional.
- ASSERT is theatre with asymmetric consequences: it discourages rational buyers and can provoke emotional ones.
- First-home buyers stretch near the final call, renovators stretch for damaged upside in emotional rooms, and ego bidders stretch when counter-bidding becomes personal; each exposes a distinct live tell while rational bidders keep their ceiling.
- Each auction owns a serialized random stream, so saving and reloading cannot reroll rival reactions or auctioneer timing.
- A room read is earned by lowering the paddle; the observation becomes stale on the next bid, and better diligence makes it more precise.
- A renovation is recommended only when its expected value exceeds its full quoted effect.
- Known structural defects must be repaired before a tenant can be placed.
- Completed improvements raise the next rent appraisal; an existing tenancy can be ended for one week's rent to reopen renovation.
- Leasing requires an upfront two-week letting fee and one vacant campaign week; the tenancy starts only after that week's ownership and interest costs close.
- New tenancies reach a review every eight held weeks. Renewing preserves occupancy; testing a higher market rent can improve income or send a low-demand tenant out, requiring another leasing campaign.
- The review card labels whether the proposed ASK is appraisal-supported, demand-supported, or above appraisal with vacancy risk; the resolution uses that same visible classification.
- A due review pauses week advancement from every screen and routes the player to the affected tenancy, so the decision cannot be bypassed through another holding.
- Rent supports cashflow and borrowing capacity, but debt, ownership costs, and scheduled maintenance remain visible.
- Spare cash can reduce a selected property's principal, trading liquidity for lower debt and more bank headroom.
- A clean, leased home held for four weeks can release equity up to 80% LVR and the bank limit; the cash funds growth while debt and interest rise.
- Each property reports its current equity, loan-to-value ratio, and annual interest rate so refinance capacity is legible rather than magical.
- Each holding also reports its own net weekly cashflow after effective rent, management, interest, property costs, and current maintenance losses.
- The safe-cash target grows by $8,000 per door (with an $18,000 starter minimum), and both bidder terms and live guidance flag bids that breach it.
- A sale outcome separates the hammer price from fees, loan repayment, settlement cash released, and true deal profit.
- Every advanced week closes a visible statement for rent, management, interest, property costs, net cashflow, and debt-funded shortfall.
- Maintenance checks occur on a disclosed schedule, reduce collected rent until repaired, and never arrive as uncontrolled random punishment.
- Walking away preserves capital and can build discipline reputation when a rival overheats.
- The season ledger values attendance, settled purchases, disciplined exits, post-auction buys, completed sales, and realized profit.
- Unused weekly registrations are recorded as restraint, then shown beside purchase discipline, final rent, and realised profit in the campaign debrief.
- Named bidders recur across authored Saturdays. Completed rooms persist appearances, wins, stretch behaviour, and highest prices in the player's notebook, so recognition becomes earned information.
- The campaign resolves immediately when all three portfolio conditions are true, including after settlement, leasing, or a required repair; success never demands an artificial extra week of costs.
- A failed season identifies each exact shortfall and names the largest normalized constraint, turning the final ledger into strategy feedback for the next portfolio rather than a generic game-over screen.

## Content structure

Twelve authored properties cover rental holds, quiet bargains, risky fixers, pretty traps, land plays, renovator bait, hot-suburb FOMO, and auction traps. Six market pulses rotate suburb demand, borrowing power, renovation appetite, premium liquidity, and rental conditions. The weekly schedule rotates deterministically so patience changes what appears without hiding the system behind uncontrolled randomness.

The listing art reinforces the asset read: compact sites use taller terrace-like facades, premium homes show balconies, columns, and landscaping, while large blocks shrink the dwelling into a visible yard with fencing and outbuildings.
