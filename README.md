# zero-alloc-lob

**A deterministic, zero-allocation Limit Order Book(LOB) engine written in Rust.**

**⚠ Status**: Active Development / Private Alpha


**🎯 Goal**: < 500ns matching latency (tick-to-trade)


## 📖 Overview

**zero-alloc-lob** is a high-performance matching engine designed for environments where tail latency matters more than average latency (e.g., HFT, Market Making, and Exchange Infrastructure).

Unlike standard matching engines that rely on dynamic memory allocation (causing non-deterministic heap fragmentation and GC/allocator pauses), this engine operates entirely on pre-allocated memory structures. It guarantees O(1) or amortized O(1) memory operations on the hot path.

## ⚡ Key Features

**Zero Dynamic Allocation**: All order objects and nodes are managed via static Arenas and Object Pools provided by llt-rs. No malloc/free calls during the trading session.

**Cache Locality**: Optimized for L1/L2 cache hits by using contiguous memory layouts rather than pointer chasing.

**Deterministic Execution**: The engine state is purely a function of the input sequence, making it ideal for replay-based debugging and high-fidelity backtesting.

**Safe Concurrency**: Built on Rust's safety guarantees, ensuring memory safety without the overhead of garbage collection.


# 🏗 Architecture

The engine is built on top of the llt-rs (Low Latency Toolkit) ecosystem.

## Core Dependencies

**llt-rs**: Provides the underlying memory primitives.

feature = ["arena_allocator"]: Used for storing Order Nodes (Red-Black Tree or Splay Tree nodes).


## 🚀 Performance Benchmarks

Benchmarks run on Apple M1/M2 Pro (3.2 GHz).

--------------------------------------------------------------------------------
Metric               Condition                           Result       Complexity 
--------------------------------------------------------------------------------
**Place Order**       Top of Book (Best Bid/Ask)          ~74 ns       O(1)

**Match Execution**   Single Trade                        ~72 ns       O(1)

**Deep Insertion**    Middle of 5,000 Orders              ~4.36 µs     O(N)

---------------------------------------------------------------------------------


### Analysis of Results

**Hot Path (~74ns)**: The engine achieves sub-100ns latency for updates at the best price level. This is due to the pointer-based design avoiding all syscalls.

**Deep Book (~1.7ns per node)**: While O(N) insertion is slower, the linear scan speed proves the cache benefits of the Arena. Traversing orders takes ~1.7ns/hop, indicating a near 100% L1 Cache Hit rate.

## 📦 Installation & Usage

This library is currently private. It relies on the local llt-rs crate.


## Data Layout

**Orders**: Stored as NonNull pointers in a pre-allocated byte buffer.

**Indexing**: HashMap<OrderId, OrderPtr> for O(1) cancellation lookups.

**Recycling**: Canceled orders are pushed to a Vec<OrderPtr> stack, allowing O(1) memory reuse without fragmentation.


## PROJECT LAYOUT

```
zero-alloc-lob/
├── Cargo.toml
├── README.md
├── benches/              # Criterion benchmarks (CRITICAL for this project)
│   └── latency.rs
├── src/
│   ├── lib.rs            # Public API
│   ├── engine/           # The matching logic
│   │   ├── mod.rs
│   │   ├── book.rs       # The Book struct
│   │   └── matcher.rs    # The execution logic
│   └── storage/          # The memory layout
│       ├── mod.rs
│       └── layout.rs     # Where we use llt-rs arenas
└── examples/
    └── simple_match.rs   # A runnable example

```
