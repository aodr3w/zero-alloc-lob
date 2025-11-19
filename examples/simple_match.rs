use zero_alloc_lob::engine::book::OrderBook;
use zero_alloc_lob::storage::layout::Side;

fn main() {
    println!("--- Order Book Initialization ---");
    let initial_capacity = 100_000;
    let mut book = OrderBook::new("BTC-USDT", initial_capacity);
    println!("> Arena Capacity: {} bytes", book.capacity_bytes());

    // --- 1. FIRST ORDER: Allocate New Memory ---
    let order_id_1 = 101;
    let price_1 = 50_000;
    let quantity = 100;

    println!("\n--- 1. Placing Order 101 (Initial Allocation) ---");
    book.place_limit_order(order_id_1, Side::Sell, price_1, quantity)
        .unwrap();

    let used_after_1 = book.used_bytes();
    let slot_size = used_after_1;

    println!(
        "    Used Bytes: {} ({} bytes/slot)",
        used_after_1, slot_size
    );
    println!("    Active Orders: {}", book.active_orders());

    // --- 2. CANCEL ORDER 1: Creates a Memory Hole (Recyclable Slot) ---
    println!("\n--- 2. Cancelling Order 101 (Creates Free Slot) ---");
    book.cancel_order(order_id_1).unwrap();

    let used_after_cancel = book.used_bytes();
    let free_slots = book.free_slots();

    println!("    Used Bytes (Unchanged): {}", used_after_cancel); // Should be the same
    println!("    Active Orders: {}", book.active_orders());
    println!("    Free Slots Available: {}", free_slots);

    assert_eq!(used_after_1, used_after_cancel);
    assert_eq!(free_slots, 1);

    // --- 3. SECOND ORDER: Recycle Canceled Memory Slot ---
    let order_id_2 = 102;
    let price_2 = 50_001;

    println!("\n--- 3. Placing Order 102 (Recycling Slot) ---");
    book.place_limit_order(order_id_2, Side::Sell, price_2, quantity)
        .unwrap();

    let used_after_2 = book.used_bytes();
    let free_slots_after_reuse = book.free_slots();

    println!("    Used Bytes: {}", used_after_2);
    println!("    Active Orders: {}", book.active_orders());
    println!("    Free Slots Available: {}", free_slots_after_reuse);

    // THE CRITICAL TEST: Used bytes must NOT increase!
    assert_eq!(
        used_after_1, used_after_2,
        "Memory leak detected! Used bytes should not increase after recycling."
    );
    assert_eq!(free_slots_after_reuse, 0, "Free list was not drained.");

    println!("\n✅ SUCCESS: Memory slot was recycled. Zero dynamic allocation maintained.");

    // Final state cleanup
    book.cancel_order(order_id_2).unwrap();
}
