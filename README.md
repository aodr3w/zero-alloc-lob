# zero-alloc-lob

A deterministic, zero-allocation Limit Order Book (LOB) engine written in Rust.


## 📖 Overview

**zero-alloc-lob** is a high-performance matching engine designed for environments where tail latency matters more than average latency (e.g., HFT, Market Making, and Exchange Infrastructure).

Unlike standard matching engines that rely on dynamic memory allocation (causing non-deterministic heap fragmentation and allocator pauses), this engine operates entirely on pre-allocated memory structures. It guarantees O(1) or amortized O(1) memory operations on the hot path.

## ⚡ Key Features

**Zero Dynamic Allocation**: All order objects are managed via a static Arena (Bump Allocator) and recycled via an O(1) FreeList. No malloc/free calls occur during the trading session.

**Cache Locality**: Optimized for L1/L2 cache hits by using contiguous memory layouts rather than random heap pointer chasing.

**Deterministic Execution**: The engine state is purely a function of the input sequence, making it ideal for replay-based debugging and high-fidelity backtesting.

**Price-Time Priority**: Implements standard matching logic using intrusive linked lists.


## 🚀 Performance Benchmarks

Benchmarks run on Apple M1/M2 Pro (3.2 GHz).

--------------------------------------------------------------------------------
Metric               Condition                           Result       Complexity 
--------------------------------------------------------------------------------
**Place Order**       Top of Book (Best Bid/Ask)          ~74 ns       O(1)

**Match Execution**   Single Trade                        ~72 ns       O(1)

**Deep Insertion**    Middle of 5,000 Orders              ~4.36 µs     O(N)

---------------------------------------------------------------------------------


### Analysis

**Hot Path (~74ns)**: The engine achieves sub-100ns latency for updates at the best price level. This is due to the pointer-based design avoiding all syscalls.

**Deep Book (~1.7ns per node)**: While O(N) insertion is slower (using a Linked List), the linear scan speed proves the cache benefits of the Arena. Traversing orders takes ~1.7ns/hop, indicating a near 100% L1 Cache Hit rate.



# 🏗 Architecture

The engine is built on top of the llt-rs (Low Latency Toolkit) ecosystem.


## 📦 Installation & Usage

This library is currently private. It relies on the local llt-rs crate.


## Memory Layout

**Orders**: `NonNull` pointers into a pre-allocated byte buffer `(Arena)`

**Indexing**: `HashMap<OrderId, OrderPtr>` for O(1) cancellation lookups.

**Recycling**: Canceled orders are pushed to a `Vec<OrderPtr>` stack (acting as a Free List), allowing O(1) memory reuse without fragmentation or allocation.


## 📦 Installation & Usage

### git clone (ssh)

```
git clone git@github.com:aodr3w/zero-alloc-lob.git
cd zero-alloc-lob

# Run the example
cargo run --example simple_match

# Run benchmarks
cargo bench

```
### Example

```
use zero_alloc_lob::engine::book::OrderBook;
use zero_alloc_lob::storage::layout::Side;

fn main() {
    // 1. Warm Up: Pre-allocate memory for 1 million orders
    let mut book = OrderBook::new("BTC-USDT", 1_000_000);

    // 2. The Hot Path (Zero Allocation)
    // Maker: Places a passive order
    book.place_limit_order(101, Side::Buy, 50_000, 100).unwrap(); 
    
    // Taker: Matches against the resting order
    book.place_limit_order(102, Side::Sell, 50_000, 50).unwrap(); 
}

```


## PROJECT LAYOUT

```
zero-alloc-lob/
├── Cargo.toml
├── README.md
├── benches/             
│   └── latency.rs   # Criterion benchmarks
├── src/
│   ├── lib.rs            # Public API
│   ├── engine/           # The matching logic
│   │   ├── mod.rs
│   │   ├── book.rs       # The Book struct (Arena Owner)
│   │   └── matcher.rs    # The execution logic
│   └── storage/          # The memory layout
│       ├── mod.rs
│       └── layout.rs     # Arena-compatible structs
└── examples/
    └── simple_match.rs  # Runnable example

```
