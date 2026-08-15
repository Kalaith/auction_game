use crate::model::{Auction, AuctionStatus, RivalRecord};

pub fn record_completed_room(notebook: &mut Vec<RivalRecord>, auction: &Auction) {
    let Some(status) = auction.status.as_ref() else {
        return;
    };
    let winner = match status {
        AuctionStatus::SoldToNpc(name) => Some(name.as_str()),
        _ => None,
    };

    for bidder in &auction.bidders {
        let won = winner == Some(bidder.name.as_str());
        if let Some(record) = notebook
            .iter_mut()
            .find(|record| record.name == bidder.name)
        {
            record.auctions_met += 1;
            record.auctions_won += u32::from(won);
            record.highest_room_price = record.highest_room_price.max(auction.current_bid);
            record.stretches_seen += u32::from(bidder.stretch_bid_used);
        } else {
            notebook.push(RivalRecord {
                name: bidder.name.clone(),
                bidder_type: bidder.bidder_type,
                auctions_met: 1,
                auctions_won: u32::from(won),
                highest_room_price: auction.current_bid,
                stretches_seen: u32::from(bidder.stretch_bid_used),
            });
        }
    }
}

#[cfg(test)]
mod tests;
