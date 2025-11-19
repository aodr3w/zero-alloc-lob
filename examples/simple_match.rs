use zero_alloc_lob::engine::book::OrderBook;
use zero_alloc_lob::storage::layout::Side;

fn main() {
    println!("Initializing Zero-Allocation Order Book...");
    let mut book = OrderBook::new("BTC-USDT", 100_000);

    // 1. Place a SELL Order (Maker)
    // Sell 1.0 BTC @ 50,000
    println!("\n> [1] Placing SELL: 1.0 @ 50,000");
    let (ptr1, trades1) = book.place_limit_order(1, Side::Sell, 50_000, 100).unwrap();
    println!("    Status: Rested (Addr: {:p})", ptr1.unwrap().as_ptr());
    println!("    Trades: {}", trades1.len());

    // 2. Place a SELL Order (Maker)
    // Sell 0.5 BTC @ 51,000
    println!("\n> [2] Placing SELL: 0.5 @ 51,000");
    let (ptr2, _) = book.place_limit_order(2, Side::Sell, 51_000, 50).unwrap();
    println!("    Status: Rested (Addr: {:p})", ptr2.unwrap().as_ptr());

    // 3. Place a BUY Order (Taker)
    // Buy 1.2 BTC @ 52,000
    // This should eat the entire 1.0 @ 50,000 and part of the 0.5 @ 51,000?
    // WAIT: Our current implementation is "Stack" (LIFO) insert for simplicity.
    // So it will match the 51,000 order FIRST (because it was inserted last and is at the head).
    // This highlights why we need Sorted Insert in Phase 2!

    println!("\n> [3] Placing BUY: 1.2 @ 52,000 (Crosses Spread)");
    let (ptr3, trades3) = book.place_limit_order(3, Side::Buy, 52_000, 120).unwrap();

    println!(
        "    Status: {:?}",
        if ptr3.is_some() { "Rested" } else { "Filled" }
    );
    println!("    Trades Generated: {}", trades3.len());

    for (i, trade) in trades3.iter().enumerate() {
        println!(
            "    Trade #{}: Maker={} Taker={} Price={} Qty={}",
            i + 1,
            trade.maker_id.0,
            trade.taker_id.0,
            trade.price.0,
            trade.quantity.0
        );
    }

    println!("\n> Final Memory Usage: {} bytes", book.used_bytes());
}
