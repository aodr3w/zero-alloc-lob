use crate::engine::book::OrderBook;
use crate::storage::layout::{OrderId, Price, Quantity, Side};

#[derive(Debug, Clone)]
pub struct Trade {
    pub maker_id: OrderId,
    pub taker_id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
    pub maker_side: Side,
}

/*
Executes an incoming order against the book
Returns a tuple.
1. The remaining quantity of the incoming order (if any).
2. A list of Trade events generated.
*/
pub fn execute_match(
    book: &mut OrderBook,
    taker_id: OrderId,
    taker_side: Side,
    taker_price: Price,
    mut taker_qty: Quantity,
) -> (Quantity, Vec<Trade>) {
    let mut trades = Vec::with_capacity(16); //Pre-allocate for common case

    loop {
        //1 if incoming is filled, stop.
        if taker_qty.0 == 0 {
            break;
        }
        //2 Get the best order on the OPPOSITE side.
        let best_match_ptr = match taker_side {
            Side::Buy => book.best_ask,  // Buyer looks at Asks
            Side::Sell => book.best_bid, // Seller looks at Bids
        };

        //3. if the book is empty on that side, stop.
        let mut maker_ptr = match best_match_ptr {
            Some(ptr) => ptr,
            None => break,
        };

        //4. Access the maker order data
        // SAFETY: We hold a mutable reference to `book`, so no one else can touch the arena.
        // The pointer is guaranteed valid as long as we don't reset the arena.

        let maker_order = unsafe { maker_ptr.as_mut() };

        //5. Check Price Crossing (Limit logic)
        let crosses = match taker_side {
            Side::Buy => taker_price >= maker_order.price, // Buyer willing to pay >= Ask,
            Side::Sell => taker_price <= maker_order.price, // Seller willing to accept <= Bid
        };

        if !crosses {
            break; // Spread is not crossed, stop matching.
        }

        //6. MATCH FOUND - Calculate Trade Quantity
        let trade_qty = std::cmp::min(taker_qty.0, maker_order.qty.0);

        //7 Generate Event
        trades.push(Trade {
            maker_id: maker_order.id,
            taker_id,
            price: maker_order.price,
            quantity: Quantity(trade_qty),
            maker_side: maker_order.side,
        });

        // 8. Update Quantities
        taker_qty.0 -= trade_qty;
        maker_order.qty.0 -= trade_qty;

        // 9. If Maker is filled, remove from book
        if maker_order.qty.0 == 0 {
            // Delegate removal to the book to handle pointer rewiring
            book.remove_order(maker_ptr);
        }
    }
    (taker_qty, trades)
}
