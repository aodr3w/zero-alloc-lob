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


### 🚀 Performance Benchmarks

Benchmarks run on Apple M1/M2 Pro (3.2 GHz).

| Metric | Condition | Result (Mean) | Complexity |
| :--- | :--- | :--- | :--- |
| **Place Order** | Top of Book (Best Bid/Ask) | **~74 ns** | **O(1)** |
| **Match Execution** | Single Trade | **~72 ns** | **O(1)** |
| **Deep Insertion** | Middle of 5,000 Orders | **~4.36 μs** | **O(N)** |


### Analysis

**Hot Path (~74ns)**: The engine achieves sub-100ns latency for updates at the best price level. This is due to the pointer-based design avoiding all syscalls.

**Deep Book (~1.7ns per node)**: While O(N) insertion is slower (using a Linked List), the linear scan speed proves the cache benefits of the Arena. Traversing orders takes ~1.7ns/hop, indicating a near 100% L1 Cache Hit rate.



# 🏗 Architecture

The engine is built on top of the llt-rs (Low Latency Toolkit) ecosystem.

```
crate: https://crates.io/crates/llt-rs

github: https://github.com/aodr3w/llt-rs

```



## 📦 Installation & Usage

This is an exhibition project demonstrating low-latency systems programming techniques.

### 1. Setup

Clone the repository and enter the directory:

```

git clone git@github.com:aodr3w/zero-alloc-lob.git
cd zero-alloc-lob

```

### 2. Run Example

See the engine in action with the provided example scenario:

```
cargo run --example simple_match

```

### 3. Run Latency Benchmarks

Reproduce the performance metrics using Criterion:

```
cargo bench

```

## Memory Layout

**Orders**: `NonNull` pointers into a pre-allocated byte buffer `(Arena)`

**Indexing**: `HashMap<OrderId, OrderPtr>` for O(1) cancellation lookups.

**Recycling**: Canceled orders are pushed to a `Vec<OrderPtr>` stack (acting as a Free List), allowing O(1) memory reuse without fragmentation or allocation.



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
