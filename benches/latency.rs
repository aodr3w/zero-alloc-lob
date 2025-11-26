use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use zero_alloc_lob::engine::book::OrderBook;
use zero_alloc_lob::storage::layout::Side;

fn benchmark_place_order(c: &mut Criterion) {
    // Setup: Create a book with enough capacity so we don't OOM during the test
    let mut book = OrderBook::new("BTC-USDT", 1_000_000);

    // Pre-fill the book with some orders so we aren't inserting into an empty list (too easy)
    // We add 100 orders to simulate a "live" book traversal cost.
    for i in 0..100 {
        book.place_limit_order(i, Side::Sell, 50_000 + i, 100)
            .unwrap();
    }

    let next_id = 1000;

    c.bench_function("place_limit_order_no_match", |b| {
        b.iter(|| {
            // We place a Buy order that WON'T match (50,000 < 50,000 + i)
            // This tests the "Insertion Latency" (Alloc + Sorted Insert)
            let _ = black_box(book.place_limit_order(
                black_box(next_id),
                black_box(Side::Buy),
                black_box(40_000),
                black_box(10),
            ));

            // Cleanup: We immediately cancel it to keep the book state stable for the next run
            let _ = black_box(book.cancel_order(next_id));

            // Note: We recycle the ID to keep the map size stable
        })
    });
}

fn benchmark_match_order(c: &mut Criterion) {
    let mut book = OrderBook::new("BTC-USDT", 1_000_000);

    // Scenario: Book has 10,000 Sell orders.
    // We want to measure the time it takes to MATCH against the top one.
    for i in 0..10_000 {
        book.place_limit_order(i, Side::Sell, 50_000 + i, 100)
            .unwrap();
    }

    let taker_id = 20_000;

    c.bench_function("execute_match_single", |b| {
        b.iter(|| {
            // Place a Buy order that crosses the spread (Matches the top Sell at 50,000)
            // We use Fill-and-Kill style (qty 10 vs 100) so the resting order isn't removed,
            // just reduced. This keeps the book structure stable.
            let _ = black_box(book.place_limit_order(
                black_box(taker_id),
                black_box(Side::Buy),
                black_box(55_000), // Crosses everything
                black_box(10),     // Small qty
            ));

            // Note: In a real match benchmark, managing state is hard because orders disappear.
            // This specific micro-benchmark measures the allocation + match logic overhead.
        })
    });
}

fn benchmark_deep_insertion(c: &mut Criterion) {
    let mut book = OrderBook::new("BTC-USDT", 1_000_000);

    // Setup: Create 5,000 Sell orders ordered by price.
    // Prices: 50,000, 50,001, ..., 54,999
    // The Linked List will be 5,000 nodes long.
    for i in 0..5_000 {
        book.place_limit_order(i, Side::Sell, 50_000 + i, 100)
            .unwrap();
    }

    let next_id = 10_000;

    c.bench_function("place_order_middle_of_book_5k", |b| {
        b.iter(|| {
            // We place a Sell order at 52,500.
            // This is worse than 50,000 but better than 54,999.
            // It should land roughly in the middle (index 2,500).
            // The engine MUST walk 2,500 pointers to find the spot.
            let _ = black_box(book.place_limit_order(
                black_box(next_id),
                black_box(Side::Sell),
                black_box(52_500),
                black_box(10),
            ));

            let _ = black_box(book.cancel_order(next_id));
        })
    });
}

criterion_group!(
    benches,
    benchmark_place_order,
    benchmark_match_order,
    benchmark_deep_insertion
);
criterion_main!(benches);
